//! # P02 — Concurrent Purchases
//!
//! **Scenario:** 100 different accounts try to buy from a product with
//! stock = 10, all at the same time.
//!
//! **Expected behaviour:**
//! - Exactly 10 purchases succeed.
//! - The remaining 90 receive `SoldOut`.
//! - Final stock is 0.
//!
//! **Concepts tested:**
//! - Thread-safe stock decrement under contention
//! - Correct total success count under concurrency
//! - Using `tokio::sync::Barrier` to align goroutines

mod common;

use common::*;
use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::task::JoinHandle;

#[tokio::test]
async fn test_exactly_stock_count_succeeds() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = Arc::new(MockFlashSaleService::new(redis.clone(), db.clone()));

    let stock: u32 = 10;
    let requesters: u32 = 100;
    service.add_product(make_product("prod-1", stock));

    let barrier = Arc::new(Barrier::new(requesters as usize));

    // -- Act: spawn all requests concurrently ---------------------------
    let handles: Vec<JoinHandle<MockPurchaseResult>> = (0..requesters)
        .map(|i| {
            let svc = Arc::clone(&service);
            let bar = Arc::clone(&barrier);
            tokio::spawn(async move {
                // Wait until every task is ready so they all fire at once.
                bar.wait().await;
                let req = make_request("p02", "prod-1", &format!("acct-{i}"));
                svc.purchase(&req).await
            })
        })
        .collect();

    // -- Collect results ------------------------------------------------
    let mut success_count: u32 = 0;
    let mut sold_out_count: u32 = 0;

    for handle in handles {
        match handle.await.unwrap() {
            MockPurchaseResult::Success { .. } => success_count += 1,
            MockPurchaseResult::SoldOut => sold_out_count += 1,
            other => panic!("unexpected result: {other}"),
        }
    }

    // -- Assert ---------------------------------------------------------
    assert_eq!(
        success_count, stock,
        "exactly {stock} purchases should succeed, got {success_count}"
    );
    assert_eq!(
        sold_out_count,
        requesters - stock,
        "{} should get SoldOut, got {sold_out_count}",
        requesters - stock
    );
    assert_eq!(
        redis.get_stock("prod-1"),
        0,
        "final stock must be 0"
    );
}
