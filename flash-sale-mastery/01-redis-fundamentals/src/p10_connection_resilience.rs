//! # Exercise 10: Connection Resilience
//!
//! ## Learning Objective
//! Implement retry logic with exponential backoff for Redis operations, and
//! build resilient wrappers that handle transient connection failures gracefully.
//!
//! ## Flash Sale Context
//! During a flash sale, Redis may experience brief connection drops due to
//! network hiccups, failover events, or momentary overload. A resilient client
//! retries failed operations with exponential backoff before giving up. This
//! prevents a single transient failure from cascading into a full outage.
//!
//! ## Instructions
//! 1. Implement `execute_with_retry` that retries an async operation with backoff
//! 2. Implement `resilient_get` that uses the retry wrapper for GET operations
//! 3. Implement `resilient_set` that uses the retry wrapper for SET operations
//! 4. Implement `backoff_duration` to calculate delay for a given attempt
//!
//! ## Hints
//! - Exponential backoff: delay = base * 2^attempt (with jitter)
//! - Cap the maximum delay (e.g., 5 seconds)
//! - Only retry on transient errors (connection, timeout)
//! - Use `tokio::time::sleep` for async delays

use deadpool_redis::Connection as RedisConnection;

/// Error type for resilient operations.
#[derive(Debug, thiserror::Error)]
pub enum ResilientError {
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("Max retries ({max_retries}) exhausted. Last error: {last_error}")]
    MaxRetriesExceeded {
        max_retries: usize,
        last_error: String,
    },

    #[error("Non-retryable error: {0}")]
    NonRetryable(String),
}

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: usize,
    /// Base delay in milliseconds for exponential backoff.
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds (cap).
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
/// Uses exponential backoff: base_delay * 2^attempt, capped at max_delay.
///
/// # Arguments
/// * `config` - Retry configuration
/// * `attempt` - Current attempt number (0-indexed)
pub fn backoff_duration(config: &RetryConfig, attempt: usize) -> std::time::Duration {
    // TODO: Calculate 2^attempt * base_delay_ms, capped at max_delay_ms
    // TODO: Return as Duration
    todo!("Implement backoff_duration")
}

/// Execute an async operation with retry logic and exponential backoff.
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation` - Async closure that returns Result
///
/// # Returns
/// The result of the operation, or MaxRetriesExceeded if all retries fail.
pub async fn execute_with_retry<T, F, Fut>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, ResilientError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, ResilientError>>,
{
    // TODO: Loop up to max_retries + 1 attempts
    // TODO: On transient error, sleep for backoff_duration and retry
    // TODO: On non-transient error, return immediately
    // TODO: On success, return the result
    todo!("Implement execute_with_retry")
}

/// Resilient GET that retries on transient failures.
pub async fn resilient_get(
    conn: &mut RedisConnection,
    key: &str,
    config: &RetryConfig,
) -> Result<Option<String>, ResilientError> {
    // TODO: Use execute_with_retry to wrap a GET command
    // Note: Since conn is borrowed, you may need to restructure
    todo!("Implement resilient_get")
}

/// Resilient SET that retries on transient failures.
pub async fn resilient_set(
    conn: &mut RedisConnection,
    key: &str,
    value: &str,
    config: &RetryConfig,
) -> Result<(), ResilientError> {
    // TODO: Use execute_with_retry to wrap a SET command
    todo!("Implement resilient_set")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_duration_basic() {
        let config = RetryConfig {
            max_retries: 5,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        };
        let d0 = backoff_duration(&config, 0);
        let d1 = backoff_duration(&config, 1);
        let d2 = backoff_duration(&config, 2);
        assert_eq!(d0, std::time::Duration::from_millis(100));
        assert_eq!(d1, std::time::Duration::from_millis(200));
        assert_eq!(d2, std::time::Duration::from_millis(400));
    }

    #[test]
    fn test_backoff_duration_capped() {
        let config = RetryConfig {
            max_retries: 10,
            base_delay_ms: 1000,
            max_delay_ms: 5000,
        };
        // 2^10 * 1000 = 1,024,000ms but should be capped at 5000ms
        let d = backoff_duration(&config, 10);
        assert_eq!(d, std::time::Duration::from_millis(5000));
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
            base_delay_ms: 1, // Very short for testing
            max_delay_ms: 10,
        };
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_clone = attempts.clone();

        let result = execute_with_retry(&config, || {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(ResilientError::Redis(
                        deadpool_redis::redis::RedisError::from((
                            deadpool_redis::redis::ErrorKind::IoError,
                            "connection refused",
                        )),
                    ))
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
            Err::<i32, _>(ResilientError::Redis(
                deadpool_redis::redis::RedisError::from((
                    deadpool_redis::redis::ErrorKind::IoError,
                    "connection refused",
                )),
            ))
        })
        .await;
        assert!(result.is_err(), "Should fail after max retries");
        match result {
            Err(ResilientError::MaxRetriesExceeded { max_retries, .. }) => {
                assert_eq!(max_retries, 2);
            }
            other => panic!("Expected MaxRetriesExceeded, got: {other:?}"),
        }
    }
}
