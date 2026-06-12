//! # Purchase Route Handler
//!
//! POST /purchase -- attempts a flash sale purchase for a product/account pair.
//!
//! ## Exercise
//!
//! 1. Parse the JSON body into a `PurchaseRequest`.
//! 2. Validate required fields (product_id, account_id, idempotency_key).
//! 3. Call the stock service's `check_and_decrement_stock`.
//! 4. On success, generate a voucher and enqueue an order.
//! 5. Map the `PurchaseResult` into a `PurchaseResponse`.
//! 6. Write tests for success, sold out, duplicate, and missing-field cases.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::middleware::error_handler;
use crate::models::purchase::{PurchaseRequest, PurchaseResponse, PurchaseStatus};
use crate::services::order_service::PendingOrder;
use crate::AppState;

/// POST /purchase handler.
#[cfg(feature = "solution")]
pub async fn handle_purchase(
    State(state): State<AppState>,
    Json(body): Json<PurchaseRequest>,
) -> Result<(StatusCode, Json<PurchaseResponse>), axum::response::Response> {
    // Validate required fields.
    if body.product_id.is_empty() || body.account_id.is_empty() || body.idempotency_key.is_empty() {
        return Err(error_handler::json_error_response(
            StatusCode::BAD_REQUEST,
            "invalid_request",
            "Missing required fields: product_id, account_id, idempotency_key",
        ));
    }

    tracing::info!(
        product_id = %body.product_id,
        account_id = %body.account_id,
        idempotency_key = %body.idempotency_key,
        "Purchase attempt"
    );

    // Check circuit breaker before hitting Redis.
    if !state.circuit_breaker.can_execute("redis") {
        tracing::warn!("Circuit breaker open, rejecting purchase");
        let response = PurchaseResponse {
            status: PurchaseStatus::SoldOut,
            voucher_code: None,
            message: "Service temporarily unavailable. Please try again later.".to_string(),
        };
        return Ok((StatusCode::SERVICE_UNAVAILABLE, Json(response)));
    }

    // Call the stock service (atomic Lua script).
    let result = state
        .stock_service
        .check_and_decrement_stock(&body.product_id, &body.account_id, &body.idempotency_key)
        .await;

    let purchase_result = match result {
        Ok(r) => {
            state.circuit_breaker.record_success("redis");
            r
        }
        Err(e) => {
            state.circuit_breaker.record_failure("redis");
            tracing::error!(error = %e, "Stock service error");
            return Err(error_handler::json_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "service_error",
                "An error occurred processing your request",
            ));
        }
    };

    // Generate voucher and queue order on success.
    let voucher = match &purchase_result {
        crate::models::purchase::PurchaseResult::Success { product_id, account_id } => {
            let v = state.voucher_service.generate_voucher(product_id, account_id);
            let _ = state.voucher_service.store_voucher(&v).await;

            // Fire-and-forget notification
            let notif_svc = state.notification_service.clone();
            let acc = account_id.clone();
            let code = v.code.clone();
            tokio::spawn(async move {
                let _ = notif_svc.send_purchase_notification(&acc, &code).await;
            });

            // Queue order for async processing
            let order = PendingOrder {
                order_id: uuid::Uuid::new_v4().to_string(),
                product_id: product_id.clone(),
                account_id: account_id.clone(),
                voucher_code: v.code.clone(),
                created_at: chrono::Utc::now(),
            };
            let _ = state.order_queue.queue_order(order).await;

            Some(v)
        }
        crate::models::purchase::PurchaseResult::SoldOut => {
            // Notify ops team
            let notif_svc = state.notification_service.clone();
            let pid = body.product_id.clone();
            tokio::spawn(async move {
                let _ = notif_svc.send_sold_out_notification(&pid).await;
            });
            None
        }
        _ => None,
    };

    let response = purchase_result.into_response(voucher);
    let status = match response.status {
        PurchaseStatus::Success => StatusCode::OK,
        PurchaseStatus::SoldOut => StatusCode::OK,
        PurchaseStatus::AlreadyClaimed => StatusCode::CONFLICT,
        PurchaseStatus::VoucherLimitReached => StatusCode::OK,
        PurchaseStatus::IdempotentReplay => StatusCode::OK,
        PurchaseStatus::RateLimited => StatusCode::TOO_MANY_REQUESTS,
    };

    Ok((status, Json(response)))
}

#[cfg(not(feature = "solution"))]
pub async fn handle_purchase(
    State(_state): State<AppState>,
    Json(_body): Json<PurchaseRequest>,
) -> Result<(StatusCode, Json<PurchaseResponse>), axum::response::Response> {
    todo!("Implement the purchase handler: validate, call stock service, generate voucher, queue order")
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_missing_fields_returns_400() {
        let app = crate::tests::test_app().await;
        let Some(app) = app else {
            eprintln!("SKIP: Redis not available");
            return;
        };

        let req = Request::builder()
            .method("POST")
            .uri("/purchase")
            .header("Content-Type", "application/json")
            .body(Body::from(r#"{"product_id":"","account_id":"","idempotency_key":""}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
