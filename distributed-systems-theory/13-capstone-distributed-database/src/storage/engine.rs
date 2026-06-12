//! In-memory key-value storage engine.
//!
//! This module provides a simple hash-map-backed storage engine with support for
//! put, get, delete, and prefix-scan operations. It is the core persistence layer
//! that the Raft state machine and API handlers write to.

use std::collections::HashMap;

/// A key-value storage engine backed by an in-memory hash map.
#[derive(Debug, Clone)]
pub struct StorageEngine {
    data: HashMap<String, String>,
}

impl StorageEngine {
    /// Create a new, empty storage engine.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Insert or overwrite a key-value pair.
    pub fn put(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    /// Look up a value by key. Returns `None` if the key does not exist.
    pub fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    /// Delete a key-value pair. Returns `true` if the key existed, `false` otherwise.
    pub fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    /// Return all key-value pairs whose key starts with the given prefix.
    ///
    /// Results are returned in an unordered iterator; for deterministic output
    /// the caller should sort the results.
    pub fn scan(&self, prefix: &str) -> Vec<(String, String)> {
        self.data
            .iter()
            .filter(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Return the number of key-value pairs stored.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Return `true` if the engine contains no entries.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get() {
        let mut engine = StorageEngine::new();
        engine.put("key".into(), "value".into());
        assert_eq!(engine.get("key"), Some("value".into()));
    }

    #[test]
    fn test_delete_existing() {
        let mut engine = StorageEngine::new();
        engine.put("key".into(), "value".into());
        assert!(engine.delete("key"));
        assert_eq!(engine.get("key"), None);
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut engine = StorageEngine::new();
        assert!(!engine.delete("missing"));
    }

    #[test]
    fn test_scan_prefix() {
        let mut engine = StorageEngine::new();
        engine.put("user:1".into(), "Alice".into());
        engine.put("user:2".into(), "Bob".into());
        engine.put("post:1".into(), "Hello".into());

        let mut results = engine.scan("user:");
        results.sort();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], ("user:1".into(), "Alice".into()));
        assert_eq!(results[1], ("user:2".into(), "Bob".into()));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut engine = StorageEngine::new();
        assert!(engine.is_empty());
        assert_eq!(engine.len(), 0);
        engine.put("a".into(), "1".into());
        assert!(!engine.is_empty());
        assert_eq!(engine.len(), 1);
    }

    #[test]
    fn test_overwrite() {
        let mut engine = StorageEngine::new();
        engine.put("key".into(), "v1".into());
        engine.put("key".into(), "v2".into());
        assert_eq!(engine.get("key"), Some("v2".into()));
    }
}
