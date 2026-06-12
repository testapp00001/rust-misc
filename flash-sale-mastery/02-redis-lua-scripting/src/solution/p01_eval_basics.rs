//! # Solution 01: Redis EVAL Basics
//!
//! Complete implementation of EVAL and EVALSHA fundamentals.

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// A trivial Lua script that returns the integer 42.
const SIMPLE_SCRIPT: &str = r#"
return 42
"#;

/// A Lua script that concatenates all KEYS and ARGV into a formatted string.
///
/// Returns "key1,key2:arg1,arg2" (keys comma-joined, colon, args comma-joined).
const KEYS_ARGS_SCRIPT: &str = r#"
local result = ''
for i, key in ipairs(KEYS) do
    result = result .. key
    if i < #KEYS then result = result .. ',' end
end
result = result .. ':'
for i, arg in ipairs(ARGV) do
    result = result .. arg
    if i < #ARGV then result = result .. ',' end
end
return result
"#;

/// Execute a simple Lua script that returns the integer 42.
pub async fn eval_simple_script(conn: &mut RedisConnection) -> Result<i64> {
    let result: i64 = Script::new(SIMPLE_SCRIPT)
        .invoke_async(conn)
        .await?;
    Ok(result)
}

/// Execute a Lua script that receives KEYS and ARGV and returns a
/// formatted string in the shape `"key1,key2:arg1,arg2"`.
pub async fn eval_with_keys_and_args(
    conn: &mut RedisConnection,
    keys: Vec<String>,
    args: Vec<String>,
) -> Result<String> {
    let script = Script::new(KEYS_ARGS_SCRIPT);
    let mut invocation = script.prepare_invoke();
    for key in &keys {
        invocation.key(key.as_str());
    }
    for arg in &args {
        invocation.arg(arg.as_str());
    }
    let result: String = invocation.invoke_async(conn).await?;
    Ok(result)
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

    #[tokio::test]
    async fn test_eval_simple_script() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("Should get connection");
        let result = eval_simple_script(&mut conn)
            .await
            .expect("Script should execute");
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_eval_with_keys_and_args() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("Should get connection");
        let keys = vec!["key1".to_string(), "key2".to_string()];
        let args = vec!["arg1".to_string(), "arg2".to_string()];
        let result = eval_with_keys_and_args(&mut conn, keys, args)
            .await
            .expect("Script should execute");
        assert_eq!(result, "key1,key2:arg1,arg2");
    }

    #[tokio::test]
    async fn test_eval_empty_keys_and_args() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("Should get connection");
        let result = eval_with_keys_and_args(&mut conn, vec![], vec![])
            .await
            .expect("Script should execute with empty inputs");
        assert_eq!(result, ":");
    }
}
