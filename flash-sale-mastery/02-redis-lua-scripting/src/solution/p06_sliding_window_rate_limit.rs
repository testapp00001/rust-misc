//! # Solution 06: Sliding Window Rate Limit
//!
//! Complete implementation of sliding window rate limiting via sorted sets.

use anyhow::Result;
use deadpool_redis::Connection as RedisConnection;
use redis::Script;

/// Result of a rate limit check.
#[derive(Debug, Clone, PartialEq)]
pub enum RateLimitResult {
    /// Request is allowed. Contains the remaining quota in the window.
    Allowed { remaining: u64 },
    /// Request is denied; the rate limit has been exceeded.
    Denied,
}

/// Sliding window rate limit Lua script.
///
/// Uses a sorted set where scores are timestamps. Old entries are evicted,
/// the remaining count is checked against the limit, and if allowed the
/// new entry is added.
const RATE_LIMIT_SCRIPT: &str = r#"
-- KEYS[1] = rate limit sorted set key
-- ARGV[1] = window_ms
-- ARGV[2] = max_requests
-- ARGV[3] = now_ms (current timestamp)
-- ARGV[4] = unique request id

local key = KEYS[1]
local window_ms = tonumber(ARGV[1])
local max_requests = tonumber(ARGV[2])
local now = tonumber(ARGV[3])
local request_id = ARGV[4]

-- Evict entries outside the sliding window
local window_start = now - window_ms
redis.call('ZREMRANGEBYSCORE', key, '-inf', tostring(window_start))

-- Count entries still in the window
local current_count = redis.call('ZCARD', key)

if current_count < max_requests then
    -- Allowed: add the new entry and refresh the TTL
    redis.call('ZADD', key, tostring(now), request_id)
    redis.call('PEXPIRE', key, window_ms)
    return {1, max_requests - current_count - 1}  -- Allowed, remaining
else
    return {0, 0}  -- Denied
end
"#;

/// Check whether a request is allowed under the sliding window rate limit.
pub async fn check_rate_limit(
    conn: &mut RedisConnection,
    key: &str,
    window_ms: u64,
    max_requests: u64,
) -> Result<RateLimitResult> {
    // Generate a unique request ID from the current timestamp in nanoseconds
    let now_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let request_id = format!("{now_ns}");

    // Current time in milliseconds
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    let result: Vec<i64> = Script::new(RATE_LIMIT_SCRIPT)
        .key(key)
        .arg(window_ms.to_string())
        .arg(max_requests.to_string())
        .arg(now_ms.to_string())
        .arg(&request_id)
        .invoke_async(conn)
        .await?;

    match result.as_slice() {
        [1, remaining] => Ok(RateLimitResult::Allowed {
            remaining: *remaining as u64,
        }),
        _ => Ok(RateLimitResult::Denied),
    }
}

/// Clean up rate limit data for a key (test helper).
pub async fn cleanup_rate_limit(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<()> {
    let _: () = redis::cmd("DEL")
        .arg(key)
        .query_async(conn)
        .await?;
    Ok(())
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

    /// Requests within the limit should be allowed.
    #[tokio::test]
    async fn test_within_limit() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_rl_within";

        cleanup_rate_limit(&mut conn, key).await.unwrap();

        for i in 0..5 {
            let result = check_rate_limit(&mut conn, key, 1000, 10).await.unwrap();
            assert_eq!(
                result,
                RateLimitResult::Allowed { remaining: 10 - i - 1 },
                "Request {i} should be allowed"
            );
        }

        cleanup_rate_limit(&mut conn, key).await.unwrap();
    }

    /// Requests exceeding the limit should be denied.
    #[tokio::test]
    async fn test_exceeded_limit() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_rl_exceeded";

        cleanup_rate_limit(&mut conn, key).await.unwrap();

        // Exhaust the limit
        for _ in 0..3 {
            check_rate_limit(&mut conn, key, 1000, 3).await.unwrap();
        }

        // Next should be denied
        let result = check_rate_limit(&mut conn, key, 1000, 3).await.unwrap();
        assert_eq!(result, RateLimitResult::Denied);

        cleanup_rate_limit(&mut conn, key).await.unwrap();
    }

    /// The window should slide: old entries expire and new ones are allowed.
    #[tokio::test]
    async fn test_window_sliding() {
        let pool = match try_connect().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };
        let mut conn = pool.get().await.expect("connection");
        let key = "test_rl_sliding";

        cleanup_rate_limit(&mut conn, key).await.unwrap();

        // Use a very short window (100ms) with limit 2
        let r1 = check_rate_limit(&mut conn, key, 100, 2).await.unwrap();
        assert!(matches!(r1, RateLimitResult::Allowed { .. }));
        let r2 = check_rate_limit(&mut conn, key, 100, 2).await.unwrap();
        assert!(matches!(r2, RateLimitResult::Allowed { .. }));
        let r3 = check_rate_limit(&mut conn, key, 100, 2).await.unwrap();
        assert_eq!(r3, RateLimitResult::Denied);

        // Wait for the window to slide past
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        // Should be allowed again
        let r4 = check_rate_limit(&mut conn, key, 100, 2).await.unwrap();
        assert!(matches!(r4, RateLimitResult::Allowed { .. }));

        cleanup_rate_limit(&mut conn, key).await.unwrap();
    }
}
