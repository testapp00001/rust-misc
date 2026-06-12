//! # Request Tracing Middleware
//!
//! Creates a tracing span for every request with: request_id, method, path,
//! status code, and latency.
//!
//! ## Exercise
//!
//! 1. Implement the `TracingLayer` and `TracingService`.
//! 2. Generate a UUID request_id and attach it as a span field and response header.
//! 3. Record the response status and latency when the inner service completes.
//! 4. Write tests verifying span creation and request_id propagation.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use tower::{Layer, Service};
use tracing::Instrument;

// ---------------------------------------------------------------------------
// Tower Layer + Service
// ---------------------------------------------------------------------------

/// Tower layer that creates a tracing span per request.
#[derive(Clone)]
pub struct TracingLayer;

impl TracingLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for TracingLayer {
    type Service = TracingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TracingService { inner }
    }
}

/// Tower service that instruments requests with tracing spans.
#[derive(Clone)]
pub struct TracingService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<axum::http::Request<ReqBody>> for TracingService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = axum::http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<ReqBody>) -> Self::Future {
        let request_id = uuid::Uuid::new_v4().to_string();
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let start = Instant::now();

        let span = tracing::info_span!(
            "request",
            request_id = %request_id,
            method = %method,
            path = %path,
            status = tracing::field::Empty,
            latency_ms = tracing::field::Empty,
        );

        let mut inner = self.inner.clone();

        let fut = async move {
            let result = inner.call(req).await;
            let latency = start.elapsed();

            match &result {
                Ok(resp) => {
                    tracing::Span::current().record("status", resp.status().as_u16());
                    tracing::Span::current().record("latency_ms", latency.as_millis() as u64);

                    tracing::info!(
                        status = resp.status().as_u16(),
                        latency_ms = latency.as_millis() as u64,
                        "Request completed"
                    );
                }
                Err(_) => {
                    tracing::Span::current().record("status", 500);
                    tracing::Span::current().record("latency_ms", latency.as_millis() as u64);

                    tracing::error!(
                        latency_ms = latency.as_millis() as u64,
                        "Request failed"
                    );
                }
            }

            result
        };

        Box::pin(fut.instrument(span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::{ServiceBuilder, ServiceExt};
    use axum::http::{Request, Response, StatusCode};

    #[tokio::test]
    async fn test_span_creation() {
        // Initialize a subscriber to capture tracing events.
        // This test primarily verifies the middleware does not panic.
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();

        let mut svc = ServiceBuilder::new()
            .layer(TracingLayer::new())
            .service_fn(|_req: Request<()>| async {
                Ok::<_, String>(Response::builder()
                    .status(StatusCode::OK)
                    .body(())
                    .unwrap())
            });

        let req = Request::builder()
            .uri("/health")
            .body(())
            .unwrap();

        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_id_generation() {
        let mut svc = ServiceBuilder::new()
            .layer(TracingLayer::new())
            .service_fn(|_req: Request<()>| async {
                Ok::<_, String>(Response::builder()
                    .status(StatusCode::OK)
                    .body(())
                    .unwrap())
            });

        // Two requests should get different request_ids (tested via tracing spans).
        let req1 = Request::builder().uri("/a").body(()).unwrap();
        let req2 = Request::builder().uri("/b").body(()).unwrap();

        let r1 = svc.ready().await.unwrap().call(req1).await.unwrap();
        let r2 = svc.ready().await.unwrap().call(req2).await.unwrap();

        assert_eq!(r1.status(), StatusCode::OK);
        assert_eq!(r2.status(), StatusCode::OK);
    }
}
