//! # Lesson 09: Secret Management
//!
//! ## The Problem
//!
//! Applications need secrets to function:
//! - Database passwords
//! - API keys
//! - JWT signing keys
//! - TLS private keys
//! - Encryption keys
//!
//! These secrets are the keys to the kingdom. If they leak, everything is compromised.
//!
//! ## Common Secret Leaks
//!
//! | Leak Vector | How It Happens |
//! |-------------|---------------|
//! | Source code | Hardcoded secrets committed to Git |
//! | Environment | Secrets in Docker images, CI logs, process listings |
//! | Logs | Secrets accidentally logged in error messages |
//! | Memory dumps | Secrets persisting in memory after use |
//! | Config files | Secrets in .env files committed to repo |
//!
//! ## Defense: The `secrecy` and `zeroize` Crates
//!
//! - `Secret<T>` from the `secrecy` crate prevents accidental logging (Debug impl is redacted)
//! - `Zeroize` from the `zeroize` crate clears memory when a value is dropped
//! - Together, they minimize the window where secrets exist in memory
//!
//! ## Secret Lifecycle
//!
//! ```text
//! 1. Generation: Create cryptographically random secret
//! 2. Storage: Load from environment variable or secret manager
//! 3. Usage: Use the secret for its purpose (sign, encrypt, authenticate)
//! 4. Rotation: Replace with new secret, invalidate old one
//! 5. Zeroization: Clear from memory when no longer needed
//! ```
//!
//! ## Attack Context
//!
//! - **Hardcoded secrets**: Attacker finds API key in source code. Defense: env vars.
//! - **Log leakage**: Secret appears in a debug log. Defense: secrecy::Secret<T>.
//! - **Memory scraping**: Attacker reads process memory. Defense: zeroize on drop.
//! - **No rotation**: Same secret used for years. Defense: regular rotation with overlap.

use zeroize::{Zeroize, ZeroizeOnDrop};

/// A managed secret with metadata for lifecycle management.
#[derive(Clone)]
pub struct ManagedSecret {
    /// The secret value (zeroized on drop)
    value: Secret<Vec<u8>>,
    /// When the secret was created (Unix timestamp)
    pub created_at: u64,
    /// When the secret expires (Unix timestamp)
    pub expires_at: u64,
    /// Unique identifier for this secret version
    pub key_id: String,
    /// Whether this secret has been rotated out
    pub is_active: bool,
}

impl std::fmt::Debug for ManagedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedSecret")
            .field("key_id", &self.key_id)
            .field("created_at", &self.created_at)
            .field("expires_at", &self.expires_at)
            .field("is_active", &self.is_active)
            .field("value", &"[REDACTED]")
            .finish()
    }
}

/// A secret store that manages multiple secrets with rotation support.
#[derive(Debug)]
pub struct SecretStore {
    /// All secrets indexed by key_id
    secrets: Vec<ManagedSecret>,
}

impl SecretStore {
    /// Create a new empty secret store.
    pub fn new() -> Self {
        Self {
            secrets: Vec::new(),
        }
    }
}

/// Exercise 1: Create a managed secret from raw bytes.
///
/// Steps:
/// 1. Generate a key_id in the format "key-{timestamp}-{random_hex}"
///    For simplicity, use "key-{timestamp}" format (timestamp from created_at)
/// 2. Wrap the value in Secret<Vec<u8>>
/// 3. Set created_at and expires_at (created_at + lifetime_secs)
/// 4. Mark as active
pub fn create_secret(
    value: Vec<u8>,
    created_at: u64,
    lifetime_secs: u64,
) -> ManagedSecret {
    todo!("Create a managed secret with metadata")
}

/// Exercise 2: Load a secret from an environment variable.
///
/// Read the environment variable with the given name.
/// If it exists and is not empty, create a ManagedSecret from it.
/// If it does not exist or is empty, return None.
///
/// Use `created_at` as the current time and `lifetime_secs` for expiration.
pub fn load_secret_from_env(
    env_var_name: &str,
    created_at: u64,
    lifetime_secs: u64,
) -> Option<ManagedSecret> {
    todo!("Load a secret from an environment variable")
}

/// Exercise 3: Check if a secret has expired.
///
/// A secret is expired if current_time >= expires_at OR if is_active is false.
pub fn is_secret_expired(secret: &ManagedSecret, current_time: u64) -> bool {
    todo!("Check if a secret has expired")
}

/// Exercise 4: Rotate a secret in the store.
///
/// Steps:
/// 1. Deactivate all existing secrets for the same purpose (mark is_active = false)
/// 2. Add the new secret to the store
/// 3. Return the key_id of the new secret
///
/// The "purpose" is determined by the key_id prefix (everything before the last "-{timestamp}")
/// For simplicity, deactivate all secrets that are currently active before adding the new one.
pub fn rotate_secret(store: &mut SecretStore, new_secret: ManagedSecret) -> String {
    todo!("Rotate a secret in the store")
}

/// Exercise 5: Get the active secret for use.
///
/// Return the first active, non-expired secret in the store.
/// If no active secret is found, return None.
pub fn get_active_secret(store: &SecretStore, current_time: u64) -> Option<&ManagedSecret> {
    todo!("Get the active secret for use")
}

/// Exercise 6: Implement secure secret comparison.
///
/// Compare two secret byte slices in constant time to prevent timing attacks.
/// The `ring` crate provides `constant_time_eq` for this purpose.
///
/// For this exercise, implement a simple constant-time comparison:
/// - If lengths differ, return false (but still do work to avoid length-based timing)
/// - XOR each byte pair and accumulate any differences
/// - Return true only if no differences found
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement constant-time secret comparison")
}

/// Exercise 7: Zeroize a secret's memory.
///
/// Create a struct `SensitiveBuffer` that:
/// - Holds a Vec<u8>
/// - Implements Drop to zero out the buffer contents before deallocation
/// - Uses the zeroize crate's Zeroize trait
///
/// Return a new SensitiveBuffer with the given data.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SensitiveBuffer {
    pub data: Vec<u8>,
}

pub fn create_sensitive_buffer(data: Vec<u8>) -> SensitiveBuffer {
    todo!("Create a zeroizing sensitive buffer")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_secret_metadata() {
        let secret = create_secret(b"my-api-key".to_vec(), 1700000000, 86400);
        assert_eq!(secret.created_at, 1700000000);
        assert_eq!(secret.expires_at, 1700086400);
        assert!(secret.is_active);
        assert!(secret.key_id.starts_with("key-"));
    }

    #[test]
    fn test_secret_debug_redacted() {
        let secret = create_secret(b"super-secret".to_vec(), 1700000000, 86400);
        let debug_str = format!("{:?}", secret);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("super-secret"));
    }

    #[test]
    fn test_is_secret_expired_not_expired() {
        let secret = create_secret(b"key".to_vec(), 1700000000, 86400);
        assert!(!is_secret_expired(&secret, 1700000001));
    }

    #[test]
    fn test_is_secret_expired_expired() {
        let secret = create_secret(b"key".to_vec(), 1700000000, 86400);
        assert!(is_secret_expired(&secret, 1700086401));
    }

    #[test]
    fn test_is_secret_expired_inactive() {
        let mut secret = create_secret(b"key".to_vec(), 1700000000, 86400);
        secret.is_active = false;
        assert!(is_secret_expired(&secret, 1700000001));
    }

    #[test]
    fn test_rotate_secret_deactivates_old() {
        let mut store = SecretStore::new();
        let old = create_secret(b"old-key".to_vec(), 1700000000, 86400);
        let old_id = old.key_id.clone();
        store.secrets.push(old);

        let new = create_secret(b"new-key".to_vec(), 1700001000, 86400);
        let new_id = rotate_secret(&mut store, new);

        assert_ne!(new_id, old_id);
        // Old should be deactivated
        let old_secret = store.secrets.iter().find(|s| s.key_id == old_id).unwrap();
        assert!(!old_secret.is_active);
        // New should be active
        let new_secret = store.secrets.iter().find(|s| s.key_id == new_id).unwrap();
        assert!(new_secret.is_active);
    }

    #[test]
    fn test_get_active_secret() {
        let mut store = SecretStore::new();
        let mut secret = create_secret(b"key".to_vec(), 1700000000, 86400);
        secret.is_active = false; // deactivated
        store.secrets.push(secret);

        let active = create_secret(b"active-key".to_vec(), 1700001000, 86400);
        store.secrets.push(active);

        let result = get_active_secret(&store, 1700001001);
        assert!(result.is_some());
        assert_eq!(result.unwrap().key_id, store.secrets[1].key_id);
    }

    #[test]
    fn test_get_active_secret_none() {
        let store = SecretStore::new();
        assert!(get_active_secret(&store, 1700000000).is_none());
    }

    #[test]
    fn test_constant_time_eq_same() {
        assert!(constant_time_eq(b"secret", b"secret"));
    }

    #[test]
    fn test_constant_time_eq_different() {
        assert!(!constant_time_eq(b"secret", b"differ"));
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        assert!(!constant_time_eq(b"short", b"much-longer-value"));
    }

    #[test]
    fn test_sensitive_buffer_creation() {
        let buf = create_sensitive_buffer(b"sensitive data".to_vec());
        assert_eq!(buf.data, b"sensitive data");
    }

    #[test]
    fn test_load_secret_from_env_missing() {
        let result = load_secret_from_env("NONEXISTENT_SECRET_VAR_12345", 1700000000, 86400);
        assert!(result.is_none());
    }
}
