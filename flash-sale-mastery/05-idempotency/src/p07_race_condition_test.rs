//! # Exercise 07: Race Condition Tests
//!
//! ## Learning Objective
//! Write rigorous tests that expose race conditions in idempotency
//! implementations. These tests use barrier synchronization to ensure
//! that multiple requests arrive at the exact same moment, testing the
//! atomic guarantees of the underlying store.
//!
//! ## Flash Sale Context
//! The critical invariant: **no matter how requests are timed, a user must
//! never receive more than one voucher per purchase intent**. This module
//! creates worst-case timing scenarios to verify this invariant holds.
//!
//! ## Instructions
//! 1. Implement a simple in-memory idempotency store (for unit testing)
//! 2. Implement a Redis-backed store with proper atomic operations
//! 3. Write tests that use `tokio::sync::Barrier` to synchronize request timing
//! 4. Verify that exactly one request proceeds in each race scenario
//! 5. Test: two simultaneous requests, N simultaneous requests, Redis NX race
//!
//! ## Hints
//! - `tokio::sync::Barrier::new(n)` creates a barrier that releases after n tasks call `wait()`
//! - Spawn all tasks, have them all wait on the barrier, then proceed
//! - After the barrier, each task attempts to acquire the idempotency key
//! - Count how many get `Proceed` vs `Duplicate` -- exactly 1 should get `Proceed`
//! - Use `std::sync::atomic::AtomicUsize` to count without a Mutex

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Barrier;

/// Result of an idempotency check.
#[derive(Debug, Clone, PartialEq)]
pub enum IdempotencyResult {
    /// This is the first request. Proceed with processing.
    Proceed,
    /// This is a duplicate. Return the cached result.
    Duplicate(String),
}

/// A simple in-memory idempotency store for testing race conditions.
///
/// This store must be safe for concurrent access from multiple async tasks.
pub struct InMemoryIdempotencyStore {
    // TODO: Add fields for thread-safe storage
    // Hint: use Arc<Mutex<HashMap<String, String>>> or Arc<DashMap<String, String>>
    todo: (), // placeholder
}

impl InMemoryIdempotencyStore {
    pub fn new() -> Self {
        // TODO: Initialize the store
        todo!("Implement InMemoryIdempotencyStore::new")
    }

    /// Atomically check and mark a key as seen.
    ///
    /// This operation must be atomic: if two tasks call this simultaneously
    /// with the same key, exactly one should get `Proceed`.
    pub fn check_and_mark(&self, key: &str) -> IdempotencyResult {
        // TODO: Atomically check if the key exists
        // If not, insert it and return Proceed
        // If yes, return Duplicate with the existing value
        todo!("Implement atomic check_and_mark")
    }

    /// Store a result for a key (for Duplicate returns).
    pub fn store_result(&self, key: &str, result: &str) {
        // TODO: Update the stored result for this key
        todo!("Implement store_result")
    }

    /// Clear a key (simulate lock expiry for testing).
    pub fn clear_key(&self, key: &str) {
        // TODO: Remove the key from the store
        todo!("Implement clear_key")
    }
}

/// Create a test scenario where N tasks all try to acquire the same key
/// at the exact same moment.
///
/// Returns the number of tasks that got `Proceed` (should be exactly 1).
pub async fn race_n_tasks(store: &InMemoryIdempotencyStore, key: &str, n: usize) -> usize {
    // TODO: Create a Barrier with count n
    // TODO: Spawn n tasks that:
    //   1. Wait on the barrier
    //   2. Call store.check_and_mark(key)
    //   3. Return whether they got Proceed
    // TODO: Join all tasks and count how many got Proceed
    todo!("Implement race_n_tasks")
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Pool};
    use deadpool_redis::redis::AsyncCommands;

    const REDIS_URL: &str = "redis://127.0.0.1:6379";

    fn test_key(suffix: &str) -> String {
        format!("race:test:p07:{}:{}", std::process::id(), suffix)
    }

    async fn try_redis_pool() -> Result<Pool, String> {
        let cfg = Config::from_url(REDIS_URL);
        cfg.builder()
            .map_err(|e| format!("Could not create pool: {e}"))?
            .max_size(16)
            .build()
            .map_err(|e| format!("Could not create pool: {e}"))
    }

    /// Test: two requests arrive at the exact same moment.
    /// Only one should proceed.
    #[tokio::test]
    async fn test_two_simultaneous_requests() {
        let store = InMemoryIdempotencyStore::new();
        let proceed_count = race_n_tasks(&store, &test_key("two_sim"), 2).await;
        assert_eq!(
            proceed_count, 1,
            "Exactly 1 out of 2 simultaneous requests should proceed"
        );
    }

    /// Test: 100 requests arrive at the exact same moment.
    /// Only one should proceed.
    #[tokio::test]
    async fn test_one_hundred_simultaneous_requests() {
        let store = InMemoryIdempotencyStore::new();
        let proceed_count = race_n_tasks(&store, &test_key("100_sim"), 100).await;
        assert_eq!(
            proceed_count, 1,
            "Exactly 1 out of 100 simultaneous requests should proceed"
        );
    }

    /// Test: Redis SET NX race condition.
    /// Multiple tasks compete to SET NX the same key.
    #[tokio::test]
    async fn test_redis_set_nx_race() {
        let pool = match try_redis_pool().await {
            Ok(p) => p,
            Err(msg) => {
                eprintln!("SKIP: {msg}");
                return;
            }
        };

        let key = test_key("redis_nx_race");
        let n = 50;
        let barrier = Arc::new(Barrier::new(n));
        let proceed_count = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..n {
            let pool_clone = pool.clone();
            let key_clone = key.clone();
            let barrier_clone = barrier.clone();
            let count_clone = proceed_count.clone();

            handles.push(tokio::spawn(async move {
                barrier_clone.wait().await;

                let mut conn = pool_clone.get().await.expect("get connection");
                // SET NX: returns true if key was set (new), false if exists
                let result: bool = redis::cmd("SET")
                    .arg(&key_clone)
                    .arg("processing")
                    .arg("NX")
                    .arg("EX")
                    .arg(60)
                    .query_async(&mut *conn)
                    .await
                    .expect("SET NX");

                if result {
                    count_clone.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }

        for h in handles {
            h.await.expect("task panicked");
        }

        let total = proceed_count.load(Ordering::SeqCst);
        assert_eq!(
            total, 1,
            "Exactly 1 out of {n} Redis SET NX attempts should succeed"
        );
    }

    /// Test: no duplicate vouchers are ever issued, regardless of timing.
    ///
    /// This is the ultimate invariant test. It simulates the full flow:
    /// check idempotency -> generate voucher -> store result, and verifies
    /// that under concurrent access, exactly one voucher is generated.
    #[tokio::test]
    async fn test_no_duplicate_vouchers_issued() {
        let store = InMemoryIdempotencyStore::new();
        let key = test_key("no_dup_vouchers");
        let n = 50;
        let barrier = Arc::new(Barrier::new(n));
        let voucher_count = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for i in 0..n {
            let store_ref = &store;
            let key_clone = key.clone();
            let barrier_clone = barrier.clone();
            let count_clone = voucher_count.clone();

            // We need to use unsafe to share the store reference across tasks
            // In a real implementation, the store would be wrapped in Arc
            handles.push(async move {
                barrier_clone.wait().await;

                let result = store_ref.check_and_mark(&key_clone);
                match result {
                    IdempotencyResult::Proceed => {
                        // Simulate voucher generation
                        let voucher = format!("VCH-{i:04}");
                        store_ref.store_result(&key_clone, &voucher);
                        count_clone.fetch_add(1, Ordering::SeqCst);
                    }
                    IdempotencyResult::Duplicate(_) => {
                        // Duplicate -- no voucher generated
                    }
                }
            });
        }

        futures::future::join_all(handles).await;

        let total = voucher_count.load(Ordering::SeqCst);
        assert_eq!(
            total, 1,
            "Exactly 1 voucher must be issued, got {total}. INVARIANT VIOLATION!"
        );
    }

    /// Test: sequential requests are handled correctly (sanity check).
    #[tokio::test]
    async fn test_sequential_requests() {
        let store = InMemoryIdempotencyStore::new();
        let key = test_key("sequential");

        let r1 = store.check_and_mark(&key);
        store.store_result(&key, "VCH-0001");
        let r2 = store.check_and_mark(&key);

        assert_eq!(r1, IdempotencyResult::Proceed);
        assert_eq!(
            r2,
            IdempotencyResult::Duplicate("VCH-0001".to_string())
        );
    }

    /// Test: after a lock expires (simulated), a new request can proceed.
    ///
    /// This tests the scenario where a request acquires the lock but crashes
    /// before completing. The lock should eventually expire so other requests
    /// can proceed.
    #[tokio::test]
    async fn test_lock_expiry_allows_retry() {
        // This test verifies the concept; actual TTL-based expiry would need
        // a real Redis or a time-aware in-memory store.
        let store = InMemoryIdempotencyStore::new();
        let key = test_key("lock_expiry");

        // First request acquires
        let r1 = store.check_and_mark(&key);
        assert_eq!(r1, IdempotencyResult::Proceed);

        // Simulate lock expiry by clearing the key
        // (In a real system, this happens automatically via Redis TTL)
        store.clear_key(&key);

        // A new request should be able to proceed
        let r2 = store.check_and_mark(&key);
        assert_eq!(r2, IdempotencyResult::Proceed);
    }
}
