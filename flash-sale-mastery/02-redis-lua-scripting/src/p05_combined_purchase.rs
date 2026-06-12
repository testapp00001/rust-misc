//! # Exercise 05: Combined Atomic Purchase -- THE MASTER SCRIPT
//!
//! ## Learning Objective
//! Compose every individual check (stock, claim, voucher, idempotency)
//! into a single Lua script that atomically executes the full purchase
//! flow. This is the most important exercise in the entire module.
//!
//! ## Flash Sale Context
//! A real flash sale purchase requires four atomic guarantees:
//!
//! 1. **Stock**: inventory must not go negative
//! 2. **Claim**: each account may only purchase once
//! 3. **Voucher**: the product's voucher cap must not be exceeded
//! 4. **Idempotency**: duplicate request IDs must return the same result
//!
//! If these are separate scripts, a crash between them leaves the system
//! in an inconsistent state. A single Lua script either succeeds entirely
//! or fails entirely -- there is no partial state.
//!
//! ## Instructions
//! 1. Define the combined Lua script as a `const &str` with inline comments
//! 2. Implement `execute_purchase` that invokes the script and parses the
//!    multi-element return array into `PurchaseResult`
//! 3. Implement helper functions for setup and verification
//!
//! ## Hints
//! - Return a Lua table where element 1 is a status code:
//!   1=Success, 2=SoldOut, 3=AlreadyClaimed, 4=VoucherLimit, 5=IdempotentReplay
//! - On success, element 2 is the voucher code
//! - On idempotent replay, element 2 is the cached result string
//! - Always store the idempotency result via SETEX before returning
//! - Roll back stock (INCR) if the voucher limit check fails after decrement
//!
//! ## Lua Script Key Layout
//!
//! | Index | Key                     | Purpose              |
//! |-------|-------------------------|----------------------|
//! | 1     | `stock:{product_id}`    | Remaining inventory  |
//! | 2     | `claims:{product_id}`   | Per-account claims   |
//! | 3     | `vouchers:{product_id}` | Voucher counter      |
//! | 4     | `idemp:{request_id}`    | Idempotency cache    |

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;
use serde::{Deserialize, Serialize};

/// TTL (seconds) for the idempotency cache entry.
const TTL_SECS: u64 = 3600;

/// Request parameters for a combined purchase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseRequest {
    pub product_id: String,
    pub account_id: String,
    pub request_id: String,
    pub max_vouchers: u64,
}

/// Result of a combined purchase attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum PurchaseResult {
    /// Purchase succeeded. Contains the generated voucher code.
    Success { voucher_code: String },
    /// The product is sold out.
    SoldOut,
    /// This account has already claimed this product.
    AlreadyClaimed,
    /// The product's voucher limit has been reached.
    VoucherLimitReached,
    /// This request_id was already processed. Returns the cached result.
    IdempotentReplay { original_result: String },
}

/// Combined atomic purchase Lua script.
///
/// Performs idempotency check, account claim check, stock check,
/// voucher limit check, and all mutations in a single atomic block.
///
/// KEYS: [stock, claims, vouchers, idempotency]
/// ARGV: [account_id, max_vouchers, request_id, ttl_secs]
///
/// Returns a Lua table:
///   {1, voucher_code}           -- Success
///   {2}                         -- SoldOut
///   {3}                         -- AlreadyClaimed
///   {4}                         -- VoucherLimitReached
///   {5, cached_result_string}   -- IdempotentReplay
const PURCHASE_SCRIPT: &str = r#"
-- KEYS[1] = stock key
-- KEYS[2] = claims set key
-- KEYS[3] = voucher counter key
-- KEYS[4] = idempotency key
-- ARGV[1] = account_id
-- ARGV[2] = max_vouchers (string)
-- ARGV[3] = request_id
-- ARGV[4] = ttl_seconds (string)

-- TODO: Implement the combined purchase script
-- Step 1: Idempotency check -- GET KEYS[4]; if exists, return {5, value}
-- Step 2: Account claim -- SISMEMBER KEYS[2] ARGV[1]; if member, store idemp result, return {3}
-- Step 3: Stock check -- GET KEYS[1]; if nil or <= 0, store idemp result, return {2}
-- Step 4: Voucher limit -- GET KEYS[3]; if count >= max, roll back stock, store idemp, return {4}
-- Step 5: Mutate -- DECR stock, SADD claim, INCR voucher
-- Step 6: Generate voucher code and store idemp result
-- Step 7: Return {1, voucher_code}
"#;

/// Execute the full atomic purchase flow.
pub async fn execute_purchase(
    conn: &mut RedisConnection,
    req: &PurchaseRequest,
) -> Result<PurchaseResult> {
    // TODO: Build all 4 keys and 4 args, invoke PURCHASE_SCRIPT, parse result
    todo!("Implement: invoke the combined Lua script and parse the return array")
}

/// Set up stock for a product (test helper).
pub async fn setup_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    quantity: i64,
) -> Result<()> {
    // TODO: SET stock:<product_id>
    todo!()
}

/// Get current stock (test helper).
pub async fn get_stock(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64> {
    // TODO: GET stock:<product_id>
    todo!()
}

/// Get current voucher count (test helper).
pub async fn get_voucher_count(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<u64> {
    // TODO: GET vouchers:<product_id>
    todo!()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Pool, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn try_connect() -> Result<Pool, String> {
        Config::from_url(REDIS_URL)
            .builder(Some(Runtime::Tokio1))
            .max_size(64)
            .build()
            .map_err(|e| format!("Could not create pool: {e}"))
    }

    async fn cleanup_all(conn: &mut RedisConnection, product_id: &str) {
        for prefix in &["stock:", "claims:", "vouchers:"] {
            let _: () = redis::cmd("DEL")
                .arg(format!("{prefix}{product_id}"))
                .query_async(&mut *conn)
                .await
                .unwrap_or(());
        }
    }

    async fn cleanup_idemp(conn: &mut RedisConnection, request_id: &str) {
        let _: () = redis::cmd("DEL")
            .arg(format!("idemp:{request_id}"))
            .query_async(&mut *conn)
            .await
            .unwrap_or(());
    }

    /// Happy path: valid request should succeed with a voucher code.
    #[tokio::test]
    async fn test_successful_purchase() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_success";
        let rid = "test_success_req1";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
        setup_stock(&mut conn, pid, 10).await.unwrap();

        let req = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid.to_string(),
            max_vouchers: 5,
        };

        let result = execute_purchase(&mut conn, &req).await.unwrap();
        match result {
            PurchaseResult::Success { voucher_code } => {
                assert!(!voucher_code.is_empty());
            }
            other => panic!("Expected Success, got {other:?}"),
        }

        let stock = get_stock(&mut conn, pid).await.unwrap();
        assert_eq!(stock, 9);

        let vouchers = get_voucher_count(&mut conn, pid).await.unwrap();
        assert_eq!(vouchers, 1);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
    }

    /// Sold-out product should return SoldOut.
    #[tokio::test]
    async fn test_sold_out() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_combined_soldout";
        let rid = "test_combined_soldout_req";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
        // No stock set => sold out

        let req = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid.to_string(),
            max_vouchers: 5,
        };

        let result = execute_purchase(&mut conn, &req).await.unwrap();
        assert_eq!(result, PurchaseResult::SoldOut);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
    }

    /// Duplicate account should return AlreadyClaimed.
    #[tokio::test]
    async fn test_already_claimed() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_combined_claimed";
        let rid1 = "test_combined_claimed_r1";
        let rid2 = "test_combined_claimed_r2";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid1).await;
        cleanup_idemp(&mut conn, rid2).await;
        setup_stock(&mut conn, pid, 10).await.unwrap();

        let req1 = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid1.to_string(),
            max_vouchers: 5,
        };
        let r1 = execute_purchase(&mut conn, &req1).await.unwrap();
        assert!(matches!(r1, PurchaseResult::Success { .. }));

        // Same account, different request ID
        let req2 = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid2.to_string(),
            max_vouchers: 5,
        };
        let r2 = execute_purchase(&mut conn, &req2).await.unwrap();
        assert_eq!(r2, PurchaseResult::AlreadyClaimed);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid1).await;
        cleanup_idemp(&mut conn, rid2).await;
    }

    /// Voucher limit exhausted should return VoucherLimitReached.
    #[tokio::test]
    async fn test_voucher_limit() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_combined_vlimit";
        let rid = "test_combined_vlimit_req";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
        setup_stock(&mut conn, pid, 10).await.unwrap();

        let req = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid.to_string(),
            max_vouchers: 0, // Zero limit
        };

        let result = execute_purchase(&mut conn, &req).await.unwrap();
        assert_eq!(result, PurchaseResult::VoucherLimitReached);

        // Stock should be unchanged (rolled back)
        let stock = get_stock(&mut conn, pid).await.unwrap();
        assert_eq!(stock, 10);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
    }

    /// Same request_id should return IdempotentReplay.
    #[tokio::test]
    async fn test_idempotent_replay() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_combined_idemp";
        let rid = "test_combined_idemp_req";

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
        setup_stock(&mut conn, pid, 10).await.unwrap();

        let req = PurchaseRequest {
            product_id: pid.to_string(),
            account_id: "acct_1".to_string(),
            request_id: rid.to_string(),
            max_vouchers: 5,
        };

        let r1 = execute_purchase(&mut conn, &req).await.unwrap();
        assert!(matches!(r1, PurchaseResult::Success { .. }));

        let r2 = execute_purchase(&mut conn, &req).await.unwrap();
        assert!(matches!(r2, PurchaseResult::IdempotentReplay { .. }));

        // Stock should only have been decremented once
        let stock = get_stock(&mut conn, pid).await.unwrap();
        assert_eq!(stock, 9);

        cleanup_all(&mut conn, pid).await;
        cleanup_idemp(&mut conn, rid).await;
    }

    /// CRITICAL: 500 concurrent threads, stock=100, max_vouchers=100.
    /// Exactly 100 should succeed. Stock and voucher count must match.
    #[test]
    fn test_concurrent_purchase() {
        let pool = match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(try_connect())
        {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let pid = "concurrent_purchase";

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            cleanup_all(&mut conn, pid).await;
            setup_stock(&mut conn, pid, 100).await.unwrap();
        });

        let mut handles = vec![];
        for i in 0..500 {
            let p = pool.clone();
            let id = pid.to_string();
            handles.push(std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut conn = p.get().await.unwrap();
                    let req = PurchaseRequest {
                        product_id: id.clone(),
                        account_id: format!("acct_{i}"),
                        request_id: format!("req_{i}"),
                        max_vouchers: 100,
                    };
                    execute_purchase(&mut conn, &req).await
                })
            }));
        }

        let mut success_count = 0i64;
        let mut sold_out_count = 0i64;
        for handle in handles {
            match handle.join().unwrap() {
                Ok(PurchaseResult::Success { .. }) => success_count += 1,
                Ok(PurchaseResult::SoldOut) => sold_out_count += 1,
                Ok(other) => panic!("Unexpected result: {other:?}"),
                Err(e) => panic!("Unexpected error: {e}"),
            }
        }

        assert_eq!(success_count, 100, "Exactly 100 should succeed");
        assert_eq!(sold_out_count, 400, "400 should see SoldOut");

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            let stock = get_stock(&mut conn, pid).await.unwrap();
            assert_eq!(stock, 0, "Stock must be 0");
            let vouchers = get_voucher_count(&mut conn, pid).await.unwrap();
            assert_eq!(vouchers, 100, "Voucher count must be 100");
            cleanup_all(&mut conn, pid).await;
        });
    }
}
