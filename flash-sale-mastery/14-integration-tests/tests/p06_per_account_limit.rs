//! # P06 — Per-Account Claim Limit
//!
//! **Scenario:** A single account tries to buy the same product 5 times,
//! each time with a *different* idempotency key.
//!
//! **Expected behaviour:**
//! - The first request succeeds.
//! - Requests 2–5 return `AlreadyClaimed`.
//! - Stock drops by exactly 1.
//!
//! **Concepts tested:**
//! - Per-account claim tracking
//! - Preventing the same user from hogging inventory
//! - Interaction between idempotency and claim limits

mod common;

use common::*;
use std::sync::Arc;

#[tokio::test]
async fn test_account_can_only_claim_once() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-1", 100));

    let account = "greedy-account";

    // -- Act: 5 attempts with different idempotency keys ----------------
    let mut results = Vec::with_capacity(5);
    for i in 0..5 {
        let request = MockPurchaseRequest {
            product_id: "prod-1".to_string(),
            account_id: account.to_string(),
            idempotency_key: make_idempotency_key("p06", &format!("{account}-{i}")),
        };
        results.push(service.purchase(&request).await);
    }

    // -- Assert ---------------------------------------------------------

    // First attempt succeeds.
    assert!(
        matches!(&results[0], MockPurchaseResult::Success { .. }),
        "first attempt should succeed, got: {}",
        results[0]
    );

    // Attempts 2-5 are rejected with AlreadyClaimed.
    for (i, result) in results.iter().enumerate().skip(1) {
        assert!(
            matches!(result, MockPurchaseResult::AlreadyClaimed),
            "attempt #{i} should be AlreadyClaimed, got: {result}"
        );
    }

    // Only 1 unit of stock consumed.
    assert_eq!(redis.get_stock("prod-1"), 99);
    assert_eq!(db.count_orders("prod-1"), 1);
}

#[tokio::test]
async fn test_different_accounts_are_independent() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-1", 100));

    // Two different accounts each make one purchase.
    let req_a = make_request("p06-indep", "prod-1", "account-A");
    let req_b = make_request("p06-indep", "prod-1", "account-B");

    // -- Act ------------------------------------------------------------
    let result_a = service.purchase(&req_a).await;
    let result_b = service.purchase(&req_b).await;

    // -- Assert ---------------------------------------------------------
    assert!(matches!(result_a, MockPurchaseResult::Success { .. }));
    assert!(matches!(result_b, MockPurchaseResult::Success { .. }));
    assert_eq!(redis.get_stock("prod-1"), 98);
}
