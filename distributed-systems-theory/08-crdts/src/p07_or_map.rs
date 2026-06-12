//! # Exercise: OR-Map (Observed-Remove Map)
//!
//! ## Theory
//!
//! An OR-Map (Observed-Remove Map) combines an OR-Set for keys with arbitrary
//! CRDT values. Keys can be added and removed (with add-wins semantics for
//! concurrent operations), and values can be updated independently.
//!
//! This is one of the most useful CRDTs for building distributed applications.
//! It's used in systems like Riak and Automerge.
//!
//! ## Proof / Intuition
//!
//! The OR-Map maintains:
//! - An OR-Set for keys (providing add/remove semantics)
//! - A mapping from keys to values (which can be any CRDT)
//!
//! Merging an OR-Map merges both the key set and each key's value independently.
//! Since each component is itself a CRDT, the merge preserves all algebraic
//! properties.
//!
//! ## Implementation Task
//!
//! Implement `ORMap<K, V>` with:
//! - `insert(key, value)` - add or update a key-value pair
//! - `remove(key)` - remove a key
//! - `get(key)` - get a value by key
//! - `merge(other)` - merge keys and values
//!
//! ## Verification
//!
//! Verify insert/remove work, merge properties hold, concurrent operations
//! are handled correctly.
//!
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

/// Global unique tag counter for OR-Map keys.
static TAG_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Unique tag for map key additions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UniqueTag(u64);

impl UniqueTag {
    fn new() -> Self {
        Self(TAG_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

/// An Observed-Remove Map CRDT.
///
/// Keys behave like an OR-Set (add-wins concurrent semantics).
/// Values can be any type that supports clone and equality.
#[derive(Debug, Clone)]
pub struct ORMap<K: Eq + Hash + Clone, V: Clone + PartialEq> {
    /// Key -> set of tags.
    key_tags: HashMap<K, HashSet<UniqueTag>>,
    /// Tags that have been removed.
    removed_tags: HashSet<UniqueTag>,
    /// Key -> value (only for keys with surviving tags).
    values: HashMap<K, V>,
}

impl<K: Eq + Hash + Clone, V: Clone + PartialEq> ORMap<K, V> {
    /// Create a new empty OR-Map.
    pub fn new() -> Self {
        Self {
            key_tags: HashMap::new(),
            removed_tags: HashSet::new(),
            values: HashMap::new(),
        }
    }

    /// Check if a key has surviving tags (is alive).
    fn is_alive(&self, key: &K) -> bool {
        self.key_tags
            .get(key)
            .map(|tags| tags.iter().any(|t| !self.removed_tags.contains(t)))
            .unwrap_or(false)
    }

    /// Insert or update a key-value pair.
    pub fn insert(&mut self, key: K, value: V) {
        let tag = UniqueTag::new();
        self.key_tags.entry(key.clone()).or_default().insert(tag);
        self.values.insert(key, value);
    }

    /// Remove a key (and its value).
    pub fn remove(&mut self, key: &K) {
        if let Some(tags) = self.key_tags.remove(key) {
            self.removed_tags.extend(tags);
        }
        self.values.remove(key);
    }

    /// Get a value by key (only if key is alive).
    pub fn get(&self, key: &K) -> Option<&V> {
        if self.is_alive(key) {
            self.values.get(key)
        } else {
            None
        }
    }

    /// Check if a key exists (is alive).
    pub fn contains_key(&self, key: &K) -> bool {
        self.is_alive(key)
    }

    /// Get the number of alive key-value pairs.
    pub fn len(&self) -> usize {
        self.key_tags
            .keys()
            .filter(|k| self.is_alive(k))
            .count()
    }

    /// Check if the map is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get all alive keys.
    pub fn keys(&self) -> Vec<&K> {
        self.key_tags
            .keys()
            .filter(|k| self.is_alive(k))
            .collect()
    }

    /// Merge with another OR-Map.
    pub fn merge(&mut self, other: &ORMap<K, V>) {
        // Merge key tags
        for (key, tags) in &other.key_tags {
            let entry = self.key_tags.entry(key.clone()).or_default();
            entry.extend(tags);
        }

        // Merge removed tags
        self.removed_tags.extend(&other.removed_tags);

        // Merge values for alive keys
        for (key, value) in &other.values {
            if self.is_alive(key) {
                // Only update if the key is alive in the merged state
                self.values.insert(key.clone(), value.clone());
            }
        }

        // Clean up dead keys from values
        let dead_keys: Vec<K> = self
            .values
            .keys()
            .filter(|k| !self.is_alive(k))
            .cloned()
            .collect();
        for key in dead_keys {
            self.values.remove(&key);
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone + PartialEq> Default for ORMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut map = ORMap::new();
        map.insert("key1", "value1");
        map.insert("key2", "value2");

        assert_eq!(map.get(&"key1"), Some(&"value1"));
        assert_eq!(map.get(&"key2"), Some(&"value2"));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_remove_key() {
        let mut map = ORMap::new();
        map.insert("key1", "value1");
        map.insert("key2", "value2");

        map.remove(&"key1");
        assert_eq!(map.get(&"key1"), None);
        assert_eq!(map.get(&"key2"), Some(&"value2"));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_merge_combines_keys() {
        let mut a = ORMap::new();
        a.insert("a", 1);
        a.insert("b", 2);

        let mut b = ORMap::new();
        b.insert("b", 20);
        b.insert("c", 3);

        a.merge(&b);

        assert_eq!(a.get(&"a"), Some(&1));
        assert_eq!(a.get(&"c"), Some(&3));
        assert_eq!(a.len(), 3);
    }

    #[test]
    fn test_merge_is_idempotent() {
        let mut map = ORMap::new();
        map.insert("a", 1);
        map.insert("b", 2);
        map.remove(&"a");

        let original = map.clone();
        map.merge(&original.clone());

        // After idempotent merge, map should have same alive keys
        assert_eq!(map.len(), original.len());
        assert_eq!(map.get(&"b"), Some(&2));
    }

    #[test]
    fn test_concurrent_insert_wins() {
        let mut a = ORMap::new();
        let mut b = ORMap::new();

        a.insert("x", 1);
        b.insert("x", 2);

        a.merge(&b);

        // Both values should be present (concurrent)
        // The key "x" should have both tags, so it's alive
        assert!(a.contains_key(&"x"), "Concurrent inserts: key should survive");
    }
}
