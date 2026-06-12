//! # Solution 06: Stock Reconciliation
//!
//! Complete implementation of stock reconciliation between Redis and database.

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
    pub product_id: String,
    pub redis_stock: i64,
    pub db_stock: i64,
    pub in_sync: bool,
    pub fix_applied: bool,
    pub status: String,
}

fn redis_key(product_id: &str) -> String {
    format!("redis:stock:{}", product_id)
}

fn db_key(product_id: &str) -> String {
    format!("db:stock:{}", product_id)
}

/// Initialize both Redis and DB stock for a product.
pub async fn initialize_stores(
    conn: &mut RedisConnection,
    product_id: &str,
    quantity: i64,
) -> Result<(), ReconciliationError> {
    cmd("SET")
        .arg(redis_key(product_id))
        .arg(quantity)
        .query_async::<()>(conn)
        .await?;
    cmd("SET")
        .arg(db_key(product_id))
        .arg(quantity)
        .query_async::<()>(conn)
        .await?;
    Ok(())
}

/// Simulate a Redis decrement (faster, happens first).
pub async fn decrement_redis(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64, ReconciliationError> {
    let val: i64 = cmd("DECR")
        .arg(redis_key(product_id))
        .query_async(conn)
        .await?;
    Ok(val)
}

/// Simulate a database decrement (slower, happens after Redis).
pub async fn decrement_db(
    conn: &mut RedisConnection,
    product_id: &str,
) -> Result<i64, ReconciliationError> {
    let val: i64 = cmd("DECR")
        .arg(db_key(product_id))
        .query_async(conn)
        .await?;
    Ok(val)
}

/// Reconcile stock between Redis and database.
///
/// ## Decision Logic
///
/// | Redis | DB   | Meaning                | Action                    |
/// |-------|------|------------------------|---------------------------|
/// | ==    | ==   | In sync                | No action                 |
/// | <     | ==   | Redis ahead (lag)      | Normal during sale, log   |
/// | ==    | <    | DB ahead (error)       | Fix Redis to match DB     |
/// | <     | <    | Both decremented       | Normal, different rates   |
///
/// The database is the **source of truth**. If Redis and DB disagree,
/// we trust the database and correct Redis.
pub async fn reconcile_stock(
    conn: &mut RedisConnection,
    product_id: &str,
    threshold: i64,
) -> Result<ReconciliationReport, ReconciliationError> {
    let r_stock: i64 = cmd("GET")
        .arg(redis_key(product_id))
        .query_async(conn)
        .await
        .unwrap_or(0);
    let d_stock: i64 = cmd("GET")
        .arg(db_key(product_id))
        .query_async(conn)
        .await
        .unwrap_or(0);

    let diff = (r_stock - d_stock).abs();

    if diff > threshold {
        return Err(ReconciliationError::ThresholdExceeded {
            redis_stock: r_stock,
            db_stock: d_stock,
            diff,
        });
    }

    if r_stock == d_stock {
        return Ok(ReconciliationReport {
            product_id: product_id.to_string(),
            redis_stock: r_stock,
            db_stock: d_stock,
            in_sync: true,
            fix_applied: false,
            status: "In sync".to_string(),
        });
    }

    if r_stock < d_stock {
        // Redis ahead of DB -- normal during async writes
        // Redis has decremented more than DB. This is expected when
        // Redis writes complete before DB writes.
        Ok(ReconciliationReport {
            product_id: product_id.to_string(),
            redis_stock: r_stock,
            db_stock: d_stock,
            in_sync: false,
            fix_applied: false,
            status: format!(
                "Redis ahead of DB by {} (normal async lag during sale)",
                d_stock - r_stock
            ),
        })
    } else {
        // DB ahead of Redis -- error state
        // DB has decremented more than Redis. This means a Redis write
        // was lost. Fix Redis to match DB (source of truth).
        cmd("SET")
            .arg(redis_key(product_id))
            .arg(d_stock)
            .query_async::<()>(conn)
            .await?;

        Ok(ReconciliationReport {
            product_id: product_id.to_string(),
            redis_stock: d_stock, // After fix
            db_stock: d_stock,
            in_sync: true, // Now in sync after fix
            fix_applied: true,
            status: format!(
                "DB ahead of Redis by {} -- fixed Redis to match DB (source of truth)",
                r_stock - d_stock
            ),
        })
    }
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
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Pool error: {e}"))?;
        pool.get().await.map_err(|e| format!("Conn error: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection, product_id: &str) {
        let _: Result<(), _> = cmd("DEL")
            .arg(redis_key(product_id))
            .arg(db_key(product_id))
            .query_async(conn)
            .await;
    }

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
        assert!(report.in_sync);
        assert!(!report.fix_applied);
        cleanup(&mut conn, TEST_PRODUCT).await;
    }

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
        decrement_redis(&mut conn, TEST_PRODUCT)
            .await
            .expect("Redis decrement");
        let report = reconcile_stock(&mut conn, TEST_PRODUCT, 10)
            .await
            .expect("Should reconcile");
        assert!(!report.in_sync);
        assert_eq!(report.redis_stock, 99);
        assert_eq!(report.db_stock, 100);
        assert!(!report.fix_applied, "Should not fix Redis-ahead case");
        cleanup(&mut conn, TEST_PRODUCT).await;
    }

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
        decrement_db(&mut conn, TEST_PRODUCT)
            .await
            .expect("DB decrement");
        let report = reconcile_stock(&mut conn, TEST_PRODUCT, 10)
            .await
            .expect("Should reconcile");
        assert!(!report.in_sync);
        assert!(report.fix_applied);
        let redis_val: i64 = cmd("GET")
            .arg(redis_key(TEST_PRODUCT))
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(redis_val, 99, "Redis should be fixed to match DB");
        cleanup(&mut conn, TEST_PRODUCT).await;
    }

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
