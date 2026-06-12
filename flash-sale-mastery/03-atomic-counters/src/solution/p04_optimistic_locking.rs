//! # Solution 04: Optimistic Locking
//!
//! Complete implementation of optimistic concurrency control using version
//! numbers in Redis hashes, simulating database-style optimistic locking.

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

fn product_key(product_id: &str) -> String {
    format!("product:{}", product_id)
}

/// Initialize a product with stock and version in a Redis hash.
pub async fn initialize_product(
    conn: &mut RedisConnection,
    product_id: &str,
    stock: i64,
) -> Result<(), OptimisticError> {
    let key = product_key(product_id);
    let mut pipe = Pipeline::new();
    pipe.hset(&key, "stock", stock);
    pipe.hset(&key, "version", 1i64);
    pipe.query_async::<()>(conn).await?;
    Ok(())
}

/// Read current product data (stock and version) from Redis hash.
pub async fn get_product(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<ProductData, OptimisticError> {
    let key = product_key(product_id);
    let (stock, version): (i64, i64) = cmd("HMGET")
        .arg(&key)
        .arg("stock")
        .arg("version")
        .query_async(conn)
        .await?;
    Ok(ProductData { stock, version })
}

/// Update stock using optimistic locking with version check.
///
/// ## SQL Equivalent
/// ```sql
/// UPDATE products
/// SET stock = stock - 1, version = version + 1
/// WHERE id = ? AND version = ?
/// ```
///
/// ## How It Works (Redis Simulation)
/// 1. WATCH the product hash key
/// 2. HMGET stock and version
/// 3. If version != expected_version, return false (conflict)
/// 4. If stock < delta, return InsufficientStock
/// 5. MULTI/EXEC: HSET new stock and new version
/// 6. If EXEC aborted (key changed), return false
///
/// ## Trade-offs
/// - **Pros:** No locks held during read, high throughput under low contention
/// - **Cons:** Wasted work on conflicts, degrades under high contention
/// - **When to use:** Low-to-moderate contention, read-heavy workloads
pub async fn update_stock_optimistic(
    conn: &mut RedisConnection,
    product_id: &str,
    expected_version: i64,
    delta: i64,
) -> Result<bool, OptimisticError> {
    let key = product_key(product_id);

    // Step 1: WATCH the product key
    cmd("WATCH")
        .arg(&key)
        .query_async::<()>(conn)
        .await?;

    // Step 2: Read current state
    let (current_stock, current_version): (i64, i64) = cmd("HMGET")
        .arg(&key)
        .arg("stock")
        .arg("version")
        .query_async(conn)
        .await?;

    // Step 3: Version check
    if current_version != expected_version {
        return Ok(false);
    }

    // Step 4: Stock check
    if current_stock < delta {
        return Err(OptimisticError::InsufficientStock {
            need: delta,
            have: current_stock,
        });
    }

    // Step 5: Atomic update
    let new_stock = current_stock - delta;
    let new_version = current_version + 1;

    let mut pipe = Pipeline::new();
    pipe.atomic();
    pipe.hset(&key, "stock", new_stock);
    pipe.hset(&key, "version", new_version);
    let result: Value = pipe.query_async(conn).await?;

    // Step 6: Check if EXEC succeeded
    Ok(!matches!(result, Value::Nil))
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
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Pool error: {e}"))?;
        pool.get().await.map_err(|e| format!("Conn error: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection) {
        let _: Result<(), _> = cmd("DEL")
            .arg(format!("product:{}", TEST_PRODUCT))
            .query_async(conn)
            .await;
    }

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
        update_stock_optimistic(&mut conn, TEST_PRODUCT, 1, 1)
            .await
            .expect("First update");
        let result = update_stock_optimistic(&mut conn, TEST_PRODUCT, 1, 1)
            .await
            .expect("Should not error");
        assert!(!result, "Should detect version conflict with stale version");
        cleanup(&mut conn).await;
    }

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
            let product = get_product(&mut conn, TEST_PRODUCT)
                .await
                .expect("Should read");
            if product.stock <= 0 {
                break;
            }
            match update_stock_optimistic(&mut conn, TEST_PRODUCT, product.version, 1).await {
                Ok(true) => total_success += 1,
                Ok(false) => continue,
                Err(_) => break,
            }
        }

        let product = get_product(&mut conn, TEST_PRODUCT)
            .await
            .expect("Should read");
        assert_eq!(total_success, initial_stock, "Total successful decrements should equal initial stock");
        assert_eq!(product.stock, 0, "Stock should be exactly 0");
        assert!(product.stock >= 0, "Stock must never go negative");
        cleanup(&mut conn).await;
    }
}
