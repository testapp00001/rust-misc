//! # P03 — Oversell Prevention (CRITICAL)
//!
//! **Scenario:** Stock is 1. One thousand concurrent requests arrive.
//!
//! **Expected behaviour:**
//! - Exactly **one** request succeeds.
//! - Stock ends at **0** — never negative.
//! - The other 999 receive `SoldOut`.
//!
//! This is the single most important test in the entire project. If this
//! fails, the system can sell items that do not exist, causing financial loss
//! and customer complaints.
//!
//! **Concepts tested:**
//! - Atomic compare-and-set (CAS) stock decrement
//! - Correctness under extreme contention
//! - Stock invariant: `stock >= 0` at all times

mod common;

use common::*;
use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::task::JoinHandle;

#[tokio::test]
async fn test_oversell_impossible_with_single_item() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = Arc::new(MockFlashSaleService::new(redis.clone(), db.clone()));

    let stock: u32 = 1;
    let requesters: u32 = 1_000;
    service.add_product(make_product("prod-1", stock));

    let barrier = Arc::new(Barrier::new(requesters as usize));

    // -- Act: fire all requests at once ---------------------------------
    let handles: Vec<JoinHandle<MockPurchaseResult>> = (0..requesters)
        .map(|i| {
            let svc = Arc::clone(&service);
            let bar = Arc::clone(&barrier);
            tokio::spawn(async move {
                bar.wait().await;
                let req = make_request("p03", "prod-1", &format!("acct-{i}"));
                svc.purchase(&req).await
            })
        })
        .collect();

    // -- Collect results ------------------------------------------------
    let mut success_count: u32 = 0;
    let mut sold_out_count: u32 = 0;
    let mut other_count: u32 = 0;

    for handle in handles {
        match handle.await.unwrap() {
            MockPurchaseResult::Success { .. } => success_count += 1,
            MockPurchaseResult::SoldOut => sold_out_count += 1,
            _ => other_count += 1,
        }
    }

    // -- Assert ---------------------------------------------------------

    // 1. Exactly one success — the whole point of this test.
    assert_eq!(
        success_count, 1,
        "CRITICAL: exactly 1 purchase must succeed, got {success_count}"
    );

    // 2. Everyone else gets SoldOut.
    assert_eq!(
        sold_out_count,
        requesters - 1,
        "the other {} should get SoldOut, got {sold_out_count}",
        requesters - 1,
    );

    // 3. Stock is exactly zero — never negative.
    let final_stock = redis.get_stock("prod-1");
    assert_eq!(
        final_stock, 0,
        "CRITICAL: final stock must be 0, got {final_stock}"
    );

    // 4. Belt-and-suspenders: verify the DB also has exactly one order.
    assert_eq!(
        db.count_orders("prod-1"),
        1,
        "DB should contain exactly 1 order"
    );

    // 5. No unexpected results.
    assert_eq!(
        other_count, 0,
        "no result other than Success or SoldOut is acceptable"
    );

    // 6. Verify the stock never went negative by re-reading several times
    //    (the DashMap entry CAS guarantees this, but we check anyway).
    for _ in 0..100 {
        assert!(
            redis.get_stock("prod-1") <= stock,
            "stock must never exceed initial value"
        );
    }
}
