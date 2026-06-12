//! # Exercise 06: Sliding Window Rate Limit
//!
//! ## Learning Objective
//! Implement a sliding window rate limiter using a Redis sorted set and a
//! Lua script. The sliding window provides smoother rate limiting than a
//! fixed window by considering request timestamps within a rolling period.
//!
//! ## Flash Sale Context
//! Even with stock limits, an abusive client can flood the API with
//! thousands of requests per second, starving legitimate users. A sliding
//! window rate limiter per client IP (or account) caps the request rate
//! while allowing legitimate bursts.
//!
//! ## Instructions
//! 1. Define a Lua script that uses a sorted set as a sliding window
//! 2. Implement `check_rate_limit` returning `RateLimitResult`
//! 3. Implement `cleanup_rate_limit` for test teardown
//!
//! ## Hints
//! - Use `ZADD key timestamp member` to record each request
//! - Use `ZREMRANGEBYSCORE key -inf window_start` to evict old entries
//! - Use `ZCARD key` to count entries in the window
//! - Set `PEXPIRE key window_ms` for automatic cleanup
//! - Member values should be unique (timestamp + random, or use a counter)

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
/// KEYS[1]: sorted set key for this client
/// ARGV[1]: window size in milliseconds
/// ARGV[2]: max requests in the window
/// ARGV[3]: current timestamp in milliseconds
/// ARGV[4]: unique request identifier (for the sorted set member)
///
/// Returns: {1, remaining} if allowed, {0, 0} if denied
const RATE_LIMIT_SCRIPT: &str = r#"
-- KEYS[1] = rate limit sorted set key
-- ARGV[1] = window_ms
-- ARGV[2] = max_requests
-- ARGV[3] = now_ms (current timestamp)
-- ARGV[4] = unique request id

-- TODO: Implement sliding window rate limiting
-- 1. Compute window_start = now - window_ms
-- 2. ZREMRANGEBYSCORE to evict old entries
-- 3. ZCARD to count current entries
-- 4. If count < max, ZADD and PEXPIRE, return {1, remaining}
-- 5. Otherwise return {0, 0}
"#;

/// Check whether a request is allowed under the sliding window rate limit.
pub async fn check_rate_limit(
    conn: &mut RedisConnection,
    key: &str,
    window_ms: u64,
    max_requests: u64,
) -> Result<RateLimitResult> {
    // TODO: Generate a unique request ID, compute current timestamp, invoke script
    todo!("Implement rate limit check using RATE_LIMIT_SCRIPT")
}

/// Clean up rate limit data for a key (test helper).
pub async fn cleanup_rate_limit(
    conn: &mut RedisConnection,
    key: &str,
) -> Result<()> {
    // TODO: DEL the key
    todo!("Implement: delete the rate limit key")
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
