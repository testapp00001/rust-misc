//! # Exercise 06: Stock Reconciliation
//!
//! ## Learning Objective
//! Implement reconciliation between two data stores (Redis and database)
//! to detect and fix stock discrepancies that inevitably occur in
//! distributed systems.
//!
//! ## Flash Sale Context
//! In a production flash sale, stock is tracked in both Redis (fast,
//! in-memory) and a database (durable, authoritative). During normal
//! operation, Redis may be slightly ahead of the database because
//! writes are asynchronous. After the sale, reconciliation compares
//! both stores and fixes mismatches.
//!
//! ## Common Scenarios
//! - **Redis ahead of DB:** Normal during async writes. The Redis count
//!   was decremented but the DB write hasn't completed yet.
//! - **DB ahead of Redis:** Error state. Possibly a failed Redis write
//!   after a successful DB write. Redis needs to be corrected.
//! - **Mismatch beyond threshold:** Alert for manual investigation.
//!
//! ## Instructions
//! 1. Implement `initialize_stores` to set up matching Redis and DB counters
//! 2. Implement `reconcile_stock` that compares both stores
//! 3. Implement `fix_mismatch` that corrects the discrepancy
//! 4. Handle each mismatch scenario appropriately
//!
//! ## Hints
//! - Use two Redis keys to simulate Redis stock and DB stock
//! - Reconciliation reads both and compares
//! - If Redis > DB: Redis is ahead (normal), log but don't fix
//! - If DB > Redis: Redis is behind (error), fix Redis to match DB
//! - The DB is the source of truth for reconciliation
//!
//! ## Trade-offs
//! - **Pros:** Catches drift before it causes overselling or lost sales,
//!   automated correction reduces manual intervention
//! - **Cons:** Adds complexity, fix operations can cause brief inconsistency,
//!   false positives during normal async operation
//! - **When to use:** Always in production flash sale systems, run periodically

use deadpool_redis::redis::cmd;
use deadpool_redis::Connection as RedisConnection;

/// Error type for reconciliation operations.
#[derive(Debug, thiserror::Error)]
pub enum ReconciliationError {
    #[error("Redis error: {0}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("Mismatch threshold exceeded: Redis={redis_stock}, DB={db_stock}, diff={diff}")]
    ThresholdExceeded {
        redis_stock: i64,
        db_stock: i64,
        diff: i64,
    },
}

/// Report comparing stock between Redis and database.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReconciliationReport {
    /// Product identifier.
    pub product_id: String,
    /// Stock value in Redis.
    pub redis_stock: i64,
    /// Stock value in database.
    pub db_stock: i64,
    /// Whether the stores are in sync.
    pub in_sync: bool,
    /// Whether a fix was applied.
    pub fix_applied: bool,
    /// Human-readable description of the status.
    pub status: String,
}

/// Initialize both Redis and DB stock for a product.
///
/// Uses two Redis keys to simulate the two stores:
/// - `redis:stock:{product_id}` for Redis
/// - `db:stock:{product_id}` for the database
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
/// * `quantity` - Initial stock quantity (same in both stores)
pub async fn initialize_stores(
    conn: &mut RedisConnection,
    product_id: &str,
    quantity: i64,
) -> Result<(), ReconciliationError> {
    // TODO: SET redis:stock:{product_id} = quantity
    // TODO: SET db:stock:{product_id} = quantity
    todo!("Initialize both stores with matching stock")
}

/// Simulate a Redis decrement (faster, happens first).
pub async fn decrement_redis(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64, ReconciliationError> {
    // TODO: DECR redis:stock:{product_id}
    todo!("Simulate Redis-side decrement")
}

/// Simulate a database decrement (slower, happens after Redis).
pub async fn decrement_db(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64, ReconciliationError> {
    // TODO: DECR db:stock:{product_id}
    todo!("Simulate DB-side decrement")
}

/// Reconcile stock between Redis and database.
///
/// Compares both stores and returns a report. If there's a mismatch
/// where DB > Redis (error state), fixes Redis to match DB.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
/// * `threshold` - Maximum acceptable difference before alerting
pub async fn reconcile_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    threshold: i64,
) -> Result<ReconciliationReport, ReconciliationError> {
    // TODO: Read redis:stock:{product_id} and db:stock:{product_id}
    // TODO: Compare the two values
    // TODO: If in sync, return report with in_sync = true
    // TODO: If Redis > DB (Redis ahead), this is normal during sale
    //   - Return report with status "Redis ahead of DB (normal async lag)"
    // TODO: If DB > Redis (Redis behind), this is an error
    //   - Fix Redis to match DB
    //   - Return report with fix_applied = true
    // TODO: If difference > threshold, return ThresholdExceeded error
    todo!("Implement stock reconciliation")
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";
    const TEST_PRODUCT: &str = "test:recon:product:42";

    async fn get_conn() -> Result<RedisConnection, String> {
        let cfg = Config::from_url(REDIS_URL);
        let pool = cfg
            .builder(Some(Runtime::Tokio1))
            .build()
            .map_err(|e| format!("Pool error: {e}"))?;
        pool.get().await.map_err(|e| format!("Conn error: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection, product_id: &str) {
        let redis_key = format!("redis:stock:{}", product_id);
        let db_key = format!("db:stock:{}", product_id);
        let _: Result<(), _> = cmd("DEL")
            .arg(&redis_key)
            .arg(&db_key)
            .query_async(conn)
            .await;
    }

    /// Test in-sync state.
    #[tokio::test]
    async fn test_in_sync() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn, TEST_PRODUCT).await;

        initialize_stores(&mut conn, TEST_PRODUCT, 100)
            .await
            .expect("Should initialize");

        let report = reconcile_stock(&mut conn, TEST_PRODUCT, 10)
            .await
            .expect("Should reconcile");
        assert!(report.in_sync, "Stores should be in sync");
        assert!(!report.fix_applied, "No fix needed");

        cleanup(&mut conn, TEST_PRODUCT).await;
    }

    /// Test Redis ahead of DB (normal during sale).
    #[tokio::test]
    async fn test_redis_ahead() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn, TEST_PRODUCT).await;

        initialize_stores(&mut conn, TEST_PRODUCT, 100)
            .await
            .expect("Should initialize");

        // Simulate Redis decrement without DB decrement
        decrement_redis(&mut conn, TEST_PRODUCT)
            .await
            .expect("Redis decrement");

        let report = reconcile_stock(&mut conn, TEST_PRODUCT, 10)
            .await
            .expect("Should reconcile");
        assert!(!report.in_sync, "Should detect mismatch");
        assert_eq!(report.redis_stock, 99);
        assert_eq!(report.db_stock, 100);
        // Redis ahead is normal -- no fix applied
        assert!(!report.fix_applied, "Should not fix Redis-ahead case");

        cleanup(&mut conn, TEST_PRODUCT).await;
    }

    /// Test DB ahead of Redis (error state, fix applied).
    #[tokio::test]
    async fn test_db_ahead_fixes_redis() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn, TEST_PRODUCT).await;

        initialize_stores(&mut conn, TEST_PRODUCT, 100)
            .await
            .expect("Should initialize");

        // Simulate DB decrement without Redis decrement (error state)
        decrement_db(&mut conn, TEST_PRODUCT)
            .await
            .expect("DB decrement");

        let report = reconcile_stock(&mut conn, TEST_PRODUCT, 10)
            .await
            .expect("Should reconcile");
        assert!(!report.in_sync, "Should detect mismatch");
        assert!(report.fix_applied, "Should fix DB-ahead case");

        // After fix, both should match DB value
        let redis_key = format!("redis:stock:{}", TEST_PRODUCT);
        let redis_val: i64 = cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(redis_val, 99, "Redis should be fixed to match DB");

        cleanup(&mut conn, TEST_PRODUCT).await;
    }

    /// Test reconciliation fixes mismatches.
    #[tokio::test]
    async fn test_reconciliation_fixes() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn, TEST_PRODUCT).await;

        initialize_stores(&mut conn, TEST_PRODUCT, 100)
            .await
            .expect("Should initialize");

        // Simulate multiple Redis-only decrements (lag)
        for _ in 0..5 {
            decrement_redis(&mut conn, TEST_PRODUCT)
                .await
                .expect("Redis decrement");
        }

        let report = reconcile_stock(&mut conn, TEST_PRODUCT, 10)
            .await
            .expect("Should reconcile");
        assert_eq!(report.redis_stock, 95);
        assert_eq!(report.db_stock, 100);

        cleanup(&mut conn, TEST_PRODUCT).await;
    }
}
