//! # Solution 02: Redis Atomic Counter
//!
//! Complete implementation of Redis-based atomic counter with oversell prevention.

use deadpool_redis::redis::cmd;
use deadpool_redis::Connection as RedisConnection;

/// Error type for Redis counter operations.
#[derive(Debug, thiserror::Error)]
pub enum RedisCounterError {
    #[error("Redis error: {0}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("Stock exhausted: no items remaining")]
    StockExhausted,

    #[error("Pool error: {0}")]
    PoolError(#[from] deadpool_redis::PoolError),
}

/// Result of a decrement operation.
#[derive(Debug, Clone, PartialEq)]
pub struct DecrementResult {
    /// The new stock value after decrement.
    pub new_value: i64,
    /// Whether this request successfully claimed an item.
    pub claimed: bool,
}

/// Initialize stock for a product in Redis.
pub async fn initialize_stock(
    conn: &mut RedisConnection,
    key: &str,
    quantity: i64,
) -> Result<(), RedisCounterError> {
    cmd("SET")
        .arg(key)
        .arg(quantity)
        .query_async::<()>(conn)
        .await?;
    Ok(())
}

/// Atomically decrement stock by 1 using the DECR-then-check pattern.
///
/// ## How It Works
///
/// 1. `DECR key` atomically subtracts 1 and returns the new value
/// 2. If new_value >= 0, the claim succeeded
/// 3. If new_value < 0, we oversold -- revert with `INCR` and return error
///
/// ## Trade-offs
/// - **Pros:** Works across all instances, single Redis command, well-understood
/// - **Cons:** Brief oversell window (value goes negative then reverts),
///   extra INCR round-trip on exhaustion, not truly atomic check-and-decrement
/// - **When to use:** Most flash sale scenarios where brief negative state is acceptable
///
/// ## Better Alternative
/// A Lua script (Module 02) can do check-and-decrement atomically:
/// ```lua
/// local stock = tonumber(redis.call('GET', KEYS[1]))
/// if stock > 0 then
///     redis.call('DECR', KEYS[1])
///     return stock - 1
/// else
///     return -1
/// end
/// ```
pub async fn decrement(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<DecrementResult, RedisCounterError> {
    let new_value: i64 = cmd("DECR").arg(key).query_async(conn).await?;

    if new_value >= 0 {
        Ok(DecrementResult {
            new_value,
            claimed: true,
        })
    } else {
        // Revert the decrement since stock is exhausted
        let _: i64 = cmd("INCR").arg(key).query_async(conn).await?;
        Err(RedisCounterError::StockExhausted)
    }
}

/// Get the current stock value.
pub async fn get_stock(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<i64, RedisCounterError> {
    let value: i64 = cmd("GET").arg(key).query_async(conn).await?;
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";
    const TEST_KEY: &str = "test:atomic:counter:p02";

    async fn get_conn() -> Result<RedisConnection, String> {
        let cfg = Config::from_url(REDIS_URL);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Pool error: {e}"))?;
        pool.get().await.map_err(|e| format!("Conn error: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection) {
        let _: Result<(), _> = cmd("DEL").arg(TEST_KEY).query_async(conn).await;
    }

    #[tokio::test]
    async fn test_basic_decrement() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;
        initialize_stock(&mut conn, TEST_KEY, 10)
            .await
            .expect("Should initialize stock");
        let result = decrement(&mut conn, TEST_KEY)
            .await
            .expect("Should decrement");
        assert!(result.claimed);
        assert_eq!(result.new_value, 9);
        cleanup(&mut conn).await;
    }

    #[tokio::test]
    async fn test_oversell_prevention() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;
        initialize_stock(&mut conn, TEST_KEY, 2)
            .await
            .expect("Should initialize stock");
        decrement(&mut conn, TEST_KEY).await.expect("Decrement 1");
        decrement(&mut conn, TEST_KEY).await.expect("Decrement 2");
        let result = decrement(&mut conn, TEST_KEY).await;
        assert!(result.is_err(), "Should reject oversell");
        let stock = get_stock(&mut conn, TEST_KEY)
            .await
            .expect("Should read stock");
        assert_eq!(stock, 0, "Stock should be exactly 0, not negative");
        cleanup(&mut conn).await;
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;
        let initial = 100i64;
        initialize_stock(&mut conn, TEST_KEY, initial)
            .await
            .expect("Should initialize stock");
        let mut handles = Vec::new();
        for _ in 0..10 {
            let url = REDIS_URL.to_string();
            let key = TEST_KEY.to_string();
            handles.push(tokio::spawn(async move {
                let cfg = Config::from_url(&url);
                let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
                let mut success = 0;
                for _ in 0..20 {
                    let mut c = pool.get().await.unwrap();
                    if decrement(&mut c, &key).await.is_ok() {
                        success += 1;
                    }
                }
                success
            }));
        }
        let mut total_success = 0i64;
        for h in handles {
            total_success += h.await.unwrap();
        }
        assert_eq!(total_success, initial, "Total claims should equal initial stock");
        let stock = get_stock(&mut conn, TEST_KEY)
            .await
            .expect("Should read stock");
        assert_eq!(stock, 0, "Stock should be exactly 0");
        cleanup(&mut conn).await;
    }
}
