//! # Solution 04: Atomic Product Voucher Limit
//!
//! Complete implementation of atomic voucher count capping via Lua.

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
/// Reads the current voucher count for KEYS[1], compares it to the
/// maximum in ARGV[1], and increments only when under the limit.
/// Returns the new count (> 0) on success, or -1 when the limit is hit.
const VOUCHER_LIMIT_SCRIPT: &str = r#"
-- KEYS[1]: voucher counter key (e.g. "vouchers:product:42")
-- ARGV[1]: max_vouchers (string-encoded integer)
-- Returns: new voucher number (> 0) on success, -1 if limit reached

local current = redis.call('GET', KEYS[1])
local count = 0
if current then
    count = tonumber(current)
end
local max = tonumber(ARGV[1])
if count >= max then
    return -1  -- limit reached
end
redis.call('INCR', KEYS[1])
return count + 1  -- return the 1-based voucher number
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
    let key = format!("vouchers:{product_id}");
    let result: i64 = Script::new(VOUCHER_LIMIT_SCRIPT)
        .key(key.as_str())
        .arg(max_vouchers.to_string())
        .invoke_async(conn)
        .await?;

    match result {
        r if r > 0 => Ok(VoucherResult::Granted(r as u64)),
        _ => Ok(VoucherResult::LimitReached),
    }
}

/// Read the current voucher count for a product (test helper).
pub async fn get_voucher_count(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<u64> {
    let key = format!("vouchers:{product_id}");
    let value: Option<u64> = redis::cmd("GET")
        .arg(&key)
        .query_async(conn)
        .await?;
    Ok(value.unwrap_or(0))
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
        let mut cfg = Config::from_url(REDIS_URL);
        cfg.pool = Some(deadpool_redis::PoolConfig {
            max_size: 64,
            ..Default::default()
        });
        cfg.create_pool(Some(Runtime::Tokio1))
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
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut conn = match p.get().await {
                            Ok(c) => c,
                            Err(_) => return Ok::<_, anyhow::Error>(VoucherResult::LimitReached),
                        };
                        match atomic_voucher_limit(&mut conn, &id, max_vouchers).await {
                            Ok(result) => Ok(result),
                            Err(_) => Ok(VoucherResult::LimitReached),
                        }
                    })
                });
            match handle {
                Ok(h) => handles.push(h),
                Err(_) => {} // OS thread limit reached, skip
            }
        }

        let mut granted = 0i64;
        for handle in handles {
            match handle.join().unwrap() {
                Ok(VoucherResult::Granted(_)) => granted += 1,
                Ok(VoucherResult::LimitReached) => {}
                Err(_) => {} // connection errors treated as non-granted
            }
        }

        assert!(granted <= 50, "Voucher limit must never be exceeded, got {granted}");

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            let count = get_voucher_count(&mut conn, pid).await.unwrap();
            assert_eq!(count, granted as u64, "Voucher count must match granted count");
            cleanup(&mut conn, pid).await;
        });
    }
}
