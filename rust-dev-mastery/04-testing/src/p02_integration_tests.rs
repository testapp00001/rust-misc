//! # Lesson 2: Integration Tests
//!
//! Integration tests verify that multiple components work together.
//! This lesson covers the tests/ directory pattern, shared helpers,
//! test fixtures, and testing against real I/O.

use std::collections::HashMap;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// A module to test (simulating what integration tests would exercise)
// ---------------------------------------------------------------------------

/// A simple key-value store that persists to files.
pub struct KvStore {
    data: HashMap<String, String>,
    path: PathBuf,
}

impl KvStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            data: HashMap::new(),
            path,
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn keys(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }

    /// Save to file (simulated).
    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string(&self.data)
            .map_err(|e| format!("serialize: {}", e))?;
        std::fs::write(&self.path, json)
            .map_err(|e| format!("write: {}", e))?;
        Ok(())
    }

    /// Load from file (simulated).
    pub fn load(path: &PathBuf) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("read: {}", e))?;
        let data: HashMap<String, String> = serde_json::from_str(&content)
            .map_err(|e| format!("parse: {}", e))?;
        Ok(Self {
            data,
            path: path.clone(),
        })
    }
}

// ---------------------------------------------------------------------------
// Test fixtures and helpers
// ---------------------------------------------------------------------------

/// A test fixture that sets up a temporary directory.
pub struct TestFixture {
    pub temp_dir: tempfile::TempDir,
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            temp_dir: tempfile::tempdir().expect("failed to create temp dir"),
        }
    }

    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    pub fn file_path(&self, name: &str) -> PathBuf {
        self.path().join(name)
    }

    /// Create a pre-populated KvStore.
    pub fn create_store(&self, name: &str, entries: &[(&str, &str)]) -> PathBuf {
        let path = self.file_path(name);
        let mut store = KvStore::new(path.clone());
        for (k, v) in entries {
            store.set(k, v);
        }
        store.save().unwrap();
        path
    }
}

/// Shared test data builder.
pub struct TestDataBuilder {
    entries: Vec<(String, String)>,
}

impl TestDataBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn with(mut self, key: &str, value: &str) -> Self {
        self.entries
            .push((key.to_string(), value.to_string()));
        self
    }

    pub fn build(self) -> Vec<(String, String)> {
        self.entries
    }

    pub fn as_str_refs(&self) -> Vec<(&str, &str)> {
        self.entries
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }
}

impl Default for TestDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Test helper functions
// ---------------------------------------------------------------------------

/// Assert that two string slices are equal (better error messages).
pub fn assert_str_eq(actual: &str, expected: &str) {
    assert_eq!(
        actual, expected,
        "\n  expected: \"{}\"\n  actual:   \"{}\"",
        expected, actual
    );
}

/// Assert that a store contains a specific key-value pair.
pub fn assert_store_contains(store: &KvStore, key: &str, value: &str) {
    match store.get(key) {
        Some(v) => assert_str_eq(v, value),
        None => panic!("store does not contain key '{}'", key),
    }
}

/// Assert that a store does not contain a key.
pub fn assert_store_missing(store: &KvStore, key: &str) {
    assert!(
        store.get(key).is_none(),
        "store should not contain key '{}' but has value '{}'",
        key,
        store.get(key).unwrap()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Basic integration tests using fixtures
    // -----------------------------------------------------------------------

    #[test]
    fn test_store_set_and_get() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("test.json");
        let mut store = KvStore::new(path);

        store.set("key1", "value1");
        store.set("key2", "value2");

        assert_store_contains(&store, "key1", "value1");
        assert_store_contains(&store, "key2", "value2");
        assert_store_missing(&store, "key3");
    }

    #[test]
    fn test_store_delete() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("test.json");
        let mut store = KvStore::new(path);

        store.set("key1", "value1");
        assert!(store.delete("key1"));
        assert_store_missing(&store, "key1");
        assert!(!store.delete("key1"));
    }

    #[test]
    fn test_store_len() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("test.json");
        let mut store = KvStore::new(path);

        assert!(store.is_empty());
        store.set("a", "1");
        store.set("b", "2");
        assert_eq!(store.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Persistence integration tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_save_and_load() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("persist.json");
        let mut store = KvStore::new(path.clone());

        store.set("host", "localhost");
        store.set("port", "8080");
        store.save().unwrap();

        let loaded = KvStore::load(&path).unwrap();
        assert_store_contains(&loaded, "host", "localhost");
        assert_store_contains(&loaded, "port", "8080");
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = KvStore::load(&PathBuf::from("/nonexistent/path.json"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_invalid_json() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("bad.json");
        std::fs::write(&path, "not json").unwrap();

        let result = KvStore::load(&path);
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // Test data builder integration
    // -----------------------------------------------------------------------

    #[test]
    fn test_data_builder() {
        let data = TestDataBuilder::new()
            .with("name", "Alice")
            .with("email", "alice@example.com")
            .with("role", "admin")
            .build();

        assert_eq!(data.len(), 3);
        assert_eq!(data[0].0, "name");
    }

    #[test]
    fn test_fixture_create_store() {
        let fixture = TestFixture::new();
        let path = fixture.create_store("config.json", &[("debug", "true"), ("port", "3000")]);

        let store = KvStore::load(&path).unwrap();
        assert_store_contains(&store, "debug", "true");
        assert_store_contains(&store, "port", "3000");
    }

    // -----------------------------------------------------------------------
    // Multi-step integration test
    // -----------------------------------------------------------------------

    #[test]
    fn test_full_workflow() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("workflow.json");

        // Step 1: Create and populate
        let mut store = KvStore::new(path.clone());
        store.set("app_name", "myapp");
        store.set("version", "1.0.0");
        store.set("env", "test");

        // Step 2: Save
        store.save().unwrap();

        // Step 3: Load in a new instance
        let mut loaded = KvStore::load(&path).unwrap();
        assert_eq!(loaded.len(), 3);

        // Step 4: Modify and save again
        loaded.set("version", "1.1.0");
        loaded.delete("env");
        loaded.save().unwrap();

        // Step 5: Reload and verify
        let final_store = KvStore::load(&path).unwrap();
        assert_store_contains(&final_store, "app_name", "myapp");
        assert_store_contains(&final_store, "version", "1.1.0");
        assert_store_missing(&final_store, "env");
    }

    // -----------------------------------------------------------------------
    // Keys listing
    // -----------------------------------------------------------------------

    #[test]
    fn test_store_keys() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("keys.json");
        let mut store = KvStore::new(path);

        store.set("a", "1");
        store.set("b", "2");
        store.set("c", "3");

        let mut keys = store.keys();
        keys.sort();
        assert_eq!(keys, vec!["a", "b", "c"]);
    }

    // -----------------------------------------------------------------------
    // Edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_key() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("test.json");
        let mut store = KvStore::new(path);

        store.set("", "empty key");
        assert_store_contains(&store, "", "empty key");
    }

    #[test]
    fn test_empty_value() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("test.json");
        let mut store = KvStore::new(path);

        store.set("key", "");
        assert_store_contains(&store, "key", "");
    }

    #[test]
    fn test_overwrite_value() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("test.json");
        let mut store = KvStore::new(path);

        store.set("key", "old");
        store.set("key", "new");
        assert_store_contains(&store, "key", "new");
        assert_eq!(store.len(), 1);
    }

    // -----------------------------------------------------------------------
    // Test helper function tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_assert_str_eq_pass() {
        assert_str_eq("hello", "hello");
    }

    #[test]
    #[should_panic]
    fn test_assert_str_eq_fail() {
        assert_str_eq("hello", "world");
    }

    #[test]
    fn test_assert_store_contains_pass() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("test.json");
        let mut store = KvStore::new(path);
        store.set("key", "value");
        assert_store_contains(&store, "key", "value");
    }

    #[test]
    fn test_assert_store_missing_pass() {
        let fixture = TestFixture::new();
        let path = fixture.file_path("test.json");
        let store = KvStore::new(path);
        assert_store_missing(&store, "key");
    }

    #[test]
    fn test_test_data_builder_as_str_refs() {
        let builder = TestDataBuilder::new()
            .with("a", "1")
            .with("b", "2");
        let refs = builder.as_str_refs();
        assert_eq!(refs, vec![("a", "1"), ("b", "2")]);
    }

    // -----------------------------------------------------------------------
    // Multiple stores in same fixture
    // -----------------------------------------------------------------------

    #[test]
    fn test_multiple_stores() {
        let fixture = TestFixture::new();

        let path1 = fixture.create_store("store1.json", &[("a", "1")]);
        let path2 = fixture.create_store("store2.json", &[("b", "2")]);

        let store1 = KvStore::load(&path1).unwrap();
        let store2 = KvStore::load(&path2).unwrap();

        assert_store_contains(&store1, "a", "1");
        assert_store_missing(&store1, "b");

        assert_store_contains(&store2, "b", "2");
        assert_store_missing(&store2, "a");
    }
}
