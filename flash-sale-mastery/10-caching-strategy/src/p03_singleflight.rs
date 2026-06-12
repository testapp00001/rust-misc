//! # Exercise 03: Singleflight Pattern
//!
//! ## Learning Objective
//! Implement the singleflight pattern: when multiple concurrent requests ask
//! for the same key, only one execution happens. Other callers wait for and
//! receive the same result.
//!
//! ## Flash Sale Context
//! During a flash sale, multiple API requests may need the same data (e.g.,
//! product details) at the same time. Without singleflight, each request
//! independently queries the database. With singleflight, the first request
//! starts the query and all subsequent requests for the same key piggyback
//! on that single query.
//!
//! ## Instructions
//! 1. Implement `Singleflight::new` to create a new singleflight instance
//! 2. Implement `Singleflight::call` to deduplicate concurrent requests
//!    - If a request for the key is already in-flight, wait for its result
//!    - If not, start a new execution and share the result with waiters
//! 3. Implement `Singleflight::in_flight_count` to report how many keys are
//!    currently being loaded
//!
//! ## Hints
//! - Use `tokio::sync::watch` channels to broadcast results to waiters
//! - Wrap results in `Arc` so they can be shared across tasks
//! - Use `DashMap` to track in-flight requests by key

use std::future::Future;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::watch;

/// Error type for singleflight operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum SingleflightError {
    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Sender dropped before result was available")]
    SenderDropped,
}

/// An entry tracking an in-flight request.
struct InflightEntry<T: Clone> {
    rx: watch::Receiver<Option<Arc<Result<T, SingleflightError>>>>,
}

/// Singleflight ensures at most one execution per key at a time.
pub struct Singleflight<T: Clone + Send + Sync + 'static> {
    in_flight: DashMap<String, InflightEntry<T>>,
}

impl<T: Clone + Send + Sync + 'static> Singleflight<T> {
    /// Create a new singleflight instance.
    pub fn new() -> Self {
        // TODO: Initialize with an empty DashMap
        todo!("Implement Singleflight::new")
    }

    /// Execute an async function, deduplicating concurrent calls for the same key.
    ///
    /// If a request for `key` is already in-flight, this call waits for that
    /// request's result instead of starting a new execution.
    ///
    /// # Arguments
    /// * `key` - The deduplication key
    /// * `f` - Async function to execute if no in-flight request exists
    pub async fn call<F, Fut>(
        &self,
        key: &str,
        f: F,
    ) -> Result<T, SingleflightError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, SingleflightError>>,
    {
        // TODO: Step 1: Check if there's already an in-flight request for this key
        // TODO: Step 2: If yes, clone the receiver, wait for the result, return it
        // TODO: Step 3: If no, create a watch channel, insert into in_flight map
        // TODO: Step 4: Execute the function
        // TODO: Step 5: Send the result through the channel
        // TODO: Step 6: Remove from in_flight map and return result
        todo!("Implement Singleflight::call")
    }

    /// Get the number of keys currently being loaded.
    pub fn in_flight_count(&self) -> usize {
        // TODO: Return the number of entries in the in_flight DashMap
        todo!("Implement Singleflight::in_flight_count")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_single_call_succeeds() {
        let sf: Singleflight<String> = Singleflight::new();
        let result = sf
            .call("key1", || async {
                Ok("hello".to_string())
            })
            .await
            .expect("Should succeed");
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_concurrent_calls_result_in_single_execution() {
        let sf = Arc::new(Singleflight::<String>::new());
        let exec_count = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..100 {
            let sf = Arc::clone(&sf);
            let exec_count = Arc::clone(&exec_count);
            handles.push(tokio::spawn(async move {
                sf.call("shared_key", || {
                    let exec_count = Arc::clone(&exec_count);
                    async move {
                        exec_count.fetch_add(1, Ordering::SeqCst);
                        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                        Ok("shared_result".to_string())
                    }
                })
                .await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.expect("Task should not panic"));
        }

        for result in &results {
            assert_eq!(
                result.as_ref().unwrap(),
                "shared_result"
            );
        }

        let total_executions = exec_count.load(Ordering::SeqCst);
        assert_eq!(
            total_executions, 1,
            "Expected exactly 1 execution, got {total_executions}"
        );
    }

    #[tokio::test]
    async fn test_different_keys_execute_independently() {
        let sf = Arc::new(Singleflight::<String>::new());
        let exec_count = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for key in &["key_a", "key_b", "key_c"] {
            let sf = Arc::clone(&sf);
            let exec_count = Arc::clone(&exec_count);
            let key = key.to_string();
            let key_clone = key.clone();
            handles.push(tokio::spawn(async move {
                sf.call(&key, || {
                    let exec_count = Arc::clone(&exec_count);
                    async move {
                        exec_count.fetch_add(1, Ordering::SeqCst);
                        Ok(format!("result_{key_clone}"))
                    }
                })
                .await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.expect("Task should not panic"));
        }

        // Each key should execute independently
        let total_executions = exec_count.load(Ordering::SeqCst);
        assert_eq!(
            total_executions, 3,
            "Expected 3 executions for 3 distinct keys, got {total_executions}"
        );
    }

    #[tokio::test]
    async fn test_error_propagated_to_all_waiters() {
        let sf = Arc::new(Singleflight::<String>::new());

        let mut handles = Vec::new();
        for _ in 0..10 {
            let sf = Arc::clone(&sf);
            handles.push(tokio::spawn(async move {
                sf.call("fail_key", || async {
                    Err(SingleflightError::RequestFailed("DB down".into()))
                })
                .await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.expect("Task should not panic"));
        }

        // All requests should get the error
        for result in &results {
            assert!(result.is_err(), "All requests should receive the error");
        }
    }
}
