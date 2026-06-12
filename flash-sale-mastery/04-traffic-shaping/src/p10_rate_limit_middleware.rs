//! # Exercise 10: Rate Limit Middleware (Axum/Tower)
//!
//! ## Learning Objective
//! Build a proper Tower middleware that integrates rate limiting into an Axum
//! HTTP server. Learn the `Layer` and `Service` patterns used throughout the
//! Axum ecosystem, and how to return proper HTTP 429 responses with
//! `Retry-After` headers.
//!
//! ## Flash Sale Context
//! The rate limit middleware sits at the edge of the API, inspecting every
//! incoming request before it reaches the handler. It extracts the client IP,
//! checks against the rate limiter, and either forwards the request or
//! returns a 429 Too Many Requests response with a `Retry-After` header.
//!
//! ## Instructions
//! 1. Implement `RateLimitLayer` that implements `tower::Layer`
//! 2. Implement `RateLimitService` that implements `tower::Service`
//! 3. The service should:
//!    - Extract client IP from the request
//!    - Check the rate limiter
//!    - Forward the request if allowed
//!    - Return 429 with `Retry-After` header if rate limited
//!
//! ## Hints
//! - `tower::Layer` requires implementing `layer<S>(inner: S) -> RateLimitService<S>`
//! - `tower::Service<Request>` requires `poll_ready`, `call`, and associated types
//! - Use `axum::extract::ConnectInfo` or `X-Forwarded-For` for IP extraction
//! - The `Retry-After` header value should be in seconds

use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use axum::http::header;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Configuration for the rate limit middleware.
#[derive(Debug, Clone)]
pub struct RateLimitMiddlewareConfig {
    /// Maximum requests per window per IP.
    pub max_requests: u64,
    /// Window duration in milliseconds.
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

/// Tower Layer for rate limiting.
///
/// This is the outer wrapper that gets cloned for each connection.
/// It holds the shared rate limiter configuration.
#[derive(Clone)]
pub struct RateLimitLayer {
    // TODO: Add shared rate limiter state (Arc<Mutex<...>> or DashMap)
}

impl RateLimitLayer {
    /// Create a new rate limit layer with the given configuration.
    pub fn new(config: RateLimitMiddlewareConfig) -> Self {
        todo!("Implement rate limit layer creation")
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        todo!("Wrap the inner service with rate limiting")
    }
}

/// Tower Service for rate limiting.
///
/// This wraps the inner service and intercepts requests to check rate limits
/// before forwarding them.
pub struct RateLimitService<S> {
    // TODO: Add inner service and shared rate limiter state
    _phantom: std::marker::PhantomData<S>,
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
        todo!("Delegate poll_ready to inner service")
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        todo!("Implement rate limit check and request forwarding")
    }
}

/// Extract the client IP address from a request.
///
/// Checks `X-Forwarded-For` header first, then falls back to "unknown".
fn extract_client_ip(req: &Request<Body>) -> String {
    todo!("Extract client IP from request headers")
}

/// Build a 429 Too Many Requests response with Retry-After header.
fn rate_limit_response(retry_after_secs: u32) -> Response<Body> {
    todo!("Build 429 response with Retry-After header")
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
        assert!(
            resp.headers().contains_key(header::RETRY_AFTER),
            "429 response should include Retry-After header"
        );
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
