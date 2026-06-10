//! # Lesson 07: Key Versioning and Metadata Management
//!
//! ## What Is Key Versioning?
//!
//! Every cryptographic key should carry metadata that describes its purpose, lifetime,
//! algorithm, and version. Key versioning allows systems to:
//!
//! 1. **Support multiple keys simultaneously** during rotation
//! 2. **Select the correct key** for decryption based on the version tag
//! 3. **Enforce policies** like expiry and algorithm deprecation
//! 4. **Audit** key usage across the system
//!
//! ## Key Metadata Fields
//!
//! ```text
//! KeyMetadata {
//!     id:            "user-encryption-v3"
//!     version:       3
//!     algorithm:     "AES-256-GCM"
//!     purpose:       "encryption" | "signing" | "authentication"
//!     created_at:    1700000000
//!     expires_at:    1700090000
//!     status:        "active" | "retired" | "compromised"
//!     rotated_from:  Some("user-encryption-v2")
//! }
//! ```
//!
//! ## Attack Scenario: Version Confusion
//!
//! An attacker sends a message encrypted with an old, compromised key version.
//! If the system doesn't validate key versions, it happily decrypts with the
//! compromised key. Proper version management rejects messages with retired keys
//! (or at least flags them for audit).
//!
//! ## Security Notes
//!
//! - Never reuse a version number, even after a key is retired
//! - Include key version in ciphertext header (not secret, just metadata)
//! - Reject operations with "compromised" status keys
//! - Log all key version lookups for audit

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

/// Status of a key version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStatus {
    Active,
    Retired,
    Compromised,
}

/// Purpose of a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyPurpose {
    Encryption,
    Signing,
    Authentication,
}

/// Full metadata for a versioned key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    pub id: String,
    pub version: u32,
    pub algorithm: String,
    pub purpose: KeyPurpose,
    pub created_at: u64,
    pub expires_at: u64,
    pub status: KeyStatus,
    pub rotated_from: Option<String>,
    pub key_material: Vec<u8>,
}

/// A key registry that manages all key versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRegistry {
    pub keys: Vec<KeyMetadata>,
}

impl KeyRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        KeyRegistry { keys: Vec::new() }
    }
}

/// Exercise 1: Create a new key with metadata.
///
/// Generate a 32-byte random key and attach the given metadata.
/// Version should be 1 if `rotated_from` is None, otherwise previous version + 1.
///
/// Hints:
/// - Use `ring::SystemRandom` for key material
/// - Set status to `KeyStatus::Active`
pub fn create_key(
    id: &str,
    algorithm: &str,
    purpose: KeyPurpose,
    current_timestamp: u64,
    validity_seconds: u64,
    rotated_from: Option<&str>,
    version: u32,
) -> KeyMetadata {
    todo!("Create a new key with full metadata")
}

/// Exercise 2: Register a key in the registry.
///
/// Add a KeyMetadata to the registry. If a key with the same id and version
/// already exists, return an error.
///
/// Hints:
/// - Check for duplicates by (id, version)
/// - Push to the keys vec if no duplicate
pub fn register_key(registry: &mut KeyRegistry, key: KeyMetadata) -> Result<(), String> {
    todo!("Register a key in the registry")
}

/// Exercise 3: Look up the active key for a given ID.
///
/// Search the registry for the key with the given id that has `status == Active`.
/// Return the metadata if found, None otherwise.
///
/// Hints:
/// - Use `.iter().find()` with a predicate matching both id and status
pub fn find_active_key<'a>(registry: &'a KeyRegistry, key_id: &str) -> Option<&'a KeyMetadata> {
    todo!("Find the active key for a given ID")
}

/// Exercise 4: Look up a specific key version.
///
/// Find a key by its id AND version number, regardless of status.
/// This is needed for decrypting old data.
pub fn find_key_version<'a>(registry: &'a KeyRegistry, key_id: &str, version: u32) -> Option<&'a KeyMetadata> {
    todo!("Find a specific key version")
}

/// Exercise 5: Rotate a key — retire the current active and create a new version.
///
/// 1. Find the current active key for the given id
/// 2. Set its status to Retired
/// 3. Create a new key with version + 1, rotated_from pointing to old version
/// 4. Register the new key
///
/// Hints:
/// - Use `find_active_key` to find current
/// - Mutate its status to Retired
/// - Create new key with `create_key`
/// - Register it
pub fn rotate_key(
    registry: &mut KeyRegistry,
    key_id: &str,
    current_timestamp: u64,
    validity_seconds: u64,
) -> Result<u32, String> {
    todo!("Rotate a key: retire old, create new")
}

/// Exercise 6: Mark a key version as compromised.
///
/// Find the key by id and version, set status to Compromised.
/// This should trigger re-encryption of data encrypted with this key.
pub fn mark_compromised(
    registry: &mut KeyRegistry,
    key_id: &str,
    version: u32,
) -> Result<(), String> {
    todo!("Mark a key version as compromised")
}

/// Exercise 7: List all versions of a key, ordered by version.
///
/// Return a vector of (version, status) tuples for the given key id.
pub fn list_versions(registry: &KeyRegistry, key_id: &str) -> Vec<(u32, KeyStatus)> {
    todo!("List all versions of a key")
}

/// Exercise 8: Get key material for decryption, rejecting compromised keys.
///
/// Given a key_id and version, return the key material ONLY if the key's status
/// is not Compromised. Return an error for compromised keys.
pub fn get_decryption_key(
    registry: &KeyRegistry,
    key_id: &str,
    version: u32,
) -> Result<Vec<u8>, String> {
    todo!("Get decryption key, rejecting compromised keys")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_registry() -> KeyRegistry {
        let mut registry = KeyRegistry::new();
        let key = create_key("data-key", "AES-256-GCM", KeyPurpose::Encryption, 1000, 86400, None, 1);
        register_key(&mut registry, key).unwrap();
        registry
    }

    #[test]
    fn test_create_key_metadata() {
        let key = create_key("test", "AES-256-GCM", KeyPurpose::Encryption, 1000, 86400, None, 1);
        assert_eq!(key.id, "test");
        assert_eq!(key.version, 1);
        assert_eq!(key.algorithm, "AES-256-GCM");
        assert_eq!(key.status, KeyStatus::Active);
        assert_eq!(key.key_material.len(), 32);
    }

    #[test]
    fn test_register_and_find() {
        let registry = setup_registry();
        let key = find_active_key(&registry, "data-key");
        assert!(key.is_some());
        assert_eq!(key.unwrap().version, 1);
    }

    #[test]
    fn test_register_duplicate_rejected() {
        let mut registry = setup_registry();
        let key = create_key("data-key", "AES-256-GCM", KeyPurpose::Encryption, 1000, 86400, None, 1);
        let result = register_key(&mut registry, key);
        assert!(result.is_err(), "Duplicate (id, version) should be rejected");
    }

    #[test]
    fn test_rotate_key_retires_old() {
        let mut registry = setup_registry();
        let new_version = rotate_key(&mut registry, "data-key", 5000, 86400).unwrap();
        assert_eq!(new_version, 2);

        let old = find_key_version(&registry, "data-key", 1);
        assert_eq!(old.unwrap().status, KeyStatus::Retired);

        let new = find_active_key(&registry, "data-key");
        assert!(new.is_some());
        assert_eq!(new.unwrap().version, 2);
    }

    #[test]
    fn test_mark_compromised() {
        let mut registry = setup_registry();
        mark_compromised(&mut registry, "data-key", 1).unwrap();
        let key = find_key_version(&registry, "data-key", 1);
        assert_eq!(key.unwrap().status, KeyStatus::Compromised);
    }

    #[test]
    fn test_compromised_key_rejected_for_decryption() {
        let mut registry = setup_registry();
        mark_compromised(&mut registry, "data-key", 1).unwrap();
        let result = get_decryption_key(&registry, "data-key", 1);
        assert!(result.is_err(), "Compromised key should be rejected");
    }

    #[test]
    fn test_list_versions() {
        let mut registry = setup_registry();
        rotate_key(&mut registry, "data-key", 5000, 86400).unwrap();
        rotate_key(&mut registry, "data-key", 10000, 86400).unwrap();
        let versions = list_versions(&registry, "data-key");
        assert_eq!(versions.len(), 3);
        assert_eq!(versions[0].0, 1);
        assert_eq!(versions[1].0, 2);
        assert_eq!(versions[2].0, 3);
    }

    #[test]
    fn test_find_nonexistent_key() {
        let registry = setup_registry();
        assert!(find_active_key(&registry, "nonexistent").is_none());
        assert!(find_key_version(&registry, "data-key", 99).is_none());
    }
}
