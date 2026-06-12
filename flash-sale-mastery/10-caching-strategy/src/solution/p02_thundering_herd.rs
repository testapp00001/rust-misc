//! # Solution 02: Thundering Herd Prevention
//!
//! Complete implementation of mutex-based stampede protection for cache loads.

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
        Self {
            cache: DashMap::new(),
            locks: DashMap::new(),
        }
    }

    /// Get a value from the cache without loading on miss.
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.cache.get(key).map(|v| v.clone())
    }

    /// Get a value from the cache, loading it on miss with stampede protection.
    pub async fn get_or_load<F, Fut>(
        &self,
        key: &str,
        loader: F,
    ) -> Result<serde_json::Value, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<serde_json::Value, CacheError>>,
    {
        // Fast path: check cache without locking
        if let Some(value) = self.get(key) {
            return Ok(value);
        }

        // Get or create a per-key lock
        let lock = self
            .locks
            .entry(key.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();

        // Acquire the lock -- only one request per key enters here
        let _guard = lock.lock().await;

        // Double-check: another request may have loaded while we waited
        if let Some(value) = self.get(key) {
            return Ok(value);
        }

        // We hold the lock and the cache is still empty -- load from source
        let value = loader().await?;
        self.cache.insert(key.to_string(), value.clone());
        Ok(value)
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
