//! # P04 — Duplicate Request (Idempotency)
//!
//! **Scenario:** A single account sends the *same* purchase request 10 times
//! with the same idempotency key.
//!
//! **Expected behaviour:**
//! - The first request succeeds and issues a voucher.
//! - Subsequent requests return `Duplicate` with the cached result.
//! - Only 1 voucher is issued; stock drops by exactly 1.
//!
//! **Concepts tested:**
//! - Idempotency key tracking
//! - Cached result replay
//! - Preventing double-spending

mod common;

use common::*;
use std::sync::Arc;

#[tokio::test]
async fn test_duplicate_requests_return_cached_result() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-1", 10));

    // All 10 requests share the same idempotency key.
    let shared_key = "idem-key-abc123".to_string();
    let request = MockPurchaseRequest {
        product_id: "prod-1".to_string(),
        account_id: "account-42".to_string(),
        idempotency_key: shared_key.clone(),
    };

    // -- Act: send the same request 10 times ----------------------------
    let mut results = Vec::with_capacity(10);
    for _ in 0..10 {
        results.push(service.purchase(&request).await);
    }

    // -- Assert ---------------------------------------------------------

    // First result is a genuine success.
    match &results[0] {
        MockPurchaseResult::Success { voucher_id, remaining_stock } => {
            assert!(!voucher_id.is_empty());
            assert_eq!(*remaining_stock, 9);
        }
        other => panic!("first request should be Success, got: {other}"),
    }

    // All subsequent results are Duplicates wrapping the original success.
    for (i, result) in results.iter().enumerate().skip(1) {
        match result {
            MockPurchaseResult::Duplicate { cached } => {
                match cached.as_ref() {
                    MockPurchaseResult::Success { remaining_stock, .. } => {
                        assert_eq!(*remaining_stock, 9, "cached stock should be 9");
                    }
                    other => panic!(
                        "duplicate #{i} should wrap Success, got: {other}"
                    ),
                }
            }
            other => panic!("request #{i} should be Duplicate, got: {other}"),
        }
    }

    // Only 1 voucher and 1 order in the system.
    assert_eq!(redis.get_stock("prod-1"), 9);
    assert_eq!(db.count_orders("prod-1"), 1);
    assert_eq!(db.count_vouchers("prod-1"), 1);
}
