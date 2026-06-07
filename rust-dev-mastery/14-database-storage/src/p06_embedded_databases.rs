//! # Embedded Databases
//!
//! Embedded databases run in-process, requiring no external server. This lesson
//! covers SQLite patterns, key-value stores, and when to use embedded vs
//! client-server databases.
//!
//! ## Key Concepts
//! - SQLite as an embedded database
//! - Key-value store patterns
//! - Zero-configuration database selection
//! - Embedded vs client-server trade-offs
//! - In-memory databases for testing
//! - Concurrent access patterns

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

// ---------------------------------------------------------------------------
// 1. Key-Value Store Trait
// ---------------------------------------------------------------------------

/// A trait for simple key-value storage.
pub trait KeyValueStore: Send + Sync {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn put(&self, key: &str, value: &[u8]) -> Result<(), StoreError>;
    fn delete(&self, key: &str) -> Result<bool, StoreError>;
    fn contains(&self, key: &str) -> bool;
    fn iter_prefix(&self, prefix: &str) -> Vec<(String, Vec<u8>)>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

// ---------------------------------------------------------------------------
// 2. In-Memory Key-Value Store
// ---------------------------------------------------------------------------

/// A thread-safe in-memory key-value store backed by a BTreeMap.
/// Useful for testing and as a reference implementation.
#[derive(Debug, Clone)]
pub struct MemoryStore {
    data: Arc<RwLock<BTreeMap<String, Vec<u8>>>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn with_entries(entries: Vec<(&str, &[u8])>) -> Self {
        let store = Self::new();
        for (key, value) in entries {
            store.put(key, value).unwrap();
        }
        store
    }

    /// Get all keys (for debugging).
    pub fn keys(&self) -> Vec<String> {
        self.data.read().unwrap().keys().cloned().collect()
    }

    /// Clear all data.
    pub fn clear(&self) {
        self.data.write().unwrap().clear();
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyValueStore for MemoryStore {
    fn get(&self, key: &str) -> Option<Vec<u8>> {
        self.data.read().unwrap().get(key).cloned()
    }

    fn put(&self, key: &str, value: &[u8]) -> Result<(), StoreError> {
        self.data
            .write()
            .unwrap()
            .insert(key.to_string(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<bool, StoreError> {
        Ok(self.data.write().unwrap().remove(key).is_some())
    }

    fn contains(&self, key: &str) -> bool {
        self.data.read().unwrap().contains_key(key)
    }

    fn iter_prefix(&self, prefix: &str) -> Vec<(String, Vec<u8>)> {
        self.data
            .read()
            .unwrap()
            .range(prefix.to_string()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    fn len(&self) -> usize {
        self.data.read().unwrap().len()
    }

    fn is_empty(&self) -> bool {
        self.data.read().unwrap().is_empty()
    }
}

// ---------------------------------------------------------------------------
// 3. Database Types
// ---------------------------------------------------------------------------

/// Supported embedded database backends.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DatabaseBackend {
    /// SQLite — full SQL support.
    Sqlite,
    /// Simple in-memory key-value store.
    Memory,
}

impl DatabaseBackend {
    pub fn description(&self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite - full SQL, file-based or in-memory",
            Self::Memory => "In-memory key-value store, no persistence",
        }
    }

    pub fn supports_sql(&self) -> bool {
        matches!(self, Self::Sqlite)
    }

    pub fn supports_persistence(&self) -> bool {
        matches!(self, Self::Sqlite)
    }

    pub fn supports_concurrent_writers(&self) -> bool {
        false // Both SQLite and our memory store serialize writes
    }
}

// ---------------------------------------------------------------------------
// 4. Simple Table Store (SQLite-like patterns)
// ---------------------------------------------------------------------------

/// A table-like store that demonstrates SQLite patterns without actual SQL.
/// Stores rows as ordered maps of column name -> value.
#[derive(Debug)]
pub struct TableStore {
    name: String,
    columns: Vec<String>,
    rows: BTreeMap<u64, BTreeMap<String, String>>,
    next_id: u64,
    indexes: BTreeMap<String, BTreeMap<String, Vec<u64>>>,
}

impl TableStore {
    pub fn new(name: impl Into<String>, columns: Vec<String>) -> Self {
        Self {
            name: name.into(),
            columns,
            rows: BTreeMap::new(),
            next_id: 1,
            indexes: BTreeMap::new(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Insert a row. Returns the generated ID.
    pub fn insert(&mut self, values: BTreeMap<String, String>) -> Result<u64, StoreError> {
        // Validate columns
        for key in values.keys() {
            if !self.columns.contains(key) {
                return Err(StoreError::InvalidColumn(key.clone()));
            }
        }

        let id = self.next_id;
        self.next_id += 1;

        // Update indexes
        for (col, idx) in &mut self.indexes {
            if let Some(val) = values.get(col) {
                idx.entry(val.clone()).or_default().push(id);
            }
        }

        self.rows.insert(id, values);
        Ok(id)
    }

    /// Get a row by ID.
    pub fn get(&self, id: u64) -> Option<&BTreeMap<String, String>> {
        self.rows.get(&id)
    }

    /// Update a row.
    pub fn update(
        &mut self,
        id: u64,
        values: BTreeMap<String, String>,
    ) -> Result<(), StoreError> {
        if !self.rows.contains_key(&id) {
            return Err(StoreError::NotFound(format!("row {id}")));
        }

        for key in values.keys() {
            if !self.columns.contains(key) {
                return Err(StoreError::InvalidColumn(key.clone()));
            }
        }

        self.rows.insert(id, values);
        Ok(())
    }

    /// Delete a row.
    pub fn delete(&mut self, id: u64) -> bool {
        self.rows.remove(&id).is_some()
    }

    /// Get all rows.
    pub fn all(&self) -> &BTreeMap<u64, BTreeMap<String, String>> {
        &self.rows
    }

    /// Count rows.
    pub fn count(&self) -> usize {
        self.rows.len()
    }

    /// Create an index on a column.
    pub fn create_index(&mut self, column: &str) -> Result<(), StoreError> {
        if !self.columns.contains(&column.to_string()) {
            return Err(StoreError::InvalidColumn(column.into()));
        }

        let mut idx: BTreeMap<String, Vec<u64>> = BTreeMap::new();
        for (id, row) in &self.rows {
            if let Some(val) = row.get(column) {
                idx.entry(val.clone()).or_default().push(*id);
            }
        }
        self.indexes.insert(column.into(), idx);
        Ok(())
    }

    /// Find rows by indexed column value.
    pub fn find_by_index(&self, column: &str, value: &str) -> Vec<u64> {
        self.indexes
            .get(column)
            .and_then(|idx| idx.get(value))
            .cloned()
            .unwrap_or_default()
    }

    /// Search rows where a column contains the given text.
    pub fn search(&self, column: &str, query: &str) -> Vec<(u64, &BTreeMap<String, String>)> {
        let query_lower = query.to_lowercase();
        self.rows
            .iter()
            .filter(|(_, row)| {
                row.get(column)
                    .map(|v| v.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .map(|(id, row)| (*id, row))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 5. Store Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("key not found: {0}")]
    NotFound(String),

    #[error("invalid column: {0}")]
    InvalidColumn(String),

    #[error("store is read-only")]
    ReadOnly,

    #[error("IO error: {0}")]
    Io(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_basic() {
        let store = MemoryStore::new();
        assert!(store.is_empty());

        store.put("key1", b"value1").unwrap();
        assert_eq!(store.get("key1"), Some(b"value1".to_vec()));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_memory_store_overwrite() {
        let store = MemoryStore::new();
        store.put("key", b"old").unwrap();
        store.put("key", b"new").unwrap();
        assert_eq!(store.get("key"), Some(b"new".to_vec()));
    }

    #[test]
    fn test_memory_store_delete() {
        let store = MemoryStore::new();
        store.put("key", b"value").unwrap();
        assert!(store.delete("key").unwrap());
        assert!(!store.delete("nonexistent").unwrap());
        assert!(store.get("key").is_none());
    }

    #[test]
    fn test_memory_store_contains() {
        let store = MemoryStore::new();
        store.put("exists", b"yes").unwrap();
        assert!(store.contains("exists"));
        assert!(!store.contains("missing"));
    }

    #[test]
    fn test_memory_store_iter_prefix() {
        let store = MemoryStore::new();
        store.put("user:1", b"alice").unwrap();
        store.put("user:2", b"bob").unwrap();
        store.put("post:1", b"hello").unwrap();

        let users = store.iter_prefix("user:");
        assert_eq!(users.len(), 2);
        assert!(users.iter().all(|(k, _)| k.starts_with("user:")));
    }

    #[test]
    fn test_memory_store_with_entries() {
        let store = MemoryStore::with_entries(vec![
            ("a", b"1"),
            ("b", b"2"),
            ("c", b"3"),
        ]);
        assert_eq!(store.len(), 3);
    }

    #[test]
    fn test_memory_store_clear() {
        let store = MemoryStore::new();
        store.put("key", b"val").unwrap();
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn test_memory_store_keys() {
        let store = MemoryStore::new();
        store.put("z", b"1").unwrap();
        store.put("a", b"2").unwrap();
        store.put("m", b"3").unwrap();

        let keys = store.keys();
        assert_eq!(keys, vec!["a", "m", "z"]); // BTreeMap is ordered
    }

    #[test]
    fn test_database_backend() {
        assert!(DatabaseBackend::Sqlite.supports_sql());
        assert!(!DatabaseBackend::Memory.supports_sql());
        assert!(DatabaseBackend::Sqlite.supports_persistence());
        assert!(!DatabaseBackend::Memory.supports_persistence());
        assert!(!DatabaseBackend::Sqlite.supports_concurrent_writers());
    }

    #[test]
    fn test_table_store_insert_and_get() {
        let mut store = TableStore::new("users", vec!["name".into(), "email".into()]);

        let mut values = BTreeMap::new();
        values.insert("name".into(), "Alice".into());
        values.insert("email".into(), "alice@example.com".into());

        let id = store.insert(values).unwrap();
        assert_eq!(id, 1);

        let row = store.get(id).unwrap();
        assert_eq!(row.get("name").unwrap(), "Alice");
        assert_eq!(row.get("email").unwrap(), "alice@example.com");
    }

    #[test]
    fn test_table_store_update() {
        let mut store = TableStore::new("users", vec!["name".into()]);

        let mut values = BTreeMap::new();
        values.insert("name".into(), "Alice".into());
        let id = store.insert(values).unwrap();

        let mut update = BTreeMap::new();
        update.insert("name".into(), "Alice Updated".into());
        store.update(id, update).unwrap();

        assert_eq!(store.get(id).unwrap().get("name").unwrap(), "Alice Updated");
    }

    #[test]
    fn test_table_store_delete() {
        let mut store = TableStore::new("users", vec!["name".into()]);

        let mut values = BTreeMap::new();
        values.insert("name".into(), "Alice".into());
        let id = store.insert(values).unwrap();

        assert!(store.delete(id));
        assert!(store.get(id).is_none());
        assert!(!store.delete(999));
    }

    #[test]
    fn test_table_store_invalid_column() {
        let mut store = TableStore::new("users", vec!["name".into()]);

        let mut values = BTreeMap::new();
        values.insert("invalid_col".into(), "val".into());
        assert!(store.insert(values).is_err());
    }

    #[test]
    fn test_table_store_update_not_found() {
        let mut store = TableStore::new("users", vec!["name".into()]);

        let mut values = BTreeMap::new();
        values.insert("name".into(), "val".into());
        assert!(store.update(999, values).is_err());
    }

    #[test]
    fn test_table_store_index() {
        let mut store = TableStore::new("users", vec!["name".into(), "role".into()]);

        let mut v1 = BTreeMap::new();
        v1.insert("name".into(), "Alice".into());
        v1.insert("role".into(), "admin".into());
        let id1 = store.insert(v1).unwrap();

        let mut v2 = BTreeMap::new();
        v2.insert("name".into(), "Bob".into());
        v2.insert("role".into(), "user".into());
        store.insert(v2).unwrap();

        let mut v3 = BTreeMap::new();
        v3.insert("name".into(), "Charlie".into());
        v3.insert("role".into(), "admin".into());
        let id3 = store.insert(v3).unwrap();

        store.create_index("role").unwrap();
        let admins = store.find_by_index("role", "admin");
        assert_eq!(admins, vec![id1, id3]);
    }

    #[test]
    fn test_table_store_search() {
        let mut store = TableStore::new("users", vec!["name".into()]);

        let mut v1 = BTreeMap::new();
        v1.insert("name".into(), "Alice Smith".into());
        store.insert(v1).unwrap();

        let mut v2 = BTreeMap::new();
        v2.insert("name".into(), "Bob Jones".into());
        store.insert(v2).unwrap();

        let mut v3 = BTreeMap::new();
        v3.insert("name".into(), "Alice Johnson".into());
        store.insert(v3).unwrap();

        let results = store.search("name", "alice");
        assert_eq!(results.len(), 2);

        let results = store.search("name", "bob");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_table_store_count() {
        let mut store = TableStore::new("items", vec!["val".into()]);
        assert_eq!(store.count(), 0);

        let mut v = BTreeMap::new();
        v.insert("val".into(), "1".into());
        store.insert(v).unwrap();
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_table_store_name() {
        let store = TableStore::new("my_table", vec!["col".into()]);
        assert_eq!(store.name(), "my_table");
        assert_eq!(store.columns(), &["col"]);
    }

    #[test]
    fn test_store_error_display() {
        let err = StoreError::NotFound("key".into());
        assert!(err.to_string().contains("key"));

        let err = StoreError::InvalidColumn("col".into());
        assert!(err.to_string().contains("col"));

        let err = StoreError::ReadOnly;
        assert!(!err.to_string().is_empty());
    }
}
