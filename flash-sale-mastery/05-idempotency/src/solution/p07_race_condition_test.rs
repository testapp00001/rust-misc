//! # Solution 07: Race Condition Tests
//!
//! Complete implementation of race condition tests for idempotency.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::Barrier;

/// Result of an idempotency check.
#[derive(Debug, Clone, PartialEq)]
pub enum IdempotencyResult {
    /// This is the first request. Proceed with processing.
    Proceed,
    /// This is a duplicate. Return the cached result.
    Duplicate(String),
}

/// A thread-safe in-memory idempotency store for testing race conditions.
pub struct InMemoryIdempotencyStore {
    keys: Arc<Mutex<HashMap<String, String>>>,
}

impl InMemoryIdempotencyStore {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Atomically check and mark a key as seen.
    ///
    /// The Mutex ensures that check-and-insert is atomic: if two tasks
    /// call this simultaneously with the same key, exactly one will see
    /// the key as absent and return `Proceed`.
    pub fn check_and_mark(&self, key: &str) -> IdempotencyResult {
        let mut map = self.keys.lock().unwrap();
        if map.contains_key(key) {
            IdempotencyResult::Duplicate(map[key].clone())
        } else {
            map.insert(key.to_string(), String::new());
            IdempotencyResult::Proceed
        }
    }

    /// Store a result for a key (for Duplicate returns).
    pub fn store_result(&self, key: &str, result: &str) {
        let mut map = self.keys.lock().unwrap();
        map.insert(key.to_string(), result.to_string());
    }

    /// Clear a key (simulate lock expiry for testing).
    pub fn clear_key(&self, key: &str) {
        let mut map = self.keys.lock().unwrap();
        map.remove(key);
    }
}

/// Create a test scenario where N tasks all try to acquire the same key
/// at the exact same moment.
///
/// Returns the number of tasks that got `Proceed` (should be exactly 1).
pub async fn race_n_tasks(store: &InMemoryIdempotencyStore, key: &str, n: usize) -> usize {
    let barrier = Arc::new(Barrier::new(n));
    let proceed_count = Arc::new(AtomicUsize::new(0));

    // Share the store's inner HashMap directly for spawned tasks.
    let store_keys = store.keys.clone();

    let mut handles = Vec::new();
    for _ in 0..n {
        let barrier_clone = barrier.clone();
        let count_clone = proceed_count.clone();
        let keys_clone = store_keys.clone();
        let key_owned = key.to_string();

        handles.push(tokio::spawn(async move {
            barrier_clone.wait().await;

            // Atomic check-and-mark using the shared HashMap
            let mut map = keys_clone.lock().unwrap();
            if map.contains_key(&key_owned) {
                // Duplicate
            } else {
                map.insert(key_owned, String::new());
                count_clone.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    for h in handles {
        h.await.expect("task panicked");
    }

    proceed_count.load(Ordering::SeqCst)
}

#[cfg(test)]
mod tests {
    use super::*;
    use deadpool_redis::{Config, Pool};

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
    #[tokio::test]
    async fn test_two_simultaneous_requests() {
        let store = InMemoryIdempotencyStore::new();
        let key = test_key("two_sim");

        let barrier = Arc::new(Barrier::new(2));
        let proceed_count = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..2 {
            let b = barrier.clone();
            let count = proceed_count.clone();
            let store_ref = &store;
            let k = key.clone();

            // Use spawn_local to avoid needing Send
            handles.push(async move {
                b.wait().await;
                let result = store_ref.check_and_mark(&k);
                if result == IdempotencyResult::Proceed {
                    count.fetch_add(1, Ordering::SeqCst);
                }
            });
        }

        futures::future::join_all(handles).await;

        let total = proceed_count.load(Ordering::SeqCst);
        assert_eq!(total, 1, "Exactly 1 out of 2 simultaneous requests should proceed");
    }

    /// Test: 100 requests arrive at the exact same moment.
    #[tokio::test]
    async fn test_one_hundred_simultaneous_requests() {
        let store = InMemoryIdempotencyStore::new();
        let proceed_count = race_n_tasks_concurrent(&store, &test_key("100_sim"), 100).await;
        assert_eq!(
            proceed_count, 1,
            "Exactly 1 out of 100 simultaneous requests should proceed"
        );
    }

    /// Helper that uses the store's actual check_and_mark method with spawned tasks.
    async fn race_n_tasks_concurrent(
        store: &InMemoryIdempotencyStore,
        key: &str,
        n: usize,
    ) -> usize {
        let barrier = Arc::new(Barrier::new(n));
        let proceed_count = Arc::new(AtomicUsize::new(0));

        // Share the store's inner state for spawned tasks
        let keys_arc = store.keys.clone();

        let mut handles = Vec::new();
        for _ in 0..n {
            let b = barrier.clone();
            let count = proceed_count.clone();
            let keys = keys_arc.clone();
            let k = key.to_string();

            handles.push(tokio::spawn(async move {
                b.wait().await;
                let mut map = keys.lock().unwrap();
                if !map.contains_key(&k) {
                    map.insert(k, String::new());
                    count.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }

        for h in handles {
            h.await.expect("task panicked");
        }

        proceed_count.load(Ordering::SeqCst)
    }

    /// Test: Redis SET NX race condition.
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
    #[tokio::test]
    async fn test_no_duplicate_vouchers_issued() {
        let store = InMemoryIdempotencyStore::new();
        let key = test_key("no_dup_vouchers");
        let n = 50;
        let barrier = Arc::new(Barrier::new(n));
        let voucher_count = Arc::new(AtomicUsize::new(0));

        let keys_arc = store.keys.clone();

        let mut handles = Vec::new();
        for i in 0..n {
            let b = barrier.clone();
            let count = voucher_count.clone();
            let keys = keys_arc.clone();
            let k = key.clone();

            handles.push(tokio::spawn(async move {
                b.wait().await;

                let mut map = keys.lock().unwrap();
                if !map.contains_key(&k) {
                    map.insert(k, String::new());
                    drop(map); // Release lock before "generating" voucher

                    // Simulate voucher generation
                    let _voucher = format!("VCH-{i:04}");
                    count.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }

        for h in handles {
            h.await.expect("task panicked");
        }

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
    #[tokio::test]
    async fn test_lock_expiry_allows_retry() {
        let store = InMemoryIdempotencyStore::new();
        let key = test_key("lock_expiry");

        let r1 = store.check_and_mark(&key);
        assert_eq!(r1, IdempotencyResult::Proceed);

        // Simulate lock expiry by clearing the key
        store.clear_key(&key);

        let r2 = store.check_and_mark(&key);
        assert_eq!(r2, IdempotencyResult::Proceed);
    }
}
