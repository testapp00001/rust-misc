//! # Solution 10: Connection Resilience
//!
//! Complete implementation of retry logic with exponential backoff for Redis operations.

use deadpool_redis::redis::RedisError;
use deadpool_redis::Connection as RedisConnection;
use std::time::Duration;

/// Error type for resilient operations.
#[derive(Debug, thiserror::Error)]
pub enum ResilientError {
    #[error("Redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("Max retries ({max_retries}) exhausted. Last error: {last_error}")]
    MaxRetriesExceeded {
        max_retries: usize,
        last_error: String,
    },

    #[error("Non-retryable error: {0}")]
    NonRetryable(String),
}

impl ResilientError {
    /// Whether this error is transient and worth retrying.
    fn is_transient(&self) -> bool {
        match self {
            ResilientError::Redis(err) => {
                matches!(
                    err.kind(),
                    deadpool_redis::redis::ErrorKind::IoError
                        | deadpool_redis::redis::ErrorKind::BusyLoadingError
                )
            }
            _ => false,
        }
    }
}

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: usize,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

/// Calculate the backoff duration for a given attempt number.
pub fn backoff_duration(config: &RetryConfig, attempt: usize) -> Duration {
    let delay = config.base_delay_ms.saturating_mul(1u64 << attempt);
    let capped = delay.min(config.max_delay_ms);
    Duration::from_millis(capped)
}

/// Execute an async operation with retry logic and exponential backoff.
///
/// The `operation` closure must be `FnMut` because it captures mutable state
/// (like a connection reference) and is called multiple times.
pub async fn execute_with_retry<T, F, Fut>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, ResilientError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, ResilientError>>,
{
    let mut last_error = String::new();
    let total_attempts = config.max_retries + 1;

    for attempt in 0..total_attempts {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(err) => {
                if !err.is_transient() {
                    return Err(err);
                }
                if attempt == config.max_retries {
                    return Err(ResilientError::MaxRetriesExceeded {
                        max_retries: config.max_retries,
                        last_error: err.to_string(),
                    });
                }
                last_error = err.to_string();
                let delay = backoff_duration(config, attempt);
                tokio::time::sleep(delay).await;
            }
        }
    }

    Err(ResilientError::MaxRetriesExceeded {
        max_retries: config.max_retries,
        last_error,
    })
}

/// Resilient GET that retries on transient failures.
pub async fn resilient_get(
    conn: &mut RedisConnection,
    key: &str,
    config: &RetryConfig,
) -> Result<Option<String>, ResilientError> {
    let mut last_error = String::new();
    let total_attempts = config.max_retries + 1;

    for attempt in 0..total_attempts {
        let result: Result<Option<String>, RedisError> = deadpool_redis::redis::cmd("GET")
            .arg(key)
            .query_async(&mut *conn)
            .await;
        match result {
            Ok(val) => return Ok(val),
            Err(err) => {
                let op_err = ResilientError::Redis(err);
                if !op_err.is_transient() || attempt == config.max_retries {
                    return Err(op_err);
                }
                last_error = op_err.to_string();
                let delay = backoff_duration(config, attempt);
                tokio::time::sleep(delay).await;
            }
        }
    }

    Err(ResilientError::MaxRetriesExceeded {
        max_retries: config.max_retries,
        last_error,
    })
}

/// Resilient SET that retries on transient failures.
pub async fn resilient_set(
    conn: &mut RedisConnection,
    key: &str,
    value: &str,
    config: &RetryConfig,
) -> Result<(), ResilientError> {
    let mut last_error = String::new();
    let total_attempts = config.max_retries + 1;

    for attempt in 0..total_attempts {
        let result: Result<(), RedisError> = deadpool_redis::redis::cmd("SET")
            .arg(key)
            .arg(value)
            .query_async(&mut *conn)
            .await;
        match result {
            Ok(()) => return Ok(()),
            Err(err) => {
                let op_err = ResilientError::Redis(err);
                if !op_err.is_transient() || attempt == config.max_retries {
                    return Err(op_err);
                }
                last_error = op_err.to_string();
                let delay = backoff_duration(config, attempt);
                tokio::time::sleep(delay).await;
            }
        }
    }

    Err(ResilientError::MaxRetriesExceeded {
        max_retries: config.max_retries,
        last_error,
    })
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
        format!("test:resilient:{}:{}", suffix, std::process::id())
    }

    #[test]
    fn test_backoff_duration_basic() {
        let config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        };
        assert_eq!(backoff_duration(&config, 0), Duration::from_millis(100));
        assert_eq!(backoff_duration(&config, 1), Duration::from_millis(200));
        assert_eq!(backoff_duration(&config, 2), Duration::from_millis(400));
    }

    #[test]
    fn test_backoff_duration_capped() {
        let config = RetryConfig {
            max_retries: 10,
            base_delay_ms: 1000,
            max_delay_ms: 5000,
        };
        assert_eq!(backoff_duration(&config, 10), Duration::from_millis(5000));
    }

    #[tokio::test]
    async fn test_execute_with_retry_success_first_try() {
        let config = RetryConfig::default();
        let result = execute_with_retry(&config, || async { Ok::<_, ResilientError>(42) })
            .await
            .expect("should succeed");
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_execute_with_retry_success_after_failures() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let config = RetryConfig {
            max_retries: 3,
            base_delay_ms: 1,
            max_delay_ms: 10,
        };
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let result = execute_with_retry(&config, || {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(ResilientError::Redis(RedisError::from((
                        deadpool_redis::redis::ErrorKind::IoError,
                        "connection refused",
                    ))))
                } else {
                    Ok(42)
                }
            }
        })
        .await
        .expect("should succeed after retries");
        assert_eq!(result, 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_execute_with_retry_max_retries_exceeded() {
        let config = RetryConfig {
            max_retries: 2,
            base_delay_ms: 1,
            max_delay_ms: 10,
        };
        let result = execute_with_retry(&config, || async {
            Err::<i32, _>(ResilientError::Redis(RedisError::from((
                deadpool_redis::redis::ErrorKind::IoError,
                "connection refused",
            ))))
        })
        .await;
        assert!(result.is_err());
        match result {
            Err(ResilientError::MaxRetriesExceeded { max_retries, .. }) => {
                assert_eq!(max_retries, 2);
            }
            other => panic!("Expected MaxRetriesExceeded, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_resilient_get_and_set() {
        let mut conn = match get_conn().await {
            Some(c) => c,
            None => {
                eprintln!("SKIP: Could not connect to Redis at {REDIS_URL}");
                return;
            }
        };
        let key = test_key("resilient_ops");
        let config = RetryConfig::default();
        resilient_set(&mut conn, &key, "flash_value", &config)
            .await
            .expect("resilient_set failed");
        let val = resilient_get(&mut conn, &key, &config)
            .await
            .expect("resilient_get failed");
        assert_eq!(val.as_deref(), Some("flash_value"));
    }
}
