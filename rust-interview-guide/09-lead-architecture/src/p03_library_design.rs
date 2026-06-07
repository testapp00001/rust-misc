/// Problem: Library Design
///
/// Master library design in Rust.
///
/// Key Concepts:
/// - Public API
/// - Error handling
/// - Documentation
/// - Testing
/// - Versioning

/// Problem 1: Public API design
/// Design clean public API
pub struct Library {
    data: std::collections::HashMap<String, String>,
}

impl Library {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Problem 2: Error handling
/// Design error types
#[derive(Debug)]
pub enum LibraryError {
    NotFound(String),
    InvalidInput(String),
    Internal(String),
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LibraryError::NotFound(msg) => write!(f, "Not found: {}", msg),
            LibraryError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            LibraryError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for LibraryError {}

/// Problem 3: Builder pattern
/// Design builder for library
pub struct LibraryBuilder {
    capacity: Option<usize>,
    case_sensitive: bool,
}

impl LibraryBuilder {
    pub fn new() -> Self {
        Self {
            capacity: None,
            case_sensitive: true,
        }
    }

    pub fn capacity(mut self, capacity: usize) -> Self {
        self.capacity = Some(capacity);
        self
    }

    pub fn case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    pub fn build(self) -> Library {
        Library::new()
    }
}

/// Problem 4: Iterator support
/// Support iterators
pub struct LibraryIterator<'a> {
    iter: std::collections::hash_map::Iter<'a, String, String>,
}

impl<'a> Iterator for LibraryIterator<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// Problem 5: Trait implementations
/// Implement standard traits
impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Library {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Library")
            .field("len", &self.len())
            .finish()
    }
}

/// Problem 6: Generic support
/// Support generics
pub struct GenericStore<K, V> {
    data: std::collections::HashMap<K, V>,
}

impl<K: Eq + std::hash::Hash, V> GenericStore<K, V> {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    pub fn set(&mut self, key: K, value: V) {
        self.data.insert(key, value);
    }
}

/// Problem 7: Feature flags
/// Support feature flags
pub struct FeatureFlags {
    flags: std::collections::HashMap<String, bool>,
}

impl FeatureFlags {
    pub fn new() -> Self {
        Self {
            flags: std::collections::HashMap::new(),
        }
    }

    pub fn enable(&mut self, feature: &str) {
        self.flags.insert(feature.to_string(), true);
    }

    pub fn disable(&mut self, feature: &str) {
        self.flags.insert(feature.to_string(), false);
    }

    pub fn is_enabled(&self, feature: &str) -> bool {
        self.flags.get(feature).copied().unwrap_or(false)
    }
}

/// Problem 8: Plugin system
/// Design plugin system
pub trait Plugin {
    fn name(&self) -> &str;
    fn execute(&self, input: &str) -> String;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn execute(&self, input: &str) -> Vec<String> {
        self.plugins.iter().map(|p| p.execute(input)).collect()
    }
}

/// Problem 9: Event system
/// Design event system
pub struct EventBus {
    listeners: std::collections::HashMap<String, Vec<Box<dyn Fn(&str)>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: std::collections::HashMap::new(),
        }
    }

    pub fn subscribe(&mut self, event: &str, callback: Box<dyn Fn(&str)>) {
        self.listeners
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
    }

    pub fn publish(&self, event: &str, data: &str) {
        if let Some(callbacks) = self.listeners.get(event) {
            for callback in callbacks {
                callback(data);
            }
        }
    }
}

/// Problem 10: Middleware support
/// Support middleware
pub trait LibMiddleware {
    fn process(&self, input: &str) -> String;
}

pub struct MiddlewareStack {
    middlewares: Vec<Box<dyn LibMiddleware>>,
}

impl MiddlewareStack {
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add(&mut self, middleware: Box<dyn LibMiddleware>) {
        self.middlewares.push(middleware);
    }

    pub fn execute(&self, input: &str) -> String {
        let mut result = input.to_string();
        for middleware in &self.middlewares {
            result = middleware.process(&result);
        }
        result
    }
}

/// Problem 11: Configuration support
/// Support configuration
pub struct LibConfig {
    settings: std::collections::HashMap<String, String>,
}

impl LibConfig {
    pub fn new() -> Self {
        Self {
            settings: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.settings.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.settings.insert(key.to_string(), value.to_string());
    }
}

/// Problem 12: Logging support
/// Support logging
pub trait LibLogger {
    fn log(&self, level: &str, message: &str);
}

pub struct ConsoleLibLogger;

impl LibLogger for ConsoleLibLogger {
    fn log(&self, level: &str, message: &str) {
        println!("[{}] {}", level, message);
    }
}

/// Problem 13: Testing support
/// Design for testability

/// Problem 14: Documentation
/// Document library
/// # Examples
///
/// ```
/// use lead_architecture::p03_library_design::Library;
///
/// let mut lib = Library::new();
/// lib.set("key", "value");
/// assert_eq!(lib.get("key"), Some("value"));
/// ```
pub fn documented_function() -> i32 {
    42
}

/// Problem 15: Versioning
/// Support versioning
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api() {
        let mut lib = Library::new();
        lib.set("key", "value");
        assert_eq!(lib.get("key"), Some("value"));
        assert!(lib.contains("key"));
        assert_eq!(lib.len(), 1);
    }

    #[test]
    fn test_error_handling() {
        let error = LibraryError::NotFound("key".to_string());
        assert!(error.to_string().contains("Not found"));
    }

    #[test]
    fn test_builder() {
        let lib = LibraryBuilder::new()
            .capacity(100)
            .case_sensitive(true)
            .build();
        assert_eq!(lib.len(), 0);
    }

    #[test]
    fn test_generic_store() {
        let mut store = GenericStore::new();
        store.set("key", 42);
        assert_eq!(store.get(&"key"), Some(&42));
    }

    #[test]
    fn test_feature_flags() {
        let mut flags = FeatureFlags::new();
        flags.enable("feature1");
        assert!(flags.is_enabled("feature1"));
        assert!(!flags.is_enabled("feature2"));
    }

    #[test]
    fn test_version() {
        let version = Version::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }
}
