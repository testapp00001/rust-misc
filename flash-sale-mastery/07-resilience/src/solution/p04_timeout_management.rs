//! # Solution 04: Timeout Management
//!
//! Complete implementation of per-operation timeout management with cascading
//! timeout budgets using tokio's timeout facilities.

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
        Self {
            redis: Duration::from_millis(50),
            database: Duration::from_millis(500),
            payment: Duration::from_secs(5),
            overall: Duration::from_secs(8),
        }
    }
}

/// Execute an async operation with a timeout.
///
/// Uses `tokio::time::timeout` to bound the operation's execution time.
pub async fn with_timeout<F, T, E>(duration: Duration, op: F) -> Result<T, TimeoutError>
where
    F: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    match tokio::time::timeout(duration, op).await {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => Err(TimeoutError::OperationFailed(e.to_string())),
        Err(_) => Err(TimeoutError::TimedOut(duration)),
    }
}

/// A budget that tracks remaining time across multiple sub-operations.
///
/// Ensures that cascading timeouts don't exceed the overall deadline.
pub struct TimeoutBudget {
    deadline: Duration,
    start: std::time::Instant,
}

impl TimeoutBudget {
    /// Create a new timeout budget with the given total deadline.
    pub fn new(deadline: Duration) -> Self {
        Self {
            deadline,
            start: std::time::Instant::now(),
        }
    }

    /// Execute a sub-operation, deducting from the remaining budget.
    ///
    /// Uses `min(max_timeout, remaining_budget)` as the effective timeout.
    pub async fn execute<F, T, E>(
        &self,
        max_timeout: Duration,
        op: F,
    ) -> Result<T, TimeoutError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let remaining = self.remaining();
        if remaining.is_zero() {
            return Err(TimeoutError::BudgetExhausted(Duration::ZERO));
        }

        let effective_timeout = std::cmp::min(max_timeout, remaining);
        with_timeout(effective_timeout, op).await
    }

    /// Get the remaining time in the budget.
    pub fn remaining(&self) -> Duration {
        let elapsed = self.start.elapsed();
        if elapsed >= self.deadline {
            Duration::ZERO
        } else {
            self.deadline - elapsed
        }
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
        assert!(budget.remaining() > Duration::from_millis(90));
    }

    #[tokio::test]
    async fn test_timeout_budget_deducts_time() {
        let budget = TimeoutBudget::new(Duration::from_millis(200));
        let result = budget
            .execute(Duration::from_millis(100), async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                Ok::<_, String>("done")
            })
            .await;
        assert!(result.is_ok());
        assert!(budget.remaining() < Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_timeout_budget_exhausted() {
        let budget = TimeoutBudget::new(Duration::from_millis(50));
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
        let r1 = budget
            .execute(Duration::from_millis(100), async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok::<_, String>("step 1")
            })
            .await;
        assert!(r1.is_ok());
        let r2 = budget
            .execute(Duration::from_millis(100), async {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok::<_, String>("step 2")
            })
            .await;
        assert!(r2.is_ok());
    }
}
