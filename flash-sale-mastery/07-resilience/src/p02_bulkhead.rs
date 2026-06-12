//! # Exercise 02: Bulkhead
//!
//! ## Learning Objective
//!
//! Implement the Bulkhead pattern to isolate failures and limit concurrency per
//! operation type. Just as a ship has watertight compartments that prevent a hull
//! breach from flooding the entire vessel, bulkheads prevent one overloaded subsystem
//! from consuming all available resources.
//!
//! ## Flash Sale Context
//!
//! During a flash sale, different operations have different resource requirements and
//! failure modes. Stock checking is fast and frequent, order creation is slow and
//! critical, and notifications are non-critical. Without bulkheads, a surge in
//! notification requests could exhaust all available connections, starving order
//! creation. Each operation type gets its own concurrency limit.
//!
//! ## Instructions
//!
//! 1. Define a `Bulkhead` struct that wraps a tokio Semaphore
//! 2. Implement `new(name, max_concurrency)` constructor
//! 3. Implement `execute<F, T>(op: F)` that acquires a semaphore permit before running the operation
//! 4. Create factory functions for flash sale bulkheads: stock_check, order_creation, notification
//!
//! ## Hints
//!
//! - Use `tokio::sync::Semaphore` for concurrency limiting
//! - `Semaphore::acquire()` returns a `SemaphorePermit` that releases on drop
//! - Store the permit in a variable so it isn't dropped before the operation completes
//! - Use `Arc` to share bulkheads across tasks if needed

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during bulkhead operations.
#[derive(Debug, Error)]
pub enum BulkheadError {
    #[error("bulkhead '{name}' is full: {current}/{max} slots in use")]
    BulkheadFull {
        name: String,
        current: usize,
        max: usize,
    },
    #[error("operation failed: {0}")]
    OperationFailed(String),
    #[error("semaphore closed")]
    SemaphoreClosed,
}

/// Bulkhead that limits concurrency for a specific operation type.
///
/// Uses a tokio Semaphore to enforce the maximum number of concurrent operations.
/// When the limit is reached, new requests are rejected immediately rather than
/// queuing indefinitely.
pub struct Bulkhead {
    /// Human-readable name for this bulkhead (e.g., "stock_check")
    name: String,
    /// Maximum number of concurrent operations allowed
    max_concurrency: usize,
    // TODO: Add a tokio::sync::Semaphore field for concurrency control
}

impl Bulkhead {
    /// Create a new bulkhead with the given concurrency limit.
    ///
    /// # Arguments
    /// * `name` - A human-readable name for this bulkhead
    /// * `max_concurrency` - Maximum number of concurrent operations
    ///
    /// # Returns
    /// A new `Bulkhead` ready to accept operations
    pub fn new(name: impl Into<String>, max_concurrency: usize) -> Self {
        // TODO: Initialize the bulkhead with a semaphore sized to max_concurrency
        todo!("Implement Bulkhead::new")
    }

    /// Execute an async operation through the bulkhead.
    ///
    /// Acquires a semaphore permit before executing the operation. If no permits
    /// are available, returns an error immediately (fail-fast, not queue).
    ///
    /// # Arguments
    /// * `op` - An async closure that returns `Result<T, E>`
    ///
    /// # Returns
    /// `Ok(T)` if the operation succeeds, or a `BulkheadError` if the bulkhead
    /// is full or the operation fails
    pub async fn execute<F, T, E>(&self, op: F) -> Result<T, BulkheadError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // TODO: Implement bulkhead execution:
        //   1. Try to acquire a semaphore permit (try_acquire, not acquire)
        //   2. If permit not available, return BulkheadFull error
        //   3. Execute the operation while holding the permit
        //   4. Map the operation error to BulkheadError::OperationFailed
        //   5. The permit is automatically released when dropped
        todo!("Implement Bulkhead::execute")
    }

    /// Get the name of this bulkhead.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the maximum concurrency for this bulkhead.
    pub fn max_concurrency(&self) -> usize {
        self.max_concurrency
    }
}

/// Create the standard set of bulkheads for a flash sale system.
///
/// Returns three bulkheads with appropriate concurrency limits:
/// - stock_check: Higher concurrency (fast, read-heavy)
/// - order_creation: Lower concurrency (slow, write-heavy)
/// - notification: Medium concurrency (non-critical, batched)
pub fn flash_sale_bulkheads() -> (Bulkhead, Bulkhead, Bulkhead) {
    // TODO: Create three bulkheads with appropriate concurrency limits
    //   - stock_check: 50 concurrent operations
    //   - order_creation: 10 concurrent operations
    //   - notification: 20 concurrent operations
    todo!("Implement flash_sale_bulkheads")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_basic_execution() {
        let bulkhead = Bulkhead::new("test", 5);
        let result = bulkhead
            .execute(async { Ok::<_, String>("success") })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_concurrency_limited() {
        let bulkhead = Arc::new(Bulkhead::new("test", 2));
        let active = Arc::new(AtomicUsize::new(0));
        let max_observed = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..10 {
            let bh = bulkhead.clone();
            let active = active.clone();
            let max_observed = max_observed.clone();
            handles.push(tokio::spawn(async move {
                bh.execute(async {
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    max_observed.fetch_max(current, Ordering::SeqCst);
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    active.fetch_sub(1, Ordering::SeqCst);
                    Ok::<_, String>(())
                })
                .await
            }));
        }

        // Some should fail because bulkhead is full
        let mut successes = 0;
        let mut failures = 0;
        for h in handles {
            match h.await.unwrap() {
                Ok(_) => successes += 1,
                Err(_) => failures += 1,
            }
        }
        // At least some should succeed
        assert!(successes > 0, "some operations should succeed");
        // Max concurrent should not exceed bulkhead limit
        assert!(max_observed.load(Ordering::SeqCst) <= 2);
    }

    #[tokio::test]
    async fn test_one_bulkhead_full_doesnt_block_others() {
        let stock = Bulkhead::new("stock_check", 1);
        let order = Bulkhead::new("order_creation", 1);

        // Fill the stock bulkhead with a slow operation
        let stock_clone = Arc::new(stock);
        let sc = stock_clone.clone();
        let hold = tokio::spawn(async move {
            sc.execute(async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok::<_, String>(())
            })
            .await
        });

        // Small delay to ensure the first operation holds the permit
        tokio::time::sleep(Duration::from_millis(5)).await;

        // Stock bulkhead should be full now
        let result = stock_clone
            .execute(async { Ok::<_, String>("queued") })
            .await;
        assert!(result.is_err());

        // Order bulkhead should still work
        let result = order
            .execute(async { Ok::<_, String>("order ok") })
            .await;
        assert!(result.is_ok());

        hold.await.unwrap().unwrap();
    }

    #[tokio::test]
    async fn test_operation_failure_propagates() {
        let bulkhead = Bulkhead::new("test", 5);
        let result: Result<(), BulkheadError> = bulkhead
            .execute(async { Err::<(), _>("something broke") })
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            BulkheadError::OperationFailed(msg) => assert_eq!(msg, "something broke"),
            other => panic!("expected OperationFailed, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_flash_sale_bulkheads() {
        let (stock, order, notification) = flash_sale_bulkheads();
        assert_eq!(stock.name(), "stock_check");
        assert_eq!(stock.max_concurrency(), 50);
        assert_eq!(order.name(), "order_creation");
        assert_eq!(order.max_concurrency(), 10);
        assert_eq!(notification.name(), "notification");
        assert_eq!(notification.max_concurrency(), 20);
    }
}
