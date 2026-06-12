//! # P07 — Per-Product Voucher Limit
//!
//! **Scenario:** A product has stock = 100 but a voucher cap of 5.
//! Ten different accounts try to buy.
//!
//! **Expected behaviour:**
//! - The first 5 succeed (vouchers issued).
//! - Accounts 6–10 receive `VoucherLimitReached`.
//! - Stock drops by 5 (not 10).
//!
//! **Concepts tested:**
//! - Per-product voucher cap
//! - Atomic voucher counter
//! - Interaction between stock and voucher limits

mod common;

use common::*;
use std::sync::Arc;
use tokio::sync::Barrier;
use tokio::task::JoinHandle;

#[tokio::test]
async fn test_voucher_limit_enforced() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = Arc::new(MockFlashSaleService::new(redis.clone(), db.clone()));

    let max_vouchers: u32 = 5;
    service.add_product(make_product_with_limit("prod-1", 100, max_vouchers));

    let accounts: u32 = 10;
    let barrier = Arc::new(Barrier::new(accounts as usize));

    // -- Act: all accounts purchase concurrently ------------------------
    let handles: Vec<JoinHandle<MockPurchaseResult>> = (0..accounts)
        .map(|i| {
            let svc = Arc::clone(&service);
            let bar = Arc::clone(&barrier);
            tokio::spawn(async move {
                bar.wait().await;
                let req = make_request("p07", "prod-1", &format!("acct-{i}"));
                svc.purchase(&req).await
            })
        })
        .collect();

    let mut success_count: u32 = 0;
    let mut limit_count: u32 = 0;

    for handle in handles {
        match handle.await.unwrap() {
            MockPurchaseResult::Success { .. } => success_count += 1,
            MockPurchaseResult::VoucherLimitReached => limit_count += 1,
            other => panic!("unexpected result: {other}"),
        }
    }

    // -- Assert ---------------------------------------------------------
    assert_eq!(
        success_count, max_vouchers,
        "exactly {max_vouchers} should succeed, got {success_count}"
    );
    assert_eq!(
        limit_count,
        accounts - max_vouchers,
        "{} should get VoucherLimitReached, got {limit_count}",
        accounts - max_vouchers,
    );

    // Stock decremented by exactly the number of successes.
    let expected_stock = 100 - max_vouchers;
    assert_eq!(
        redis.get_stock("prod-1"),
        expected_stock,
        "stock should be {expected_stock}"
    );
    assert_eq!(db.count_vouchers("prod-1"), max_vouchers);
}

#[tokio::test]
async fn test_voucher_limit_with_exact_boundary() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    let max_vouchers: u32 = 3;
    service.add_product(make_product_with_limit("prod-1", 100, max_vouchers));

    // -- Act: exactly max_vouchers requests -----------------------------
    for i in 0..max_vouchers {
        let req = make_request("p07-boundary", "prod-1", &format!("acct-{i}"));
        let result = service.purchase(&req).await;
        assert!(
            matches!(result, MockPurchaseResult::Success { .. }),
            "request #{i} should succeed"
        );
    }

    // One more should fail.
    let req = make_request("p07-boundary", "prod-1", "acct-over");
    let result = service.purchase(&req).await;
    assert!(
        matches!(result, MockPurchaseResult::VoucherLimitReached),
        "request beyond limit should be VoucherLimitReached, got: {result}"
    );
}
