//! # Exercise 04: Optimistic Locking
//!
//! ## Learning Objective
//! Implement optimistic concurrency control using version numbers.
//! This pattern is used in databases (SQL `WHERE version = ?`) and
//! can be simulated in Redis with hash fields and WATCH/MULTI/EXEC.
//!
//! ## Flash Sale Context
//! In a real system, stock lives in a database. The optimistic locking
//! pattern reads the current stock and version, then attempts an update
//! only if the version hasn't changed. If another request modified the
//! row between our read and write, the version check fails and we retry.
//!
//! SQL equivalent:
//! ```sql
//! UPDATE products
//! SET stock = stock - 1, version = version + 1
//! WHERE id = ? AND version = ?
//! ```
//!
//! ## Instructions
//! 1. Implement `initialize_product` to create a product with stock and version
//! 2. Implement `update_stock_optimistic` using WATCH/MULTI/EXEC on a Redis hash
//! 3. The function should only update if the version matches
//! 4. Implement `get_product` to read current stock and version
//!
//! ## Hints
//! - Use Redis HASH: `HSET product:42 stock 100 version 1`
//! - WATCH the hash key, then HMGET stock and version
//! - In MULTI/EXEC, use HSET to update both stock and version
//! - If version changed between WATCH and EXEC, the transaction aborts
//!
//! ## Trade-offs
//! - **Pros:** No locks held during read phase, high throughput under low
//!   contention, no deadlock possible
//! - **Cons:** Wasted work on conflicts (read + compute + retry), performance
//!   degrades under high contention, ABA problem without monotonic versions
//! - **When to use:** Low-to-moderate contention, read-heavy workloads,
//!   when holding a lock during the full operation is expensive

use deadpool_redis::redis::{cmd, Pipeline, Value};
use deadpool_redis::Connection as RedisConnection;

/// Error type for optimistic locking operations.
#[derive(Debug, thiserror::Error)]
pub enum OptimisticError {
    #[error("Redis error: {0}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("Version conflict: expected {expected}, actual {actual}")]
    VersionConflict { expected: i64, actual: i64 },

    #[error("Insufficient stock: need {need}, have {have}")]
    InsufficientStock { need: i64, have: i64 },

    #[error("Max retries ({0}) exceeded")]
    MaxRetriesExceeded(usize),
}

/// Product data read from the store.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductData {
    pub stock: i64,
    pub version: i64,
}

/// Initialize a product with stock and version in a Redis hash.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
/// * `stock` - Initial stock quantity
pub async fn initialize_product(
    conn: &mut RedisConnection,
    product_id: &str,
    stock: i64,
) -> Result<(), OptimisticError> {
    // TODO: Use HSET to set "stock" and "version" fields on the product key
    // TODO: Key format: "product:{product_id}"
    // TODO: Set stock = stock, version = 1
    todo!("Implement product initialization in Redis hash")
}

/// Read current product data (stock and version).
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
pub async fn get_product(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<ProductData, OptimisticError> {
    // TODO: Use HMGET to read "stock" and "version" fields
    // TODO: Parse both as i64
    // TODO: Return ProductData { stock, version }
    todo!("Implement product read from Redis hash")
}

/// Update stock using optimistic locking with version check.
///
/// Only updates if the current version matches `expected_version`.
/// Atomically decrements stock by `delta` and increments version.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
/// * `expected_version` - Version we expect (from our earlier read)
/// * `delta` - Amount to decrement (typically 1)
///
/// # Returns
/// `true` if the update succeeded, `false` if version conflict.
pub async fn update_stock_optimistic(
    conn: &mut RedisConnection,
    product_id: &str,
    expected_version: i64,
    delta: i64,
) -> Result<bool, OptimisticError> {
    // TODO: WATCH the product key
    // TODO: Read current stock and version via HMGET
    // TODO: If version != expected_version, return false (conflict)
    // TODO: If stock < delta, return InsufficientStock error
    // TODO: In MULTI/EXEC, HSET the new stock (stock - delta) and new version (version + 1)
    // TODO: Check if EXEC succeeded or was aborted
    todo!("Implement optimistic stock update")
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";
    const TEST_PRODUCT: &str = "test:opt:product:42";

    async fn get_conn() -> Result<RedisConnection, String> {
        let cfg = Config::from_url(REDIS_URL);
        let pool = cfg
            .builder(Some(Runtime::Tokio1))
            .build()
            .map_err(|e| format!("Pool error: {e}"))?;
        pool.get().await.map_err(|e| format!("Conn error: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection) {
        let _: Result<(), _> = cmd("DEL")
            .arg(format!("product:{}", TEST_PRODUCT))
            .query_async(conn)
            .await;
    }

    /// Test basic optimistic update.
    #[tokio::test]
    async fn test_basic_optimistic_update() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;

        initialize_product(&mut conn, TEST_PRODUCT, 100)
            .await
            .expect("Should initialize");

        let result = update_stock_optimistic(&mut conn, TEST_PRODUCT, 1, 1)
            .await
            .expect("Should update");
        assert!(result, "Update should succeed with correct version");

        let product = get_product(&mut conn, TEST_PRODUCT)
            .await
            .expect("Should read");
        assert_eq!(product.stock, 99);
        assert_eq!(product.version, 2);

        cleanup(&mut conn).await;
    }

    /// Test version conflict detection.
    #[tokio::test]
    async fn test_version_conflict() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;

        initialize_product(&mut conn, TEST_PRODUCT, 100)
            .await
            .expect("Should initialize");

        // First update succeeds
        update_stock_optimistic(&mut conn, TEST_PRODUCT, 1, 1)
            .await
            .expect("First update");

        // Second update with stale version should fail
        let result = update_stock_optimistic(&mut conn, TEST_PRODUCT, 1, 1)
            .await
            .expect("Should not error");
        assert!(!result, "Should detect version conflict with stale version");

        cleanup(&mut conn).await;
    }

    /// Test concurrent updates -- stock never goes negative.
    #[tokio::test]
    async fn test_concurrent_updates_stock_never_negative() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;

        let initial_stock = 50i64;
        initialize_product(&mut conn, TEST_PRODUCT, initial_stock)
            .await
            .expect("Should initialize");

        let mut total_success = 0i64;
        for _ in 0..100 {
            // Each attempt: read version, try update
            let product = get_product(&mut conn, TEST_PRODUCT)
                .await
                .expect("Should read");
            if product.stock <= 0 {
                break;
            }
            match update_stock_optimistic(&mut conn, TEST_PRODUCT, product.version, 1).await {
                Ok(true) => total_success += 1,
                Ok(false) => continue, // Version conflict, retry
                Err(_) => break,
            }
        }

        let product = get_product(&mut conn, TEST_PRODUCT)
            .await
            .expect("Should read");
        assert_eq!(
            total_success, initial_stock,
            "Total successful decrements should equal initial stock"
        );
        assert_eq!(product.stock, 0, "Stock should be exactly 0");
        assert!(product.stock >= 0, "Stock must never go negative");

        cleanup(&mut conn).await;
    }
}
