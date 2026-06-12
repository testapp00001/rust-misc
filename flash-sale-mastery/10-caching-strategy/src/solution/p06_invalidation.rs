//! # Solution 06: Cache Invalidation Strategies
//!
//! Complete implementation of cache invalidation at multiple granularities.

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
        Self {
            cache: DashMap::new(),
        }
    }

    /// Get a value from the cache.
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.cache.get(key).map(|v| v.clone())
    }

    /// Put a value into the cache.
    pub fn put(&self, key: String, value: serde_json::Value) {
        self.cache.insert(key, value);
    }

    /// Invalidate a single key.
    pub fn invalidate_key(&self, key: &str) -> bool {
        self.cache.remove(key).is_some()
    }

    /// Invalidate all keys that start with the given pattern (prefix).
    pub fn invalidate_pattern(&self, pattern: &str) -> usize {
        let before = self.cache.len();
        self.cache.retain(|key, _| !key.starts_with(pattern));
        before - self.cache.len()
    }

    /// Invalidate all entries in the cache.
    pub fn invalidate_all(&self) {
        self.cache.clear();
    }

    /// Handle a stock change event by invalidating related cache entries.
    pub fn on_stock_change(&self, product_id: &str) {
        // Invalidate all entries related to this product
        let prefixes = [
            format!("product:{product_id}"),
            format!("stock:{product_id}"),
            format!("inventory:{product_id}"),
        ];

        for prefix in &prefixes {
            self.invalidate_pattern(prefix);
        }
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
