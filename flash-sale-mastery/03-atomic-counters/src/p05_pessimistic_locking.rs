//! # Exercise 05: Pessimistic Locking
//!
//! ## Learning Objective
//! Implement pessimistic concurrency control using distributed locks.
//! This simulates the `SELECT FOR UPDATE` pattern from relational
//! databases, where a row is locked before reading and modifying it.
//!
//! ## Flash Sale Context
//! Pessimistic locking acquires an exclusive lock before reading stock,
//! ensuring no other request can modify it until we release the lock.
//! This prevents all conflicts but reduces throughput because requests
//! serialize through the lock.
//!
//! SQL equivalent:
//! ```sql
//! BEGIN;
//! SELECT stock FROM products WHERE id = ? FOR UPDATE;
//! -- stock is now locked, no other transaction can read it
//! UPDATE products SET stock = stock - 1 WHERE id = ?;
//! COMMIT;
//! ```
//!
//! ## Instructions
//! 1. Implement `acquire_lock` using Redis `SET key holder NX EX ttl`
//! 2. Implement `release_lock` using a Lua script for atomic check-and-delete
//! 3. Implement `update_stock_pessimistic` that acquires lock, reads, updates, releases
//!
//! ## Hints
//! - `SET key value NX EX ttl` only sets if key doesn't exist, with expiry
//! - Release must check the holder matches (prevent releasing someone else's lock)
//! - Use a Lua script for atomic release: `if redis.call('GET', KEYS[1]) == ARGV[1] then redis.call('DEL', KEYS[1]) end`
//! - Generate a unique holder ID (e.g., UUID) for each lock acquisition
//!
//! ## Trade-offs
//! - **Pros:** Simple mental model, no wasted work (no retries), guaranteed progress
//! - **Cons:** Reduced throughput (serialized access), risk of deadlock, lock holder
//!   crash can block others until TTL expires, distributed lock complexity
//! - **When to use:** High contention scenarios, when retry cost is high,
//!   operations that must complete within a time bound

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
///
/// In production, this would be a UUID or pod ID + request ID.
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
/// Uses `SET key holder NX EX ttl` to atomically create a lock
/// only if it doesn't already exist, with an automatic expiry.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Lock key (e.g., "lock:stock:product:42")
/// * `holder` - Unique ID identifying who holds this lock
/// * `ttl` - Lock time-to-live in seconds (auto-release safety net)
///
/// # Returns
/// `true` if the lock was acquired, `false` if already held by another.
pub async fn acquire_lock(
    conn: &mut RedisConnection,
    key: &str,
    holder: &str,
    ttl: u64,
) -> Result<bool, PessimisticError> {
    // TODO: Use SET key holder NX EX ttl
    // TODO: The result is "OK" if acquired, Nil if already locked
    // TODO: Return true if "OK", false if Nil
    todo!("Implement distributed lock acquisition")
}

/// Release a distributed lock in Redis.
///
/// Uses a Lua script to atomically check the holder and delete the key.
/// This prevents releasing a lock that was acquired by another process
/// after our lock expired.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Lock key
/// * `holder` - Our holder ID (must match current lock holder)
pub async fn release_lock(
    conn: &mut RedisConnection,
    key: &str,
    holder: &str,
) -> Result<(), PessimisticError> {
    // TODO: Use a Lua script:
    //   if redis.call('GET', KEYS[1]) == ARGV[1] then
    //       return redis.call('DEL', KEYS[1])
    //   else
    //       return 0
    //   end
    // TODO: Execute the script with key and holder as arguments
    todo!("Implement atomic lock release")
}

/// Update stock using pessimistic locking.
///
/// Acquires a distributed lock, reads current stock, decrements if
/// sufficient, and releases the lock.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `product_id` - Product identifier
/// * `delta` - Amount to decrement (typically 1)
///
/// # Returns
/// The new stock value after decrement.
pub async fn update_stock_pessimistic(
    conn: &mut RedisConnection,
    product_id: &str,
    delta: i64,
) -> Result<i64, PessimisticError> {
    // TODO: Build lock key from product_id
    // TODO: Generate holder ID
    // TODO: Try to acquire lock (with reasonable TTL, e.g., 5 seconds)
    // TODO: If lock failed, return LockFailed error
    // TODO: Read current stock from the product key
    // TODO: If stock < delta, release lock and return InsufficientStock
    // TODO: Update stock (SET or DECRBY)
    // TODO: Release lock
    // TODO: Return new stock value
    todo!("Implement pessimistic stock update")
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
            .builder(Some(Runtime::Tokio1))
            .build()
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

    /// Test lock acquisition and release.
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

        // Second acquisition should fail (different holder)
        let holder2 = "test-holder-2";
        let acquired2 = acquire_lock(&mut conn, &lock_key, holder2, 5)
            .await
            .expect("Should attempt lock");
        assert!(!acquired2, "Should not acquire held lock");

        // Release and re-acquire
        release_lock(&mut conn, &lock_key, holder)
            .await
            .expect("Should release lock");

        let acquired3 = acquire_lock(&mut conn, &lock_key, holder2, 5)
            .await
            .expect("Should attempt lock");
        assert!(acquired3, "Should acquire after release");

        cleanup(&mut conn, TEST_PRODUCT).await;
    }

    /// Test lock auto-expiry (TTL).
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

        // Acquire with 1-second TTL
        let acquired = acquire_lock(&mut conn, &lock_key, holder, 1)
            .await
            .expect("Should attempt lock");
        assert!(acquired, "Should acquire lock");

        // Wait for TTL to expire
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Another holder should now be able to acquire
        let holder2 = "ttl-holder-2";
        let acquired2 = acquire_lock(&mut conn, &lock_key, holder2, 5)
            .await
            .expect("Should attempt lock");
        assert!(acquired2, "Should acquire after TTL expiry");

        cleanup(&mut conn, TEST_PRODUCT).await;
    }

    /// Test stock update with pessimistic locking -- stock never goes negative.
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

        // Initialize stock
        let stock_key = format!("stock:{}", TEST_PRODUCT);
        let _: () = cmd("SET")
            .arg(&stock_key)
            .arg(5i64)
            .query_async(&mut conn)
            .await
            .unwrap();

        // Decrement 5 times (should all succeed)
        for i in 1..=5 {
            let new_val = update_stock_pessimistic(&mut conn, TEST_PRODUCT, 1)
                .await
                .unwrap_or_else(|e| panic!("Decrement {} should succeed: {}", i, e));
            assert_eq!(new_val, 5 - i);
        }

        // 6th decrement should fail
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
