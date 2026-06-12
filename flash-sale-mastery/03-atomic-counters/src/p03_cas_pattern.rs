//! # Exercise 03: Compare-and-Swap Pattern
//!
//! ## Learning Objective
//! Implement the Compare-and-Swap (CAS) pattern using Redis WATCH/MULTI/EXEC
//! transactions. CAS is a fundamental concurrency control mechanism used in
//! databases, caches, and distributed systems.
//!
//! ## Flash Sale Context
//! Sometimes you need to update stock conditionally -- e.g., only decrement
//! if the current value matches what you expect. CAS prevents lost updates
//! when multiple operations read the same value and try to modify it. The
//! WATCH command monitors a key for changes; if it changes before EXEC,
//! the transaction is aborted and must be retried.
//!
//! ## Instructions
//! 1. Implement `cas_update` using WATCH/MULTI/EXEC
//! 2. Implement a `cas_retry_loop` that retries on conflict
//! 3. Implement `decrement_cas` that uses the retry loop to safely decrement
//!
//! ## Hints
//! - WATCH a key, then check its value inside MULTI/EXEC
//! - If EXEC returns None (aborted), retry the whole operation
//! - Set a maximum retry count to prevent infinite loops
//! - The CAS loop: WATCH -> read -> MULTI -> conditional write -> EXEC -> check
//!
//! ## Trade-offs
//! - **Pros:** Works without special server-side commands, general-purpose,
//!   no false negatives (always correct)
//! - **Cons:** High latency under contention (many retries), O(n) retries in
//!   worst case, extra round-trips for WATCH and read
//! - **When to use:** Complex conditional updates, low-to-moderate contention,
//!   when you need to update multiple keys atomically

use deadpool_redis::redis::{cmd, Pipeline, Value};
use deadpool_redis::Connection as RedisConnection;

/// Error type for CAS operations.
#[derive(Debug, thiserror::Error)]
pub enum CasError {
    #[error("Redis error: {0}")]
    RedisError(#[from] deadpool_redis::redis::RedisError),

    #[error("CAS conflict: value changed from {expected} to {actual}")]
    Conflict { expected: i64, actual: i64 },

    #[error("Max retries ({0}) exceeded")]
    MaxRetriesExceeded(usize),

    #[error("Stock exhausted")]
    StockExhausted,
}

/// Perform a single Compare-and-Swap attempt.
///
/// Watches the key, reads its value, and if it matches `expected`,
/// atomically sets it to `new_value` inside a transaction.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - The key to update
/// * `expected` - Expected current value
/// * `new_value` - Value to set if current matches expected
///
/// # Returns
/// `true` if the swap succeeded, `false` if the value changed (conflict).
pub async fn cas_update(
    conn: &mut RedisConnection,
    key: &str,
    expected: i64,
    new_value: i64,
) -> Result<bool, CasError> {
    // TODO: WATCH the key to monitor for changes
    // TODO: GET the current value
    // TODO: If current != expected, return false (conflict detected early)
    // TODO: Build an atomic pipeline (MULTI/EXEC) with SET key new_value
    // TODO: Execute the pipeline and check the result
    //   - If EXEC succeeded (result is not Nil), return true
    //   - If EXEC was aborted (result is Nil), return false
    todo!("Implement single CAS attempt using WATCH/MULTI/EXEC")
}

/// Retry a CAS operation up to `max_retries` times.
///
/// On conflict, re-reads the current value and retries with the updated
/// expected value. This is the standard CAS retry pattern.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - The key to update
/// * `expected` - Initial expected value
/// * `new_fn` - Function that computes the new value from the current value
/// * `max_retries` - Maximum number of retry attempts
pub async fn cas_retry_loop<F>(
    conn: &mut RedisConnection,
    key: &str,
    mut expected: i64,
    new_fn: F,
    max_retries: usize,
) -> Result<i64, CasError>
where
    F: Fn(i64) -> i64,
{
    // TODO: Loop up to max_retries times:
    //   1. Call cas_update(conn, key, expected, new_fn(expected))
    //   2. If Ok(true), return the new value
    //   3. If Ok(false), read the current value and update expected
    //   4. If Err, propagate
    // TODO: If loop exits without success, return MaxRetriesExceeded
    todo!("Implement CAS retry loop")
}

/// Decrement stock using CAS pattern.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Stock counter key
/// * `max_retries` - Maximum retry attempts
pub async fn decrement_cas(
    conn: &mut RedisConnection,
    key: &str,
    max_retries: usize,
) -> Result<i64, CasError> {
    // TODO: Read the current stock value
    // TODO: If current <= 0, return StockExhausted
    // TODO: Use cas_retry_loop with a function that:
    //   - Returns v - 1 if v > 0
    //   - Returns 0 if v <= 0 (guard)
    todo!("Implement CAS-based decrement")
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";
    const TEST_KEY: &str = "test:atomic:counter:p03";

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

    /// Test successful CAS swap when value matches.
    #[tokio::test]
    async fn test_successful_swap() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;
        let _: () = cmd("SET")
            .arg(TEST_KEY)
            .arg(100i64)
            .query_async(&mut conn)
            .await
            .unwrap();

        let result = cas_update(&mut conn, TEST_KEY, 100, 99)
            .await
            .expect("CAS should succeed");
        assert!(result, "CAS should succeed when value matches");

        let current: i64 = cmd("GET")
            .arg(TEST_KEY)
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(current, 99);
        cleanup(&mut conn).await;
    }

    /// Test failed CAS when value has changed.
    #[tokio::test]
    async fn test_failed_swap() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;
        let _: () = cmd("SET")
            .arg(TEST_KEY)
            .arg(100i64)
            .query_async(&mut conn)
            .await
            .unwrap();

        // Try to CAS with wrong expected value
        let result = cas_update(&mut conn, TEST_KEY, 50, 49)
            .await
            .expect("CAS should not error");
        assert!(!result, "CAS should fail when value doesn't match");

        let current: i64 = cmd("GET")
            .arg(TEST_KEY)
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(current, 100, "Value should be unchanged");
        cleanup(&mut conn).await;
    }

    /// Test CAS retry loop convergence.
    #[tokio::test]
    async fn test_retry_loop_convergence() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;
        let _: () = cmd("SET")
            .arg(TEST_KEY)
            .arg(100i64)
            .query_async(&mut conn)
            .await
            .unwrap();

        let new_val = cas_retry_loop(&mut conn, TEST_KEY, 100, |v| v - 1, 10)
            .await
            .expect("Retry loop should converge");
        assert_eq!(new_val, 99);
        cleanup(&mut conn).await;
    }

    /// Test stock never goes negative with CAS decrement.
    #[tokio::test]
    async fn test_cas_stock_never_negative() {
        let mut conn = match get_conn().await {
            Ok(c) => c,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        cleanup(&mut conn).await;
        let _: () = cmd("SET")
            .arg(TEST_KEY)
            .arg(2i64)
            .query_async(&mut conn)
            .await
            .unwrap();

        decrement_cas(&mut conn, TEST_KEY, 10)
            .await
            .expect("Decrement 1");
        decrement_cas(&mut conn, TEST_KEY, 10)
            .await
            .expect("Decrement 2");
        let result = decrement_cas(&mut conn, TEST_KEY, 10).await;
        assert!(result.is_err(), "Should fail when stock is 0");

        let stock: i64 = cmd("GET")
            .arg(TEST_KEY)
            .query_async(&mut conn)
            .await
            .unwrap();
        assert_eq!(stock, 0, "Stock should be exactly 0");
        cleanup(&mut conn).await;
    }
}
