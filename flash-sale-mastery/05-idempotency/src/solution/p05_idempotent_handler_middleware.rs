//! # Solution 05: Idempotent Handler Middleware
//!
//! Complete implementation of an Axum/Tower middleware for transparent idempotency.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use axum::body::Body;
use axum::http::{Method, Request, Response, StatusCode};
use tower::{Layer, Service};
use serde::{Deserialize, Serialize};

/// Cached response stored in the idempotency store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    pub status_code: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Trait for idempotency stores used by the middleware.
#[async_trait::async_trait]
pub trait IdempotencyStore: Send + Sync + 'static {
    /// Check if a key exists and return the cached response if so.
    async fn check(&self, key: &str) -> Result<Option<CachedResponse>, IdempotencyError>;

    /// Store a response for a key.
    async fn store(
        &self,
        key: &str,
        response: &CachedResponse,
    ) -> Result<(), IdempotencyError>;
}

/// Errors from the idempotency middleware.
#[derive(Debug, thiserror::Error)]
pub enum IdempotencyError {
    #[error("Store error: {0}")]
    Store(String),

    #[error("Missing idempotency key header")]
    MissingKey,

    #[error("Invalid idempotency key: {0}")]
    InvalidKey(String),
}

/// Tower Layer that adds idempotency checking to services.
///
/// Wraps an `IdempotencyStore` and applies it as middleware to any inner service.
#[derive(Clone)]
pub struct IdempotencyLayer<Store> {
    store: Store,
}

impl<Store> IdempotencyLayer<Store> {
    /// Create a new idempotency layer with the given store.
    pub fn new(store: Store) -> Self {
        Self { store }
    }
}

impl<Store: Clone, Inner> Layer<Inner> for IdempotencyLayer<Store> {
    type Service = IdempotencyService<Store, Inner>;

    fn layer(&self, inner: Inner) -> Self::Service {
        IdempotencyService {
            store: self.store.clone(),
            inner,
        }
    }
}

/// Tower Service that checks idempotency before forwarding to the inner service.
#[derive(Clone)]
pub struct IdempotencyService<Store, Inner> {
    store: Store,
    inner: Inner,
}

impl<Store, Inner, ReqBody> Service<Request<ReqBody>> for IdempotencyService<Store, Inner>
where
    Store: IdempotencyStore + Clone,
    Inner: Service<Request<ReqBody>, Response = Response<Body>> + Clone + Send + 'static,
    Inner::Future: Send + 'static,
    Inner::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    ReqBody: Send + 'static,
{
    type Response = Response<Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let method = request.method().clone();
        let store = self.store.clone();
        let mut inner = self.inner.clone();

        // Extract idempotency key from headers
        let idempotency_key = request
            .headers()
            .get("idempotency-key")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Box::pin(async move {
            // GET, HEAD, OPTIONS are naturally idempotent -- pass through
            if method == Method::GET || method == Method::HEAD || method == Method::OPTIONS {
                return inner.call(request).await.map_err(Into::into);
            }

            // For POST/PUT/PATCH, require an idempotency key
            let key = match idempotency_key {
                Some(k) if !k.is_empty() => k,
                _ => {
                    return Ok(Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(Body::from(r#"{"error":"Missing Idempotency-Key header"}"#))
                        .unwrap());
                }
            };

            // Check the store
            match store.check(&key).await {
                Ok(Some(cached)) => {
                    // Duplicate request -- return cached response
                    let mut builder = Response::builder()
                        .status(StatusCode::from_u16(cached.status_code)
                            .unwrap_or(StatusCode::OK));

                    for (name, value) in &cached.headers {
                        builder = builder.header(name.as_str(), value.as_str());
                    }

                    Ok(builder.body(Body::from(cached.body)).unwrap())
                }
                Ok(None) => {
                    // First time -- forward to the inner service
                    let response = inner.call(request).await.map_err(Into::into)?;

                    // Cache the response
                    let status = response.status().as_u16();
                    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
                        .await
                        .unwrap_or_default();

                    let cached = CachedResponse {
                        status_code: status,
                        headers: vec![],
                        body: body_bytes.to_vec(),
                    };
                    let _ = store.store(&key, &cached).await;

                    // Return the response
                    Ok(Response::builder()
                        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK))
                        .body(Body::from(body_bytes))
                        .unwrap())
                }
                Err(e) => {
                    // Store error -- fail closed
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from(format!(r#"{{"error":"{e}"}}"#)))
                        .unwrap())
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Method, StatusCode};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    /// A simple in-memory idempotency store for testing.
    #[derive(Clone)]
    struct InMemoryStore {
        data: Arc<Mutex<HashMap<String, CachedResponse>>>,
        check_count: Arc<Mutex<usize>>,
        store_count: Arc<Mutex<usize>>,
    }

    impl InMemoryStore {
        fn new() -> Self {
            Self {
                data: Arc::new(Mutex::new(HashMap::new())),
                check_count: Arc::new(Mutex::new(0)),
                store_count: Arc::new(Mutex::new(0)),
            }
        }

        fn get_check_count(&self) -> usize {
            *self.check_count.lock().unwrap()
        }

        fn get_store_count(&self) -> usize {
            *self.store_count.lock().unwrap()
        }
    }

    #[async_trait::async_trait]
    impl IdempotencyStore for InMemoryStore {
        async fn check(&self, key: &str) -> Result<Option<CachedResponse>, IdempotencyError> {
            *self.check_count.lock().unwrap() += 1;
            let data = self.data.lock().unwrap();
            Ok(data.get(key).cloned())
        }

        async fn store(
            &self,
            key: &str,
            response: &CachedResponse,
        ) -> Result<(), IdempotencyError> {
            *self.store_count.lock().unwrap() += 1;
            let mut data = self.data.lock().unwrap();
            data.insert(key.to_string(), response.clone());
            Ok(())
        }
    }

    /// Test that a GET request passes through without idempotency check.
    #[tokio::test]
    async fn test_get_request_passthrough() {
        let store = InMemoryStore::new();
        let layer = IdempotencyLayer::new(store.clone());

        // Create a simple inner service that returns 200
        let inner = tower::service_fn(|_req: Request<Body>| async {
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from("OK"))
                    .unwrap(),
            )
        });

        let mut service = layer.layer(inner);

        let request = Request::builder()
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = service.call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // The store should NOT have been checked for GET requests
        assert_eq!(store.get_check_count(), 0, "GET should not check the store");
    }

    /// Test that a POST request with idempotency key is processed on first call.
    #[tokio::test]
    async fn test_post_first_request() {
        let store = InMemoryStore::new();
        let layer = IdempotencyLayer::new(store.clone());

        let inner = tower::service_fn(|_req: Request<Body>| async {
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from(r#"{"status":"processed"}"#))
                    .unwrap(),
            )
        });

        let mut service = layer.layer(inner);

        let request = Request::builder()
            .method(Method::POST)
            .header("idempotency-key", "test-key-001")
            .body(Body::empty())
            .unwrap();

        let response = service.call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(store.get_store_count(), 1, "First POST should store the result");
    }

    /// Test that a duplicate POST returns the cached response.
    #[tokio::test]
    async fn test_post_duplicate_returns_cached() {
        let store = InMemoryStore::new();
        let layer = IdempotencyLayer::new(store.clone());

        let call_count = Arc::new(Mutex::new(0usize));
        let call_count_clone = call_count.clone();

        let inner = tower::service_fn(move |_req: Request<Body>| {
            let count = call_count_clone.clone();
            async move {
                *count.lock().unwrap() += 1;
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
                    Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from(r#"{"status":"processed"}"#))
                        .unwrap(),
                )
            }
        });

        let mut service = layer.layer(inner);

        // First request
        let req1 = Request::builder()
            .method(Method::POST)
            .header("idempotency-key", "dup-key")
            .body(Body::empty())
            .unwrap();
        let resp1 = service.call(req1).await.unwrap();
        assert_eq!(resp1.status(), StatusCode::OK);

        // Second request with same key
        let req2 = Request::builder()
            .method(Method::POST)
            .header("idempotency-key", "dup-key")
            .body(Body::empty())
            .unwrap();
        let resp2 = service.call(req2).await.unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);

        // The inner service should have been called only once
        assert_eq!(*call_count.lock().unwrap(), 1, "Handler should be called once");
    }

    /// Test that different idempotency keys are processed independently.
    #[tokio::test]
    async fn test_different_keys_processed_independently() {
        let store = InMemoryStore::new();
        let layer = IdempotencyLayer::new(store.clone());

        let call_count = Arc::new(Mutex::new(0usize));
        let call_count_clone = call_count.clone();

        let inner = tower::service_fn(move |_req: Request<Body>| {
            let count = call_count_clone.clone();
            async move {
                *count.lock().unwrap() += 1;
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
                    Response::builder()
                        .status(StatusCode::OK)
                        .body(Body::from("OK"))
                        .unwrap(),
                )
            }
        });

        let mut service = layer.layer(inner);

        for i in 0..3 {
            let req = Request::builder()
                .method(Method::POST)
                .header("idempotency-key", format!("key-{i}"))
                .body(Body::empty())
                .unwrap();
            service.call(req).await.unwrap();
        }

        assert_eq!(*call_count.lock().unwrap(), 3, "Each key should be processed independently");
    }

    /// Test that POST without Idempotency-Key header returns 400.
    #[tokio::test]
    async fn test_post_without_key_returns_400() {
        let store = InMemoryStore::new();
        let layer = IdempotencyLayer::new(store.clone());

        let inner = tower::service_fn(|_req: Request<Body>| async {
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from("OK"))
                    .unwrap(),
            )
        });

        let mut service = layer.layer(inner);

        let request = Request::builder()
            .method(Method::POST)
            .body(Body::empty())
            .unwrap();

        let response = service.call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
