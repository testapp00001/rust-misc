//! # Exercise 04: Cache-Aside Pattern
//!
//! ## Learning Objective
//! Implement the cache-aside (lazy loading) pattern with TTL-based expiration.
//! The application checks the cache first; on a miss, it loads from the source,
//! stores the result in cache, and returns it.
//!
//! ## Flash Sale Context
//! Cache-aside is the most common caching pattern. The application code is
//! responsible for checking the cache, loading from DB on miss, and populating
//! the cache. TTL ensures stale data is eventually refreshed. This pattern is
//! ideal for product data that changes infrequently but is read constantly
//! during a flash sale.
//!
//! ## Instructions
//! 1. Implement `CacheAside::new` with a configurable default TTL
//! 2. Implement `CacheAside::get` with cache-aside logic (check cache, load on miss)
//! 3. Implement `CacheAside::put` to manually insert a value with optional TTL
//! 4. Implement `CacheAside::invalidate` to remove a key from cache
//! 5. Implement `CacheAside::size` to report the number of cached entries
//!
//! ## Hints
//! - Store `(value, inserted_at, ttl)` tuples in the cache
//! - Use `std::time::Instant` for tracking insertion time
//! - Check expiration on read (lazy expiration)

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
    ///
    /// # Arguments
    /// * `default_ttl` - Default time-to-live for cached entries
    pub fn new(default_ttl: Duration) -> Self {
        // TODO: Initialize with an empty DashMap and store the default TTL
        todo!("Implement CacheAside::new")
    }

    /// Get a value from the cache, loading it on miss.
    ///
    /// If the entry exists and has not expired, return the cached value.
    /// If the entry is missing or expired, call the loader, cache the result,
    /// and return it.
    ///
    /// # Arguments
    /// * `key` - The cache key
    /// * `loader` - Async function to load the value on cache miss
    pub async fn get<F, Fut>(
        &self,
        key: &str,
        loader: F,
    ) -> Result<T, CacheError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, CacheError>>,
    {
        // TODO: Step 1: Check if key exists in cache
        // TODO: Step 2: If found, check if expired (inserted_at + ttl < now)
        // TODO: Step 3: If not expired, return the cached value
        // TODO: Step 4: If expired or missing, call the loader
        // TODO: Step 5: Store the result in cache with default TTL
        // TODO: Step 6: Return the value
        todo!("Implement CacheAside::get")
    }

    /// Manually insert a value into the cache.
    ///
    /// # Arguments
    /// * `key` - The cache key
    /// * `value` - The value to cache
    /// * `ttl` - Optional custom TTL; uses default if None
    pub fn put(&self, key: String, value: T, ttl: Option<Duration>) {
        // TODO: Create a CacheEntry with the value, current Instant, and TTL
        // TODO: Insert into the cache DashMap
        todo!("Implement CacheAside::put")
    }

    /// Remove a key from the cache.
    ///
    /// # Arguments
    /// * `key` - The cache key to invalidate
    pub fn invalidate(&self, key: &str) -> bool {
        // TODO: Remove the key from the DashMap
        // TODO: Return true if the key existed, false otherwise
        todo!("Implement CacheAside::invalidate")
    }

    /// Get the number of entries currently in the cache (including expired).
    pub fn size(&self) -> usize {
        // TODO: Return the length of the DashMap
        todo!("Implement CacheAside::size")
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
