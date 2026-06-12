//! # Exercise 02: Thundering Herd Prevention
//!
//! ## Learning Objective
//! Understand the thundering herd (cache stampede) problem and implement
//! mutex-based stampede protection so that only one request loads from the
//! database while others wait for the result.
//!
//! ## Flash Sale Context
//! When a popular cache entry expires during a flash sale, thousands of
//! concurrent requests all miss the cache simultaneously. Without protection,
//! all of them hit the database, which cannot sustain that load. The solution
//! is to use a per-key mutex: the first request acquires the lock and loads
//! from DB, while all other requests wait for the result.
//!
//! ## Instructions
//! 1. Implement `ProtectedCache::new` to create a new protected cache
//! 2. Implement `ProtectedCache::get` to retrieve a value without loading
//! 3. Implement `ProtectedCache::get_or_load` with mutex-based stampede protection
//!    - Check cache first (fast path)
//!    - If miss, acquire a per-key mutex
//!    - Double-check cache after acquiring mutex (another request may have loaded)
//!    - If still missing, call the loader function and store the result
//!
//! ## Hints
//! - Use `DashMap<String, Arc<Mutex<()>>>` for per-key locks
//! - The `tokio::sync::Mutex` is async-aware and won't block the runtime
//! - Always double-check the cache after acquiring the lock

use std::future::Future;
use std::sync::Arc;

use dashmap::DashMap;
use tokio::sync::Mutex;

/// Error type for cache operations.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache key not found: {0}")]
    KeyNotFound(String),

    #[error("Loader failed: {0}")]
    LoadFailed(String),
}

/// A cache with per-key mutex protection against thundering herd.
pub struct ProtectedCache {
    pub cache: DashMap<String, serde_json::Value>,
    pub locks: DashMap<String, Arc<Mutex<()>>>,
}

impl ProtectedCache {
    /// Create a new empty protected cache.
    pub fn new() -> Self {
        // TODO: Initialize with empty DashMap instances
        todo!("Implement ProtectedCache::new")
    }

    /// Get a value from the cache without loading on miss.
    ///
    /// # Arguments
    /// * `key` - The cache key to look up
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        // TODO: Look up the key in the cache DashMap
        // TODO: Clone and return the value if found
        todo!("Implement ProtectedCache::get")
    }

    /// Get a value from the cache, loading it on miss with stampede protection.
    ///
    /// Only one concurrent request for a given key will execute the loader.
    /// Other requests for the same key will wait and receive the same result.
    ///
    /// # Arguments
    /// * `key` - The cache key to look up
    /// * `loader` - Async function to load the value on cache miss
    pub async fn get_or_load<F, Fut>(
        &self,
        key: &str,
        loader: F,
    ) -> Result<serde_json::Value, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<serde_json::Value, CacheError>>,
    {
        // TODO: Step 1: Check cache (fast path)
        // TODO: Step 2: Get or create a per-key mutex lock
        // TODO: Step 3: Acquire the lock
        // TODO: Step 4: Double-check cache after acquiring lock
        // TODO: Step 5: Call the loader and store result in cache
        // TODO: Step 6: Return the value
        todo!("Implement ProtectedCache::get_or_load")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[tokio::test]
    async fn test_cache_hit_returns_value() {
        let cache = ProtectedCache::new();
        let value = serde_json::json!({"stock": 100});
        cache
            .cache
            .insert("product:1001".to_string(), value.clone());

        let result = cache.get("product:1001").expect("Should find key");
        assert_eq!(result, value);
    }

    #[tokio::test]
    async fn test_cache_miss_returns_none() {
        let cache = ProtectedCache::new();
        assert!(cache.get("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_get_or_load_loads_on_miss() {
        let cache = ProtectedCache::new();
        let value = cache
            .get_or_load("product:1001", || async {
                Ok(serde_json::json!({"stock": 50}))
            })
            .await
            .expect("Should load successfully");

        assert_eq!(value, serde_json::json!({"stock": 50}));
        // Verify it's now in cache
        assert!(cache.get("product:1001").is_some());
    }

    #[tokio::test]
    async fn test_only_one_db_call_for_concurrent_requests() {
        let cache = Arc::new(ProtectedCache::new());
        let call_count = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..1000 {
            let cache = Arc::clone(&cache);
            let call_count = Arc::clone(&call_count);
            handles.push(tokio::spawn(async move {
                cache
                    .get_or_load("product:hot", || {
                        let call_count = Arc::clone(&call_count);
                        async move {
                            call_count.fetch_add(1, Ordering::SeqCst);
                            // Simulate DB latency
                            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                            Ok(serde_json::json!({"stock": 1000}))
                        }
                    })
                    .await
            }));
        }

        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.expect("Task should not panic"));
        }

        // All requests should succeed
        for result in &results {
            assert!(result.is_ok(), "All requests should succeed");
            assert_eq!(
                result.as_ref().unwrap(),
                &serde_json::json!({"stock": 1000})
            );
        }

        // Only 1 DB call should have been made
        let total_calls = call_count.load(Ordering::SeqCst);
        assert_eq!(
            total_calls, 1,
            "Expected exactly 1 DB call, got {total_calls}"
        );
    }
}
