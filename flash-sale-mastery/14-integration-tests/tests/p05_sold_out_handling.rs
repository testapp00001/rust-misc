//! # P05 — Sold Out Handling
//!
//! **Scenario:** A product has stock = 0 (already exhausted). A purchase
//! request arrives.
//!
//! **Expected behaviour:**
//! - Returns `SoldOut` immediately — no contention, no wasted work.
//! - This is a fast-path check: the system must not attempt a stock decrement
//!   or voucher issuance.
//!
//! **Concepts tested:**
//! - Fast-path sold-out detection
//! - Early return without unnecessary work

mod common;

use common::*;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
async fn test_sold_out_returns_immediately() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    // Stock is 0 — the product is already exhausted.
    service.add_product(make_product("prod-1", 0));

    let request = make_request("p05", "prod-1", "account-1");

    // -- Act ------------------------------------------------------------
    let start = Instant::now();
    let result = service.purchase(&request).await;
    let elapsed = start.elapsed();

    // -- Assert ---------------------------------------------------------
    assert!(
        matches!(result, MockPurchaseResult::SoldOut),
        "expected SoldOut, got: {result}"
    );

    // The fast path should be very fast (well under 100ms in mock).
    assert!(
        elapsed.as_millis() < 100,
        "sold-out path took {}ms — expected <1ms in real Redis, <100ms in mock",
        elapsed.as_millis()
    );

    // No vouchers or orders created.
    assert_eq!(db.count_orders("prod-1"), 0);
    assert_eq!(db.count_vouchers("prod-1"), 0);

    // Stock unchanged.
    assert_eq!(redis.get_stock("prod-1"), 0);
}

#[tokio::test]
async fn test_sold_out_caches_for_idempotency() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-1", 0));

    let request = make_request("p05", "prod-1", "account-1");

    // -- Act: send twice with the same idempotency key ------------------
    let first = service.purchase(&request).await;
    let second = service.purchase(&request).await;

    // -- Assert ---------------------------------------------------------
    assert!(matches!(first, MockPurchaseResult::SoldOut));

    // The second call returns a Duplicate wrapping SoldOut.
    match second {
        MockPurchaseResult::Duplicate { cached } => {
            assert!(
                matches!(cached.as_ref(), MockPurchaseResult::SoldOut),
                "cached result should be SoldOut"
            );
        }
        other => panic!("expected Duplicate, got: {other}"),
    }
}
