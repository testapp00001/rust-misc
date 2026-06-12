//! # Exercise 02: Redis String Operations
//!
//! ## Learning Objective
//! Master Redis STRING commands (GET, SET, INCR, DECR, SETEX, SETNX) for
//! managing simple key-value data in a flash sale system.
//!
//! ## Flash Sale Context
//! The most critical operation in a flash sale is stock management. When a
//! product goes on sale with 100 units, we store the count as a Redis string.
//! Each purchase atomically decrements the counter. If the counter hits zero,
//! the sale is over for that product.
//!
//! ## Instructions
//! 1. Implement `set_stock` to initialize or update stock for a product
//! 2. Implement `get_stock` to read current stock (returns 0 if key missing)
//! 3. Implement `decrement_stock` to atomically decrement and return new value
//! 4. Implement `set_stock_if_absent` using SETNX for idempotent initialization
//!
//! ## Hints
//! - Use `redis::cmd("DECR")` or the `decr()` method on Cmd
//! - DECR on a missing key creates it with value -1, so check first
//! - SETNX returns true if the key was set, false if it already existed

use deadpool_redis::Connection as RedisConnection;

/// Error type for string operations.
#[derive(Debug, thiserror::Error)]
pub enum StockError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Stock went below zero: current={current}")]
    StockExhausted { current: i64 },
}

/// Set the stock count for a product.
///
/// # Arguments
/// * `conn` - Redis connection from pool
/// * `product_id` - Product identifier (e.g., "product:1001")
/// * `count` - Initial stock count
pub async fn set_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    count: i64,
) -> Result<(), StockError> {
    // TODO: Use redis::cmd("SET") to set the key to count
    todo!("Implement set_stock")
}

/// Get the current stock count for a product.
/// Returns 0 if the key does not exist.
///
/// # Arguments
/// * `conn` - Redis connection from pool
/// * `product_id` - Product identifier
pub async fn get_stock(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64, StockError> {
    // TODO: GET the key, return 0 if it doesn't exist (Nil)
    todo!("Implement get_stock")
}

/// Atomically decrement stock by 1 and return the new value.
/// Returns an error if stock would go below zero.
///
/// # Arguments
/// * `conn` - Redis connection from pool
/// * `product_id` - Product identifier
pub async fn decrement_stock(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64, StockError> {
    // TODO: First check current stock, then DECR if positive
    // TODO: Return StockExhausted error if stock is already 0
    todo!("Implement decrement_stock")
}

/// Set stock only if the key does not already exist (SETNX).
/// Returns true if the key was set, false if it already existed.
///
/// # Arguments
/// * `conn` - Redis connection from pool
/// * `product_id` - Product identifier
/// * `count` - Stock count to set
pub async fn set_stock_if_absent(
    conn: &mut RedisConnection,
    product_id: &str,
    count: i64,
) -> Result<bool, StockError> {
    // TODO: Use SETNX (or SET with NX flag) to set only if absent
    todo!("Implement set_stock_if_absent")
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn get_conn() -> Option<RedisConnection> {
        let cfg = Config::from_url(REDIS_URL);
        let pool = cfg
            .builder(Some(Runtime::Tokio1))
            .build()
            .ok()?;
        pool.get().await.ok()
    }

    /// Helper to generate a unique test key to avoid collisions.
    fn test_key(suffix: &str) -> String {
        format!("test:stock:{}:{}", suffix, std::process::id())
    }

    #[tokio::test]
    async fn test_set_and_get_stock() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("set_get");
        set_stock(&mut conn, &key, 100).await.expect("set_stock failed");
        let stock = get_stock(&mut conn, &key).await.expect("get_stock failed");
        assert_eq!(stock, 100);
    }

    #[tokio::test]
    async fn test_get_stock_missing_key_returns_zero() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("missing");
        let stock = get_stock(&mut conn, &key).await.expect("get_stock failed");
        assert_eq!(stock, 0, "Missing key should return 0");
    }

    #[tokio::test]
    async fn test_decrement_stock() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("decr");
        set_stock(&mut conn, &key, 3).await.expect("set_stock failed");
        let val = decrement_stock(&mut conn, &key).await.expect("decr failed");
        assert_eq!(val, 2);
        let val = decrement_stock(&mut conn, &key).await.expect("decr failed");
        assert_eq!(val, 1);
        let val = decrement_stock(&mut conn, &key).await.expect("decr failed");
        assert_eq!(val, 0);
        // Next decrement should fail
        let result = decrement_stock(&mut conn, &key).await;
        assert!(result.is_err(), "Should error when stock exhausted");
    }

    #[tokio::test]
    async fn test_set_stock_if_absent() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("setnx");
        let was_set = set_stock_if_absent(&mut conn, &key, 50)
            .await
            .expect("setnx failed");
        assert!(was_set, "First SETNX should succeed");
        let was_set = set_stock_if_absent(&mut conn, &key, 99)
            .await
            .expect("setnx failed");
        assert!(!was_set, "Second SETNX should fail (key exists)");
        let stock = get_stock(&mut conn, &key).await.expect("get failed");
        assert_eq!(stock, 50, "Value should remain from first SETNX");
    }
}
