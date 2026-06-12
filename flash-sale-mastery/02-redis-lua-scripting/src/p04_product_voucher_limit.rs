//! # Exercise 04: Atomic Product Voucher Limit -- CRITICAL
//!
//! ## Learning Objective
//! Build an atomic HINCRBY-with-ceiling check (or INCR with a cap) to
//! enforce per-product voucher limits. A product may issue at most N
//! vouchers regardless of how many concurrent requests arrive.
//!
//! ## Flash Sale Context
//! Each flash sale product has a maximum number of discount vouchers.
//! The Lua script atomically reads the current count and increments only
//! if the limit has not been reached, preventing oversubscription.
//!
//! ## Instructions
//! 1. Define a Lua script that atomically checks the voucher count against
//!    a maximum and increments only when under the limit
//! 2. Implement `atomic_voucher_limit` returning `VoucherResult`
//! 3. Implement helpers for setup and querying
//!
//! ## Hints
//! - Use `redis.call('GET', key)` / `redis.call('INCR', key)` or
//!   `redis.call('INCRBY', key, amount)`
//! - Return the new voucher number on success, -1 on limit reached
//! - Use `tonumber(ARGV[1])` to parse the max from the argument

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// Result of a voucher limit check.
#[derive(Debug, Clone, PartialEq)]
pub enum VoucherResult {
    /// Voucher granted. Contains the 1-based voucher number.
    Granted(u64),
    /// The per-product voucher limit has been reached.
    LimitReached,
}

/// Atomic voucher limit Lua script.
///
/// Reads the current voucher count for `KEYS[1]`, compares it to the
/// maximum in `ARGV[1]`, and increments only when under the limit.
/// Returns the new count (> 0) on success, or -1 when the limit is hit.
const VOUCHER_LIMIT_SCRIPT: &str = r#"
-- KEYS[1]: voucher counter key (e.g. "vouchers:product:42")
-- ARGV[1]: max_vouchers (string-encoded integer)
-- Returns: new voucher number (> 0) on success, -1 if limit reached

-- TODO: Implement atomic voucher limit check
-- 1. Read current count: GET KEYS[1]
-- 2. Parse to number (default 0 if nil)
-- 3. If count >= max, return -1
-- 4. Otherwise INCR and return the new count
"#;

/// Atomically check and increment the voucher count for a product.
///
/// Returns `VoucherResult::Granted(number)` with the 1-based voucher
/// number, or `VoucherResult::LimitReached` if the cap has been hit.
pub async fn atomic_voucher_limit(
    conn: &mut RedisConnection,
    product_id: &str,
    max_vouchers: u64,
) -> Result<VoucherResult> {
    // TODO: Build key, invoke script, interpret result
    todo!("Implement atomic voucher limit using VOUCHER_LIMIT_SCRIPT")
}

/// Read the current voucher count for a product (test helper).
pub async fn get_voucher_count(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<u64> {
    // TODO: GET vouchers:<product_id> (default 0)
    todo!("Implement: return current voucher count")
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
            .max_size(32)
            .build()
            .map_err(|e| format!("Could not create pool: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection, product_id: &str) {
        let _: () = redis::cmd("DEL")
            .arg(format!("vouchers:{product_id}"))
            .query_async(&mut *conn)
            .await
            .unwrap_or(());
    }

    /// Under the limit should return Granted with incrementing numbers.
    #[tokio::test]
    async fn test_under_limit() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_under_limit";

        cleanup(&mut conn, pid).await;

        let r1 = atomic_voucher_limit(&mut conn, pid, 5).await.unwrap();
        assert_eq!(r1, VoucherResult::Granted(1));

        let r2 = atomic_voucher_limit(&mut conn, pid, 5).await.unwrap();
        assert_eq!(r2, VoucherResult::Granted(2));

        let count = get_voucher_count(&mut conn, pid).await.unwrap();
        assert_eq!(count, 2);

        cleanup(&mut conn, pid).await;
    }

    /// At the limit should return LimitReached.
    #[tokio::test]
    async fn test_at_limit() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_at_limit";

        cleanup(&mut conn, pid).await;

        // Exhaust the limit
        for expected in 1..=3 {
            let r = atomic_voucher_limit(&mut conn, pid, 3).await.unwrap();
            assert_eq!(r, VoucherResult::Granted(expected));
        }

        // Next attempt should fail
        let r = atomic_voucher_limit(&mut conn, pid, 3).await.unwrap();
        assert_eq!(r, VoucherResult::LimitReached);

        // Count should still be 3, not 4
        let count = get_voucher_count(&mut conn, pid).await.unwrap();
        assert_eq!(count, 3);

        cleanup(&mut conn, pid).await;
    }

    /// Limit of 0 should immediately reject.
    #[tokio::test]
    async fn test_zero_limit() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_zero_limit";

        cleanup(&mut conn, pid).await;

        let r = atomic_voucher_limit(&mut conn, pid, 0).await.unwrap();
        assert_eq!(r, VoucherResult::LimitReached);

        cleanup(&mut conn, pid).await;
    }

    /// CRITICAL: 1000 concurrent requests with a limit of 50.
    /// Exactly 50 should succeed; count must never exceed 50.
    #[test]
    fn test_concurrent_limit_boundary() {
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
        let pid = "concurrent_voucher";
        let max_vouchers: u64 = 50;

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            cleanup(&mut conn, pid).await;
        });

        let mut handles = vec![];
        for _ in 0..1000 {
            let p = pool.clone();
            let id = pid.to_string();
            handles.push(std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut conn = p.get().await.unwrap();
                    atomic_voucher_limit(&mut conn, &id, max_vouchers).await
                })
            }));
        }

        let mut granted = 0i64;
        for handle in handles {
            match handle.join().unwrap() {
                Ok(VoucherResult::Granted(_)) => granted += 1,
                Ok(VoucherResult::LimitReached) => {}
                Err(e) => panic!("Unexpected error: {e}"),
            }
        }

        assert_eq!(granted, 50, "Exactly 50 vouchers should be granted");

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            let count = get_voucher_count(&mut conn, pid).await.unwrap();
            assert_eq!(count, 50, "Voucher count must be exactly 50");
            cleanup(&mut conn, pid).await;
        });
    }
}
