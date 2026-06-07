//! # Public API Design
//!
//! Designing a good public API in Rust requires careful control of what's exposed,
//! how types are used, and how the API can evolve without breaking downstream code.
//!
//! ## Key Concepts
//! - **API surface area**: Minimize what's `pub`; use `pub(crate)` for internal items
//! - **Sealing traits**: Prevent external implementations
//! - **API evolution**: Designing for backward-compatible changes
//! - **Visibility control**: Module structure as API boundary

/// A sealed trait: external crates cannot implement it.
/// This is achieved by having a private supertrait.
mod sealed {
    pub trait Sealed {}
}

/// A type that can be safely serialized to bytes.
pub trait Serializable: sealed::Sealed {
    fn to_bytes(&self) -> Vec<u8>;
}

/// Our types implement the sealed trait.
pub struct Packet {
    pub header: u32,
    pub payload: Vec<u8>,
}

impl sealed::Sealed for Packet {}
impl Serializable for Packet {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_be_bytes().to_vec();
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}

/// A configuration builder that controls its own API surface.
pub struct DatabaseConfig {
    // Private fields — not directly accessible
    connection_string: String,
    max_pool_size: u32,
    timeout_secs: u64,
    ssl_mode: SslMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

impl DatabaseConfig {
    /// Creates a new config with required fields only.
    /// Optional fields have sensible defaults.
    pub fn new(connection_string: impl Into<String>) -> Self {
        DatabaseConfig {
            connection_string: connection_string.into(),
            max_pool_size: 10,
            timeout_secs: 30,
            ssl_mode: SslMode::Prefer,
        }
    }

    // Builder methods for optional configuration
    pub fn max_pool_size(mut self, size: u32) -> Self {
        self.max_pool_size = size;
        self
    }

    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn ssl_mode(mut self, mode: SslMode) -> Self {
        self.ssl_mode = mode;
        self
    }

    // Getters for read-only access
    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }

    pub fn max_pool_size_value(&self) -> u32 {
        self.max_pool_size
    }

    pub fn timeout_value(&self) -> u64 {
        self.timeout_secs
    }

    pub fn ssl_mode_value(&self) -> SslMode {
        self.ssl_mode
    }
}

/// Demonstrates the "type alias" pattern for API stability.
/// The internal representation can change without affecting the public API.
pub mod metrics {
    /// A histogram metric. Currently backed by a simple Vec, but the
    /// implementation can change without breaking the API.
    pub struct Histogram {
        values: Vec<f64>,
    }

    impl Histogram {
        pub fn new() -> Self {
            Histogram { values: Vec::new() }
        }

        pub fn record(&mut self, value: f64) {
            self.values.push(value);
        }

        pub fn mean(&self) -> f64 {
            if self.values.is_empty() {
                return 0.0;
            }
            self.values.iter().sum::<f64>() / self.values.len() as f64
        }

        pub fn count(&self) -> usize {
            self.values.len()
        }

        // Note: we do NOT expose the internal Vec via a public method.
        // This preserves the ability to change the internal representation.
    }
}

/// An opaque type pattern: the internal structure is completely hidden.
pub struct OpaqueHandle {
    // Private: callers cannot construct this directly
    id: u64,
    data: Vec<u8>,
}

impl OpaqueHandle {
    /// Only factory method — no public constructor.
    pub fn create(data: Vec<u8>) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        OpaqueHandle {
            id: NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            data,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Demonstrates extension traits: adding methods to existing types
/// without modifying their original implementation.
pub trait StrExt {
    fn is_valid_identifier(&self) -> bool;
    fn to_snake_case(&self) -> String;
}

impl StrExt for str {
    fn is_valid_identifier(&self) -> bool {
        !self.is_empty()
            && self
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_')
            && !self.chars().next().unwrap().is_ascii_digit()
    }

    fn to_snake_case(&self) -> String {
        let mut result = String::new();
        for (i, c) in self.chars().enumerate() {
            if c.is_uppercase() && i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        }
        result
    }
}

/// A versioned API: methods are tagged with their stability level.
pub struct StableApi;

impl StableApi {
    /// Stable: will not change in future versions.
    pub fn stable_method(&self) -> i32 {
        42
    }
}

/// Marker types for API stability levels.
pub mod stability {
    /// This API is stable and will not change.
    pub struct Stable;
    /// This API is experimental and may change.
    pub struct Experimental;
    /// This API is deprecated and will be removed.
    pub struct Deprecated;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sealed_trait() {
        let packet = Packet {
            header: 1,
            payload: vec![2, 3, 4],
        };
        let bytes = packet.to_bytes();
        assert_eq!(bytes[0..4], [0, 0, 0, 1]); // header in big-endian
        assert_eq!(&bytes[4..], &[2, 3, 4]); // payload
    }

    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig::new("postgres://localhost/db");
        assert_eq!(config.connection_string(), "postgres://localhost/db");
        assert_eq!(config.max_pool_size_value(), 10);
        assert_eq!(config.timeout_value(), 30);
        assert_eq!(config.ssl_mode_value(), SslMode::Prefer);
    }

    #[test]
    fn test_database_config_builder() {
        let config = DatabaseConfig::new("postgres://localhost/db")
            .max_pool_size(50)
            .timeout(60)
            .ssl_mode(SslMode::Require);

        assert_eq!(config.max_pool_size_value(), 50);
        assert_eq!(config.timeout_value(), 60);
        assert_eq!(config.ssl_mode_value(), SslMode::Require);
    }

    #[test]
    fn test_histogram() {
        let mut hist = metrics::Histogram::new();
        assert_eq!(hist.count(), 0);
        assert_eq!(hist.mean(), 0.0);

        hist.record(10.0);
        hist.record(20.0);
        hist.record(30.0);

        assert_eq!(hist.count(), 3);
        assert!((hist.mean() - 20.0).abs() < 0.001);
    }

    #[test]
    fn test_opaque_handle() {
        let handle = OpaqueHandle::create(vec![1, 2, 3]);
        assert!(handle.id() > 0);
        assert_eq!(handle.data(), &[1, 2, 3]);
    }

    #[test]
    fn test_opaque_handle_unique_ids() {
        let h1 = OpaqueHandle::create(vec![]);
        let h2 = OpaqueHandle::create(vec![]);
        assert_ne!(h1.id(), h2.id());
    }

    #[test]
    fn test_str_ext_identifier() {
        assert!("valid_name".is_valid_identifier());
        assert!(!"123bad".is_valid_identifier());
        assert!(!"".is_valid_identifier());
        assert!(!"has spaces".is_valid_identifier());
        assert!("CamelCase".is_valid_identifier());
    }

    #[test]
    fn test_str_ext_snake_case() {
        assert_eq!("CamelCase".to_snake_case(), "camel_case");
        assert_eq!("already_snake".to_snake_case(), "already_snake");
        assert_eq!("ABC".to_snake_case(), "a_b_c");
    }

    #[test]
    fn test_stable_api() {
        let api = StableApi;
        assert_eq!(api.stable_method(), 42);
    }
}
