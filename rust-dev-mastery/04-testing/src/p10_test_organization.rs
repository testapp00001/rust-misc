//! # Lesson 10: Test Organization
//!
//! Well-organized tests are easier to maintain and debug.
//! This lesson covers test hierarchy, naming conventions, test utilities,
//! and test-only code patterns.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Production code
// ---------------------------------------------------------------------------

/// A simple cache with TTL support.
pub struct Cache<K, V> {
    entries: HashMap<K, CacheEntry<V>>,
    default_ttl_ms: u64,
    current_time_ms: u64,
}

struct CacheEntry<V> {
    value: V,
    expires_at_ms: u64,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> Cache<K, V> {
    pub fn new(default_ttl_ms: u64) -> Self {
        Self {
            entries: HashMap::new(),
            default_ttl_ms,
            current_time_ms: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.entries.insert(
            key,
            CacheEntry {
                value,
                expires_at_ms: self.current_time_ms + self.default_ttl_ms,
            },
        );
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.entries.get(key).and_then(|entry| {
            if self.current_time_ms < entry.expires_at_ms {
                Some(&entry.value)
            } else {
                None
            }
        })
    }

    pub fn remove(&mut self, key: &K) -> bool {
        self.entries.remove(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Advance the clock (for testing).
    pub fn advance_time(&mut self, ms: u64) {
        self.current_time_ms += ms;
    }

    /// Remove expired entries.
    pub fn evict_expired(&mut self) {
        let now = self.current_time_ms;
        self.entries.retain(|_, entry| now < entry.expires_at_ms);
    }
}

// ---------------------------------------------------------------------------
// Test utilities module
// ---------------------------------------------------------------------------

/// Test-only utilities. In a real project, these would be in
/// `#[cfg(test)] mod test_utils` or a separate `tests/common/` module.
pub mod test_utils {
    use super::*;

    /// Create a cache pre-populated with test data.
    pub fn create_test_cache() -> Cache<String, String> {
        let mut cache = Cache::new(1000);
        cache.insert("key1".into(), "value1".into());
        cache.insert("key2".into(), "value2".into());
        cache.insert("key3".into(), "value3".into());
        cache
    }

    /// Assert that a cache contains a specific key-value pair.
    pub fn assert_cache_contains(cache: &Cache<String, String>, key: &str, value: &str) {
        match cache.get(&key.to_string()) {
            Some(v) => assert_eq!(v, value, "cache[{}] = '{}', expected '{}'", key, v, value),
            None => panic!("cache does not contain key '{}'", key),
        }
    }

    /// Assert that a cache does not contain a key.
    pub fn assert_cache_missing(cache: &Cache<String, String>, key: &str) {
        assert!(
            cache.get(&key.to_string()).is_none(),
            "cache should not contain key '{}' but found '{}'",
            key,
            cache.get(&key.to_string()).unwrap()
        );
    }

    /// Generate a list of test keys.
    pub fn test_keys(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("key-{}", i)).collect()
    }

    /// Generate a list of test values.
    pub fn test_values(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("value-{}", i)).collect()
    }
}

// ---------------------------------------------------------------------------
// Test naming conventions
// ---------------------------------------------------------------------------

/// Demonstrates test naming patterns.
/// Good test names describe the behavior being tested.
///
/// Pattern: test_{unit}_{scenario}_{expected}
///
/// Examples:
/// - test_cache_insert_and_get
/// - test_cache_expired_entry_returns_none
/// - test_cache_evict_removes_expired
/// - test_cache_empty_returns_zero_len

#[cfg(test)]
mod tests {
    use super::*;
    use test_utils::*;

    // -----------------------------------------------------------------------
    // Group 1: Basic operations
    // -----------------------------------------------------------------------

    #[test]
    fn test_cache_new_is_empty() {
        let cache: Cache<String, String> = Cache::new(1000);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = Cache::new(1000);
        cache.insert("key".to_string(), "value".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some(&"value".to_string()));
    }

    #[test]
    fn test_cache_get_missing_key() {
        let cache: Cache<String, String> = Cache::new(1000);
        assert_eq!(cache.get(&"missing".to_string()), None);
    }

    #[test]
    fn test_cache_overwrite_value() {
        let mut cache = Cache::new(1000);
        cache.insert("key".to_string(), "old".to_string());
        cache.insert("key".to_string(), "new".to_string());
        assert_eq!(cache.get(&"key".to_string()), Some(&"new".to_string()));
    }

    #[test]
    fn test_cache_remove_existing() {
        let mut cache = Cache::new(1000);
        cache.insert("key".to_string(), "value".to_string());
        assert!(cache.remove(&"key".to_string()));
        assert_eq!(cache.get(&"key".to_string()), None);
    }

    #[test]
    fn test_cache_remove_missing() {
        let mut cache: Cache<String, String> = Cache::new(1000);
        assert!(!cache.remove(&"missing".to_string()));
    }

    // -----------------------------------------------------------------------
    // Group 2: TTL and expiration
    // -----------------------------------------------------------------------

    #[test]
    fn test_cache_entry_expires_after_ttl() {
        let mut cache = Cache::new(100);
        cache.insert("key".to_string(), "value".to_string());
        assert!(cache.get(&"key".to_string()).is_some());

        cache.advance_time(101);
        assert!(cache.get(&"key".to_string()).is_none());
    }

    #[test]
    fn test_cache_entry_valid_before_ttl() {
        let mut cache = Cache::new(100);
        cache.insert("key".to_string(), "value".to_string());

        cache.advance_time(99);
        assert!(cache.get(&"key".to_string()).is_some());
    }

    #[test]
    fn test_cache_evict_removes_expired() {
        let mut cache = Cache::new(100);
        cache.insert("a".to_string(), "1".to_string());
        cache.advance_time(50);
        cache.insert("b".to_string(), "2".to_string());

        cache.advance_time(51);
        // "a" is expired (101ms old), "b" is not (51ms old)
        cache.evict_expired();

        assert!(cache.get(&"a".to_string()).is_none());
        assert!(cache.get(&"b".to_string()).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_evict_all_expired() {
        let mut cache = Cache::new(100);
        cache.insert("a".to_string(), "1".to_string());
        cache.insert("b".to_string(), "2".to_string());

        cache.advance_time(200);
        cache.evict_expired();

        assert!(cache.is_empty());
    }

    // -----------------------------------------------------------------------
    // Group 3: Test utilities usage
    // -----------------------------------------------------------------------

    #[test]
    fn test_create_test_cache() {
        let cache = create_test_cache();
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn test_assert_cache_contains_pass() {
        let cache = create_test_cache();
        assert_cache_contains(&cache, "key1", "value1");
    }

    #[test]
    #[should_panic(expected = "does not contain key")]
    fn test_assert_cache_contains_fail() {
        let cache = create_test_cache();
        assert_cache_contains(&cache, "missing", "value");
    }

    #[test]
    fn test_assert_cache_missing_pass() {
        let cache = create_test_cache();
        assert_cache_missing(&cache, "missing");
    }

    #[test]
    #[should_panic(expected = "should not contain key")]
    fn test_assert_cache_missing_fail() {
        let cache = create_test_cache();
        assert_cache_missing(&cache, "key1");
    }

    #[test]
    fn test_test_keys() {
        let keys = test_keys(5);
        assert_eq!(keys.len(), 5);
        assert_eq!(keys[0], "key-0");
        assert_eq!(keys[4], "key-4");
    }

    #[test]
    fn test_test_values() {
        let values = test_values(3);
        assert_eq!(values, vec!["value-0", "value-1", "value-2"]);
    }

    // -----------------------------------------------------------------------
    // Group 4: Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_cache_zero_ttl() {
        let mut cache = Cache::new(0);
        cache.insert("key".to_string(), "value".to_string());
        // With 0 TTL, entry expires immediately
        cache.advance_time(1);
        assert!(cache.get(&"key".to_string()).is_none());
    }

    #[test]
    fn test_cache_large_ttl() {
        let mut cache = Cache::new(u64::MAX / 2);
        cache.insert("key".to_string(), "value".to_string());
        assert!(cache.get(&"key".to_string()).is_some());
    }

    #[test]
    fn test_cache_many_entries() {
        let mut cache = Cache::new(1000);
        for i in 0..1000 {
            cache.insert(format!("key-{}", i), format!("value-{}", i));
        }
        assert_eq!(cache.len(), 1000);
        assert_eq!(
            cache.get(&"key-500".to_string()),
            Some(&"value-500".to_string())
        );
    }

    // -----------------------------------------------------------------------
    // Group 5: Combined scenarios
    // -----------------------------------------------------------------------

    #[test]
    fn test_cache_full_lifecycle() {
        let mut cache = Cache::new(100);

        // Insert
        cache.insert("session".to_string(), "abc123".to_string());
        assert_cache_contains(&cache, "session", "abc123");

        // Advance time partially
        cache.advance_time(50);
        assert_cache_contains(&cache, "session", "abc123");

        // Insert another entry
        cache.insert("token".to_string(), "xyz".to_string());

        // Advance past first entry's TTL
        cache.advance_time(51);
        assert_cache_missing(&cache, "session");
        assert_cache_contains(&cache, "token", "xyz");

        // Evict
        cache.evict_expired();
        assert_eq!(cache.len(), 1);

        // Advance past all
        cache.advance_time(100);
        cache.evict_expired();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_update_extends_ttl() {
        let mut cache = Cache::new(100);
        cache.insert("key".to_string(), "v1".to_string());

        cache.advance_time(80);
        // Update - should reset TTL
        cache.insert("key".to_string(), "v2".to_string());

        cache.advance_time(80);
        // Still valid because TTL was reset
        assert_cache_contains(&cache, "key", "v2");
    }
}
