//! # Exercise 05: Redis Sorted Set Operations
//!
//! ## Learning Objective
//! Master Redis SORTED SET commands (ZADD, ZRANGEBYSCORE, ZREMRANGEBYSCORE,
//! ZCARD) for implementing a sliding window rate limiter.
//!
//! ## Flash Sale Context
//! During a flash sale, abusive clients may bombard the API with thousands of
//! requests per second. A sliding window rate limiter tracks each request's
//! timestamp in a sorted set. Before processing a request, we count how many
//! entries fall within the time window (e.g., last 60 seconds). If the count
//! exceeds the limit, the request is rejected.
//!
//! The sorted set's score is the timestamp, and the member is a unique request
//! ID (e.g., timestamp + random suffix to handle identical timestamps).
//!
//! ## Instructions
//! 1. Implement `record_request` to add a timestamped entry to the rate limit key
//! 2. Implement `count_requests_in_window` to count requests within a time range
//! 3. Implement `cleanup_old_entries` to remove entries older than a threshold
//! 4. Implement `is_rate_limited` to check if a client exceeds the limit
//!
//! ## Hints
//! - ZADD score is the timestamp (use f64 for sub-second precision)
//! - ZRANGEBYSCORE with min/max counts entries in a range
//! - ZCARD gives the total count (but includes expired entries until cleanup)

use deadpool_redis::Connection as RedisConnection;

/// Error type for sorted set operations.
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Rate limit exceeded: {count} requests in window (limit: {limit})")]
    LimitExceeded { count: usize, limit: usize },
}

/// Record a request at the given timestamp.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Sorted set key (e.g., "ratelimit:client:123")
/// * `timestamp` - Request timestamp (seconds since epoch, as f64)
/// * `request_id` - Unique request identifier
pub async fn record_request(
    conn: &mut RedisConnection,
    key: &str,
    timestamp: f64,
    request_id: &str,
) -> Result<(), RateLimitError> {
    // TODO: Use ZADD to add the request with timestamp as score
    todo!("Implement record_request")
}

/// Count requests within a time window [window_start, window_end].
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Sorted set key
/// * `window_start` - Start of window (inclusive)
/// * `window_end` - End of window (inclusive)
pub async fn count_requests_in_window(
    conn: &mut RedisConnection,
    key: &str,
    window_start: f64,
    window_end: f64,
) -> Result<usize, RateLimitError> {
    // TODO: Use ZRANGEBYSCORE to get entries in range, then count them
    // Alternative: Use ZCOUNT if available
    todo!("Implement count_requests_in_window")
}

/// Remove all entries with timestamps before the given threshold.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Sorted set key
/// * `before_timestamp` - Remove entries with score < this value
pub async fn cleanup_old_entries(
    conn: &mut RedisConnection,
    key: &str,
    before_timestamp: f64,
) -> Result<usize, RateLimitError> {
    // TODO: Use ZREMRANGEBYSCORE to remove old entries
    // TODO: Return the number of entries removed
    todo!("Implement cleanup_old_entries")
}

/// Check if a client is rate-limited.
/// Returns Ok(count) if within limit, Err(LimitExceeded) if over.
///
/// # Arguments
/// * `conn` - Redis connection
/// * `key` - Sorted set key
/// * `window_seconds` - Window duration in seconds
/// * `max_requests` - Maximum requests allowed in the window
/// * `current_timestamp` - Current time
pub async fn is_rate_limited(
    conn: &mut RedisConnection,
    key: &str,
    window_seconds: f64,
    max_requests: usize,
    current_timestamp: f64,
) -> Result<usize, RateLimitError> {
    // TODO: Count requests in [current_timestamp - window_seconds, current_timestamp]
    // TODO: Return count if under limit, error if over
    todo!("Implement is_rate_limited")
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Runtime};

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    async fn get_conn() -> Option<RedisConnection> {
        let cfg = Config::from_url(REDIS_URL);
        let pool = cfg.builder().ok()?.build().ok()?;
        pool.get().await.ok()
    }

    fn test_key(suffix: &str) -> String {
        format!("test:ratelimit:{}:{}", suffix, std::process::id())
    }

    #[tokio::test]
    async fn test_record_and_count_requests() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("record");
        let now = 1700000000.0_f64;
        for i in 0..5 {
            record_request(&mut conn, &key, now + i as f64, &format!("req:{i}"))
                .await
                .expect("record failed");
        }
        let count = count_requests_in_window(&mut conn, &key, now, now + 10.0)
            .await
            .expect("count failed");
        assert_eq!(count, 5, "Should count all 5 requests in window");
    }

    #[tokio::test]
    async fn test_count_requests_outside_window() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("window");
        let now = 1700000000.0_f64;
        record_request(&mut conn, &key, now - 120.0, "old_req")
            .await
            .expect("record failed");
        record_request(&mut conn, &key, now, "new_req")
            .await
            .expect("record failed");
        let count = count_requests_in_window(&mut conn, &key, now - 60.0, now)
            .await
            .expect("count failed");
        assert_eq!(count, 1, "Only recent request should be in window");
    }

    #[tokio::test]
    async fn test_cleanup_old_entries() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("cleanup");
        let now = 1700000000.0_f64;
        for i in 0..10 {
            record_request(&mut conn, &key, now - 100.0 + i as f64 * 5.0, &format!("req:{i}"))
                .await
                .expect("record failed");
        }
        let removed = cleanup_old_entries(&mut conn, &key, now - 50.0)
            .await
            .expect("cleanup failed");
        assert!(removed > 0, "Should have removed old entries");
        let remaining = count_requests_in_window(&mut conn, &key, now - 200.0, now + 100.0)
            .await
            .expect("count failed");
        assert_eq!(remaining, 10 - removed);
    }

    #[tokio::test]
    async fn test_is_rate_limited() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("limited");
        let now = 1700000000.0_f64;
        // Record 3 requests within a 60-second window with limit of 5
        for i in 0..3 {
            record_request(&mut conn, &key, now + i as f64, &format!("req:{i}"))
                .await
                .expect("record failed");
        }
        let count = is_rate_limited(&mut conn, &key, 60.0, 5, now + 2.0)
            .await
            .expect("should not be limited");
        assert_eq!(count, 3, "Should return count of 3");
        // Now set limit to 2, should be limited
        let result = is_rate_limited(&mut conn, &key, 60.0, 2, now + 2.0).await;
        assert!(result.is_err(), "Should be rate limited");
    }
}
