//! # Error Types as API Contract
//!
//! Error types are a critical part of any public API. Well-designed error types
//! tell callers what went wrong and how to recover. This module covers error
//! type design, documentation, and compatibility.
//!
//! ## Key Concepts
//! - **Error types as API contract**: Error variants are part of the public API
//! - **Error documentation**: Documenting when and why errors occur
//! - **Error compatibility**: Adding new error variants without breaking callers
//! - **Error conversion**: From/Into for ergonomic error handling

use std::fmt;

/// A well-designed error type for a storage API.
/// Each variant represents a distinct failure mode that callers may need to handle.
#[derive(Debug, Clone, PartialEq)]
pub enum StorageError {
    /// The requested key was not found.
    /// Callers should check if the key exists before reading.
    NotFound {
        /// The key that was not found.
        key: String,
    },

    /// The operation was denied due to insufficient permissions.
    /// Callers should check their credentials.
    PermissionDenied {
        /// Description of the required permission.
        required: String,
    },

    /// The storage is full and cannot accept new data.
    /// Callers should free space or use a different storage backend.
    StorageFull {
        /// Current usage in bytes.
        used: u64,
        /// Total capacity in bytes.
        capacity: u64,
    },

    /// The value is too large to store.
    ValueTooLarge {
        /// The size of the value in bytes.
        size: usize,
        /// The maximum allowed size in bytes.
        max_size: usize,
    },

    /// The key contains invalid characters.
    InvalidKey {
        /// The invalid key.
        key: String,
        /// Description of what makes it invalid.
        reason: String,
    },

    /// An internal error occurred. This is a catch-all for unexpected failures.
    /// Callers should retry or report the error.
    Internal {
        /// A human-readable description of the error.
        message: String,
    },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::NotFound { key } => write!(f, "Key not found: {key}"),
            StorageError::PermissionDenied { required } => {
                write!(f, "Permission denied: requires {required}")
            }
            StorageError::StorageFull { used, capacity } => {
                write!(f, "Storage full: {used}/{capacity} bytes used")
            }
            StorageError::ValueTooLarge { size, max_size } => {
                write!(f, "Value too large: {size} bytes (max {max_size})")
            }
            StorageError::InvalidKey { key, reason } => {
                write!(f, "Invalid key '{key}': {reason}")
            }
            StorageError::Internal { message } => write!(f, "Internal error: {message}"),
        }
    }
}

impl std::error::Error for StorageError {}

/// Error conversion: wrap common error types.
impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::Internal {
            message: format!("IO error: {err}"),
        }
    }
}

/// A result type alias for the storage API.
pub type StorageResult<T> = Result<T, StorageError>;

/// A storage API that demonstrates error type best practices.
pub struct KeyValueStore {
    data: std::collections::HashMap<String, Vec<u8>>,
    max_value_size: usize,
    capacity: usize,
}

impl KeyValueStore {
    pub fn new(capacity: usize, max_value_size: usize) -> Self {
        KeyValueStore {
            data: std::collections::HashMap::new(),
            max_value_size,
            capacity,
        }
    }

    /// Gets a value by key.
    ///
    /// # Errors
    ///
    /// Returns `StorageError::NotFound` if the key does not exist.
    pub fn get(&self, key: &str) -> StorageResult<&[u8]> {
        self.data
            .get(key)
            .map(|v| v.as_slice())
            .ok_or_else(|| StorageError::NotFound {
                key: key.to_string(),
            })
    }

    /// Sets a value for a key.
    ///
    /// # Errors
    ///
    /// - `StorageError::InvalidKey` if the key is empty
    /// - `StorageError::ValueTooLarge` if the value exceeds `max_value_size`
    /// - `StorageError::StorageFull` if the store is at capacity
    pub fn set(&mut self, key: &str, value: Vec<u8>) -> StorageResult<()> {
        if key.is_empty() {
            return Err(StorageError::InvalidKey {
                key: key.to_string(),
                reason: "Key cannot be empty".into(),
            });
        }

        if value.len() > self.max_value_size {
            return Err(StorageError::ValueTooLarge {
                size: value.len(),
                max_size: self.max_value_size,
            });
        }

        let current_size: usize = self.data.values().map(|v| v.len()).sum();
        if current_size + value.len() > self.capacity && !self.data.contains_key(key) {
            return Err(StorageError::StorageFull {
                used: current_size as u64,
                capacity: self.capacity as u64,
            });
        }

        self.data.insert(key.to_string(), value);
        Ok(())
    }

    /// Deletes a value by key.
    ///
    /// # Errors
    ///
    /// Returns `StorageError::NotFound` if the key does not exist.
    pub fn delete(&mut self, key: &str) -> StorageResult<()> {
        self.data
            .remove(key)
            .ok_or_else(|| StorageError::NotFound {
                key: key.to_string(),
            })?;
        Ok(())
    }
}

/// An error chain that provides context at each level.
#[derive(Debug)]
pub struct ChainedError {
    pub kind: ErrorKind,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    NotFound,
    Permission,
    Validation,
    Internal,
}

impl fmt::Display for ChainedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.kind, self.message)?;
        if let Some(source) = &self.source {
            write!(f, "\n  Caused by: {source}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ChainedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// A type-erased error box for APIs that need flexibility.
#[derive(Debug)]
pub struct ApiError {
    code: ErrorCode,
    message: String,
    details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    BadRequest = 400,
    Unauthorized = 401,
    Forbidden = 403,
    NotFound = 404,
    InternalError = 500,
}

impl ApiError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        ApiError {
            code,
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code as u16, self.message)
    }
}

impl std::error::Error for ApiError {}

/// Error recovery: provides methods for callers to determine retry behavior.
impl StorageError {
    /// Returns true if the error is transient and the operation might succeed if retried.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            StorageError::Internal { .. } | StorageError::StorageFull { .. }
        )
    }

    /// Returns true if the error is due to invalid input from the caller.
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            StorageError::InvalidKey { .. }
                | StorageError::ValueTooLarge { .. }
                | StorageError::NotFound { .. }
        )
    }

    /// Returns a suggested retry delay, if applicable.
    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            StorageError::StorageFull { .. } => Some(std::time::Duration::from_secs(5)),
            StorageError::Internal { .. } => Some(std::time::Duration::from_secs(1)),
            _ => None,
        }
    }
}

// serde_json is not a dependency; provide a minimal stub for the example
mod serde_json {
    #[derive(Debug, Clone)]
    pub enum Value {
        Null,
        Bool(bool),
        String(String),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_value_store_basic() {
        let mut store = KeyValueStore::new(1024, 256);
        store.set("key1", vec![1, 2, 3]).unwrap();
        assert_eq!(store.get("key1").unwrap(), &[1, 2, 3]);
    }

    #[test]
    fn test_key_value_store_not_found() {
        let store = KeyValueStore::new(1024, 256);
        let err = store.get("missing").unwrap_err();
        assert!(matches!(err, StorageError::NotFound { .. }));
    }

    #[test]
    fn test_key_value_store_empty_key() {
        let mut store = KeyValueStore::new(1024, 256);
        let err = store.set("", vec![1]).unwrap_err();
        assert!(matches!(err, StorageError::InvalidKey { .. }));
    }

    #[test]
    fn test_key_value_store_value_too_large() {
        let mut store = KeyValueStore::new(1024, 10);
        let err = store.set("key", vec![0; 20]).unwrap_err();
        assert!(matches!(err, StorageError::ValueTooLarge { .. }));
    }

    #[test]
    fn test_key_value_store_full() {
        let mut store = KeyValueStore::new(5, 5);
        store.set("a", vec![1, 2, 3, 4, 5]).unwrap();
        let err = store.set("b", vec![1]).unwrap_err();
        assert!(matches!(err, StorageError::StorageFull { .. }));
    }

    #[test]
    fn test_key_value_store_delete() {
        let mut store = KeyValueStore::new(1024, 256);
        store.set("key", vec![1]).unwrap();
        store.delete("key").unwrap();
        assert!(store.get("key").is_err());
    }

    #[test]
    fn test_error_display() {
        let err = StorageError::NotFound {
            key: "test".into(),
        };
        assert_eq!(err.to_string(), "Key not found: test");
    }

    #[test]
    fn test_error_is_retryable() {
        assert!(!StorageError::NotFound { key: "x".into() }.is_retryable());
        assert!(StorageError::Internal { message: "x".into() }.is_retryable());
        assert!(StorageError::StorageFull { used: 100, capacity: 100 }.is_retryable());
    }

    #[test]
    fn test_error_is_client_error() {
        assert!(StorageError::NotFound { key: "x".into() }.is_client_error());
        assert!(!StorageError::Internal { message: "x".into() }.is_client_error());
    }

    #[test]
    fn test_error_retry_after() {
        assert!(StorageError::StorageFull { used: 100, capacity: 100 }
            .retry_after()
            .is_some());
        assert!(StorageError::NotFound { key: "x".into() }.retry_after().is_none());
    }

    #[test]
    fn test_api_error() {
        let err = ApiError::new(ErrorCode::NotFound, "User not found");
        assert_eq!(err.code(), ErrorCode::NotFound);
        assert_eq!(err.message(), "User not found");
        assert!(err.to_string().contains("404"));
    }

    #[test]
    fn test_api_error_with_details() {
        let err = ApiError::new(ErrorCode::BadRequest, "Invalid input")
            .with_details(serde_json::Value::String("details".into()));
        assert!(err.details.is_some());
    }
}
