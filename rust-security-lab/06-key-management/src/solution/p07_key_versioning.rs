//! # Lesson 07: Key Versioning and Metadata Management (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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
    pub fn new() -> Self {
        KeyRegistry { keys: Vec::new() }
    }
}

/// Create a new key with full metadata.
///
/// Generates a 32-byte random key using the OS CSPRNG.
pub fn create_key(
    id: &str,
    algorithm: &str,
    purpose: KeyPurpose,
    current_timestamp: u64,
    validity_seconds: u64,
    rotated_from: Option<&str>,
    version: u32,
) -> KeyMetadata {
    let rng = SystemRandom::new();
    let mut key_material = vec![0u8; 32];
    rng.fill(&mut key_material).expect("Failed to generate key material");

    KeyMetadata {
        id: id.to_string(),
        version,
        algorithm: algorithm.to_string(),
        purpose,
        created_at: current_timestamp,
        expires_at: current_timestamp + validity_seconds,
        status: KeyStatus::Active,
        rotated_from: rotated_from.map(|s| s.to_string()),
        key_material,
    }
}

/// Register a key in the registry.
///
/// Rejects duplicates with the same (id, version).
pub fn register_key(registry: &mut KeyRegistry, key: KeyMetadata) -> Result<(), String> {
    if registry.keys.iter().any(|k| k.id == key.id && k.version == key.version) {
        return Err(format!("Key {} v{} already exists", key.id, key.version));
    }
    registry.keys.push(key);
    Ok(())
}

/// Find the active key for a given ID.
pub fn find_active_key<'a>(registry: &'a KeyRegistry, key_id: &str) -> Option<&'a KeyMetadata> {
    registry
        .keys
        .iter()
        .find(|k| k.id == key_id && k.status == KeyStatus::Active)
}

/// Find a specific key version (any status).
pub fn find_key_version<'a>(registry: &'a KeyRegistry, key_id: &str, version: u32) -> Option<&'a KeyMetadata> {
    registry
        .keys
        .iter()
        .find(|k| k.id == key_id && k.version == version)
}

/// Rotate a key: retire the current active, create a new version.
///
/// Returns the new version number.
pub fn rotate_key(
    registry: &mut KeyRegistry,
    key_id: &str,
    current_timestamp: u64,
    validity_seconds: u64,
) -> Result<u32, String> {
    // Find and retire the current active key
    let current = registry
        .keys
        .iter()
        .find(|k| k.id == key_id && k.status == KeyStatus::Active)
        .cloned()
        .ok_or_else(|| format!("No active key found for {}", key_id))?;

    let old_version = current.version;
    let new_version = old_version + 1;

    // Retire the old key
    for k in registry.keys.iter_mut() {
        if k.id == key_id && k.version == old_version {
            k.status = KeyStatus::Retired;
        }
    }

    // Create and register the new key
    let new_key = create_key(
        key_id,
        &current.algorithm,
        current.purpose,
        current_timestamp,
        validity_seconds,
        Some(&format!("{}-v{}", key_id, old_version)),
        new_version,
    );
    registry.keys.push(new_key);

    Ok(new_version)
}

/// Mark a key version as compromised.
pub fn mark_compromised(
    registry: &mut KeyRegistry,
    key_id: &str,
    version: u32,
) -> Result<(), String> {
    let key = registry
        .keys
        .iter_mut()
        .find(|k| k.id == key_id && k.version == version)
        .ok_or_else(|| format!("Key {} v{} not found", key_id, version))?;
    key.status = KeyStatus::Compromised;
    Ok(())
}

/// List all versions of a key, ordered by version.
pub fn list_versions(registry: &KeyRegistry, key_id: &str) -> Vec<(u32, KeyStatus)> {
    let mut versions: Vec<(u32, KeyStatus)> = registry
        .keys
        .iter()
        .filter(|k| k.id == key_id)
        .map(|k| (k.version, k.status))
        .collect();
    versions.sort_by_key(|(v, _)| *v);
    versions
}

/// Get key material for decryption, rejecting compromised keys.
pub fn get_decryption_key(
    registry: &KeyRegistry,
    key_id: &str,
    version: u32,
) -> Result<Vec<u8>, String> {
    let key = registry
        .keys
        .iter()
        .find(|k| k.id == key_id && k.version == version)
        .ok_or_else(|| format!("Key {} v{} not found", key_id, version))?;

    if key.status == KeyStatus::Compromised {
        return Err(format!("Key {} v{} is compromised", key_id, version));
    }

    Ok(key.key_material.clone())
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
        assert!(result.is_err());
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
        assert!(result.is_err());
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
