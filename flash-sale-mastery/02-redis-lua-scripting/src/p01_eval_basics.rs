//! # Exercise 01: Redis EVAL Basics
//!
//! ## Learning Objective
//! Understand how Redis EVAL and EVALSHA commands execute Lua scripts
//! atomically inside Redis. Learn to pass KEYS and ARGV to scripts and
//! handle different return types.
//!
//! ## Flash Sale Context
//! Every flash sale operation -- stock check, claim, voucher issuance --
//! will eventually be wrapped in a Lua script for atomicity. Before
//! building complex scripts you must master the basics: how Lua executes
//! inside Redis, how parameters are passed, and how results come back.
//!
//! ## Instructions
//! 1. Implement `eval_simple_script` that runs a Lua script returning `42`
//! 2. Implement `eval_with_keys_and_args` that passes keys and args to a
//!    script which concatenates them into the format `"key1,key2:arg1,arg2"`
//!
//! ## Hints
//! - Use `redis::Script::new(lua_code)` to create a script
//! - Chain `.key(value)` and `.arg(value)` to add parameters
//! - Call `.invoke_async(conn).await?` to execute
//! - The result type is inferred from the function return type

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;

/// Execute a simple Lua script that returns the integer 42.
///
/// This demonstrates the most basic EVAL usage -- a script with no keys
/// and no arguments that returns a single integer.
pub async fn eval_simple_script(conn: &mut RedisConnection) -> Result<i64> {
    // TODO: Create a redis::Script that returns 42 and invoke it
    todo!("Implement: use redis::Script to run a Lua script returning 42")
}

/// Execute a Lua script that receives KEYS and ARGV and returns a
/// formatted string.
///
/// The script should return a string in the format:
/// `"key1,key2:arg1,arg2"`
/// (keys comma-separated, colon separator, then args comma-separated).
///
/// For empty inputs it should return `":"`.
pub async fn eval_with_keys_and_args(
    conn: &mut RedisConnection,
    keys: Vec<String>,
    args: Vec<String>,
) -> Result<String> {
    // TODO: Create a redis::Script that iterates KEYS and ARGV
    // TODO: Use .key() and .arg() to pass the parameters
    todo!("Implement: pass keys and args to a Lua script and return formatted result")
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

    /// A Lua script that returns 42 should produce 42.
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

    /// Keys and args should be concatenated in the expected format.
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

    /// Empty keys and args should still produce a valid separator.
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
