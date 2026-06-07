//! # Lesson 3: Module System Mastery
//!
//! Rust's module system controls visibility, namespacing, and code organization.
//! This lesson covers the module tree, pub/pub(crate)/pub(super) visibility,
//! re-exports with `pub use`, glob imports, and the mod.rs vs filename.rs debate.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Visibility demo types
// ---------------------------------------------------------------------------

/// A publicly visible struct — any crate can use it.
#[derive(Debug, Clone, PartialEq)]
pub struct PublicConfig {
    pub name: String,
    pub max_connections: usize,
}

/// This struct is public but its fields are private.
/// Callers must use constructor and accessor methods.
#[derive(Debug, Clone)]
pub struct OpaqueHandle {
    id: u64,
    created_at: std::time::Instant,
}

impl OpaqueHandle {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            created_at: std::time::Instant::now(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn age_secs(&self) -> u64 {
        self.created_at.elapsed().as_secs()
    }
}

/// A struct with pub(crate) fields — visible within this crate only.
#[derive(Debug)]
pub struct InternalState {
    pub(crate) counter: u64,
    pub(crate) cache: HashMap<String, String>,
}

impl InternalState {
    pub fn new() -> Self {
        Self {
            counter: 0,
            cache: HashMap::new(),
        }
    }

    pub fn increment(&mut self) {
        self.counter += 1;
    }

    pub fn get_counter(&self) -> u64 {
        self.counter
    }
}

// ---------------------------------------------------------------------------
// Sub-module simulation: in real code, these would be separate files
// ---------------------------------------------------------------------------

/// Simulates a submodule's public API.
/// In a real project, this would be in a separate file like `db/mod.rs` or `db.rs`.
pub mod db {
    use std::collections::HashMap;

    /// Database connection configuration.
    #[derive(Debug, Clone)]
    pub struct ConnectionConfig {
        pub host: String,
        pub port: u16,
        pub database: String,
        pub max_pool_size: usize,
    }

    impl ConnectionConfig {
        pub fn new(host: impl Into<String>, database: impl Into<String>) -> Self {
            Self {
                host: host.into(),
                port: 5432,
                database: database.into(),
                max_pool_size: 10,
            }
        }

        pub fn with_port(mut self, port: u16) -> Self {
            self.port = port;
            self
        }

        pub fn with_pool_size(mut self, size: usize) -> Self {
            self.max_pool_size = size;
            self
        }

        pub fn connection_string(&self) -> String {
            format!("postgres://{}:{}/{}", self.host, self.port, self.database)
        }
    }

    /// A row returned from a query.
    pub type Row = HashMap<String, Value>;

    /// A database value (simplified).
    #[derive(Debug, Clone, PartialEq)]
    pub enum Value {
        Null,
        Text(String),
        Integer(i64),
        Float(f64),
        Boolean(bool),
    }

    /// Result of executing a query.
    #[derive(Debug)]
    pub struct QueryResult {
        pub columns: Vec<String>,
        pub rows: Vec<Row>,
    }

    impl QueryResult {
        pub fn new(columns: Vec<String>) -> Self {
            Self {
                columns,
                rows: Vec::new(),
            }
        }

        pub fn add_row(&mut self, row: Row) {
            self.rows.push(row);
        }

        pub fn row_count(&self) -> usize {
            self.rows.len()
        }

        pub fn is_empty(&self) -> bool {
            self.rows.is_empty()
        }
    }
}

/// Simulates a submodule for caching with re-exports.
pub mod cache {
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    /// A cache entry with expiration.
    #[derive(Debug, Clone)]
    pub struct CacheEntry<V> {
        pub value: V,
        pub inserted_at: Instant,
        pub ttl: Duration,
    }

    impl<V> CacheEntry<V> {
        pub fn new(value: V, ttl: Duration) -> Self {
            Self {
                value,
                inserted_at: Instant::now(),
                ttl,
            }
        }

        pub fn is_expired(&self) -> bool {
            self.inserted_at.elapsed() > self.ttl
        }
    }

    /// A simple key-value cache with TTL support.
    #[derive(Debug)]
    pub struct Cache<K, V> {
        entries: HashMap<K, CacheEntry<V>>,
        default_ttl: Duration,
    }

    impl<K: Eq + std::hash::Hash + Clone, V: Clone> Cache<K, V> {
        pub fn new(default_ttl: Duration) -> Self {
            Self {
                entries: HashMap::new(),
                default_ttl,
            }
        }

        pub fn insert(&mut self, key: K, value: V) {
            self.entries
                .insert(key, CacheEntry::new(value, self.default_ttl));
        }

        pub fn get(&self, key: &K) -> Option<&V> {
            self.entries.get(key).and_then(|entry| {
                if entry.is_expired() {
                    None
                } else {
                    Some(&entry.value)
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

        /// Remove all expired entries.
        pub fn evict_expired(&mut self) {
            self.entries.retain(|_, entry| !entry.is_expired());
        }
    }
}

// ---------------------------------------------------------------------------
// Re-export pattern: consolidate public API at the crate root
// ---------------------------------------------------------------------------

/// Re-export commonly used types so callers can use `use mycrate::prelude::*`.
pub mod prelude {
    pub use super::db::{ConnectionConfig, QueryResult, Value};
    pub use super::cache::{Cache, CacheEntry};
    pub use super::{OpaqueHandle, PublicConfig};
}

/// Demonstrate the module tree as a string representation.
pub fn module_tree_string() -> String {
    let mut lines = Vec::new();
    lines.push("crate".to_string());
    lines.push("  PublicConfig (pub struct)".to_string());
    lines.push("  OpaqueHandle (pub struct, private fields)".to_string());
    lines.push("  InternalState (pub struct, pub(crate) fields)".to_string());
    lines.push("  db (pub mod)".to_string());
    lines.push("    ConnectionConfig (pub struct)".to_string());
    lines.push("    Value (pub enum)".to_string());
    lines.push("    QueryResult (pub struct)".to_string());
    lines.push("  cache (pub mod)".to_string());
    lines.push("    CacheEntry (pub struct)".to_string());
    lines.push("    Cache (pub struct)".to_string());
    lines.push("  prelude (pub mod, re-exports)".to_string());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_public_config() {
        let config = PublicConfig {
            name: "production".to_string(),
            max_connections: 100,
        };
        assert_eq!(config.name, "production");
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_opaque_handle() {
        let handle = OpaqueHandle::new(42);
        assert_eq!(handle.id(), 42);
        // age might be 0 since it was just created
        let _ = handle.age_secs();
    }

    #[test]
    fn test_internal_state() {
        let mut state = InternalState::new();
        assert_eq!(state.get_counter(), 0);
        state.increment();
        state.increment();
        assert_eq!(state.get_counter(), 2);
        // pub(crate) fields accessible within the crate
        assert_eq!(state.counter, 2);
    }

    #[test]
    fn test_db_connection_config() {
        let config = db::ConnectionConfig::new("localhost", "mydb")
            .with_port(5433)
            .with_pool_size(20);
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5433);
        assert_eq!(config.database, "mydb");
        assert_eq!(config.max_pool_size, 20);
        assert_eq!(
            config.connection_string(),
            "postgres://localhost:5433/mydb"
        );
    }

    #[test]
    fn test_db_query_result() {
        let mut result = db::QueryResult::new(vec!["id".to_string(), "name".to_string()]);
        assert!(result.is_empty());

        let mut row = std::collections::HashMap::new();
        row.insert("id".to_string(), db::Value::Integer(1));
        row.insert("name".to_string(), db::Value::Text("Alice".to_string()));
        result.add_row(row);

        assert_eq!(result.row_count(), 1);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_cache_basic() {
        let mut cache = cache::Cache::new(Duration::from_secs(60));
        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some(&"value1"));
        assert_eq!(cache.get(&"key2"), None);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = cache::Cache::new(Duration::from_secs(60));
        cache.insert("key1", "value1");
        assert!(cache.remove(&"key1"));
        assert!(!cache.remove(&"key1"));
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_entry_expiration() {
        let entry = cache::CacheEntry::new("value", Duration::from_nanos(0));
        // With 0 TTL, entry should be expired immediately
        std::thread::sleep(Duration::from_millis(1));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_evict_expired() {
        let mut cache = cache::Cache::new(Duration::from_nanos(0));
        cache.insert("a", 1);
        cache.insert("b", 2);
        std::thread::sleep(Duration::from_millis(1));
        cache.evict_expired();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_prelude_re_exports() {
        // Verify that prelude items are accessible
        let config = prelude::PublicConfig {
            name: "test".to_string(),
            max_connections: 10,
        };
        let _ = prelude::OpaqueHandle::new(1);
        let _ = prelude::ConnectionConfig::new("host", "db");
        let _: prelude::Cache<&str, i32> = prelude::Cache::new(Duration::from_secs(10));
    }

    #[test]
    fn test_module_tree_string() {
        let tree = module_tree_string();
        assert!(tree.contains("crate"));
        assert!(tree.contains("PublicConfig"));
        assert!(tree.contains("db (pub mod)"));
        assert!(tree.contains("cache (pub mod)"));
        assert!(tree.contains("prelude (pub mod, re-exports)"));
    }

    #[test]
    fn test_db_value_variants() {
        assert_eq!(db::Value::Null, db::Value::Null);
        assert_ne!(db::Value::Integer(1), db::Value::Integer(2));
        assert_eq!(
            db::Value::Text("hello".to_string()),
            db::Value::Text("hello".to_string())
        );
    }
}
