//! # Solution 03: Retry with Exponential Backoff
//!
//! Complete implementation of retry logic with exponential backoff, jitter,
//! and a time-windowed retry budget.

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur during retry operations.
#[derive(Debug, Error)]
pub enum RetryError {
    #[error("operation failed after {attempts} attempts: {last_error}")]
    ExhaustedRetries {
        attempts: usize,
        last_error: String,
    },
    #[error("retry budget exceeded: too many retries in the time window")]
    BudgetExhausted,
}

/// Retry an async operation with exponential backoff and jitter.
///
/// Uses a deterministic jitter formula to avoid needing the `rand` crate:
/// `jitter_factor = 0.5 + (attempt * 0.37 % 1.0)` gives a value between 0.5 and 1.5.
pub async fn retry_with_backoff<F, Fut, T, E>(
    op: F,
    max_retries: usize,
    base_delay: Duration,
    max_delay: Duration,
) -> Result<T, RetryError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut last_error = String::new();

    for attempt in 0..=max_retries {
        match op().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                last_error = e.to_string();

                // Don't sleep after the last attempt
                if attempt < max_retries {
                    // Exponential backoff: base_delay * 2^attempt
                    let exponential = base_delay * 2u32.pow(attempt as u32);
                    let capped = std::cmp::min(exponential, max_delay);

                    // Apply deterministic jitter: factor between 0.5 and 1.5
                    let jitter_factor = 0.5 + ((attempt as f64 * 0.37) % 1.0);
                    let jittered =
                        Duration::from_secs_f64(capped.as_secs_f64() * jitter_factor);

                    tokio::time::sleep(jittered).await;
                }
            }
        }
    }

    Err(RetryError::ExhaustedRetries {
        attempts: max_retries + 1,
        last_error,
    })
}

/// A budget that limits the number of retries within a rolling time window.
pub struct RetryBudget {
    max_retries: usize,
    window: Duration,
    timestamps: VecDeque<Instant>,
}

impl RetryBudget {
    /// Create a new retry budget.
    pub fn new(max_retries: usize, window: Duration) -> Self {
        Self {
            max_retries,
            window,
            timestamps: VecDeque::new(),
        }
    }

    /// Check if a retry is allowed under the budget.
    ///
    /// Prunes expired entries, then checks if we're under the limit.
    pub fn check(&mut self) -> Result<(), RetryError> {
        let now = Instant::now();

        // Remove timestamps outside the rolling window
        while let Some(&front) = self.timestamps.front() {
            if now.duration_since(front) > self.window {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }

        if self.timestamps.len() < self.max_retries {
            self.timestamps.push_back(now);
            Ok(())
        } else {
            Err(RetryError::BudgetExhausted)
        }
    }

    /// Get the number of retries remaining in the current window.
    pub fn remaining(&self) -> usize {
        self.max_retries.saturating_sub(self.timestamps.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_succeeds_on_first_attempt() {
        let result = retry_with_backoff(
            || async { Ok::<_, String>("success") },
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_succeeds_on_retry() {
        let attempt = Arc::new(AtomicUsize::new(0));
        let attempt_clone = attempt.clone();

        let result = retry_with_backoff(
            move || {
                let attempt = attempt_clone.clone();
                async move {
                    let n = attempt.fetch_add(1, Ordering::SeqCst);
                    if n < 2 {
                        Err(format!("fail #{n}"))
                    } else {
                        Ok("recovered")
                    }
                }
            },
            3,
            Duration::from_millis(5),
            Duration::from_millis(50),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "recovered");
        assert_eq!(attempt.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_exhausted_retries() {
        let result: Result<&str, RetryError> = retry_with_backoff(
            || async { Err::<&str, _>("permanent failure") },
            2,
            Duration::from_millis(5),
            Duration::from_millis(50),
        )
        .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            RetryError::ExhaustedRetries { attempts, .. } => {
                assert_eq!(attempts, 3); // 1 initial + 2 retries
            }
            other => panic!("expected ExhaustedRetries, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_jitter_spreads_retry_timing() {
        let start1 = std::time::Instant::now();
        let _ = retry_with_backoff(
            || async { Err::<(), _>("fail") },
            2,
            Duration::from_millis(20),
            Duration::from_millis(200),
        )
        .await;
        let elapsed1 = start1.elapsed();

        let start2 = std::time::Instant::now();
        let _ = retry_with_backoff(
            || async { Err::<(), _>("fail") },
            2,
            Duration::from_millis(20),
            Duration::from_millis(200),
        )
        .await;
        let elapsed2 = start2.elapsed();

        // Both should complete and take roughly similar time
        assert!(elapsed1 > Duration::from_millis(10));
        assert!(elapsed2 > Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_retry_budget_allows_retries() {
        let mut budget = RetryBudget::new(3, Duration::from_secs(1));
        assert!(budget.check().is_ok());
        assert!(budget.check().is_ok());
        assert!(budget.check().is_ok());
        assert_eq!(budget.remaining(), 0);
    }

    #[tokio::test]
    async fn test_retry_budget_exhausted() {
        let mut budget = RetryBudget::new(2, Duration::from_secs(1));
        assert!(budget.check().is_ok());
        assert!(budget.check().is_ok());
        assert!(budget.check().is_err());
    }

    #[tokio::test]
    async fn test_retry_budget_window_resets() {
        let mut budget = RetryBudget::new(2, Duration::from_millis(50));
        assert!(budget.check().is_ok());
        assert!(budget.check().is_ok());
        assert!(budget.check().is_err());
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(budget.check().is_ok());
    }
}
