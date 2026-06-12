//! # Exercise 07: Atomic Idempotency Check
//!
//! ## Learning Objective
//! Build an atomic check-and-set for idempotency keys. A request ID is
//! either new (store the result and proceed) or a duplicate (return the
//! cached result without reprocessing).
//!
//! ## Flash Sale Context
//! Network retries, client-side retry logic, and message queue redelivery
//! can cause the same logical request to arrive multiple times. Without
//! idempotency, a retry could decrement stock again or issue a second
//! voucher. The idempotency key ensures exactly-once semantics.
//!
//! ## Instructions
//! 1. Define a Lua script that atomically checks for an existing result
//!    and stores a new one if absent
//! 2. Implement `check_idempotency` returning `IdempotencyResult`
//! 3. Implement `cleanup_idempotency` for test teardown
//!
//! ## Hints
//! - `GET` returns nil/false for missing keys in Lua
//! - Use `SETEX key ttl value` to store with a TTL
//! - Return {1, ""} for a new request, {0, cached_value} for a duplicate

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// Result of an idempotency check.
#[derive(Debug, Clone, PartialEq)]
pub enum IdempotencyResult {
    /// This is the first time this request ID has been seen.
    FirstRequest,
    /// This request ID was already processed. Contains the cached result.
    Duplicate { cached_result: String },
}

/// Atomic idempotency check-and-set Lua script.
///
/// KEYS[1]: idempotency key
/// ARGV[1]: result JSON to store (if first request)
/// ARGV[2]: TTL in seconds
///
/// Returns: {1, ""} if first request (result stored),
///          {0, cached_value} if duplicate.
const IDEMPOTENCY_SCRIPT: &str = r#"
-- KEYS[1] = idempotency key
-- ARGV[1] = result_json to store
-- ARGV[2] = ttl_seconds

-- TODO: Implement atomic idempotency check
-- 1. GET KEYS[1]
-- 2. If exists, return {0, existing_value}
-- 3. Otherwise SETEX KEYS[1] ttl result_json, return {1, ""}
"#;

/// Check whether a request ID has been processed before.
///
/// On `FirstRequest`, the caller should proceed and the result has been
/// stored with the configured TTL. On `Duplicate`, the cached result is
/// returned so the caller can replay it.
pub async fn check_idempotency(
    conn: &mut RedisConnection,
    idempotency_key: &str,
    result_json: &str,
    ttl_secs: u64,
) -> Result<IdempotencyResult> {
    // TODO: Invoke the script and parse the result
    todo!("Implement idempotency check using IDEMPOTENCY_SCRIPT")
}

/// Clean up an idempotency key (test helper).
pub async fn cleanup_idempotency(
    conn: &mut RedisConnection,
    idempotency_key: &str,
) -> Result<()> {
    // TODO: DEL the key
    todo!("Implement: delete the idempotency key")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Pool, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn try_connect() -> Result<Pool, String> {
        Config::from_url(REDIS_URL)
            .builder(Some(Runtime::Tokio1))
            .max_size(32)
            .build()
            .map_err(|e| format!("Could not create pool: {e}"))
    }

    /// First request should return FirstRequest.
    #[tokio::test]
    async fn test_first_request() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_idemp_first";

        cleanup_idempotency(&mut conn, key).await.unwrap();

        let result = check_idempotency(&mut conn, key, r#"{"status":"ok"}"#, 60)
            .await
            .unwrap();
        assert_eq!(result, IdempotencyResult::FirstRequest);

        cleanup_idempotency(&mut conn, key).await.unwrap();
    }

    /// Duplicate request should return the cached result.
    #[tokio::test]
    async fn test_duplicate_request() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_idemp_dup";

        cleanup_idempotency(&mut conn, key).await.unwrap();

        let cached = r#"{"voucher":"VCHR-001"}"#;
        let r1 = check_idempotency(&mut conn, key, cached, 60)
            .await
            .unwrap();
        assert_eq!(r1, IdempotencyResult::FirstRequest);

        let r2 = check_idempotency(&mut conn, key, "should-be-ignored", 60)
            .await
            .unwrap();
        assert_eq!(
            r2,
            IdempotencyResult::Duplicate {
                cached_result: cached.to_string()
            }
        );

        cleanup_idempotency(&mut conn, key).await.unwrap();
    }

    /// Different keys should be independent.
    #[tokio::test]
    async fn test_independent_keys() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");

        cleanup_idempotency(&mut conn, "test_idemp_ind_a").await.unwrap();
        cleanup_idempotency(&mut conn, "test_idemp_ind_b").await.unwrap();

        let r1 = check_idempotency(&mut conn, "test_idemp_ind_a", "result_a", 60)
            .await
            .unwrap();
        let r2 = check_idempotency(&mut conn, "test_idemp_ind_b", "result_b", 60)
            .await
            .unwrap();

        assert_eq!(r1, IdempotencyResult::FirstRequest);
        assert_eq!(r2, IdempotencyResult::FirstRequest);

        cleanup_idempotency(&mut conn, "test_idemp_ind_a").await.unwrap();
        cleanup_idempotency(&mut conn, "test_idemp_ind_b").await.unwrap();
    }

    /// CRITICAL: 1000 concurrent requests with the same key.
    /// Exactly 1 should be FirstRequest; the rest must be Duplicate.
    #[test]
    fn test_concurrent_duplicates() {
        let pool = match tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(try_connect())
        {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let key = "concurrent_idemp";

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            cleanup_idempotency(&mut conn, key).await.unwrap();
        });

        let mut handles = vec![];
        for i in 0..1000 {
            let p = pool.clone();
            let k = key.to_string();
            handles.push(std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut conn = p.get().await.unwrap();
                    check_idempotency(
                        &mut conn,
                        &k,
                        &format!(r#"{{"thread":{i}}}"#),
                        60,
                    )
                    .await
                })
            }));
        }

        let mut first_count = 0i64;
        let mut dup_count = 0i64;
        for handle in handles {
            match handle.join().unwrap() {
                Ok(IdempotencyResult::FirstRequest) => first_count += 1,
                Ok(IdempotencyResult::Duplicate { .. }) => dup_count += 1,
                Err(e) => panic!("Unexpected error: {e}"),
            }
        }

        assert_eq!(first_count, 1, "Exactly 1 FirstRequest");
        assert_eq!(dup_count, 999, "999 should be Duplicate");

        rt.block_on(async {
            let mut conn = pool.get().await.unwrap();
            cleanup_idempotency(&mut conn, key).await.unwrap();
        });
    }
}
