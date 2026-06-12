//! # Solution 04: Cache-Aside Pattern
//!
//! Complete implementation of cache-aside with TTL-based expiration.

use std::future::Future;
use std::time::{Duration, Instant};

use dashmap::DashMap;

/// Error type for cache-aside operations.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Loader failed: {0}")]
    LoadFailed(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

/// A cached entry with TTL tracking.
#[derive(Debug, Clone)]
struct CacheEntry<T: Clone> {
    value: T,
    inserted_at: Instant,
    ttl: Duration,
}

/// Cache-aside implementation with TTL-based expiration.
pub struct CacheAside<T: Clone + Send + Sync + 'static> {
    cache: DashMap<String, CacheEntry<T>>,
    default_ttl: Duration,
}

impl<T: Clone + Send + Sync + 'static> CacheAside<T> {
    /// Create a new cache-aside store with the given default TTL.
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            cache: DashMap::new(),
            default_ttl,
        }
    }

    /// Get a value from the cache, loading it on miss.
    pub async fn get<F, Fut>(
        &self,
        key: &str,
        loader: F,
    ) -> Result<T, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, CacheError>>,
    {
        // Check if the key exists and is not expired
        if let Some(entry) = self.cache.get(key) {
            if entry.inserted_at.elapsed() < entry.ttl {
                return Ok(entry.value.clone());
            }
            // Entry expired -- remove it
            drop(entry);
            self.cache.remove(key);
        }

        // Cache miss or expired -- load from source
        let value = loader().await?;
        let entry = CacheEntry {
            value: value.clone(),
            inserted_at: Instant::now(),
            ttl: self.default_ttl,
        };
        self.cache.insert(key.to_string(), entry);
        Ok(value)
    }

    /// Manually insert a value into the cache.
    pub fn put(&self, key: String, value: T, ttl: Option<Duration>) {
        let entry = CacheEntry {
            value,
            inserted_at: Instant::now(),
            ttl: ttl.unwrap_or(self.default_ttl),
        };
        self.cache.insert(key, entry);
    }

    /// Remove a key from the cache.
    pub fn invalidate(&self, key: &str) -> bool {
        self.cache.remove(key).is_some()
    }

    /// Get the number of entries currently in the cache (including expired).
    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_cache_hit_returns_cached_value() {
        let cache = CacheAside::new(Duration::from_secs(60));
        cache.put(
            "product:1001".to_string(),
            serde_json::json!({"name": "Widget"}),
            None,
        );

        let load_count = Arc::new(AtomicUsize::new(0));
        let result = cache
            .get("product:1001", || {
                let load_count = Arc::clone(&load_count);
                async move {
                    load_count.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::json!({"name": "From DB"}))
                }
            })
            .await
            .expect("Should succeed");

        assert_eq!(result, serde_json::json!({"name": "Widget"}));
        assert_eq!(
            load_count.load(Ordering::SeqCst),
            0,
            "Loader should not be called on cache hit"
        );
    }

    #[tokio::test]
    async fn test_cache_miss_triggers_load() {
        let cache = CacheAside::new(Duration::from_secs(60));
        let load_count = Arc::new(AtomicUsize::new(0));

        let result = cache
            .get("product:1001", || {
                let load_count = Arc::clone(&load_count);
                async move {
                    load_count.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::json!({"name": "From DB"}))
                }
            })
            .await
            .expect("Should succeed");

        assert_eq!(result, serde_json::json!({"name": "From DB"}));
        assert_eq!(load_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_ttl_expiration() {
        let cache = CacheAside::new(Duration::from_millis(50));
        let load_count = Arc::new(AtomicUsize::new(0));

        // First call: cache miss, loads from DB
        let _ = cache
            .get("key1", || {
                let load_count = Arc::clone(&load_count);
                async move {
                    load_count.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::json!("value1"))
                }
            })
            .await
            .expect("Should succeed");

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Second call: should be a miss due to expiration
        let _ = cache
            .get("key1", || {
                let load_count = Arc::clone(&load_count);
                async move {
                    load_count.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::json!("value2"))
                }
            })
            .await
            .expect("Should succeed");

        assert_eq!(
            load_count.load(Ordering::SeqCst),
            2,
            "Loader should be called twice: once before expiry, once after"
        );
    }

    #[tokio::test]
    async fn test_invalidate_removes_entry() {
        let cache = CacheAside::new(Duration::from_secs(60));
        cache.put(
            "key1".to_string(),
            serde_json::json!("value1"),
            None,
        );
        assert_eq!(cache.size(), 1);

        let removed = cache.invalidate("key1");
        assert!(removed, "Key should have been removed");
        assert_eq!(cache.size(), 0);
    }

    #[tokio::test]
    async fn test_invalidate_nonexistent_key() {
        let cache: CacheAside<serde_json::Value> =
            CacheAside::new(Duration::from_secs(60));
        let removed = cache.invalidate("nonexistent");
        assert!(!removed, "Should return false for nonexistent key");
    }
}
