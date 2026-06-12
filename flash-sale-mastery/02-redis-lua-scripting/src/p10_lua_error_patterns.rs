//! # Exercise 10: Lua Error Patterns
//!
//! ## Learning Objective
//! Understand how errors propagate from Lua scripts to the Redis client,
//! how to use `pcall` for protected calls within Lua, and how to return
//! structured error information instead of letting scripts abort.
//!
//! ## Flash Sale Context
//! A Lua script that calls `redis.call()` on a key of the wrong type
//! (e.g. SADD on a string key) raises a runtime error that aborts the
//! script and returns an error to the client. In production you want
//! graceful error handling -- structured error codes, not raw Redis errors.
//!
//! ## Instructions
//! 1. Implement `demonstrate_pcall` showing pcall vs redis.call
//! 2. Implement `demonstrate_type_error` showing type mismatch handling
//! 3. Implement `structured_error_script` returning error codes as arrays
//!
//! ## Hints
//! - `redis.call()` raises an error on failure (aborts the script)
//! - `redis.pcall()` returns `nil, error_message` on failure (no abort)
//! - Lua `pcall(fn)` catches any Lua error: returns `true, result` or `false, err`
//! - Return `{0, error_code, error_message}` for structured errors

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// Demonstrates the difference between `redis.call` (raises) and
/// `redis.pcall` (returns error).
///
/// Returns "call_result:pcall_result" where each part is either the
/// value or "ERROR:description".
pub const PCALL_DEMO_SCRIPT: &str = r#"
-- This script demonstrates pcall vs call
-- KEYS[1] = a key that might be the wrong type
--
-- We use pcall to safely attempt an operation that might fail
-- (e.g. trying to increment a set member as if it were a number)

local ok, result = pcall(function()
    return redis.call('INCR', KEYS[1])
end)

if ok then
    return "OK:" .. tostring(result)
else
    return "ERROR:" .. tostring(result)
end
"#;

/// Demonstrates type error handling.
///
/// Sets a key to a string value, then tries to treat it as a list (LPUSH).
/// Returns "CAUGHT:error_message" on type mismatch, or "UNEXPECTED" if
/// the error does not occur.
const TYPE_ERROR_SCRIPT: &str = r#"
-- KEYS[1] = a key we will set as string, then try to LPUSH
redis.call('SET', KEYS[1], 'not_a_list')

local ok, err = pcall(function()
    redis.call('LPUSH', KEYS[1], 'value')
end)

if ok then
    return "UNEXPECTED"
else
    return "CAUGHT:" .. tostring(err)
end
"#;

/// Returns structured error information as a Lua array.
///
/// Returns: {1, value} on success, {0, error_code, error_message} on error.
///
/// Demonstrates how to build a structured error protocol inside Lua
/// so the Rust side can pattern-match on error codes instead of parsing
/// raw Redis error strings.
const STRUCTURED_ERROR_SCRIPT: &str = r#"
-- KEYS[1] = key to check
-- ARGV[1] = expected type ("string", "list", "set", "hash")
-- Returns:
--   {1, value}                       if type matches
--   {0, "WRONG_TYPE", actual_type}   if type mismatch
--   {0, "NOT_FOUND", ""}             if key does not exist

-- TODO: Implement structured error handling
-- 1. Check if key exists: redis.call('EXISTS', KEYS[1])
-- 2. If not exists, return {0, "NOT_FOUND", ""}
-- 3. Get actual type: redis.call('TYPE', KEYS[1])
-- 4. If type != ARGV[1], return {0, "WRONG_TYPE", actual_type}
-- 5. Otherwise get value and return {1, value}
"#;

/// Demonstrates pcall vs redis.call behavior.
pub async fn demonstrate_pcall(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<String> {
    // TODO: Invoke PCALL_DEMO_SCRIPT
    todo!("Implement: run the pcall demo script")
}

/// Demonstrates type error handling.
pub async fn demonstrate_type_error(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<String> {
    // TODO: Invoke TYPE_ERROR_SCRIPT
    todo!("Implement: run the type error demo script")
}

/// Run the structured error script and return the result.
///
/// Returns a tuple: (success: bool, value_or_code: String, detail: String)
pub async fn structured_error_check(
    conn: &mut RedisConnection,
    key: &str,
    expected_type: &str,
) -> Result<(bool, String, String)> {
    // TODO: Invoke STRUCTURED_ERROR_SCRIPT and parse the array
    todo!("Implement: run structured error script and parse result")
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
            .max_size(16)
            .build()
            .map_err(|e| format!("Could not create pool: {e}"))
    }

    async fn cleanup(conn: &mut RedisConnection, key: &str) {
        let _: () = redis::cmd("DEL")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .unwrap_or(());
    }

    /// pcall should succeed on a valid INCR operation.
    #[tokio::test]
    async fn test_pcall_success() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_pcall_ok";

        cleanup(&mut conn, key).await;

        let result = demonstrate_pcall(&mut conn, key).await.unwrap();
        assert!(result.starts_with("OK:"), "Expected OK, got: {result}");

        cleanup(&mut conn, key).await;
    }

    /// pcall should catch a type error gracefully.
    #[tokio::test]
    async fn test_pcall_type_error() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_pcall_type_err";

        cleanup(&mut conn, key).await;

        // Set up a set type (not a string)
        let _: () = redis::cmd("SADD")
            .arg(key)
            .arg("member1")
            .query_async(&mut *conn)
            .await
            .unwrap();

        // INCR on a set key should fail, but pcall catches it
        let result = demonstrate_pcall(&mut conn, key).await.unwrap();
        assert!(
            result.starts_with("ERROR:"),
            "Expected ERROR for type mismatch, got: {result}"
        );

        cleanup(&mut conn, key).await;
    }

    /// Type error script should catch LPUSH on a string key.
    #[tokio::test]
    async fn test_type_error_caught() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_type_err";

        cleanup(&mut conn, key).await;

        let result = demonstrate_type_error(&mut conn, key).await.unwrap();
        assert!(
            result.starts_with("CAUGHT:"),
            "Expected CAUGHT for type error, got: {result}"
        );
        assert!(
            result.contains("WRONGTYPE") || result.contains("wrong"),
            "Error should mention type mismatch"
        );

        cleanup(&mut conn, key).await;
    }

    /// Structured error: key exists and type matches.
    #[tokio::test]
    async fn test_structured_success() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_struct_ok";

        cleanup(&mut conn, key).await;

        let _: () = redis::cmd("SET")
            .arg(key)
            .arg("hello")
            .query_async(&mut *conn)
            .await
            .unwrap();

        let (success, value, detail) =
            structured_error_check(&mut conn, key, "string").await.unwrap();
        assert!(success, "Should succeed for matching type");
        assert_eq!(value, "hello");
        assert!(detail.is_empty());

        cleanup(&mut conn, key).await;
    }

    /// Structured error: key does not exist.
    #[tokio::test]
    async fn test_structured_not_found() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_struct_missing";

        cleanup(&mut conn, key).await;

        let (success, code, _) =
            structured_error_check(&mut conn, key, "string").await.unwrap();
        assert!(!success, "Should fail for missing key");
        assert_eq!(code, "NOT_FOUND");
    }

    /// Structured error: key exists but type does not match.
    #[tokio::test]
    async fn test_structured_wrong_type() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_struct_wrong";

        cleanup(&mut conn, key).await;

        let _: () = redis::cmd("SET")
            .arg(key)
            .arg("value")
            .query_async(&mut *conn)
            .await
            .unwrap();

        let (success, code, actual_type) =
            structured_error_check(&mut conn, key, "list").await.unwrap();
        assert!(!success, "Should fail for type mismatch");
        assert_eq!(code, "WRONG_TYPE");
        assert_eq!(actual_type, "string");

        cleanup(&mut conn, key).await;
    }
}
