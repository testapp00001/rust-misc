//! # Exercise 02: Redis Atomic Counter
//!
//! ## Learning Objective
//! Learn how Redis's DECR command provides atomic decrement operations
//! that work across distributed instances, and how to guard against
//! overselling.
//!
//! ## Flash Sale Context
//! In a distributed flash sale, multiple API servers share the same
//! stock counter in Redis. The DECR command is atomic at the Redis
//! server level -- even 10,000 concurrent decrements will each see a
//! unique, sequential value. But DECR happily goes negative. We need
//! a guard pattern to prevent overselling.
//!
//! ## Instructions
//! 1. Implement `initialize_stock` to SET the initial stock in Redis
//! 2. Implement `decrement` that uses DECR and checks for negative results
//! 3. If DECR returns a value < 0, increment it back (revert) and return an error
//! 4. Implement `get_stock` to read current stock
//!
//! ## Hints
//! - `redis::cmd("DECR").arg(key)` returns the new value after decrement
//! - If the result is negative, immediately INCR to revert
//! - Consider using a Lua script for atomic check-and-decrement (Module 02)
//! - The pattern: DECR -> check -> (revert if negative)
//!
//! ## Trade-offs
//! - **Pros:** Works across all instances, single Redis command, well-understood
//! - **Cons:** Brief oversell window (value goes negative then reverts),
//!   extra INCR round-trip on exhaustion, not truly atomic check-and-decrement
//! - **When to use:** Most flash sale scenarios where brief negative state is acceptable

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
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Redis key for the stock counter (e.g., "stock:product:42")
/// * `quantity` - Initial stock quantity
pub async fn initialize_stock(
    conn: &mut RedisConnection,
    key: &str,
    quantity: i64,
) -> Result<(), RedisCounterError> {
    // TODO: Use cmd("SET") to initialize the key with the quantity
    // TODO: Use query_async and handle Redis errors
    todo!("Implement stock initialization in Redis")
}

/// Atomically decrement stock by 1 and return whether an item was claimed.
///
/// Uses the DECR-then-check pattern:
/// 1. DECR the key atomically (Redis guarantees atomicity)
/// 2. If the new value >= 0, the claim is successful
/// 3. If the new value < 0, revert with INCR and return StockExhausted
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Redis key for the stock counter
pub async fn decrement(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<DecrementResult, RedisCounterError> {
    // TODO: DECR the key and get the new value (i64)
    // TODO: If new_value >= 0, return DecrementResult { new_value, claimed: true }
    // TODO: If new_value < 0, INCR to revert and return StockExhausted
    todo!("Implement Redis DECR with negative guard")
}

/// Get the current stock value.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Redis key for the stock counter
pub async fn get_stock(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<i64, RedisCounterError> {
    // TODO: Use cmd("GET") to read the key
    // TODO: Parse the result as i64
    todo!("Implement stock read from Redis")
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
            .builder(Some(Runtime::Tokio1))
            .build()
            .map_err(|e| format!("Pool error: {e}"))?;
        pool.get().await.map_err(|e| format!("Conn error: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection) {
        let _: Result<(), _> = cmd("DEL").arg(TEST_KEY).query_async(conn).await;
    }

    /// Test basic stock initialization and decrement.
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

    /// Test that stock never goes negative (oversell prevention).
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

        // Decrement twice (should succeed)
        decrement(&mut conn, TEST_KEY).await.expect("Decrement 1");
        decrement(&mut conn, TEST_KEY).await.expect("Decrement 2");

        // Third decrement should fail
        let result = decrement(&mut conn, TEST_KEY).await;
        assert!(result.is_err(), "Should reject oversell");

        // Stock should be 0, not negative
        let stock = get_stock(&mut conn, TEST_KEY)
            .await
            .expect("Should read stock");
        assert_eq!(stock, 0, "Stock should be exactly 0, not negative");

        cleanup(&mut conn).await;
    }

    /// Test concurrent decrements from multiple connections.
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

        // Spawn concurrent decrements using separate connections
        let mut handles = Vec::new();
        for _ in 0..10 {
            let url = REDIS_URL.to_string();
            let key = TEST_KEY.to_string();
            handles.push(tokio::spawn(async move {
                let cfg = Config::from_url(&url);
                let pool = cfg.builder(Some(Runtime::Tokio1)).build().unwrap();
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

        assert_eq!(
            total_success, initial,
            "Total claims should equal initial stock"
        );
        let stock = get_stock(&mut conn, TEST_KEY)
            .await
            .expect("Should read stock");
        assert_eq!(stock, 0, "Stock should be exactly 0");

        cleanup(&mut conn).await;
    }
}
