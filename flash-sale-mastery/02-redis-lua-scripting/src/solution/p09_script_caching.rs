//! # Solution 09: Script Caching (EVAL vs EVALSHA)
//!
//! Complete implementation of script caching strategies.

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
/// EVAL sends the full script source on every call.
pub async fn eval_direct(
    conn: &mut RedisConnection,
    script: &str,
    num_keys: usize,
    keys: &[&str],
    args: &[&str],
) -> Result<redis::Value> {
    let mut cmd = redis::cmd("EVAL");
    cmd.arg(script).arg(num_keys);
    for key in keys {
        cmd.arg(key);
    }
    for arg in args {
        cmd.arg(arg);
    }
    let result = cmd.query_async(conn).await?;
    Ok(result)
}

/// Execute a pre-loaded script via EVALSHA.
///
/// EVALSHA sends only the 40-byte SHA1 hex digest. The script must have
/// been previously loaded into Redis's script cache.
pub async fn evalsha_direct(
    conn: &mut RedisConnection,
    sha: &str,
    num_keys: usize,
    keys: &[&str],
    args: &[&str],
) -> Result<redis::Value> {
    let mut cmd = redis::cmd("EVALSHA");
    cmd.arg(sha).arg(num_keys);
    for key in keys {
        cmd.arg(key);
    }
    for arg in args {
        cmd.arg(arg);
    }
    let result = cmd.query_async(conn).await?;
    Ok(result)
}

/// Load a script into Redis's script cache and return its SHA1 hex digest.
pub async fn load_script(
    conn: &mut RedisConnection,
    script: &str,
) -> Result<String> {
    let sha: String = redis::cmd("SCRIPT")
        .arg("LOAD")
        .arg(script)
        .query_async(conn)
        .await?;
    Ok(sha)
}

/// Check whether a script with the given SHA1 is cached in Redis.
pub async fn script_exists(
    conn: &mut RedisConnection,
    sha: &str,
) -> Result<bool> {
    let results: Vec<bool> = redis::cmd("SCRIPT")
        .arg("EXISTS")
        .arg(sha)
        .query_async(conn)
        .await?;
    Ok(results.first().copied().unwrap_or(false))
}

/// Execute a script using a caching strategy.
///
/// First attempts EVALSHA (fast path). If Redis responds with NOSCRIPT
/// (the script is not cached), falls back to EVAL (which also caches the
/// script for future EVALSHA calls).
pub async fn cached_eval(
    conn: &mut RedisConnection,
    script: &str,
    keys: &[&str],
    args: &[&str],
) -> Result<redis::Value> {
    // Compute the SHA1 of the script using Redis itself
    // We'll try EVALSHA first with a computed SHA, fall back to EVAL

    // Step 1: Load the script to get its SHA (this is idempotent)
    let sha = load_script(conn, script).await?;

    // Step 2: Try EVALSHA first
    let evalsha_result = evalsha_direct(conn, &sha, keys.len(), keys, args).await;

    match evalsha_result {
        Ok(value) => Ok(value),
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("NOSCRIPT") {
                // Script not cached; use EVAL
                let value = eval_direct(conn, script, keys.len(), keys, args).await?;
                Ok(value)
            } else {
                Err(e)
            }
        }
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

    /// EVALSHA without loading should fail.
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

        let bogus_sha = "0000000000000000000000000000000000000000";
        let result = evalsha_direct(&mut conn, bogus_sha, 0, &[], &["1"]).await;
        assert!(result.is_err(), "EVALSHA with uncached SHA should fail");
    }

    /// cached_eval should work on first call and subsequent calls.
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

        // Flush the script cache
        let _: () = redis::cmd("SCRIPT")
            .arg("FLUSH")
            .query_async(&mut *conn)
            .await
            .unwrap();

        // First call: should use EVAL path (and cache the script)
        let r1 = cached_eval(&mut conn, TEST_SCRIPT, &[], &["3", "7"])
            .await
            .unwrap();
        assert_eq!(r1, redis::Value::Int(10));

        // Second call: should use EVALSHA path (script is now cached)
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
        assert_eq!(r1, redis::Value::BulkString(b"found".to_vec()));

        // Cleanup
        let _: () = redis::cmd("DEL")
            .arg("test_cached_key")
            .query_async(&mut *conn)
            .await
            .unwrap();

        let r2 = cached_eval(&mut conn, script, &["test_cached_key"], &["default"])
            .await
            .unwrap();
        assert_eq!(r2, redis::Value::BulkString(b"default".to_vec()));
    }
}
