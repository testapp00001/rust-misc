//! # Solution 10: Lua Error Patterns
//!
//! Complete implementation of error handling patterns within Lua scripts.

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// Demonstrates the difference between `redis.call` (raises) and
/// `redis.pcall` (returns error).
const PCALL_DEMO_SCRIPT: &str = r#"
-- pcall catches errors from redis.call, preventing script abort
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
const TYPE_ERROR_SCRIPT: &str = r#"
-- Sets a key as a string, then tries to LPUSH (list operation).
-- pcall catches the WRONGTYPE error gracefully.
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
const STRUCTURED_ERROR_SCRIPT: &str = r#"
-- KEYS[1] = key to check
-- ARGV[1] = expected type ("string", "list", "set", "hash")

-- Step 1: Check if key exists
local exists = redis.call('EXISTS', KEYS[1])
if exists == 0 then
    return {0, "NOT_FOUND", ""}
end

-- Step 2: Get actual type
local type_result = redis.call('TYPE', KEYS[1])
-- Handle Redis 7.2+ where TYPE returns {ok="string"} instead of plain string
local actual_type
if type(type_result) == "table" then
    actual_type = type_result.ok
else
    actual_type = type_result
end

-- Step 3: Compare types
if actual_type ~= ARGV[1] then
    return {0, "WRONG_TYPE", actual_type}
end

-- Step 4: Get value based on type
local value
if actual_type == "string" then
    value = redis.call('GET', KEYS[1])
elseif actual_type == "list" then
    value = redis.call('LRANGE', KEYS[1], 0, -1)
elseif actual_type == "set" then
    value = redis.call('SMEMBERS', KEYS[1])
elseif actual_type == "hash" then
    value = redis.call('HGETALL', KEYS[1])
else
    value = tostring(redis.call('GET', KEYS[1]) or '')
end

return {1, tostring(value), ""}
"#;

/// Demonstrates pcall vs redis.call behavior.
pub async fn demonstrate_pcall(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<String> {
    let result: String = Script::new(PCALL_DEMO_SCRIPT)
        .key(key)
        .invoke_async(conn)
        .await?;
    Ok(result)
}

/// Demonstrates type error handling.
pub async fn demonstrate_type_error(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<String> {
    let result: String = Script::new(TYPE_ERROR_SCRIPT)
        .key(key)
        .invoke_async(conn)
        .await?;
    Ok(result)
}

/// Run the structured error script and return the result.
///
/// Returns a tuple: (success: bool, value_or_code: String, detail: String)
pub async fn structured_error_check(
    conn: &mut RedisConnection,
    key: &str,
    expected_type: &str,
) -> Result<(bool, String, String)> {
    let result: Vec<redis::Value> = Script::new(STRUCTURED_ERROR_SCRIPT)
        .key(key)
        .arg(expected_type)
        .invoke_async(conn)
        .await?;

    let extract = |v: &redis::Value| -> String {
        match v {
            redis::Value::BulkString(bytes) => String::from_utf8_lossy(bytes).to_string(),
            redis::Value::SimpleString(s) => s.clone(),
            other => format!("{other:?}"),
        }
    };

    match result.as_slice() {
        [redis::Value::Int(1), value @ _, detail @ _] => {
            Ok((true, extract(value), extract(detail)))
        }
        [redis::Value::Int(0), code @ _, detail @ _] => {
            Ok((false, extract(code), extract(detail)))
        }
        _ => Err(anyhow::anyhow!(
            "Unexpected structured error response: {result:?}"
        )),
    }
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
        let cfg = Config::from_url(REDIS_URL);
        cfg.create_pool(Some(Runtime::Tokio1))
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
