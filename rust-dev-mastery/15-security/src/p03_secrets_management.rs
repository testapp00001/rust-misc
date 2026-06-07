//! # Secrets Management
//!
//! Secrets (API keys, passwords, tokens) must never appear in source code, logs,
//! or error messages. This lesson covers patterns for securely handling secrets
//! in Rust applications.
//!
//! ## Key Concepts
//! - Zeroizing sensitive data on drop
//! - Environment variable management
//! - Secret string types that don't leak
//! - Vault integration patterns
//! - Secret rotation

use std::collections::HashMap;
use std::fmt;

// ---------------------------------------------------------------------------
// 1. Secret String (Non-Leaking)
// ---------------------------------------------------------------------------

/// A string type that prevents accidental secret leakage.
/// Does not implement Debug or Display to avoid logging secrets.
pub struct SecretString {
    inner: String,
}

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            inner: value.into(),
        }
    }

    /// Expose the secret value. Callers must be careful not to log this.
    pub fn expose(&self) -> &str {
        &self.inner
    }

    /// Get the length without exposing the value.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

// Implement Debug to hide the value
impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SecretString(***)")
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "***")
    }
}

impl Drop for SecretString {
    fn drop(&mut self) {
        // Zero out the memory
        // SAFETY: We're zeroing our own string's bytes
        unsafe {
            let bytes = self.inner.as_bytes_mut();
            for byte in bytes.iter_mut() {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Secret Container
// ---------------------------------------------------------------------------

/// A container that holds multiple named secrets.
#[derive(Debug)]
pub struct SecretStore {
    secrets: HashMap<String, SecretString>,
}

impl SecretStore {
    pub fn new() -> Self {
        Self {
            secrets: HashMap::new(),
        }
    }

    /// Load secrets from environment variables.
    pub fn from_env(prefix: &str) -> Self {
        let mut store = Self::new();
        for (key, value) in std::env::vars() {
            if key.starts_with(prefix) {
                let name = key
                    .strip_prefix(prefix)
                    .unwrap_or(&key)
                    .trim_start_matches('_')
                    .to_lowercase();
                store.set(&name, &value);
            }
        }
        store
    }

    pub fn set(&mut self, name: &str, value: &str) {
        self.secrets
            .insert(name.into(), SecretString::new(value));
    }

    pub fn get(&self, name: &str) -> Option<&SecretString> {
        self.secrets.get(name)
    }

    pub fn has(&self, name: &str) -> bool {
        self.secrets.contains_key(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<SecretString> {
        self.secrets.remove(name)
    }

    pub fn len(&self) -> usize {
        self.secrets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.secrets.is_empty()
    }

    /// List secret names (without values).
    pub fn list_names(&self) -> Vec<&str> {
        self.secrets.keys().map(|s| s.as_str()).collect()
    }
}

// ---------------------------------------------------------------------------
// 3. Zeroizing Wrapper
// ---------------------------------------------------------------------------

/// A wrapper that zeros memory on drop.
pub struct Zeroizing<T: Zeroize> {
    value: T,
}

impl<T: Zeroize> Zeroizing<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }

    pub fn inner(&self) -> &T {
        &self.value
    }
}

impl<T: Zeroize> Drop for Zeroizing<T> {
    fn drop(&mut self) {
        self.value.zeroize();
    }
}

impl<T: Zeroize + fmt::Debug> fmt::Debug for Zeroizing<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Zeroizing(***)")
    }
}

/// Trait for types that can be zeroed out.
pub trait Zeroize {
    fn zeroize(&mut self);
}

impl Zeroize for Vec<u8> {
    fn zeroize(&mut self) {
        for byte in self.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

impl Zeroize for String {
    fn zeroize(&mut self) {
        unsafe {
            let bytes = self.as_bytes_mut();
            for byte in bytes.iter_mut() {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

impl Zeroize for [u8] {
    fn zeroize(&mut self) {
        for byte in self.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Vault Integration Pattern
// ---------------------------------------------------------------------------

/// A trait for secret backends (vault, env, file, etc.)
pub trait SecretBackend: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<String>, SecretError>;
    fn list(&self) -> Result<Vec<String>, SecretError>;
}

/// Environment variable secret backend.
pub struct EnvBackend {
    prefix: String,
}

impl EnvBackend {
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl SecretBackend for EnvBackend {
    fn get(&self, key: &str) -> Result<Option<String>, SecretError> {
        let env_key = format!("{}_{}", self.prefix, key.to_uppercase());
        Ok(std::env::var(&env_key).ok())
    }

    fn list(&self) -> Result<Vec<String>, SecretError> {
        Ok(std::env::vars()
            .filter(|(k, _)| k.starts_with(&self.prefix))
            .map(|(k, _)| {
                k.strip_prefix(&self.prefix)
                    .unwrap_or(&k)
                    .trim_start_matches('_')
                    .to_lowercase()
            })
            .collect())
    }
}

/// In-memory secret backend for testing.
#[derive(Debug, Default)]
pub struct MemoryBackend {
    secrets: HashMap<String, String>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_secret(mut self, key: &str, value: &str) -> Self {
        self.secrets.insert(key.into(), value.into());
        self
    }
}

impl SecretBackend for MemoryBackend {
    fn get(&self, key: &str) -> Result<Option<String>, SecretError> {
        Ok(self.secrets.get(key).cloned())
    }

    fn list(&self) -> Result<Vec<String>, SecretError> {
        Ok(self.secrets.keys().cloned().collect())
    }
}

// ---------------------------------------------------------------------------
// 5. Secret Provider (Facade)
// ---------------------------------------------------------------------------

/// Provides secrets from a backend with caching and fallback.
pub struct SecretProvider {
    backends: Vec<Box<dyn SecretBackend>>,
    cache: HashMap<String, String>,
}

impl SecretProvider {
    pub fn new() -> Self {
        Self {
            backends: Vec::new(),
            cache: HashMap::new(),
        }
    }

    pub fn with_backend(mut self, backend: impl SecretBackend + 'static) -> Self {
        self.backends.push(Box::new(backend));
        self
    }

    /// Get a secret, checking cache first, then backends in order.
    pub fn get(&mut self, key: &str) -> Result<Option<&str>, SecretError> {
        if self.cache.contains_key(key) {
            return Ok(self.cache.get(key).map(|s| s.as_str()));
        }

        for backend in &self.backends {
            if let Some(value) = backend.get(key)? {
                self.cache.insert(key.into(), value);
                return Ok(self.cache.get(key).map(|s| s.as_str()));
            }
        }

        Ok(None)
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

// ---------------------------------------------------------------------------
// 6. Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("secret not found: {0}")]
    NotFound(String),

    #[error("backend error: {0}")]
    BackendError(String),

    #[error("access denied")]
    AccessDenied,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_string_hides_value() {
        let secret = SecretString::new("my-api-key");
        let debug = format!("{secret:?}");
        assert!(!debug.contains("my-api-key"));
        assert!(debug.contains("***"));
    }

    #[test]
    fn test_secret_string_display() {
        let secret = SecretString::new("password123");
        let display = format!("{secret}");
        assert_eq!(display, "***");
    }

    #[test]
    fn test_secret_string_expose() {
        let secret = SecretString::new("my-key");
        assert_eq!(secret.expose(), "my-key");
        assert_eq!(secret.len(), 6);
    }

    #[test]
    fn test_secret_store() {
        let mut store = SecretStore::new();
        store.set("api_key", "sk-123");
        store.set("db_password", "secret");

        assert!(store.has("api_key"));
        assert_eq!(store.get("api_key").unwrap().expose(), "sk-123");
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_secret_store_list_names() {
        let mut store = SecretStore::new();
        store.set("a", "1");
        store.set("b", "2");

        let names = store.list_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"a"));
    }

    #[test]
    fn test_secret_store_remove() {
        let mut store = SecretStore::new();
        store.set("key", "value");
        store.remove("key");
        assert!(!store.has("key"));
    }

    #[test]
    fn test_zeroizing_vec() {
        let secret = Zeroizing::new(vec![1u8, 2, 3, 4, 5]);
        assert_eq!(secret.inner(), &vec![1, 2, 3, 4, 5]);
        // After drop, memory is zeroed
        drop(secret);
    }

    #[test]
    fn test_zeroizing_string() {
        let secret = Zeroizing::new(String::from("sensitive"));
        assert_eq!(secret.inner(), "sensitive");
        drop(secret);
    }

    #[test]
    fn test_zeroizing_debug_hides() {
        let secret = Zeroizing::new(vec![1u8, 2, 3]);
        let debug = format!("{secret:?}");
        assert!(debug.contains("***"));
        assert!(!debug.contains("1"));
    }

    #[test]
    fn test_env_backend() {
        std::env::set_var("TEST_SECRET_API_KEY", "test-value");
        let backend = EnvBackend::new("TEST_SECRET");
        let value = backend.get("api_key").unwrap();
        assert_eq!(value.as_deref(), Some("test-value"));
        std::env::remove_var("TEST_SECRET_API_KEY");
    }

    #[test]
    fn test_memory_backend() {
        let backend = MemoryBackend::new()
            .with_secret("key1", "val1")
            .with_secret("key2", "val2");

        assert_eq!(backend.get("key1").unwrap(), Some("val1".into()));
        assert_eq!(backend.get("missing").unwrap(), None);

        let keys = backend.list().unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn test_secret_provider() {
        let backend = MemoryBackend::new().with_secret("api_key", "sk-123");
        let mut provider = SecretProvider::new().with_backend(backend);

        let value = provider.get("api_key").unwrap();
        assert_eq!(value, Some("sk-123"));

        let missing = provider.get("missing").unwrap();
        assert_eq!(missing, None);
    }

    #[test]
    fn test_secret_provider_caching() {
        let backend = MemoryBackend::new().with_secret("key", "value");
        let mut provider = SecretProvider::new().with_backend(backend);

        // First call
        provider.get("key").unwrap();
        // Second call should use cache
        let value = provider.get("key").unwrap();
        assert_eq!(value, Some("value"));
    }

    #[test]
    fn test_secret_provider_fallback() {
        let backend1 = MemoryBackend::new();
        let backend2 = MemoryBackend::new().with_secret("key", "from_backend2");
        let mut provider = SecretProvider::new()
            .with_backend(backend1)
            .with_backend(backend2);

        let value = provider.get("key").unwrap();
        assert_eq!(value, Some("from_backend2"));
    }

    #[test]
    fn test_secret_error_display() {
        let err = SecretError::NotFound("key".into());
        assert!(err.to_string().contains("key"));

        let err = SecretError::AccessDenied;
        assert!(!err.to_string().is_empty());
    }
}
