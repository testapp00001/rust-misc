//! # Generics vs Dynamic Dispatch
//!
//! Choosing between static dispatch (generics/monomorphization) and dynamic
//! dispatch (trait objects) is a fundamental API design decision. This module
//! covers the tradeoffs and when to use each approach.
//!
//! ## Key Concepts
//! - **Static dispatch**: Generics — zero overhead, but code bloat
//! - **Dynamic dispatch**: Trait objects — runtime cost, but smaller binary
//! - **Monomorphization cost**: Compile time and binary size
//! - **When to use each**: Performance-critical vs flexibility

/// A storage trait demonstrating both static and dynamic dispatch approaches.
pub trait Storage {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: String, value: String);
    fn delete(&mut self, key: &str) -> bool;
}

/// In-memory storage implementation.
pub struct MemoryStorage {
    data: std::collections::HashMap<String, String>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            data: std::collections::HashMap::new(),
        }
    }
}

impl Storage for MemoryStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }
}

/// Read-only storage (no modification).
pub struct ReadOnlyStorage {
    data: std::collections::HashMap<String, String>,
}

impl ReadOnlyStorage {
    pub fn new(data: std::collections::HashMap<String, String>) -> Self {
        ReadOnlyStorage { data }
    }
}

impl Storage for ReadOnlyStorage {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, _key: String, _value: String) {
        // No-op for read-only
    }

    fn delete(&mut self, _key: &str) -> bool {
        false // Cannot delete
    }
}

/// Static dispatch version: generic over the storage type.
/// The compiler generates a specialized version for each concrete type used.
pub struct StaticConfig<S: Storage> {
    storage: S,
    prefix: String,
}

impl<S: Storage> StaticConfig<S> {
    pub fn new(storage: S, prefix: impl Into<String>) -> Self {
        StaticConfig {
            storage,
            prefix: prefix.into(),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.storage.get(&format!("{}{key}", self.prefix))
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.storage
            .set(format!("{}{key}", self.prefix), value.to_string());
    }
}

/// Dynamic dispatch version: uses a trait object.
/// Only one copy of the code is compiled, with vtable indirection.
pub struct DynamicConfig {
    storage: Box<dyn Storage>,
    prefix: String,
}

impl DynamicConfig {
    pub fn new(storage: Box<dyn Storage>, prefix: impl Into<String>) -> Self {
        DynamicConfig {
            storage,
            prefix: prefix.into(),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.storage.get(&format!("{}{key}", self.prefix))
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.storage
            .set(format!("{}{key}", self.prefix), value.to_string());
    }
}

/// Demonstrates enum dispatch as an alternative to trait objects.
/// Faster than dyn dispatch because the compiler can inline and optimize.
pub enum AnyStorage {
    Memory(MemoryStorage),
    ReadOnly(ReadOnlyStorage),
}

impl Storage for AnyStorage {
    fn get(&self, key: &str) -> Option<String> {
        match self {
            AnyStorage::Memory(s) => s.get(key),
            AnyStorage::ReadOnly(s) => s.get(key),
        }
    }

    fn set(&mut self, key: String, value: String) {
        match self {
            AnyStorage::Memory(s) => s.set(key, value),
            AnyStorage::ReadOnly(s) => s.set(key, value),
        }
    }

    fn delete(&mut self, key: &str) -> bool {
        match self {
            AnyStorage::Memory(s) => s.delete(key),
            AnyStorage::ReadOnly(s) => s.delete(key),
        }
    }
}

/// A function that accepts either static or dynamic dispatch.
/// Uses `impl Trait` for the common case (static dispatch).
pub fn get_config_value(storage: &impl Storage, key: &str) -> Option<String> {
    storage.get(key)
}

/// Batch processor using generics (static dispatch).
pub fn process_batch_static<T: Storage>(storage: &T, keys: &[&str]) -> Vec<Option<String>> {
    keys.iter().map(|k| storage.get(k)).collect()
}

/// Batch processor using trait objects (dynamic dispatch).
pub fn process_batch_dynamic(storage: &dyn Storage, keys: &[&str]) -> Vec<Option<String>> {
    keys.iter().map(|k| storage.get(k)).collect()
}

/// Demonstrates the cost of monomorphization with a generic function
/// that is instantiated for many types.
pub fn generic_operation<T: std::fmt::Display + Clone>(items: &[T]) -> String {
    items
        .iter()
        .map(|item| format!("{item}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The same operation using dynamic dispatch (smaller binary).
pub fn dynamic_operation(items: &[&dyn std::fmt::Display]) -> String {
    items
        .iter()
        .map(|item| format!("{item}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A plugin system using trait objects.
/// Plugins can be loaded at runtime and invoked dynamically.
pub trait Plugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn execute(&self, input: &str) -> Result<String, String>;
}

pub struct PluginRegistry {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        PluginRegistry {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn execute(&self, plugin_name: &str, input: &str) -> Result<String, String> {
        self.plugins
            .iter()
            .find(|p| p.name() == plugin_name)
            .ok_or_else(|| format!("Plugin not found: {plugin_name}"))
            .and_then(|p| p.execute(input))
    }

    pub fn list_plugins(&self) -> Vec<(&str, &str)> {
        self.plugins
            .iter()
            .map(|p| (p.name(), p.version()))
            .collect()
    }
}

/// A concrete plugin implementation.
pub struct ReversePlugin;

impl Plugin for ReversePlugin {
    fn name(&self) -> &str {
        "reverse"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn execute(&self, input: &str) -> Result<String, String> {
        Ok(input.chars().rev().collect())
    }
}

pub struct UpperPlugin;

impl Plugin for UpperPlugin {
    fn name(&self) -> &str {
        "upper"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn execute(&self, input: &str) -> Result<String, String> {
        Ok(input.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_config() {
        let storage = MemoryStorage::new();
        let mut config = StaticConfig::new(storage, "app.");

        config.set("name", "MyApp");
        assert_eq!(config.get("name"), Some("MyApp".to_string()));
    }

    #[test]
    fn test_dynamic_config() {
        let mut config = DynamicConfig::new(Box::new(MemoryStorage::new()), "app.");
        config.set("name", "MyApp");
        assert_eq!(config.get("name"), Some("MyApp".to_string()));
    }

    #[test]
    fn test_dynamic_config_readonly() {
        let mut config = DynamicConfig::new(Box::new(ReadOnlyStorage::new(
            [("key".into(), "value".into())].into_iter().collect(),
        )), "");

        assert_eq!(config.get("key"), Some("value".to_string()));
        config.set("key", "new_value"); // No-op
        assert_eq!(config.get("key"), Some("value".to_string())); // Unchanged
    }

    #[test]
    fn test_enum_dispatch() {
        let mut storage = AnyStorage::Memory(MemoryStorage::new());
        storage.set("key".into(), "value".into());
        assert_eq!(storage.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_static_batch() {
        let mut storage = MemoryStorage::new();
        storage.set("a".into(), "1".into());
        storage.set("b".into(), "2".into());

        let results = process_batch_static(&storage, &["a", "b", "c"]);
        assert_eq!(
            results,
            vec![
                Some("1".to_string()),
                Some("2".to_string()),
                None
            ]
        );
    }

    #[test]
    fn test_dynamic_batch() {
        let mut storage = MemoryStorage::new();
        storage.set("a".into(), "1".into());

        let results = process_batch_dynamic(&storage, &["a", "missing"]);
        assert_eq!(
            results,
            vec![Some("1".to_string()), None]
        );
    }

    #[test]
    fn test_generic_operation() {
        let items = vec![1, 2, 3];
        assert_eq!(generic_operation(&items), "1, 2, 3");

        let items = vec!["a", "b", "c"];
        assert_eq!(generic_operation(&items), "a, b, c");
    }

    #[test]
    fn test_dynamic_operation() {
        let items: Vec<&dyn std::fmt::Display> = vec![&1, &"hello", &3.14];
        assert_eq!(dynamic_operation(&items), "1, hello, 3.14");
    }

    #[test]
    fn test_plugin_registry() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(ReversePlugin));
        registry.register(Box::new(UpperPlugin));

        let result = registry.execute("reverse", "hello").unwrap();
        assert_eq!(result, "olleh");

        let result = registry.execute("upper", "hello").unwrap();
        assert_eq!(result, "HELLO");
    }

    #[test]
    fn test_plugin_registry_not_found() {
        let registry = PluginRegistry::new();
        let result = registry.execute("nonexistent", "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_list() {
        let mut registry = PluginRegistry::new();
        registry.register(Box::new(ReversePlugin));
        registry.register(Box::new(UpperPlugin));

        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 2);
        assert_eq!(plugins[0].0, "reverse");
        assert_eq!(plugins[1].0, "upper");
    }
}
