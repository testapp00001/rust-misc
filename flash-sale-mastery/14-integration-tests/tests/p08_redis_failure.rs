//! # P08 — Redis Failure
//!
//! **Scenario:** Redis is unavailable when a purchase request arrives.
//!
//! **Expected behaviour:**
//! - Returns `RedisUnavailable` (maps to HTTP 503).
//! - Does **not** return an internal error (HTTP 500).
//! - The request is safely rejected — no partial state.
//!
//! **Concepts tested:**
//! - Graceful degradation when the primary store is down
//! - Fast-fail behaviour
//! - No partial writes or inconsistent state

mod common;

use common::*;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[tokio::test]
async fn test_returns_unavailable_when_redis_down() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-1", 10));

    // Simulate Redis failure.
    redis.failing.store(true, Ordering::SeqCst);

    let request = make_request("p08", "prod-1", "account-1");

    // -- Act ------------------------------------------------------------
    let result = service.purchase(&request).await;

    // -- Assert ---------------------------------------------------------
    assert!(
        matches!(result, MockPurchaseResult::RedisUnavailable),
        "expected RedisUnavailable, got: {result}"
    );

    // No stock consumed, no orders, no vouchers.
    // (Stock was set before failure, so it's still 10 in the map.)
    assert_eq!(redis.get_stock("prod-1"), 10);
    assert_eq!(db.count_orders("prod-1"), 0);

    // A failure event should be recorded.
    let events = redis.get_events();
    assert!(
        events
            .iter()
            .any(|e| matches!(e.event_type, EventType::PurchaseFailed)
                && e.message.contains("redis")),
        "a PurchaseFailed event mentioning Redis must be recorded"
    );
}

#[tokio::test]
async fn test_recovers_after_redis_returns() {
    // -- Setup ----------------------------------------------------------
    let redis = Arc::new(MockRedis::new());
    let db = Arc::new(MockDb::new());
    let service = MockFlashSaleService::new(redis.clone(), db.clone());

    service.add_product(make_product("prod-1", 10));

    // Redis fails.
    redis.failing.store(true, Ordering::SeqCst);
    let fail_result = service.purchase(&make_request("p08-rec", "prod-1", "acct-1")).await;
    assert!(matches!(fail_result, MockPurchaseResult::RedisUnavailable));

    // Redis comes back.
    redis.failing.store(false, Ordering::SeqCst);
    let ok_result = service.purchase(&make_request("p08-rec", "prod-1", "acct-2")).await;
    assert!(
        matches!(ok_result, MockPurchaseResult::Success { .. }),
        "should succeed after Redis recovery, got: {ok_result}"
    );
}
