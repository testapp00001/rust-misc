//! # P11 — Reconciliation Test
//!
//! **Scenario:** After a flash sale, Redis and the database may have
//! drifted (e.g., due to DB write failures during the sale). The
//! reconciliation job compares the two and corrects mismatches.
//!
//! **Expected behaviour:**
//! - Mismatches between Redis stock and DB order count are detected.
//! - A reconciliation report is generated.
//! - Corrections are applied.
//!
//! **Concepts tested:**
//! - Post-sale consistency checks
//! - Detecting state drift between Redis and DB
//! - Generating actionable reconciliation reports

mod common;

use common::*;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[tokio::test]
async fn test_reconciliation_detects_consistent_state() {
    // -- Setup: a clean sale with no DB failures ------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-1", 10));

    // 3 successful purchases — both Redis and DB agree.
    for i in 0..3 {
        let req = make_request("p11-clean", "prod-1", &format!("acct-{i}"));
        service.purchase(&req).await;
    }

    let mut initial_stocks = HashMap::new();
    initial_stocks.insert("prod-1".to_string(), 10u32);

    // -- Act ------------------------------------------------------------
    let result = service.reconcile(&initial_stocks);

    // -- Assert ---------------------------------------------------------
    assert!(
        result.mismatches.is_empty(),
        "no mismatches expected in clean state, found {}",
        result.mismatches.len()
    );
    assert_eq!(result.corrections, 0);
}

#[tokio::test]
async fn test_reconciliation_detects_db_drift() {
    // -- Setup: simulate DB failure during some purchases ---------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    let initial_stock: u32 = 10;
    service.add_product(make_product("prod-1", initial_stock));

    // Phase 1: DB is up — 2 purchases succeed and persist.
    for i in 0..2 {
        let req = make_request("p11-drift", "prod-1", &format!("acct-{i}"));
        service.purchase(&req).await;
    }
    assert_eq!(db.count_orders("prod-1"), 2);

    // Phase 2: DB goes down — 3 more purchases succeed in Redis but not DB.
    db.failing.store(true, Ordering::SeqCst);
    for i in 2..5 {
        let req = make_request("p11-drift", "prod-1", &format!("acct-{i}"));
        service.purchase(&req).await;
    }
    db.failing.store(false, Ordering::SeqCst);

    // Redis stock: 10 - 5 = 5.  DB orders: 2.  Drift = 3.
    assert_eq!(redis.get_stock("prod-1"), 5);
    assert_eq!(db.count_orders("prod-1"), 2);

    let mut initial_stocks = HashMap::new();
    initial_stocks.insert("prod-1".to_string(), initial_stock);

    // -- Act: reconcile -------------------------------------------------
    let result = service.reconcile(&initial_stocks);

    // -- Assert ---------------------------------------------------------
    // Expected remaining from DB perspective: 10 - 2 = 8.
    // Actual Redis stock: 5.
    // Mismatch detected.
    assert_eq!(
        result.mismatches.len(),
        1,
        "should detect 1 mismatch"
    );

    let mismatch = &result.mismatches[0];
    assert_eq!(mismatch.product_id, "prod-1");
    assert_eq!(mismatch.redis_stock, 5);
    assert_eq!(mismatch.db_orders, 2);
    assert_eq!(mismatch.initial_stock, 10);

    assert_eq!(result.corrections, 1);
}

#[tokio::test]
async fn test_reconciliation_across_multiple_products() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-a", 10));
    service.add_product(make_product("prod-b", 20));
    service.add_product(make_product("prod-c", 5));

    // prod-a: all purchases persisted (no drift).
    for i in 0..3 {
        let req = make_request("p11-multi", "prod-a", &format!("acct-{i}"));
        service.purchase(&req).await;
    }

    // prod-b: DB fails mid-sale (drift expected).
    for i in 0..2 {
        let req = make_request("p11-multi", "prod-b", &format!("acct-{i}"));
        service.purchase(&req).await;
    }
    db.failing.store(true, Ordering::SeqCst);
    for i in 2..4 {
        let req = make_request("p11-multi", "prod-b", &format!("acct-{i}"));
        service.purchase(&req).await;
    }
    db.failing.store(false, Ordering::SeqCst);

    // prod-c: no purchases (no drift).
    // (deliberately left untouched)

    let mut initial_stocks = HashMap::new();
    initial_stocks.insert("prod-a".to_string(), 10);
    initial_stocks.insert("prod-b".to_string(), 20);
    initial_stocks.insert("prod-c".to_string(), 5);

    // -- Act ------------------------------------------------------------
    let result = service.reconcile(&initial_stocks);

    // -- Assert ---------------------------------------------------------
    // Only prod-b should have a mismatch.
    assert_eq!(result.mismatches.len(), 1, "only prod-b should drift");
    assert_eq!(result.mismatches[0].product_id, "prod-b");
    assert_eq!(result.corrections, 1);

    // Verify prod-a and prod-c are clean.
    assert!(
        result
            .mismatches
            .iter()
            .all(|m| m.product_id != "prod-a" && m.product_id != "prod-c"),
        "prod-a and prod-c should have no mismatches"
    );
}
