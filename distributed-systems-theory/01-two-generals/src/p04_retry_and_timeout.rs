//! # Exercise: Retry with Timeout
//!
//! ## Theory
//!
//! While the Two Generals' Problem proves that perfect agreement is impossible over
//! an unreliable channel, practical systems use retries with timeouts to achieve
//! *sufficient* reliability. The key parameters are:
//!
//! - **Max retries**: How many times to attempt delivery before giving up.
//! - **Base timeout**: How long to wait for a response before retrying.
//! - **Backoff multiplier**: How much to increase the timeout after each failure.
//!
//! Exponential backoff (doubling the timeout each retry) is the standard approach.
//! It prevents network congestion from retries while ensuring increasingly patient
//! waits as failures accumulate.
//!
//! ## Proof / Intuition
//!
//! If each attempt has an independent probability *p* of success, the probability
//! of at least one success in *n* attempts is:
//!
//! `P(success) = 1 - (1 - p)^n`
//!
//! For p=0.5 (50% loss) and n=10 retries: `P(success) = 1 - 0.5^10 = 0.999`
//!
//! However, the Two Generals' Problem still applies to the *last* retry -- if the
//! final message is lost, agreement is still uncertain. Retries reduce the
//! *probability* of failure, but never eliminate it.
//!
//! ## Implementation Task
//!
//! Implement:
//! - `RetryPolicy` with max_retries, base_timeout, backoff_multiplier
//! - `retry_with_timeout` function that retries with exponential backoff
//!
//! ## Verification
//!
//! - Verify retry count matches expected behavior
//! - Verify exponential backoff timing
//! - Verify that success on first attempt skips retries

use std::time::{Duration, Instant};

/// Configuration for retry behavior with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (not counting the initial attempt).
    pub max_retries: u32,
    /// Initial timeout in milliseconds.
    pub base_timeout_ms: u64,
    /// Multiplier applied to timeout after each failure.
    /// 2.0 means doubling (standard exponential backoff).
    pub backoff_multiplier: f64,
    /// Maximum timeout cap in milliseconds (prevents unbounded growth).
    pub max_timeout_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_timeout_ms: 100,
            backoff_multiplier: 2.0,
            max_timeout_ms: 30_000,
        }
    }
}

impl RetryPolicy {
    /// Calculate the timeout for a given attempt number (0-indexed).
    pub fn timeout_for_attempt(&self, attempt: u32) -> Duration {
        let timeout_ms =
            self.base_timeout_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        let capped = timeout_ms.min(self.max_timeout_ms as f64) as u64;
        Duration::from_millis(capped)
    }

    /// Execute a function with retries and exponential backoff.
    ///
    /// The function `f` is called up to `max_retries + 1` times (initial + retries).
    /// Each attempt has a timeout. If the function returns `Ok`, the result is returned.
    /// If all attempts fail, the last error is returned.
    pub fn retry_with_timeout<F, T, E>(&self, mut f: F) -> Result<T, RetryResult<E>>
    where
        F: FnMut() -> Result<T, E>,
    {
        let mut last_error = None;
        let mut total_attempts = 0;

        for attempt in 0..=self.max_retries {
            total_attempts += 1;
            let timeout = self.timeout_for_attempt(attempt);
            let start = Instant::now();

            match f() {
                Ok(result) => {
                    return Ok(result);
                }
                Err(e) => {
                    let elapsed = start.elapsed();
                    if elapsed < timeout {
                        // Completed within timeout but returned error -- still a failure
                        last_error = Some(e);
                    } else {
                        // Timed out -- record error and retry
                        last_error = Some(e);
                    }
                }
            }
        }

        Err(RetryResult {
            error: last_error.unwrap(),
            attempts: total_attempts,
        })
    }
}

/// Result of a failed retry operation.
#[derive(Debug, Clone)]
pub struct RetryResult<E> {
    /// The error from the last attempt.
    pub error: E,
    /// Total number of attempts made.
    pub attempts: u32,
}

/// Async version of retry with timeout using tokio.
pub async fn retry_with_timeout_async<F, Fut, T, E>(
    policy: &RetryPolicy,
    mut f: F,
) -> Result<T, RetryResult<E>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut last_error = None;

    for attempt in 0..=policy.max_retries {
        let timeout = policy.timeout_for_attempt(attempt);

        match tokio::time::timeout(timeout, f()).await {
            Ok(Ok(result)) => {
                return Ok(result);
            }
            Ok(Err(e)) => {
                last_error = Some(e);
            }
            Err(_timeout_err) => {
                // Timeout -- will retry
            }
        }
    }

    Err(RetryResult {
        error: last_error.expect("at least one attempt should have run"),
        attempts: policy.max_retries + 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn retry_count_matches_config() {
        let policy = RetryPolicy {
            max_retries: 5,
            base_timeout_ms: 10,
            backoff_multiplier: 2.0,
            max_timeout_ms: 1000,
        };

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = policy.retry_with_timeout(|| -> Result<(), &str> {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err("always fail")
        });

        assert!(result.is_err());
        let retry_result = result.unwrap_err();
        assert_eq!(retry_result.attempts, 6); // 1 initial + 5 retries
        assert_eq!(counter.load(Ordering::SeqCst), 6);
    }

    #[test]
    fn success_on_first_attempt_skips_retries() {
        let policy = RetryPolicy {
            max_retries: 5,
            base_timeout_ms: 10,
            backoff_multiplier: 2.0,
            max_timeout_ms: 1000,
        };

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = policy.retry_with_timeout(|| -> Result<i32, &str> {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Ok(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn exponential_backoff_timing() {
        let policy = RetryPolicy {
            max_retries: 4,
            base_timeout_ms: 100,
            backoff_multiplier: 2.0,
            max_timeout_ms: 10_000,
        };

        // Verify timeout doubles each attempt
        assert_eq!(policy.timeout_for_attempt(0), Duration::from_millis(100));
        assert_eq!(policy.timeout_for_attempt(1), Duration::from_millis(200));
        assert_eq!(policy.timeout_for_attempt(2), Duration::from_millis(400));
        assert_eq!(policy.timeout_for_attempt(3), Duration::from_millis(800));
        assert_eq!(policy.timeout_for_attempt(4), Duration::from_millis(1600));
    }

    #[test]
    fn backoff_respects_max_timeout() {
        let policy = RetryPolicy {
            max_retries: 20,
            base_timeout_ms: 1000,
            backoff_multiplier: 2.0,
            max_timeout_ms: 5000,
        };

        // Even after many attempts, timeout should not exceed max
        for attempt in 0..=20 {
            let timeout = policy.timeout_for_attempt(attempt);
            assert!(
                timeout <= Duration::from_millis(5000),
                "timeout {timeout:?} exceeded max for attempt {attempt}"
            );
        }
    }

    #[test]
    fn eventual_success_after_failures() {
        let policy = RetryPolicy {
            max_retries: 5,
            base_timeout_ms: 10,
            backoff_multiplier: 2.0,
            max_timeout_ms: 1000,
        };

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = policy.retry_with_timeout(|| -> Result<i32, &str> {
            let attempt = counter_clone.fetch_add(1, Ordering::SeqCst);
            if attempt < 3 {
                Err("not yet")
            } else {
                Ok(99)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 99);
        assert_eq!(counter.load(Ordering::SeqCst), 4); // 3 failures + 1 success
    }

    #[tokio::test]
    async fn async_retry_works() {
        let policy = RetryPolicy {
            max_retries: 5,
            base_timeout_ms: 10,
            backoff_multiplier: 2.0,
            max_timeout_ms: 1000,
        };

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = retry_with_timeout_async(&policy, || {
            let c = Arc::clone(&counter_clone);
            async move {
                let attempt = c.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    Err("not yet")
                } else {
                    Ok("success")
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }
}
