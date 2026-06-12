//! # Solution 02: Atomic Stock Decrement
//!
//! Complete implementation of atomic check-and-decrement via Lua.

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

local stock = redis.call('GET', KEYS[1])
if not stock or tonumber(stock) <= 0 then
    return -1  -- sold out
end
redis.call('DECR', KEYS[1])
return tonumber(stock) - 1  -- return new stock count
"#;

/// Atomically decrement stock for a product.
///
/// Returns `StockResult::Success(remaining)` with the new stock count,
/// or `StockResult::SoldOut` if no stock remains.
pub async fn atomic_decrement(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<StockResult> {
    let key = format!("stock:{product_id}");
    let result: i64 = Script::new(STOCK_DECR_SCRIPT)
        .key(key.as_str())
        .invoke_async(conn)
        .await?;

    match result {
        r if r >= 0 => Ok(StockResult::Success(r)),
        _ => Ok(StockResult::SoldOut),
    }
}

/// Set the stock quantity for a product (test helper).
pub async fn setup_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    quantity: i64,
) -> Result<()> {
    let key = format!("stock:{product_id}");
    let _: () = redis::cmd("SET")
        .arg(&key)
        .arg(quantity)
        .query_async(conn)
        .await?;
    Ok(())
}

/// Read the current stock for a product (test helper).
pub async fn get_stock(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64> {
    let key = format!("stock:{product_id}");
    let value: Option<i64> = redis::cmd("GET")
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
            let handle = std::thread::Builder::new()
                .spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut conn = match p.get().await {
                            Ok(c) => c,
                            Err(_) => return Ok::<_, anyhow::Error>(StockResult::SoldOut),
                        };
                        match atomic_decrement(&mut conn, &id).await {
                            Ok(result) => Ok(result),
                            Err(_) => Ok(StockResult::SoldOut),
                        }
                    })
                });
            match handle {
                Ok(h) => handles.push(h),
                Err(_) => {} // OS thread limit reached, skip
            }
        }

        let mut success_count = 0i64;
        for handle in handles {
            match handle.join().unwrap() {
                Ok(StockResult::Success(_)) => success_count += 1,
                Ok(StockResult::SoldOut) => {}
                Err(_) => {} // connection errors treated as non-success
            }
        }

        assert!(
            success_count <= 1,
            "At most 1 thread should have succeeded, got {success_count}"
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
