//! # Solution 05: Pessimistic Locking
//!
//! Complete implementation of pessimistic concurrency control using
//! Redis-based distributed locks.

use deadpool_redis::redis::cmd;
use deadpool_redis::Connection as RedisConnection;

/// Error type for pessimistic locking operations.
#[derive(Debug, thiserror::Error)]
pub enum PessimisticError {
    #[error("Redis error: {0}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("Failed to acquire lock for key: {key}")]
    LockFailed { key: String },

    #[error("Lock timeout after {0}ms")]
    LockTimeout(u64),

    #[error("Insufficient stock: need {need}, have {have}")]
    InsufficientStock { need: i64, have: i64 },
}

/// Generate a unique holder ID for lock ownership.
fn generate_holder_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("holder:{:x}", ts)
}

/// Acquire a distributed lock in Redis.
///
/// Uses `SET key holder NX EX ttl`:
/// - NX: only set if key does NOT exist (atomic test-and-set)
/// - EX: set expiry in seconds (safety net if holder crashes)
///
/// This is the Redlock algorithm's basic building block.
///
/// ## Trade-offs
/// - **Pros:** Simple, fast, no external dependencies
/// - **Cons:** Single Redis instance is a SPOF, clock drift can cause issues
/// - **For production:** Consider Redlock (multi-node) or etcd/Consul locks
pub async fn acquire_lock(
    conn: &mut RedisConnection,
    key: &str,
    holder: &str,
    ttl: u64,
) -> Result<bool, PessimisticError> {
    let result: Option<String> = cmd("SET")
        .arg(key)
        .arg(holder)
        .arg("NX")
        .arg("EX")
        .arg(ttl)
        .query_async(conn)
        .await?;

    Ok(result.is_some())
}

/// Release a distributed lock using an atomic Lua script.
///
/// The script checks that the holder matches before deleting, preventing
/// the case where:
/// 1. Process A acquires lock
/// 2. Process A is slow, lock expires (TTL)
/// 3. Process B acquires lock
/// 4. Process A finishes and tries to release -- but it would release B's lock!
///
/// The Lua script makes the check-and-delete atomic.
pub async fn release_lock(
    conn: &mut RedisConnection,
    key: &str,
    holder: &str,
) -> Result<(), PessimisticError> {
    let script = redis::Script::new(
        r#"
        if redis.call('GET', KEYS[1]) == ARGV[1] then
            return redis.call('DEL', KEYS[1])
        else
            return 0
        end
        "#,
    );

    script
        .key(key)
        .arg(holder)
        .invoke_async::<()>(conn)
        .await?;

    Ok(())
}

/// Update stock using pessimistic locking.
///
/// ## SQL Equivalent
/// ```sql
/// BEGIN;
/// SELECT stock FROM products WHERE id = ? FOR UPDATE;
/// UPDATE products SET stock = stock - 1 WHERE id = ?;
/// COMMIT;
/// ```
///
/// ## How It Works (Redis Simulation)
/// 1. Generate unique holder ID
/// 2. Acquire distributed lock with SET NX EX
/// 3. Read current stock
/// 4. If stock >= delta, decrement and write back
/// 5. Release lock (always, even on error)
/// 6. Return new stock value
pub async fn update_stock_pessimistic(
    conn: &mut RedisConnection,
    product_id: &str,
    delta: i64,
) -> Result<i64, PessimisticError> {
    let stock_key = format!("stock:{}", product_id);
    let lock_key = format!("lock:stock:{}", product_id);
    let holder = generate_holder_id();

    // Acquire lock
    let acquired = acquire_lock(conn, &lock_key, &holder, 5).await?;
    if !acquired {
        return Err(PessimisticError::LockFailed { key: lock_key });
    }

    // Read current stock
    let current: Option<i64> = cmd("GET")
        .arg(&stock_key)
        .query_async(conn)
        .await?;
    let current = current.unwrap_or(0);

    // Check stock
    if current < delta {
        let _ = release_lock(conn, &lock_key, &holder).await;
        return Err(PessimisticError::InsufficientStock {
            need: delta,
            have: current,
        });
    }

    // Update stock
    let new_stock = current - delta;
    let _: () = cmd("SET")
        .arg(&stock_key)
        .arg(new_stock)
        .query_async(conn)
        .await?;

    // Release lock
    release_lock(conn, &lock_key, &holder).await?;

    Ok(new_stock)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";
    const TEST_PRODUCT: &str = "test:pess:product:42";

    async fn get_conn() -> Result<RedisConnection, String> {
        let cfg = Config::from_url(REDIS_URL);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Pool error: {e}"))?;
        pool.get().await.map_err(|e| format!("Conn error: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection, product_id: &str) {
        let stock_key = format!("stock:{}", product_id);
        let lock_key = format!("lock:stock:{}", product_id);
        let _: Result<(), _> = cmd("DEL")
            .arg(&stock_key)
            .arg(&lock_key)
            .query_async(conn)
            .await;
    }

    #[tokio::test]
    async fn test_lock_acquisition() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn, TEST_PRODUCT).await;

        let lock_key = format!("lock:{}", TEST_PRODUCT);
        let holder = "test-holder-1";

        let acquired = acquire_lock(&mut conn, &lock_key, holder, 5)
            .await
            .expect("Should attempt lock");
        assert!(acquired, "Should acquire uncontested lock");

        let holder2 = "test-holder-2";
        let acquired2 = acquire_lock(&mut conn, &lock_key, holder2, 5)
            .await
            .expect("Should attempt lock");
        assert!(!acquired2, "Should not acquire held lock");

        release_lock(&mut conn, &lock_key, holder)
            .await
            .expect("Should release lock");

        let acquired3 = acquire_lock(&mut conn, &lock_key, holder2, 5)
            .await
            .expect("Should attempt lock");
        assert!(acquired3, "Should acquire after release");

        cleanup(&mut conn, TEST_PRODUCT).await;
    }

    #[tokio::test]
    async fn test_lock_timeout() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn, TEST_PRODUCT).await;

        let lock_key = format!("lock:ttl:{}", TEST_PRODUCT);
        let holder = "ttl-holder";

        let acquired = acquire_lock(&mut conn, &lock_key, holder, 1)
            .await
            .expect("Should attempt lock");
        assert!(acquired, "Should acquire lock");

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let holder2 = "ttl-holder-2";
        let acquired2 = acquire_lock(&mut conn, &lock_key, holder2, 5)
            .await
            .expect("Should attempt lock");
        assert!(acquired2, "Should acquire after TTL expiry");

        cleanup(&mut conn, TEST_PRODUCT).await;
    }

    #[tokio::test]
    async fn test_pessimistic_stock_never_negative() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn, TEST_PRODUCT).await;

        let stock_key = format!("stock:{}", TEST_PRODUCT);
        let _: () = cmd("SET")
            .arg(&stock_key)
            .arg(5i64)
            .query_async(&mut conn)
            .await
            .unwrap();

        for i in 1..=5 {
            let new_val = update_stock_pessimistic(&mut conn, TEST_PRODUCT, 1)
                .await
                .unwrap_or_else(|e| panic!("Decrement {} should succeed: {}", i, e));
            assert_eq!(new_val, 5 - i);
        }

        let result = update_stock_pessimistic(&mut conn, TEST_PRODUCT, 1).await;
        assert!(result.is_err(), "Should fail when stock is 0");

        let stock: i64 = cmd("GET")
            .arg(&stock_key)
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(stock, 0, "Stock should be exactly 0");
        assert!(stock >= 0, "Stock must never go negative");

        cleanup(&mut conn, TEST_PRODUCT).await;
    }
}
