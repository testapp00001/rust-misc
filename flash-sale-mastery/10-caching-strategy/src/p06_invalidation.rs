//! # Exercise 06: Cache Invalidation Strategies
//!
//! ## Learning Objective
//! Implement cache invalidation at different granularities: single key,
//! pattern-based, and full flush. Also implement event-driven invalidation
//! where a stock change triggers invalidation of related cache entries.
//!
//! ## Flash Sale Context
//! When stock changes during a flash sale (a purchase succeeds, a reservation
//! expires), the cached stock value becomes stale. If the cache is not
//! invalidated, users see incorrect stock levels. This exercise teaches
//! different invalidation strategies and how to trigger them from domain events.
//!
//! ## Instructions
//! 1. Implement `InvalidateCache::new` to create a new cache
//! 2. Implement `get` and `put` for basic cache operations
//! 3. Implement `invalidate_key` to remove a single entry
//! 4. Implement `invalidate_pattern` to remove all keys matching a prefix
//! 5. Implement `invalidate_all` to flush the entire cache
//! 6. Implement `on_stock_change` to trigger event-driven invalidation
//!
//! ## Hints
//! - Pattern matching can use `key.starts_with(pattern)` for prefix matching
//! - `DashMap::retain` lets you filter entries in-place
//! - Event-driven invalidation should clear all entries related to a product

use dashmap::DashMap;

/// Error type for cache operations.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

/// A cache with multiple invalidation strategies.
pub struct InvalidateCache {
    cache: DashMap<String, serde_json::Value>,
}

impl InvalidateCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        // TODO: Initialize with an empty DashMap
        todo!("Implement InvalidateCache::new")
    }

    /// Get a value from the cache.
    ///
    /// # Arguments
    /// * `key` - The cache key to look up
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        // TODO: Look up the key and clone the value
        todo!("Implement InvalidateCache::get")
    }

    /// Put a value into the cache.
    ///
    /// # Arguments
    /// * `key` - The cache key
    /// * `value` - The value to store
    pub fn put(&self, key: String, value: serde_json::Value) {
        // TODO: Insert the key-value pair into the DashMap
        todo!("Implement InvalidateCache::put")
    }

    /// Invalidate a single key.
    ///
    /// # Arguments
    /// * `key` - The key to remove
    ///
    /// # Returns
    /// `true` if the key existed, `false` otherwise.
    pub fn invalidate_key(&self, key: &str) -> bool {
        // TODO: Remove the key from the DashMap
        // TODO: Return whether the key was present
        todo!("Implement InvalidateCache::invalidate_key")
    }

    /// Invalidate all keys that start with the given pattern (prefix).
    ///
    /// # Arguments
    /// * `pattern` - The prefix to match against cache keys
    ///
    /// # Returns
    /// The number of entries removed.
    pub fn invalidate_pattern(&self, pattern: &str) -> usize {
        // TODO: Count entries before and after calling retain
        // TODO: Use DashMap::retain to keep only entries that don't match the pattern
        todo!("Implement InvalidateCache::invalidate_pattern")
    }

    /// Invalidate all entries in the cache.
    pub fn invalidate_all(&self) {
        // TODO: Clear the entire DashMap
        todo!("Implement InvalidateCache::invalidate_all")
    }

    /// Handle a stock change event by invalidating related cache entries.
    ///
    /// When stock for a product changes, all cache entries related to that
    /// product should be invalidated (product data, stock counter, etc.).
    ///
    /// # Arguments
    /// * `product_id` - The product whose stock changed
    pub fn on_stock_change(&self, product_id: &str) {
        // TODO: Invalidate all cache entries related to this product
        // TODO: This includes keys like "product:{id}", "stock:{id}", etc.
        todo!("Implement InvalidateCache::on_stock_change")
    }

    /// Get the number of entries in the cache.
    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalidate_single_key() {
        let cache = InvalidateCache::new();
        cache.put("key1".into(), serde_json::json!("val1"));
        cache.put("key2".into(), serde_json::json!("val2"));
        assert_eq!(cache.size(), 2);

        let removed = cache.invalidate_key("key1");
        assert!(removed);
        assert_eq!(cache.size(), 1);
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key2").is_some());
    }

    #[test]
    fn test_invalidate_pattern() {
        let cache = InvalidateCache::new();
        cache.put("product:1001".into(), serde_json::json!("p1"));
        cache.put("product:1002".into(), serde_json::json!("p2"));
        cache.put("stock:1001".into(), serde_json::json!(100));
        cache.put("stock:1002".into(), serde_json::json!(50));
        cache.put("sale_config".into(), serde_json::json!("config"));

        let removed = cache.invalidate_pattern("product:");
        assert_eq!(removed, 2);
        assert_eq!(cache.size(), 3);

        // stock and sale_config should remain
        assert!(cache.get("stock:1001").is_some());
        assert!(cache.get("stock:1002").is_some());
        assert!(cache.get("sale_config").is_some());
    }

    #[test]
    fn test_invalidate_all() {
        let cache = InvalidateCache::new();
        cache.put("key1".into(), serde_json::json!("val1"));
        cache.put("key2".into(), serde_json::json!("val2"));
        cache.put("key3".into(), serde_json::json!("val3"));
        assert_eq!(cache.size(), 3);

        cache.invalidate_all();
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_on_stock_change_invalidates_related_entries() {
        let cache = InvalidateCache::new();
        cache.put(
            "product:1001".into(),
            serde_json::json!({"name": "Widget"}),
        );
        cache.put("stock:1001".into(), serde_json::json!(100));
        cache.put(
            "product:1002".into(),
            serde_json::json!({"name": "Gadget"}),
        );
        cache.put("stock:1002".into(), serde_json::json!(50));
        cache.put("sale_config".into(), serde_json::json!("config"));

        cache.on_stock_change("1001");

        // Entries for product 1001 should be gone
        assert!(
            cache.get("product:1001").is_none(),
            "product:1001 should be invalidated"
        );
        assert!(
            cache.get("stock:1001").is_none(),
            "stock:1001 should be invalidated"
        );

        // Entries for product 1002 should remain
        assert!(
            cache.get("product:1002").is_some(),
            "product:1002 should remain"
        );
        assert!(
            cache.get("stock:1002").is_some(),
            "stock:1002 should remain"
        );
        assert!(
            cache.get("sale_config").is_some(),
            "sale_config should remain"
        );
    }

    #[test]
    fn test_invalidate_nonexistent_pattern() {
        let cache = InvalidateCache::new();
        cache.put("key1".into(), serde_json::json!("val1"));

        let removed = cache.invalidate_pattern("nonexistent:");
        assert_eq!(removed, 0);
        assert_eq!(cache.size(), 1);
    }
}
