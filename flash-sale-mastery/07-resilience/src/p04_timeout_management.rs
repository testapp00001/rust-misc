//! # Exercise 04: Timeout Management
//!
//! ## Learning Objective
//!
//! Implement per-operation timeout management using tokio's timeout facilities.
//! Every call to a downstream service must have a bounded execution time. Without
//! timeouts, a single hung connection can block a thread indefinitely, eventually
//! exhausting the thread pool and causing a full system outage.
//!
//! ## Flash Sale Context
//!
//! Different operations have different latency expectations during a flash sale.
//! Redis stock checks should complete in under 50ms, database writes in under 500ms,
//! and payment processing in under 5 seconds. Cascading timeouts must be calculated
//! so inner timeouts are always shorter than outer ones -- if the overall order
//! timeout is 10 seconds, the stock check + DB write + payment must each have their
//! own timeouts that sum to less than 10 seconds.
//!
//! ## Instructions
//!
//! 1. Implement `with_timeout` that wraps an async operation with a timeout
//! 2. Define common timeout profiles for flash sale operations
//! 3. Implement cascading timeout calculation
//! 4. Create a `TimeoutBudget` that tracks remaining time across sub-operations
//!
//! ## Hints
//!
//! - Use `tokio::time::timeout(duration, future)` which returns `Result<T, Elapsed>`
//! - `tokio::time::timeout` returns `Ok(result_of_future)` on success
//! - For cascading timeouts, subtract elapsed time from the total budget
//! - Use `Instant::now()` to track elapsed time

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during timeout operations.
#[derive(Debug, Error)]
pub enum TimeoutError {
    #[error("operation timed out after {0:?}")]
    TimedOut(Duration),
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("timeout budget exhausted: {0:?} remaining")]
    BudgetExhausted(Duration),
}

/// Common timeout profiles for flash sale operations.
#[derive(Debug, Clone)]
pub struct TimeoutProfile {
    /// Timeout for Redis operations (stock checks, cache reads)
    pub redis: Duration,
    /// Timeout for database operations (order writes, inventory updates)
    pub database: Duration,
    /// Timeout for payment processing
    pub payment: Duration,
    /// Overall timeout for the entire operation
    pub overall: Duration,
}

impl TimeoutProfile {
    /// Create a default flash sale timeout profile.
    pub fn flash_sale() -> Self {
        // TODO: Return appropriate timeouts:
        //   - redis: 50ms
        //   - database: 500ms
        //   - payment: 5s
        //   - overall: 8s (must be > sum of all individual timeouts for cascading)
        todo!("Implement TimeoutProfile::flash_sale")
    }
}

/// Execute an async operation with a timeout.
///
/// # Arguments
/// * `duration` - Maximum time to wait for the operation
/// * `op` - An async closure that returns `Result<T, E>`
///
/// # Returns
/// `Ok(T)` if the operation completes within the timeout, or `TimeoutError` if it
/// times out or fails
pub async fn with_timeout<F, T, E>(duration: Duration, op: F) -> Result<T, TimeoutError>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    // TODO: Implement timeout wrapper:
    //   1. Use tokio::time::timeout(duration, op)
    //   2. If the inner future completes, propagate its result
    //   3. If the timeout fires, return TimeoutError::TimedOut(duration)
    todo!("Implement with_timeout")
}

/// A budget that tracks remaining time across multiple sub-operations.
///
/// Ensures that cascading timeouts don't exceed the overall deadline. Each
/// sub-operation draws from the remaining budget, and if the budget is exhausted,
/// subsequent operations are rejected immediately.
pub struct TimeoutBudget {
    /// Total deadline for all operations
    deadline: Duration,
    /// When the budget was created
    start: std::time::Instant,
}

impl TimeoutBudget {
    /// Create a new timeout budget with the given total deadline.
    ///
    /// # Arguments
    /// * `deadline` - Total time budget for all sub-operations
    pub fn new(deadline: Duration) -> Self {
        // TODO: Initialize the budget, recording the start time
        todo!("Implement TimeoutBudget::new")
    }

    /// Execute a sub-operation, deducting from the remaining budget.
    ///
    /// The sub-operation's timeout is set to the minimum of `max_timeout` and
    /// the remaining budget. If the budget is already exhausted, returns immediately.
    ///
    /// # Arguments
    /// * `max_timeout` - Maximum timeout for this specific sub-operation
    /// * `op` - An async closure that returns `Result<T, E>`
    ///
    /// # Returns
    /// `Ok(T)` if the operation succeeds within the remaining budget
    pub async fn execute<F, T, E>(
        &self,
        max_timeout: Duration,
        op: F,
    ) -> Result<T, TimeoutError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // TODO: Implement budget-aware timeout:
        //   1. Calculate remaining time: deadline - elapsed
        //   2. If remaining <= 0, return BudgetExhausted
        //   3. Use min(max_timeout, remaining) as the actual timeout
        //   4. Execute the operation with that timeout
        todo!("Implement TimeoutBudget::execute")
    }

    /// Get the remaining time in the budget.
    pub fn remaining(&self) -> Duration {
        // TODO: Calculate and return remaining time
        todo!("Implement TimeoutBudget::remaining")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_operation_completes_within_timeout() {
        let result = with_timeout(Duration::from_millis(100), async {
            Ok::<_, String>("fast result")
        })
        .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fast result");
    }

    #[tokio::test]
    async fn test_timeout_triggers_cancellation() {
        let result = with_timeout(Duration::from_millis(20), async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            Ok::<_, String>("should not reach here")
        })
        .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TimeoutError::TimedOut(d) => assert_eq!(d, Duration::from_millis(20)),
            other => panic!("expected TimedOut, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_operation_failure_propagates() {
        let result = with_timeout(Duration::from_millis(100), async {
            Err::<(), _>("inner failure")
        })
        .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TimeoutError::OperationFailed(msg) => assert_eq!(msg, "inner failure"),
            other => panic!("expected OperationFailed, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_timeout_profile_flash_sale() {
        let profile = TimeoutProfile::flash_sale();
        assert_eq!(profile.redis, Duration::from_millis(50));
        assert_eq!(profile.database, Duration::from_millis(500));
        assert_eq!(profile.payment, Duration::from_secs(5));
        assert_eq!(profile.overall, Duration::from_secs(8));
    }

    #[tokio::test]
    async fn test_timeout_budget_tracks_remaining() {
        let budget = TimeoutBudget::new(Duration::from_millis(100));
        // Initially, most of the budget should be remaining
        assert!(budget.remaining() > Duration::from_millis(90));
    }

    #[tokio::test]
    async fn test_timeout_budget_deducts_time() {
        let budget = TimeoutBudget::new(Duration::from_millis(200));
        // Execute a slow operation
        let result = budget
            .execute(Duration::from_millis(100), async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Ok::<_, String>("done")
            })
            .await;
        assert!(result.is_ok());
        // Remaining should be less than original
        assert!(budget.remaining() < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_timeout_budget_exhausted() {
        let budget = TimeoutBudget::new(Duration::from_millis(50));
        // Wait for budget to expire
        tokio::time::sleep(Duration::from_millis(60)).await;
        let result = budget
            .execute(Duration::from_millis(100), async {
                Ok::<_, String>("should not execute")
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_cascading_timeouts() {
        let budget = TimeoutBudget::new(Duration::from_millis(200));
        // First operation uses some budget
        let r1 = budget
            .execute(Duration::from_millis(100), async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok::<_, String>("step 1")
            })
            .await;
        assert!(r1.is_ok());
        // Second operation uses remaining budget
        let r2 = budget
            .execute(Duration::from_millis(100), async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok::<_, String>("step 2")
            })
            .await;
        assert!(r2.is_ok());
    }
}
