//! # Solution 05: Redis Sorted Set Operations
//!
//! Complete implementation of Redis SORTED SET operations for sliding window rate limiting.

use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection as RedisConnection;

/// Error type for sorted set operations.
#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Rate limit exceeded: {count} requests in window (limit: {limit})")]
    LimitExceeded { count: usize, limit: usize },
}

/// Record a request at the given timestamp.
pub async fn record_request(
    conn: &mut RedisConnection,
    key: &str,
    timestamp: f64,
    request_id: &str,
) -> Result<(), RateLimitError> {
    deadpool_redis::redis::cmd("ZADD")
        .arg(key)
        .arg(timestamp)
        .arg(request_id)
        .query_async::<()>(&mut *conn)
        .await?;
    Ok(())
}

/// Count requests within a time window [window_start, window_end].
pub async fn count_requests_in_window(
    conn: &mut RedisConnection,
    key: &str,
    window_start: f64,
    window_end: f64,
) -> Result<usize, RateLimitError> {
    let count: usize = deadpool_redis::redis::cmd("ZCOUNT")
        .arg(key)
        .arg(window_start)
        .arg(window_end)
        .query_async(&mut *conn)
        .await?;
    Ok(count)
}

/// Remove all entries with timestamps before the given threshold.
pub async fn cleanup_old_entries(
    conn: &mut RedisConnection,
    key: &str,
    before_timestamp: f64,
) -> Result<usize, RateLimitError> {
    let removed: usize = deadpool_redis::redis::cmd("ZREMRANGEBYSCORE")
        .arg(key)
        .arg("-inf")
        .arg(before_timestamp)
        .query_async(&mut *conn)
        .await?;
    Ok(removed)
}

/// Check if a client is rate-limited.
pub async fn is_rate_limited(
    conn: &mut RedisConnection,
    key: &str,
    window_seconds: f64,
    max_requests: usize,
    current_timestamp: f64,
) -> Result<usize, RateLimitError> {
    let window_start = current_timestamp - window_seconds;
    let count = count_requests_in_window(conn, key, window_start, current_timestamp).await?;
    if count >= max_requests {
        Err(RateLimitError::LimitExceeded {
            count,
            limit: max_requests,
        })
    } else {
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::Config;

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
        assert_eq!(count, 5);
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
        record_request(&mut conn, &key, now - 120.0, "old_req").await.unwrap();
        record_request(&mut conn, &key, now, "new_req").await.unwrap();
        let count = count_requests_in_window(&mut conn, &key, now - 60.0, now)
            .await
            .expect("count failed");
        assert_eq!(count, 1);
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
        let removed = cleanup_old_entries(&mut conn, &key, now - 50.0).await.unwrap();
        assert!(removed > 0);
        let remaining = count_requests_in_window(&mut conn, &key, now - 200.0, now + 100.0)
            .await
            .unwrap();
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
        for i in 0..3 {
            record_request(&mut conn, &key, now + i as f64, &format!("req:{i}"))
                .await
                .unwrap();
        }
        assert_eq!(is_rate_limited(&mut conn, &key, 60.0, 5, now + 2.0).await.unwrap(), 3);
        assert!(is_rate_limited(&mut conn, &key, 60.0, 2, now + 2.0).await.is_err());
    }
}
