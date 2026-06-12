//! # Exercise 02: Atomic Stock Decrement -- CRITICAL
//!
//! ## Learning Objective
//! Build an atomic check-and-decrement operation using a Lua script.
//! This is the single most important flash sale primitive: ensuring stock
//! never goes negative even when thousands of requests arrive simultaneously.
//!
//! ## Flash Sale Context
//! When 10,000 users hit "Buy" at the same instant, a naive GET-then-DECR
//! approach has a race window where multiple requests read the same stock
//! value before any decrement lands. A Lua script performs the check and
//! decrement in one atomic step inside Redis's single-threaded event loop.
//!
//! ## Instructions
//! 1. Define a Lua script as a `const &str` that:
//!    - Reads the stock value for `KEYS[1]`
//!    - If stock is nil or <= 0, returns `-1` (sold out)
//!    - Otherwise decrements stock and returns the new value
//! 2. Implement `atomic_decrement` that executes this script
//! 3. Implement `setup_stock` and `get_stock` helpers for testing
//!
//! ## Hints
//! - Use `redis::Script::new()` for the Lua script
//! - `redis.call('GET', key)` returns `false` in Lua for missing keys
//! - Always call `tonumber()` before arithmetic in Lua
//! - The script must NOT decrement when stock is already 0

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// Result of an atomic stock decrement attempt.
#[derive(Debug, Clone, PartialEq)]
pub enum StockResult {
    /// Decrement succeeded. Contains the remaining stock count.
    Success(i64),
    /// Item is sold out (stock was 0 or negative).
    SoldOut,
}

/// Atomic stock decrement Lua script.
///
/// Checks if stock > 0, then atomically decrements.
/// Returns the new stock count (>= 0) on success, or -1 if sold out.
const STOCK_DECR_SCRIPT: &str = r#"
-- KEYS[1]: stock key (e.g. "stock:product:42")
-- Returns: remaining stock (>= 0) on success, -1 if sold out

-- TODO: Implement the atomic stock decrement logic
-- 1. Read the current stock: redis.call('GET', KEYS[1])
-- 2. If nil or <= 0, return -1
-- 3. Otherwise DECR and return the new value
"#;

/// Atomically decrement stock for a product.
///
/// Returns `StockResult::Success(remaining)` with the new stock count,
/// or `StockResult::SoldOut` if no stock remains.
pub async fn atomic_decrement(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<StockResult> {
    // TODO: Build the key, invoke the Lua script, interpret the result
    todo!("Implement atomic decrement using the STOCK_DECR_SCRIPT")
}

/// Set the stock quantity for a product (test helper).
pub async fn setup_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    quantity: i64,
) -> Result<()> {
    // TODO: SET the stock key
    todo!("Implement: SET stock:<product_id> to quantity")
}

/// Read the current stock for a product (test helper).
pub async fn get_stock(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64> {
    // TODO: GET the stock key (return 0 if key is missing)
    todo!("Implement: GET stock:<product_id>")
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
            .arg(format!("stock:{product_id}"))
            .query_async(&mut *conn)
            .await
            .unwrap_or(());
    }

    /// Decrement from stock of 5 should yield Success(4).
    #[tokio::test]
    async fn test_normal_decrement() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_normal";

        cleanup(&mut conn, pid).await;
        setup_stock(&mut conn, pid, 5).await.unwrap();

        let result = atomic_decrement(&mut conn, pid).await.unwrap();
        assert_eq!(result, StockResult::Success(4));

        let remaining = get_stock(&mut conn, pid).await.unwrap();
        assert_eq!(remaining, 4);

        cleanup(&mut conn, pid).await;
    }

    /// Missing key (no stock set) should return SoldOut.
    #[tokio::test]
    async fn test_sold_out_no_key() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_soldout_nokey";

        cleanup(&mut conn, pid).await;

        let result = atomic_decrement(&mut conn, pid).await.unwrap();
        assert_eq!(result, StockResult::SoldOut);

        cleanup(&mut conn, pid).await;
    }

    /// Stock of 0 should return SoldOut without going negative.
    #[tokio::test]
    async fn test_sold_out_zero_stock() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_soldout_zero";

        cleanup(&mut conn, pid).await;
        setup_stock(&mut conn, pid, 0).await.unwrap();

        let result = atomic_decrement(&mut conn, pid).await.unwrap();
        assert_eq!(result, StockResult::SoldOut);

        cleanup(&mut conn, pid).await;
    }

    /// Decrement to 0 then next attempt should be SoldOut.
    #[tokio::test]
    async fn test_decrement_to_zero_then_sold_out() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let pid = "test_to_zero";

        cleanup(&mut conn, pid).await;
        setup_stock(&mut conn, pid, 1).await.unwrap();

        let result = atomic_decrement(&mut conn, pid).await.unwrap();
        assert_eq!(result, StockResult::Success(0));

        let result = atomic_decrement(&mut conn, pid).await.unwrap();
        assert_eq!(result, StockResult::SoldOut);

        cleanup(&mut conn, pid).await;
    }

    /// CRITICAL TEST: 1000 concurrent threads race for the last item.
    /// Stock must NEVER go negative. Exactly 1 thread should succeed.
    #[test]
    fn test_concurrent_last_item() {
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
        let pid = "concurrent_last";

        // Setup: stock = 1
        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            cleanup(&mut conn, pid).await;
            setup_stock(&mut conn, pid, 1).await.unwrap();
        });

        // Spawn 1000 threads each trying to buy
        let mut handles = vec![];
        for _ in 0..1000 {
            let p = pool.clone();
            let id = pid.to_string();
            handles.push(std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut conn = p.get().await.unwrap();
                    atomic_decrement(&mut conn, &id).await
                })
            }));
        }

        let mut success_count = 0i64;
        for handle in handles {
            match handle.join().unwrap() {
                Ok(StockResult::Success(_)) => success_count += 1,
                Ok(StockResult::SoldOut) => {}
                Err(e) => panic!("Unexpected error: {e}"),
            }
        }

        assert_eq!(
            success_count, 1,
            "Exactly 1 thread should have succeeded"
        );

        // Verify stock is exactly 0, never negative
        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            let stock = get_stock(&mut conn, pid).await.unwrap();
            assert_eq!(stock, 0, "Stock must be exactly 0, never negative");
            cleanup(&mut conn, pid).await;
        });
    }
}
