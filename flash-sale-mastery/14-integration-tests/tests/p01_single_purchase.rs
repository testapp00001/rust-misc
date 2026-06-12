//! # P01 — Single Purchase Flow
//!
//! **Scenario:** A single user buys one item from a product with stock = 10.
//!
//! **Expected behaviour:**
//! 1. The purchase succeeds and returns a voucher ID.
//! 2. Stock decrements from 10 to 9.
//! 3. An audit event is recorded.
//!
//! **Concepts tested:**
//! - Basic purchase hot path
//! - Stock decrement
//! - Voucher issuance
//! - Event recording

mod common;

use common::*;
use std::sync::Arc;

/// Verify that a single purchase succeeds, decrements stock, and issues a
/// voucher.
#[tokio::test]
async fn test_single_successful_purchase() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    let product = make_product("prod-1", 10);
    service.add_product(product);

    let request = make_request("p01", "prod-1", "account-42");

    // -- Act ------------------------------------------------------------
    let result = service.purchase(&request).await;

    // -- Assert ---------------------------------------------------------
    match &result {
        MockPurchaseResult::Success {
            voucher_id,
            remaining_stock,
        } => {
            assert!(!voucher_id.is_empty(), "voucher ID must not be empty");
            assert_eq!(*remaining_stock, 9, "stock should be 9 after one purchase");
        }
        other => panic!("expected Success, got: {other}"),
    }

    // Stock in Redis must also reflect the decrement.
    assert_eq!(redis.get_stock("prod-1"), 9);

    // At least one event should have been recorded.
    let events = redis.get_events();
    assert!(
        events.iter().any(|e| matches!(e.event_type, EventType::VoucherIssued)),
        "a VoucherIssued event must be present"
    );
    assert!(
        events.iter().any(|e| matches!(e.event_type, EventType::PurchaseCompleted)),
        "a PurchaseCompleted event must be present"
    );

    // The order should have been persisted in the DB.
    assert_eq!(db.count_orders("prod-1"), 1);
}
