//! # Stock Query Route Handler
//!
//! GET /stock/{product_id} -- returns current stock, voucher count, and sale status.
//!
//! ## Exercise
//!
//! 1. Extract the `product_id` path parameter.
//! 2. Call the stock service to get current stock info.
//! 3. Return a `StockResponse` as JSON.
//! 4. Write tests for found, not found, and inactive sale states.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use crate::models::stock::StockInfo;
use crate::resilience::fallback;
use crate::AppState;

/// GET /stock/{product_id} handler.
#[cfg(feature = "solution")]
pub async fn handle_stock_query(
    State(state): State<AppState>,
    Path(product_id): Path<String>,
) -> Result<Json<StockInfo>, StatusCode> {
    tracing::info!(product_id = %product_id, "Stock query");

    if !state.circuit_breaker.can_execute("redis") {
        tracing::warn!("Circuit breaker open, serving fallback for stock query");
        return Ok(Json(fallback::stock_fallback(&product_id)));
    }

    match state.stock_service.get_stock_info(&product_id).await {
        Ok(info) => {
            state.circuit_breaker.record_success("redis");
            Ok(Json(info))
        }
        Err(e) => {
            state.circuit_breaker.record_failure("redis");
            tracing::error!(error = %e, product_id = %product_id, "Stock query failed");
            // Serve a safe fallback instead of an error.
            Ok(Json(fallback::stock_fallback(&product_id)))
        }
    }
}

#[cfg(not(feature = "solution"))]
pub async fn handle_stock_query(
    State(_state): State<AppState>,
    Path(_product_id): Path<String>,
) -> Result<Json<StockInfo>, StatusCode> {
    todo!("Implement the stock query handler: fetch from stock service, fallback on error")
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_stock_query_not_found() {
        let app = crate::tests::test_app().await;
        let Some(app) = app else {
            eprintln!("SKIP: Redis not available");
            return;
        };

        let req = Request::builder()
            .uri("/stock/nonexistent-product")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        // Should return 200 with stock_remaining=0 (graceful handling).
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
    }
}
