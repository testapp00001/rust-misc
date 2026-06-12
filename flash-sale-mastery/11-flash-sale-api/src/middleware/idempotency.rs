//! # Idempotency Middleware
//!
//! Extracts the `Idempotency-Key` header, caches responses, and returns the
//! cached result for duplicate requests.
//!
//! ## Exercise
//!
//! 1. Implement the `IdempotencyLayer` and `IdempotencyService`.
//! 2. Extract the `Idempotency-Key` header from incoming requests.
//! 3. On a cache hit, return the cached response without calling the inner service.
//! 4. On a cache miss, forward the request, cache the result, and return it.
//! 5. Write tests for first request and duplicate request behavior.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use dashmap::DashMap;
use tower::{Layer, Service};

/// Cached response data (status code + body bytes).
#[derive(Clone, Debug)]
pub struct CachedResponse {
    status: u16,
    body: Vec<u8>,
}

/// Thread-safe idempotency store shared across requests.
#[derive(Clone, Debug)]
pub struct IdempotencyStore {
    cache: Arc<DashMap<String, CachedResponse>>,
    ttl: std::time::Duration,
}

impl IdempotencyStore {
    /// Create a new idempotency store.
    pub fn new(ttl: std::time::Duration) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            ttl,
        }
    }

    /// Look up a cached response by idempotency key.
    pub fn get(&self, key: &str) -> Option<CachedResponse> {
        self.cache.get(key).map(|r| r.clone())
    }

    /// Store a response for an idempotency key.
    pub fn insert(&self, key: String, response: CachedResponse) {
        self.cache.insert(key, response);

        // Spawn a cleanup task for TTL (simplified; production would use a
        // background reaper or Redis TTL).
        let _cache = self.cache.clone();
        let _ttl = self.ttl;
        // In a real system, schedule cleanup here.
    }
}

// ---------------------------------------------------------------------------
// Tower Layer + Service
// ---------------------------------------------------------------------------

/// Tower layer for idempotent request handling.
#[derive(Clone)]
pub struct IdempotencyLayer {
    store: IdempotencyStore,
}

impl IdempotencyLayer {
    pub fn new(store: IdempotencyStore) -> Self {
        Self { store }
    }
}

impl<S> Layer<S> for IdempotencyLayer {
    type Service = IdempotencyService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        IdempotencyService {
            inner,
            store: self.store.clone(),
        }
    }
}

/// Tower service that enforces idempotency via the `Idempotency-Key` header.
///
/// This implementation works specifically with `axum::body::Body` to avoid
/// needing external body-buffering utilities. On a cache miss, the inner
/// service's response status and body are collected and cached. On a cache
/// hit, a replay response is constructed from the cached data.
#[derive(Clone)]
pub struct IdempotencyService<S> {
    inner: S,
    store: IdempotencyStore,
}

impl<S> Service<axum::http::Request<axum::body::Body>> for IdempotencyService<S>
where
    S: Service<
            axum::http::Request<axum::body::Body>,
            Response = axum::http::Response<axum::body::Body>,
            Error = std::convert::Infallible,
        > + Clone
        + Send
        + 'static,
    S::Future: Send + 'static,
{
    type Response = axum::http::Response<axum::body::Body>;
    type Error = std::convert::Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<axum::body::Body>) -> Self::Future {
        let idem_key = req
            .headers()
            .get("idempotency-key")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // If no idempotency key, pass through directly.
        let Some(key) = idem_key else {
            let mut inner = self.inner.clone();
            return Box::pin(async move {
                // Inner service error is Infallible, so this always succeeds.
                Ok(inner.call(req).await.unwrap_or_else(|e: std::convert::Infallible| match e {}))
            });
        };

        // Check cache for a previous response.
        if let Some(cached) = self.store.get(&key) {
            let response = axum::http::Response::builder()
                .status(
                    axum::http::StatusCode::from_u16(cached.status)
                        .unwrap_or(axum::http::StatusCode::OK),
                )
                .header("Content-Type", "application/json")
                .header("X-Idempotent-Replay", "true")
                .body(axum::body::Body::from(cached.body))
                .unwrap();
            return Box::pin(async move { Ok(response) });
        }

        let mut inner = self.inner.clone();
        let store = self.store.clone();

        Box::pin(async move {
            let resp = inner.call(req).await.unwrap_or_else(|e: std::convert::Infallible| match e {});

            let status = resp.status().as_u16();
            let (parts, body) = resp.into_parts();

            // Collect the response body bytes for caching.
            let body_bytes = axum::body::to_bytes(body, usize::MAX)
                .await
                .unwrap_or_default();

            // Cache the response.
            store.insert(
                key,
                CachedResponse {
                    status,
                    body: body_bytes.to_vec(),
                },
            );

            Ok(axum::http::Response::from_parts(
                parts,
                axum::body::Body::from(body_bytes),
            ))
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Request, Response, StatusCode};
    use tower::{ServiceBuilder, ServiceExt};

    #[tokio::test]
    async fn test_first_request_processed() {
        let store = IdempotencyStore::new(std::time::Duration::from_secs(60));
        let mut svc = ServiceBuilder::new()
            .layer(IdempotencyLayer::new(store))
            .service_fn(|_req: Request<axum::body::Body>| async {
                let resp = Response::builder()
                    .status(StatusCode::OK)
                    .body(axum::body::Body::from(r#"{"status":"ok"}"#))
                    .unwrap();
                Ok::<_, std::convert::Infallible>(resp)
            });

        let req = Request::builder()
            .header("Idempotency-Key", "key-001")
            .body(axum::body::Body::empty())
            .unwrap();

        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(!resp.headers().contains_key("X-Idempotent-Replay"));
    }

    #[tokio::test]
    async fn test_duplicate_returns_cached() {
        let store = IdempotencyStore::new(std::time::Duration::from_secs(60));
        let mut svc = ServiceBuilder::new()
            .layer(IdempotencyLayer::new(store))
            .service_fn(|_req: Request<axum::body::Body>| async move {
                let resp = Response::builder()
                    .status(StatusCode::OK)
                    .body(axum::body::Body::from(r#"{"status":"ok","counter":1}"#))
                    .unwrap();
                Ok::<_, std::convert::Infallible>(resp)
            });

        // First request
        let req1 = Request::builder()
            .header("Idempotency-Key", "key-002")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp1 = svc.ready().await.unwrap().call(req1).await.unwrap();
        assert_eq!(resp1.status(), StatusCode::OK);

        // Duplicate request with same key
        let req2 = Request::builder()
            .header("Idempotency-Key", "key-002")
            .body(axum::body::Body::empty())
            .unwrap();
        let resp2 = svc.ready().await.unwrap().call(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        assert!(resp2.headers().contains_key("X-Idempotent-Replay"));
    }

    #[tokio::test]
    async fn test_no_key_passes_through() {
        let store = IdempotencyStore::new(std::time::Duration::from_secs(60));
        let mut svc = ServiceBuilder::new()
            .layer(IdempotencyLayer::new(store))
            .service_fn(|_req: Request<axum::body::Body>| async {
                let resp = Response::builder()
                    .status(StatusCode::CREATED)
                    .body(axum::body::Body::empty())
                    .unwrap();
                Ok::<_, std::convert::Infallible>(resp)
            });

        let req = Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        assert!(!resp.headers().contains_key("X-Idempotent-Replay"));
    }
}
