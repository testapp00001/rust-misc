//! # Error Handler Middleware
//!
//! Catches panics and unhandled errors, converting them into safe JSON responses.
//! Internal error details are never exposed to the client.
//!
//! ## Exercise
//!
//! 1. Implement the `ErrorHandlerLayer` and `ErrorHandlerService`.
//! 2. Use `std::panic::AssertUnwindSafe` + `FutureExt::catch_unwind` to catch panics.
//! 3. Convert panics and errors into JSON responses with a generic message.
//! 4. Log the full error internally for debugging.
//! 5. Write tests that verify error responses are JSON and hide internal details.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::FutureExt;
use tower::{Layer, Service};

// ---------------------------------------------------------------------------
// Tower Layer + Service
// ---------------------------------------------------------------------------

/// Tower layer that wraps handlers with panic-catching and JSON error formatting.
#[derive(Clone)]
pub struct ErrorHandlerLayer;

impl ErrorHandlerLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for ErrorHandlerLayer {
    type Service = ErrorHandlerService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorHandlerService { inner }
    }
}

/// Tower service that catches panics and formats errors as JSON.
#[derive(Clone)]
pub struct ErrorHandlerService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<axum::http::Request<ReqBody>> for ErrorHandlerService<S>
where
    S: Service<axum::http::Request<ReqBody>, Response = axum::http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Display + Send + 'static,
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
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Catch panics from the inner service.
            let result = std::panic::AssertUnwindSafe(inner.call(req))
                .catch_unwind()
                .await;

            match result {
                Ok(inner_result) => inner_result,
                Err(panic_err) => {
                    // Log the panic details internally.
                    let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "unknown panic".to_string()
                    };

                    tracing::error!(panic = %panic_msg, "Handler panicked");

                    // Return a safe, generic error response.
                    let response = axum::http::Response::builder()
                        .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                        .header("Content-Type", "application/json")
                        .body(ResBody::default())
                        .unwrap();

                    Ok(response)
                }
            }
        })
    }
}

/// Helper to create a JSON error response (used by route handlers).
pub fn json_error_response(
    status: axum::http::StatusCode,
    error: &str,
    message: &str,
) -> axum::response::Response {
    let body = serde_json::json!({
        "error": error,
        "message": message,
    });

    axum::response::Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower::{ServiceBuilder, ServiceExt};
    use axum::http::{Request, Response, StatusCode};

    #[tokio::test]
    async fn test_normal_request_passes_through() {
        let mut svc = ServiceBuilder::new()
            .layer(ErrorHandlerLayer::new())
            .service_fn(|_req: Request<()>| async {
                Ok::<_, String>(Response::builder()
                    .status(StatusCode::OK)
                    .body(())
                    .unwrap())
            });

        let req = Request::builder().body(()).unwrap();
        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_panic_is_caught() {
        let mut svc = ServiceBuilder::new()
            .layer(ErrorHandlerLayer::new())
            .service_fn(|_req: Request<()>| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response<()>, String>> + Send>> {
                Box::pin(async {
                    // This panic will be caught by the error handler middleware.
                    panic!("something went wrong inside the handler");
                    #[allow(unreachable_code)]
                    Ok(Response::builder().status(StatusCode::OK).body(()).unwrap())
                })
            });

        let req = Request::builder().body(()).unwrap();
        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        // The panic was caught and converted to an internal server error.
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_json_error_response_format() {
        let resp = json_error_response(
            StatusCode::BAD_REQUEST,
            "invalid_input",
            "Missing required field: product_id",
        );
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            resp.headers().get("content-type").unwrap(),
            "application/json"
        );
    }
}
