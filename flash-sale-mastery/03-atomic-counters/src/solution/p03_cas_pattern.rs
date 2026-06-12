//! # Solution 03: Compare-and-Swap Pattern
//!
//! Complete implementation of CAS using WATCH/MULTI/EXEC with retry loop.

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
/// ## How It Works
///
/// 1. `WATCH key` -- tell Redis to monitor this key for changes
/// 2. `GET key` -- read the current value
/// 3. If current != expected, return false early (no transaction needed)
/// 4. `MULTI` / `SET key new_value` / `EXEC` -- atomically set the value
/// 5. If EXEC returns Nil (key changed after WATCH), return false
///
/// This is the fundamental optimistic concurrency control primitive.
pub async fn cas_update(
    conn: &mut RedisConnection,
    key: &str,
    expected: i64,
    new_value: i64,
) -> Result<bool, CasError> {
    // Step 1: WATCH the key
    cmd("WATCH")
        .arg(key)
        .query_async::<()>(conn)
        .await?;

    // Step 2: Read current value
    let current: Option<i64> = cmd("GET").arg(key).query_async(conn).await?;
    let current = current.unwrap_or(0);

    // Step 3: Early exit if value doesn't match
    if current != expected {
        return Ok(false);
    }

    // Step 4: Execute atomic transaction
    let mut pipe = Pipeline::new();
    pipe.atomic(); // Wraps in MULTI/EXEC
    pipe.set(key, new_value);
    let result: Value = pipe.query_async(conn).await?;

    // Step 5: If EXEC returned Nil, the transaction was aborted
    Ok(!matches!(result, Value::Nil))
}

/// Retry a CAS operation up to `max_retries` times.
///
/// On conflict, re-reads the current value and retries with the updated
/// expected value. Convergence is guaranteed if the key eventually stabilizes.
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
    for attempt in 0..max_retries {
        let new_value = new_fn(expected);
        match cas_update(conn, key, expected, new_value).await {
            Ok(true) => return Ok(new_value),
            Ok(false) => {
                // Conflict: re-read current value and retry
                let current: i64 = cmd("GET")
                    .arg(key)
                    .query_async(conn)
                    .await?;
                expected = current;
                tracing::debug!(
                    attempt,
                    current,
                    "CAS conflict, retrying with updated expected value"
                );
            }
            Err(e) => return Err(e),
        }
    }
    Err(CasError::MaxRetriesExceeded(max_retries))
}

/// Decrement stock using CAS pattern with zero guard.
pub async fn decrement_cas(
    conn: &mut RedisConnection,
    key: &str,
    max_retries: usize,
) -> Result<i64, CasError> {
    let current: Option<i64> = cmd("GET").arg(key).query_async(conn).await?;
    let current = current.unwrap_or(0);

    if current <= 0 {
        return Err(CasError::StockExhausted);
    }

    cas_retry_loop(conn, key, current, |v| if v > 0 { v - 1 } else { 0 }, max_retries).await
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
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Pool error: {e}"))?;
        pool.get().await.map_err(|e| format!("Conn error: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection) {
        let _: Result<(), _> = cmd("DEL").arg(TEST_KEY).query_async(conn).await;
    }

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
        let current: i64 = cmd("GET").arg(TEST_KEY).query_async(&mut conn).await.unwrap();
        assert_eq!(current, 99);
        cleanup(&mut conn).await;
    }

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
        let result = cas_update(&mut conn, TEST_KEY, 50, 49)
            .await
            .expect("CAS should not error");
        assert!(!result, "CAS should fail when value doesn't match");
        let current: i64 = cmd("GET").arg(TEST_KEY).query_async(&mut conn).await.unwrap();
        assert_eq!(current, 100, "Value should be unchanged");
        cleanup(&mut conn).await;
    }

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
        decrement_cas(&mut conn, TEST_KEY, 10).await.expect("Decrement 1");
        decrement_cas(&mut conn, TEST_KEY, 10).await.expect("Decrement 2");
        let result = decrement_cas(&mut conn, TEST_KEY, 10).await;
        assert!(result.is_err(), "Should fail when stock is 0");
        let stock: i64 = cmd("GET").arg(TEST_KEY).query_async(&mut conn).await.unwrap();
        assert_eq!(stock, 0, "Stock should be exactly 0");
        cleanup(&mut conn).await;
    }
}
