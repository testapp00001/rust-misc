//! # Solution 02: Bulkhead
//!
//! Complete implementation of the bulkhead pattern using tokio Semaphores
//! to isolate failures and limit concurrency per operation type.

use thiserror::Error;
use tokio::sync::Semaphore;

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
pub struct Bulkhead {
    name: String,
    max_concurrency: usize,
    semaphore: Semaphore,
}

impl Bulkhead {
    /// Create a new bulkhead with the given concurrency limit.
    pub fn new(name: impl Into<String>, max_concurrency: usize) -> Self {
        Self {
            name: name.into(),
            max_concurrency,
            semaphore: Semaphore::new(max_concurrency),
        }
    }

    /// Execute an async operation through the bulkhead.
    ///
    /// Tries to acquire a semaphore permit. If no permits are available,
    /// returns an error immediately (fail-fast).
    pub async fn execute<F, T, E>(&self, op: F) -> Result<T, BulkheadError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Try to acquire a permit without blocking
        let permit = self.semaphore.try_acquire().map_err(|_| {
            let available = self.semaphore.available_permits();
            BulkheadError::BulkheadFull {
                name: self.name.clone(),
                current: self.max_concurrency - available,
                max: self.max_concurrency,
            }
        })?;

        // Execute the operation while holding the permit
        let result = op.await;

        // Permit is dropped here, releasing the slot
        drop(permit);

        result.map_err(|e| BulkheadError::OperationFailed(e.to_string()))
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
pub fn flash_sale_bulkheads() -> (Bulkhead, Bulkhead, Bulkhead) {
    (
        Bulkhead::new("stock_check", 50),
        Bulkhead::new("order_creation", 10),
        Bulkhead::new("notification", 20),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

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

        let mut successes = 0;
        for h in handles {
            if h.await.unwrap().is_ok() {
                successes += 1;
            }
        }
        assert!(successes > 0, "some operations should succeed");
        assert!(max_observed.load(Ordering::SeqCst) <= 2);
    }

    #[tokio::test]
    async fn test_one_bulkhead_full_doesnt_block_others() {
        let stock = Bulkhead::new("stock_check", 1);
        let order = Bulkhead::new("order_creation", 1);

        let stock_clone = Arc::new(stock);
        let sc = stock_clone.clone();
        let hold = tokio::spawn(async move {
            sc.execute(async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Ok::<_, String>(())
            })
            .await
        });

        tokio::time::sleep(Duration::from_millis(5)).await;

        let result = stock_clone
            .execute(async { Ok::<_, String>("queued") })
            .await;
        assert!(result.is_err());

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
