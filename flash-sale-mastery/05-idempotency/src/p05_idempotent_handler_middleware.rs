//! # Exercise 05: Idempotent Handler Middleware
//!
//! ## Learning Objective
//! Build an Axum middleware layer that transparently adds idempotency to any
//! handler. The middleware extracts an idempotency key from the request header,
//! checks the store, and either returns a cached response or lets the handler
//! process the request and caches the result.
//!
//! ## Flash Sale Context
//! Instead of adding idempotency checks to every route handler, a middleware
//! layer handles it uniformly. The frontend sends an `Idempotency-Key` header
//! with every POST request. The middleware:
//! 1. Extracts the key from the header
//! 2. Checks if this key was seen before
//! 3. If yes, returns the cached response (no handler called)
//! 4. If no, lets the handler process, caches the response, returns it
//!
//! ## Instructions
//! 1. Implement `IdempotencyLayer` that wraps an `IdempotencyStore`
//! 2. Implement `IdempotencyService` as a Tower Service
//! 3. Extract the `Idempotency-Key` header from requests
//! 4. For GET requests, pass through without idempotency checks
//! 5. For POST/PUT/PATCH, check the store and either return cached or process
//!
//! ## Hints
//! - Implement `tower::Layer<Inner>` for `IdempotencyLayer`
//! - Implement `tower::Service<Request<ReqBody>>` for `IdempotencyService`
//! - Use `request.headers().get("idempotency-key")` to extract the key
//! - Clone the inner service before calling it (Tower services are consumed by `call`)

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
///
/// This trait abstracts over Redis, database, or combined stores so the
/// middleware can work with any backend.
#[async_trait::async_trait]
pub trait IdempotencyStore: Send + Sync + 'static {
    /// Check if a key exists and mark it as seen if not.
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
    // TODO: Define the Service associated type
    // It should be IdempotencyService<Store, Inner>
    type Service = IdempotencyService<Store, Inner>;

    fn layer(&self, inner: Inner) -> Self::Service {
        // TODO: Create an IdempotencyService wrapping the inner service with the store
        todo!("Implement Layer::layer")
    }
}

/// Tower Service that checks idempotency before forwarding to the inner service.
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
        // TODO: Delegate poll_ready to the inner service
        todo!("Implement poll_ready")
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        // TODO: Check if the request method is GET/HEAD/OPTIONS -> pass through without idempotency
        // TODO: Extract the "Idempotency-Key" header
        // TODO: If no key header on a POST/PUT/PATCH -> return 400 Bad Request
        // TODO: Check the store for the key
        //   - If found -> return the cached response
        //   - If not found -> forward to inner service, cache the response, return it
        todo!("Implement idempotent request handling")
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

    // TODO: Implement IdempotencyStore for InMemoryStore
    // Use the data HashMap to store and retrieve cached responses
    // For check(): return Some if key exists, None if not
    // For store(): insert the response into the HashMap

    /// Test that a GET request passes through without idempotency check.
    #[tokio::test]
    async fn test_get_request_passthrough() {
        // TODO: Create an InMemoryStore
        // TODO: Create a simple handler that returns 200
        // TODO: Build a service with the IdempotencyLayer
        // TODO: Send a GET request (no idempotency key needed)
        // TODO: Assert the response is 200 and the store was NOT checked
        todo!("Implement test: GET passthrough")
    }

    /// Test that a POST request with idempotency key is processed on first call.
    #[tokio::test]
    async fn test_post_first_request() {
        // TODO: Create a store and service
        // TODO: Send a POST with Idempotency-Key header
        // TODO: Assert the handler was called and response is cached
        todo!("Implement test: first POST request")
    }

    /// Test that a duplicate POST returns the cached response.
    #[tokio::test]
    async fn test_post_duplicate_returns_cached() {
        // TODO: Send the same POST twice (same Idempotency-Key)
        // TODO: Assert both responses are identical
        // TODO: Assert the handler was called only once
        todo!("Implement test: duplicate POST returns cached")
    }

    /// Test that different idempotency keys are processed independently.
    #[tokio::test]
    async fn test_different_keys_processed_independently() {
        // TODO: Send two POSTs with different Idempotency-Key values
        // TODO: Assert both are processed (both hit the handler)
        todo!("Implement test: different keys independent")
    }

    /// Test that POST without Idempotency-Key header returns 400.
    #[tokio::test]
    async fn test_post_without_key_returns_400() {
        // TODO: Send a POST without the Idempotency-Key header
        // TODO: Assert the response is 400 Bad Request
        todo!("Implement test: missing key returns 400")
    }
}
