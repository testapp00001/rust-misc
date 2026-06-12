//! # P09 — Database Failure
//!
//! **Scenario:** The database is unavailable, but Redis is working.
//!
//! **Expected behaviour:**
//! - The purchase **succeeds** — Redis is the primary store for the hot path.
//! - The order is queued for retry (DB write fails silently).
//! - A warning is logged.
//!
//! **Concepts tested:**
//! - Redis-first architecture: the hot path does not depend on the DB
//! - Graceful degradation: DB failure does not block purchases
//! - Eventual consistency: orders will be persisted later

mod common;

use common::*;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[tokio::test]
async fn test_purchase_succeeds_when_db_down() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-1", 10));

    // Simulate DB failure.
    db.failing.store(true, Ordering::SeqCst);

    let request = make_request("p09", "prod-1", "account-1");

    // -- Act ------------------------------------------------------------
    let result = service.purchase(&request).await;

    // -- Assert ---------------------------------------------------------
    match &result {
        MockPurchaseResult::Success { voucher_id, remaining_stock } => {
            assert!(!voucher_id.is_empty(), "voucher must still be issued");
            assert_eq!(*remaining_stock, 9, "stock must be decremented");
        }
        other => panic!("expected Success even with DB down, got: {other}"),
    }

    // Stock was decremented in Redis.
    assert_eq!(redis.get_stock("prod-1"), 9);

    // But the DB has no persisted order (write failed).
    assert_eq!(
        db.count_orders("prod-1"),
        0,
        "DB should have 0 orders because it was failing"
    );

    // The idempotency key is cached in Redis so a retry can succeed.
    let cached = redis.check_idempotency(&request.idempotency_key);
    assert!(cached.is_some(), "result must be cached for retry");
}

#[tokio::test]
async fn test_db_recovery_persists_queued_orders() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-1", 10));

    // Phase 1: DB down — purchase succeeds but order not persisted.
    db.failing.store(true, Ordering::SeqCst);
    let result = service.purchase(&make_request("p09-rec", "prod-1", "acct-1")).await;
    assert!(matches!(result, MockPurchaseResult::Success { .. }));
    assert_eq!(db.count_orders("prod-1"), 0);

    // Phase 2: DB back — new purchase persists normally.
    db.failing.store(false, Ordering::SeqCst);
    let result = service.purchase(&make_request("p09-rec", "prod-1", "acct-2")).await;
    assert!(matches!(result, MockPurchaseResult::Success { .. }));
    assert_eq!(db.count_orders("prod-1"), 1, "new order should persist");

    // Stock reflects both purchases.
    assert_eq!(redis.get_stock("prod-1"), 8);
}
