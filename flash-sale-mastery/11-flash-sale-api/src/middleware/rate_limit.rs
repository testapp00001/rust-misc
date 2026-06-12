//! # Rate Limit Middleware
//!
//! Per-account rate limiting using an in-memory sliding window.
//!
//! ## Exercise
//!
//! 1. Implement the `RateLimitLayer` that creates `RateLimitService` instances.
//! 2. Implement the `RateLimitService` that checks the rate limiter before
//!    forwarding the request.
//! 3. Extract the `X-Account-Id` header (or `account_id` from the body) to
//!    identify the caller.
//! 4. Return `429 Too Many Requests` with a `Retry-After` header when exceeded.
//! 5. Write tests for requests within and exceeding the limit.

use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tower::{Layer, Service};

/// Sliding window rate limiter state per account.
#[derive(Debug)]
struct AccountWindow {
    /// Timestamps of recent requests within the window.
    timestamps: VecDeque<Instant>,
}

/// Thread-safe, per-account rate limiter.
#[derive(Clone, Debug)]
pub struct RateLimiter {
    windows: std::sync::Arc<DashMap<String, AccountWindow>>,
    max_requests: u64,
    window: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(max_requests: u64, window: Duration) -> Self {
        Self {
            windows: std::sync::Arc::new(DashMap::new()),
            max_requests,
            window,
        }
    }

    /// Check whether `account_id` is allowed to make a request right now.
    ///
    /// Returns `true` if allowed, `false` if rate-limited.
    pub fn check(&self, account_id: &str) -> bool {
        let now = Instant::now();
        let mut entry = self
            .windows
            .entry(account_id.to_string())
            .or_insert_with(|| AccountWindow {
                timestamps: VecDeque::new(),
            });

        // Remove timestamps outside the window
        while let Some(&front) = entry.timestamps.front() {
            if now.duration_since(front) > self.window {
                entry.timestamps.pop_front();
            } else {
                break;
            }
        }

        if (entry.timestamps.len() as u64) < self.max_requests {
            entry.timestamps.push_back(now);
            true
        } else {
            false
        }
    }

    /// Calculate how long until the account can make another request.
    pub fn retry_after(&self, account_id: &str) -> Duration {
        let entry = self.windows.get(account_id);
        match entry.and_then(|e| e.timestamps.front().copied()) {
            Some(oldest) => {
                let elapsed = oldest.elapsed();
                if elapsed < self.window {
                    self.window - elapsed
                } else {
                    Duration::ZERO
                }
            }
            None => Duration::ZERO,
        }
    }
}

// ---------------------------------------------------------------------------
// Tower Layer + Service
// ---------------------------------------------------------------------------

/// Tower layer that wraps requests with per-account rate limiting.
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: RateLimiter,
}

impl RateLimitLayer {
    pub fn new(limiter: RateLimiter) -> Self {
        Self { limiter }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: self.limiter.clone(),
        }
    }
}

/// Tower service that enforces per-account rate limits.
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: RateLimiter,
}

impl<S, ReqBody, ResBody> Service<axum::http::Request<ReqBody>> for RateLimitService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = axum::http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    ReqBody: Send + 'static,
    ResBody: Default + Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        // Extract account_id from the X-Account-Id header.
        let account_id = req
            .headers()
            .get("x-account-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("anonymous")
            .to_string();

        let allowed = self.limiter.check(&account_id);

        if !allowed {
            let retry_after = self.limiter.retry_after(&account_id);
            let retry_secs = retry_after.as_secs().max(1);

            let response = axum::http::Response::builder()
                .status(axum::http::StatusCode::TOO_MANY_REQUESTS)
                .header("Retry-After", retry_secs.to_string())
                .header("Content-Type", "application/json")
                .body(ResBody::default())
                .unwrap();

            return Box::pin(async move { Ok(response) });
        }

        let mut inner = self.inner.clone();
        Box::pin(async move { inner.call(req).await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::{ServiceBuilder, ServiceExt};
    use axum::http::{Request, Response, StatusCode};

    fn test_limiter() -> RateLimiter {
        RateLimiter::new(3, Duration::from_secs(1))
    }

    #[tokio::test]
    async fn test_requests_within_limit() {
        let limiter = test_limiter();
        let mut svc = ServiceBuilder::new()
            .layer(RateLimitLayer::new(limiter))
            .service_fn(|_req: Request<()>| async {
                Ok::<_, String>(Response::new(()))
            });

        for _ in 0..3 {
            let req = Request::builder()
                .header("x-account-id", "acct-1")
                .body(())
                .unwrap();
            let resp = svc.ready().await.unwrap().call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    async fn test_requests_exceeding_limit() {
        let limiter = test_limiter();
        let mut svc = ServiceBuilder::new()
            .layer(RateLimitLayer::new(limiter))
            .service_fn(|_req: Request<()>| async {
                Ok::<_, String>(Response::new(()))
            });

        // Use up the quota
        for _ in 0..3 {
            let req = Request::builder()
                .header("x-account-id", "acct-2")
                .body(())
                .unwrap();
            svc.ready().await.unwrap().call(req).await.unwrap();
        }

        // Next request should be rate-limited
        let req = Request::builder()
            .header("x-account-id", "acct-2")
            .body(())
            .unwrap();
        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(resp.headers().contains_key("Retry-After"));
    }

    #[tokio::test]
    async fn test_different_accounts_independent() {
        let limiter = test_limiter();
        let mut svc = ServiceBuilder::new()
            .layer(RateLimitLayer::new(limiter))
            .service_fn(|_req: Request<()>| async {
                Ok::<_, String>(Response::new(()))
            });

        // Exhaust acct-3
        for _ in 0..3 {
            let req = Request::builder()
                .header("x-account-id", "acct-3")
                .body(())
                .unwrap();
            svc.ready().await.unwrap().call(req).await.unwrap();
        }

        // acct-4 should still be allowed
        let req = Request::builder()
            .header("x-account-id", "acct-4")
            .body(())
            .unwrap();
        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
