//! # Exercise 03: Retry with Exponential Backoff
//!
//! ## Learning Objective
//!
//! Implement retry logic with exponential backoff and jitter. When a transient
//! failure occurs (network blip, brief overload), retrying after a short delay
//! often succeeds. Exponential backoff increases the delay between retries to
//! avoid overwhelming a struggling service, and jitter randomizes delays to
//! prevent synchronized retry storms (thundering herd).
//!
//! ## Flash Sale Context
//!
//! During a flash sale, Redis or the payment gateway might briefly return errors
//! under load. Without retry logic, every transient error becomes a user-visible
//! failure. With naive retry (immediate, constant delay), thousands of clients
//! retry simultaneously, creating a thundering herd that prevents recovery.
//! Exponential backoff with jitter spreads retries over time.
//!
//! ## Instructions
//!
//! 1. Implement `retry_with_backoff` that retries an async operation with configurable backoff
//! 2. Add jitter: multiply each delay by a random factor between 0.5 and 1.5
//! 3. Implement a retry budget that limits retries within a time window
//! 4. Create a `RetryBudget` struct for tracking retry counts
//!
//! ## Hints
//!
//! - Use `tokio::time::sleep` for async delays
//! - Use `Duration::min(delay, max_delay)` to cap the backoff
//! - For jitter without `rand`, use a simple formula: `delay * (0.5 + some_fraction)`
//! - Track retry timestamps in a VecDeque for the time-window budget

use std::time::Duration;
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
/// # Arguments
/// * `op` - An async closure that returns `Result<T, E>`. Called on each attempt.
/// * `max_retries` - Maximum number of retry attempts (not counting the initial attempt)
/// * `base_delay` - Initial delay between retries
/// * `max_delay` - Maximum delay between retries (backoff caps at this value)
///
/// # Returns
/// `Ok(T)` if any attempt succeeds, or `RetryError::ExhaustedRetries` if all fail
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
    // TODO: Implement retry with exponential backoff:
    //   1. Try the operation up to max_retries + 1 times
    //   2. On failure, calculate delay: base_delay * 2^attempt
    //   3. Cap the delay at max_delay
    //   4. Apply jitter: multiply delay by a factor between 0.5 and 1.5
    //   5. Sleep for the jittered delay before retrying
    //   6. If all attempts fail, return ExhaustedRetries with the last error
    //
    // For jitter without the `rand` crate, you can use a deterministic approach:
    //   let jitter_factor = 0.5 + (attempt as f64 * 0.37 % 1.0);
    //   let jittered = Duration::from_secs_f64(delay.as_secs_f64() * jitter_factor);
    todo!("Implement retry_with_backoff")
}

/// A budget that limits the number of retries within a rolling time window.
///
/// This prevents a single failing operation from consuming excessive resources
/// through endless retries. If the budget is exhausted, new retries are rejected
/// immediately.
pub struct RetryBudget {
    /// Maximum retries allowed within the time window
    max_retries: usize,
    /// Duration of the rolling window
    window: Duration,
    // TODO: Add a field to track retry timestamps (e.g., Vec<Instant>)
}

impl RetryBudget {
    /// Create a new retry budget.
    ///
    /// # Arguments
    /// * `max_retries` - Maximum retries allowed within the time window
    /// * `window` - Duration of the rolling time window
    pub fn new(max_retries: usize, window: Duration) -> Self {
        // TODO: Initialize the retry budget
        todo!("Implement RetryBudget::new")
    }

    /// Check if a retry is allowed under the budget.
    ///
    /// If allowed, records the retry timestamp and returns `Ok(())`.
    /// If the budget is exhausted, returns `Err(RetryError::BudgetExhausted)`.
    ///
    /// # Returns
    /// `Ok(())` if the retry is within budget, `Err` otherwise
    pub fn check(&mut self) -> Result<(), RetryError> {
        // TODO: Implement budget check:
        //   1. Remove timestamps outside the rolling window
        //   2. If current count < max_retries, record timestamp and return Ok
        //   3. Otherwise return Err(BudgetExhausted)
        todo!("Implement RetryBudget::check")
    }

    /// Get the number of retries remaining in the current window.
    pub fn remaining(&self) -> usize {
        // TODO: Return the number of retries still available
        todo!("Implement RetryBudget::remaining")
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
        // Run two retry sequences and verify they don't take exactly the same time
        // (jitter should cause variation)
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

        // Both should complete (all retries exhausted) and take roughly similar time
        // but the jitter means they won't be exactly the same
        // We just verify they both complete without panicking
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
        assert!(budget.check().is_err()); // budget exhausted
    }

    #[tokio::test]
    async fn test_retry_budget_window_resets() {
        let mut budget = RetryBudget::new(2, Duration::from_millis(50));
        assert!(budget.check().is_ok());
        assert!(budget.check().is_ok());
        assert!(budget.check().is_err());
        // Wait for window to reset
        tokio::time::sleep(Duration::from_millis(60)).await;
        assert!(budget.check().is_ok());
    }
}
