//! # Exercise 09: Script Caching (EVAL vs EVALSHA)
//!
//! ## Learning Objective
//! Understand how Redis caches Lua scripts via SHA1 digests, the
//! difference between EVAL (send full script) and EVALSHA (send only the
//! hash), and how to implement a caching strategy for repeated execution.
//!
//! ## Flash Sale Context
//! The purchase script from exercise 05 is executed on every single
//! request. Sending the full Lua source on every call wastes bandwidth.
//! EVALSHA sends only a 40-byte SHA1, cutting wire overhead dramatically.
//!
//! ## Instructions
//! 1. Implement `eval_direct` that sends a script via EVAL
//! 2. Implement `evalsha_direct` that executes a pre-loaded script via EVALSHA
//! 3. Implement `load_script` that loads a script into Redis's cache
//! 4. Implement `cached_eval` that tries EVALSHA first, falls back to EVAL
//!
//! ## Hints
//! - `redis::cmd("EVAL").arg(script).arg(num_keys).arg(key)...` for raw EVAL
//! - `redis::cmd("EVALSHA").arg(sha).arg(num_keys).arg(key)...` for EVALSHA
//! - `redis::cmd("SCRIPT").arg("LOAD").arg(script)` returns the SHA1
//! - `redis::cmd("SCRIPT").arg("EXISTS").arg(sha)` checks if a script is cached
//! - `redis::Script::new()` internally does EVALSHA with EVAL fallback

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;

/// A simple test script that returns the sum of its arguments.
pub const TEST_SCRIPT: &str = r#"
local sum = 0
for i = 1, #ARGV do
    sum = sum + tonumber(ARGV[i])
end
return sum
"#;

/// Execute a Lua script via the raw EVAL command.
///
/// EVAL sends the full script source on every call. This is simple but
/// wastes bandwidth for frequently executed scripts.
pub async fn eval_direct(
    conn: &mut RedisConnection,
    script: &str,
    num_keys: usize,
    keys: &[&str],
    args: &[&str],
) -> Result<redis::Value> {
    // TODO: Build an EVAL command with the script, num_keys, keys, and args
    todo!("Implement raw EVAL command")
}

/// Execute a pre-loaded script via EVALSHA.
///
/// EVALSHA sends only the 40-byte SHA1 hex digest. The script must have
/// been previously loaded via `load_script` or a prior EVAL.
pub async fn evalsha_direct(
    conn: &mut RedisConnection,
    sha: &str,
    num_keys: usize,
    keys: &[&str],
    args: &[&str],
) -> Result<redis::Value> {
    // TODO: Build an EVALSHA command with the sha, num_keys, keys, and args
    todo!("Implement raw EVALSHA command")
}

/// Load a script into Redis's script cache and return its SHA1 hex digest.
pub async fn load_script(
    conn: &mut RedisConnection,
    script: &str,
) -> Result<String> {
    // TODO: Use SCRIPT LOAD command
    todo!("Implement SCRIPT LOAD and return the SHA1")
}

/// Check whether a script with the given SHA1 is cached in Redis.
pub async fn script_exists(
    conn: &mut RedisConnection,
    sha: &str,
) -> Result<bool> {
    // TODO: Use SCRIPT EXISTS command
    todo!("Implement SCRIPT EXISTS check")
}

/// Execute a script using a caching strategy.
///
/// First attempts EVALSHA (fast path). If Redis responds with NOSCRIPT
/// (the script is not cached), falls back to EVAL (which also caches the
/// script for future EVALSHA calls).
///
/// This mirrors what `redis::Script::invoke_async` does internally.
pub async fn cached_eval(
    conn: &mut RedisConnection,
    script: &str,
    keys: &[&str],
    args: &[&str],
) -> Result<redis::Value> {
    // TODO: Try EVALSHA first, catch NOSCRIPT, fall back to EVAL
    todo!("Implement EVALSHA-first with EVAL fallback")
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

    /// EVAL should execute a script and return the correct result.
    #[tokio::test]
    async fn test_eval_direct() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");

        let result = eval_direct(&mut conn, TEST_SCRIPT, 0, &[], &["10", "20", "30"])
            .await
            .unwrap();
        // Lua returns an integer
        assert_eq!(result, redis::Value::Int(60));
    }

    /// SCRIPT LOAD should return a 40-character hex SHA1.
    #[tokio::test]
    async fn test_load_script() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");

        let sha = load_script(&mut conn, TEST_SCRIPT).await.unwrap();
        assert_eq!(sha.len(), 40, "SHA1 hex digest should be 40 characters");
        assert!(
            sha.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA1 should be hex"
        );
    }

    /// EVALSHA should work after SCRIPT LOAD.
    #[tokio::test]
    async fn test_evalsha_after_load() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");

        let sha = load_script(&mut conn, TEST_SCRIPT).await.unwrap();

        let exists = script_exists(&mut conn, &sha).await.unwrap();
        assert!(exists, "Script should exist after LOAD");

        let result = evalsha_direct(&mut conn, &sha, 0, &[], &["5", "15"])
            .await
            .unwrap();
        assert_eq!(result, redis::Value::Int(20));
    }

    /// EVALSHA without loading should fail (or fall back in cached_eval).
    #[tokio::test]
    async fn test_evalsha_without_load_fails() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");

        // Use a bogus SHA that is definitely not cached
        let bogus_sha = "0000000000000000000000000000000000000000";
        let result = evalsha_direct(&mut conn, bogus_sha, 0, &[], &["1"]).await;
        assert!(result.is_err(), "EVALSHA with uncached SHA should fail");
    }

    /// cached_eval should work on first call (EVAL) and subsequent calls
    /// (EVALSHA) transparently.
    #[tokio::test]
    async fn test_cached_eval_first_and_subsequent() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");

        // Ensure the script is NOT cached
        let sha = {
            // Compute SHA manually via SCRIPT LOAD then flush
            let sha = load_script(&mut conn, TEST_SCRIPT).await.unwrap();
            let _: () = redis::cmd("SCRIPT")
                .arg("FLUSH")
                .query_async(&mut *conn)
                .await
                .unwrap();
            sha
        };

        let exists = script_exists(&mut conn, &sha).await.unwrap();
        assert!(!exists, "Script should not exist after FLUSH");

        // First call: should use EVAL (and cache the script)
        let r1 = cached_eval(&mut conn, TEST_SCRIPT, &[], &["3", "7"])
            .await
            .unwrap();
        assert_eq!(r1, redis::Value::Int(10));

        // Second call: should use EVALSHA (script is now cached)
        let r2 = cached_eval(&mut conn, TEST_SCRIPT, &[], &["100", "200"])
            .await
            .unwrap();
        assert_eq!(r2, redis::Value::Int(300));
    }

    /// Cached EVAL should handle keys as well as args.
    #[tokio::test]
    async fn test_cached_eval_with_keys() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");

        let script = r#"
            local val = redis.call('GET', KEYS[1])
            if val then return val end
            return ARGV[1]
        "#;

        // Set a key for testing
        let _: () = redis::cmd("SET")
            .arg("test_cached_key")
            .arg("found")
            .query_async(&mut *conn)
            .await
            .unwrap();

        let r1 = cached_eval(&mut conn, script, &["test_cached_key"], &["default"])
            .await
            .unwrap();
        assert_eq!(r1, redis::Value::Data(b"found".to_vec()));

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg("test_cached_key")
            .query_async(&mut *conn)
            .await
            .unwrap();

        let r2 = cached_eval(&mut conn, script, &["test_cached_key"], &["default"])
            .await
            .unwrap();
        assert_eq!(r2, redis::Value::Data(b"default".to_vec()));
    }
}
