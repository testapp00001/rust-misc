//! # Backward Compatibility
//!
//! Adding features without breaking existing code is essential for library
//! maintainability. This module covers non-breaking additions, default trait
//! methods, and extension traits.
//!
//! ## Key Concepts
//! - **Non-breaking additions**: New methods, new trait impls, new enum variants
//! - **Default trait methods**: Adding methods with defaults doesn't break implementors
//! - **Extension traits**: New functionality via new traits on existing types
//! - **Sealed traits**: Preventing external implementations for future flexibility

/// A trait with default methods — can add new methods without breaking implementors.
pub trait ConfigProvider {
    fn get(&self, key: &str) -> Option<&str>;

    // Default methods — existing implementors don't need to update
    fn get_or_default<'a>(&'a self, key: &str, default: &'a str) -> &'a str {
        self.get(key).unwrap_or(default)
    }

    fn get_required(&self, key: &str) -> Result<&str, ConfigError> {
        self.get(key)
            .ok_or_else(|| ConfigError::MissingKey(key.to_string()))
    }

    fn contains(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    MissingKey(String),
}

/// A simple config that implements the trait — only needs to implement `get`.
pub struct SimpleConfig {
    data: std::collections::HashMap<String, String>,
}

impl SimpleConfig {
    pub fn new(data: std::collections::HashMap<String, String>) -> Self {
        SimpleConfig { data }
    }
}

impl ConfigProvider for SimpleConfig {
    fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
}

/// An enum that can have new variants added using #[non_exhaustive].
/// External match statements must have a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    // Future: Trace, Fatal, etc. can be added without breaking
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            _ => write!(f, "UNKNOWN"),
        }
    }
}

/// A struct that is #[non_exhaustive] — external code cannot construct it
/// directly, giving us freedom to add fields.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub struct LogRecord {
    pub level: LogLevel,
    pub message: String,
    pub target: String,
}

impl LogRecord {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        LogRecord {
            level,
            message: message.into(),
            target: String::new(),
        }
    }
}

/// Extension trait pattern: add methods to existing types via a new trait.
/// This is the primary way to add non-breaking functionality.
pub trait StrAnalysis {
    fn word_count(&self) -> usize;
    fn is_palindrome(&self) -> bool;
    fn char_frequency(&self) -> std::collections::HashMap<char, usize>;
}

impl StrAnalysis for str {
    fn word_count(&self) -> usize {
        self.split_whitespace().count()
    }

    fn is_palindrome(&self) -> bool {
        let cleaned: String = self.chars().filter(|c| c.is_alphanumeric()).collect();
        let lower = cleaned.to_lowercase();
        lower == lower.chars().rev().collect::<String>()
    }

    fn char_frequency(&self) -> std::collections::HashMap<char, usize> {
        let mut freq = std::collections::HashMap::new();
        for c in self.chars() {
            *freq.entry(c).or_insert(0) += 1;
        }
        freq
    }
}

/// A trait that uses the "sealed" pattern to allow adding required methods
/// in future versions without breaking external implementors.
pub trait Event: sealed::Sealed {
    fn event_type(&self) -> &str;
}

mod sealed {
    pub trait Sealed {}
}

/// Only our types can implement Event.
pub struct ClickEvent {
    pub x: i32,
    pub y: i32,
}

impl sealed::Sealed for ClickEvent {}
impl Event for ClickEvent {
    fn event_type(&self) -> &str {
        "click"
    }
}

/// Demonstrates the "type evolution" pattern: adding new functionality
/// to existing types without modifying them.
pub struct EnhancedVec<T> {
    inner: Vec<T>,
}

impl<T> EnhancedVec<T> {
    pub fn new() -> Self {
        EnhancedVec { inner: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.inner.push(item);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Extension methods for EnhancedVec.
impl<T: Clone> EnhancedVec<T> {
    pub fn duplicate(&self) -> Self {
        EnhancedVec {
            inner: self.inner.clone(),
        }
    }
}

impl<T: PartialEq> EnhancedVec<T> {
    pub fn contains(&self, item: &T) -> bool {
        self.inner.contains(item)
    }

    pub fn dedup(&mut self) {
        self.inner.dedup();
    }
}

/// A builder that maintains backward compatibility by using Option for
/// new fields with sensible defaults.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    // Original fields (v1.0)
    pub endpoint: String,
    pub timeout_ms: u64,

    // Added in v1.1 (optional with defaults)
    pub retry_count: u32,
    pub retry_delay_ms: u64,

    // Added in v1.2 (optional with defaults)
    pub tls_enabled: bool,
    pub max_connections: u32,
}

impl ServiceConfig {
    pub fn new(endpoint: impl Into<String>) -> Self {
        ServiceConfig {
            endpoint: endpoint.into(),
            timeout_ms: 5000,
            retry_count: 3,
            retry_delay_ms: 1000,
            tls_enabled: false,
            max_connections: 100,
        }
    }

    // Builder methods for each optional field
    pub fn timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    pub fn retry_count(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    pub fn retry_delay_ms(mut self, ms: u64) -> Self {
        self.retry_delay_ms = ms;
        self
    }

    pub fn tls_enabled(mut self, enabled: bool) -> Self {
        self.tls_enabled = enabled;
        self
    }

    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_provider_get() {
        let config = SimpleConfig::new(
            [("host".into(), "localhost".into())]
                .into_iter()
                .collect(),
        );

        assert_eq!(config.get("host"), Some("localhost"));
        assert_eq!(config.get("missing"), None);
    }

    #[test]
    fn test_config_provider_default_methods() {
        let config = SimpleConfig::new(
            [("host".into(), "localhost".into())]
                .into_iter()
                .collect(),
        );

        assert_eq!(config.get_or_default("host", "default"), "localhost");
        assert_eq!(config.get_or_default("missing", "default"), "default");
        assert!(config.contains("host"));
        assert!(!config.contains("missing"));
    }

    #[test]
    fn test_config_provider_required() {
        let config = SimpleConfig::new(
            [("host".into(), "localhost".into())]
                .into_iter()
                .collect(),
        );

        assert_eq!(config.get_required("host"), Ok("localhost"));
        assert!(config.get_required("missing").is_err());
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(format!("{}", LogLevel::Debug), "DEBUG");
        assert_eq!(format!("{}", LogLevel::Error), "ERROR");
    }

    #[test]
    fn test_log_level_match_with_wildcard() {
        let level = LogLevel::Info;
        let msg = match level {
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
            _ => "unknown", // Required for #[non_exhaustive]
        };
        assert_eq!(msg, "info");
    }

    #[test]
    fn test_log_record() {
        let record = LogRecord::new(LogLevel::Info, "test message");
        assert_eq!(record.level, LogLevel::Info);
        assert_eq!(record.message, "test message");
    }

    #[test]
    fn test_str_analysis() {
        assert_eq!("hello world".word_count(), 2);
        assert!("racecar".is_palindrome());
        assert!(!"hello".is_palindrome());
        assert!("A man a plan a canal Panama"
            .is_palindrome());
    }

    #[test]
    fn test_char_frequency() {
        let freq = "aab".char_frequency();
        assert_eq!(freq.get(&'a'), Some(&2));
        assert_eq!(freq.get(&'b'), Some(&1));
        assert_eq!(freq.get(&'c'), None);
    }

    #[test]
    fn test_enhanced_vec() {
        let mut v = EnhancedVec::new();
        v.push(1);
        v.push(2);
        v.push(3);
        assert_eq!(v.len(), 3);
        assert!(v.contains(&2));
        assert!(!v.contains(&4));
    }

    #[test]
    fn test_enhanced_vec_duplicate() {
        let mut v = EnhancedVec::new();
        v.push(1);
        v.push(2);

        let dup = v.duplicate();
        assert_eq!(dup.len(), 2);
    }

    #[test]
    fn test_service_config_defaults() {
        let config = ServiceConfig::new("https://api.example.com");
        assert_eq!(config.endpoint, "https://api.example.com");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.retry_delay_ms, 1000);
        assert!(!config.tls_enabled);
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_service_config_builder() {
        let config = ServiceConfig::new("https://api.example.com")
            .timeout_ms(10000)
            .retry_count(5)
            .tls_enabled(true)
            .max_connections(50);

        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.retry_count, 5);
        assert!(config.tls_enabled);
        assert_eq!(config.max_connections, 50);
    }
}
