//! # Solution 10: Rate Limit Middleware (Axum/Tower)
//!
//! Complete implementation of a Tower middleware for rate limiting with Axum.

use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use axum::http::header;
use dashmap::DashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};

/// Configuration for the rate limit middleware.
#[derive(Debug, Clone)]
pub struct RateLimitMiddlewareConfig {
    pub max_requests: u64,
    pub window_ms: u64,
}

impl Default for RateLimitMiddlewareConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_ms: 1000,
        }
    }
}

/// Per-IP sliding window counter state.
#[derive(Debug)]
struct WindowState {
    prev_count: u64,
    curr_count: u64,
    window_start: Instant,
}

impl WindowState {
    fn new() -> Self {
        Self {
            prev_count: 0,
            curr_count: 0,
            window_start: Instant::now(),
        }
    }

    fn try_acquire(&mut self, window_ms: u64, max_requests: u64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.window_start).as_millis() as u64;

        if elapsed >= window_ms {
            self.prev_count = self.curr_count;
            self.curr_count = 0;
            self.window_start = now;
        }

        let fraction = if elapsed >= window_ms {
            0.0
        } else {
            1.0 - (elapsed as f64 / window_ms as f64)
        };
        let weighted = self.prev_count as f64 * fraction + self.curr_count as f64;

        if weighted < max_requests as f64 {
            self.curr_count += 1;
            true
        } else {
            false
        }
    }

    fn retry_after_ms(&self, window_ms: u64) -> u64 {
        let elapsed = self.window_start.elapsed().as_millis() as u64;
        if elapsed >= window_ms {
            0
        } else {
            window_ms - elapsed
        }
    }
}

/// Shared rate limiter state across all service clones.
#[derive(Clone)]
struct SharedState {
    config: RateLimitMiddlewareConfig,
    windows: Arc<DashMap<String, WindowState>>,
}

/// Tower Layer for rate limiting.
#[derive(Clone)]
pub struct RateLimitLayer {
    state: SharedState,
}

impl RateLimitLayer {
    /// Create a new rate limit layer with the given configuration.
    pub fn new(config: RateLimitMiddlewareConfig) -> Self {
        Self {
            state: SharedState {
                config,
                windows: Arc::new(DashMap::new()),
            },
        }
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            state: self.state.clone(),
        }
    }
}

/// Tower Service for rate limiting.
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    state: SharedState,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let client_ip = extract_client_ip(&req);
        let mut inner = self.inner.clone();
        let state = self.state.clone();

        Box::pin(async move {
            // Check rate limit
            let (allowed, retry_after_ms) = {
                let mut entry = state.windows
                    .entry(client_ip)
                    .or_insert_with(WindowState::new);
                let allowed = entry.try_acquire(state.config.window_ms, state.config.max_requests);
                let retry = entry.retry_after_ms(state.config.window_ms);
                (allowed, retry)
            };

            if allowed {
                inner.call(req).await
            } else {
                let retry_after = ((retry_after_ms + 999) / 1000).max(1);
                Ok(rate_limit_response(retry_after as u32))
            }
        })
    }
}

/// Extract the client IP address from a request.
fn extract_client_ip(req: &Request<Body>) -> String {
    req.headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Build a 429 Too Many Requests response with Retry-After header.
fn rate_limit_response(retry_after_secs: u32) -> Response<Body> {
    let body = serde_json::json!({
        "error": "Too Many Requests",
        "retry_after": retry_after_secs,
    });

    Response::builder()
        .status(StatusCode::TOO_MANY_REQUESTS)
        .header(header::RETRY_AFTER, retry_after_secs.to_string())
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::ServiceExt;

    async fn handler(_req: Request<Body>) -> Result<Response<Body>, std::convert::Infallible> {
        Ok(Response::new(Body::from("OK")))
    }

    #[tokio::test]
    async fn test_middleware_allows_normal_traffic() {
        let config = RateLimitMiddlewareConfig {
            max_requests: 100,
            window_ms: 1000,
        };
        let layer = RateLimitLayer::new(config);
        let mut service = layer.layer(tower::service_fn(handler));

        let req = Request::builder()
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_middleware_returns_429() {
        let config = RateLimitMiddlewareConfig {
            max_requests: 2,
            window_ms: 1000,
        };
        let layer = RateLimitLayer::new(config);
        let mut service = layer.layer(tower::service_fn(handler));

        for _ in 0..2 {
            let req = Request::builder()
                .header("X-Forwarded-For", "10.0.0.1")
                .body(Body::empty())
                .unwrap();
            let resp = service.ready().await.unwrap().call(req).await.unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
        }

        let req = Request::builder()
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let resp = service.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(resp.headers().contains_key(header::RETRY_AFTER));
    }

    #[tokio::test]
    async fn test_middleware_per_ip_isolation() {
        let config = RateLimitMiddlewareConfig {
            max_requests: 1,
            window_ms: 1000,
        };
        let layer = RateLimitLayer::new(config);
        let mut service = layer.layer(tower::service_fn(handler));

        let req = Request::builder()
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let resp = service.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let req = Request::builder()
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let resp = service.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

        let req = Request::builder()
            .header("X-Forwarded-For", "10.0.0.2")
            .body(Body::empty())
            .unwrap();
        let resp = service.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_proper_retry_after_header() {
        let config = RateLimitMiddlewareConfig {
            max_requests: 1,
            window_ms: 1000,
        };
        let layer = RateLimitLayer::new(config);
        let mut service = layer.layer(tower::service_fn(handler));

        let req = Request::builder()
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        service.ready().await.unwrap().call(req).await.unwrap();

        let req = Request::builder()
            .header("X-Forwarded-For", "10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let resp = service.ready().await.unwrap().call(req).await.unwrap();
        let retry_after = resp.headers()
            .get(header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok());
        assert!(retry_after.is_some(), "Retry-After should be a valid integer");
        assert!(retry_after.unwrap() > 0, "Retry-After should be positive");
    }
}
