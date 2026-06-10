//! # Lesson 05: Key Rotation Strategy
//!
//! ## What Is Key Rotation?
//!
//! Key rotation is the practice of periodically replacing cryptographic keys with new ones.
//! Old keys are kept (in a "retired" state) to decrypt existing data, but new data is
//! encrypted with the new key.
//!
//! ```text
//! Timeline:
//!   [Key v1]---encrypt data Jan-Jun---|
//!                                      [Key v2]---encrypt data Jul-Dec---|
//!                                                                        [Key v3]---
//!
//! Decryption:  data from Jan needs Key v1, data from Jul needs Key v2
//! ```
//!
//! ## Why Rotate Keys?
//!
//! 1. **Limit breach impact**: If a key is compromised, only data encrypted during
//!    that key's lifetime is exposed
//! 2. **Compliance**: PCI-DSS, HIPAA, and NIST require periodic key rotation
//! 3. **Cryptographic wear-out**: After encrypting too much data with one key,
//!    the security margin decreases (birthday bound on nonces)
//! 4. **Forward secrecy**: Compromising the current key doesn't reveal past data
//!    if past keys were securely destroyed
//!
//! ## Attack Scenario: No Key Rotation
//!
//! A company uses the same AES-256 key for 10 years. An attacker compromises the key
//! in year 10 and decrypts ALL data ever encrypted. With rotation every 90 days, only
//! 90 days of data would be exposed.
//!
//! ## Strategies
//!
//! | Strategy | When to rotate | Complexity |
//! |----------|---------------|------------|
//! | Time-based | Every N days | Low |
//! | Usage-based | After N encryptions | Medium |
//! | Event-based | On compromise suspicion | Reactive |
//! | Hybrid | Combination | High |

use serde::{Deserialize, Serialize};

/// A versioned encryption key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedKey {
    pub version: u32,
    pub key: Vec<u8>,
    pub created_at: u64,   // Unix timestamp
    pub expires_at: u64,   // Unix timestamp
    pub active: bool,      // true = encrypt with this, false = decrypt only
}

/// An encrypted record tagged with the key version used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedRecord {
    pub key_version: u32,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Exercise 1: Create a new versioned key.
///
/// Given the current highest version number, create a new key with version + 1.
/// The key should be 32 random bytes, active = true, with created_at and expires_at set.
///
/// Hints:
/// - Use `ring::SystemRandom` for the key
/// - `created_at` = current_timestamp, `expires_at` = current_timestamp + validity_seconds
pub fn create_new_key(current_version: u32, current_timestamp: u64, validity_seconds: u64) -> VersionedKey {
    todo!("Create a new versioned key")
}

/// Exercise 2: Encrypt data with the current active key.
///
/// Given a set of versioned keys, find the active one and use it to encrypt.
/// Return an EncryptedRecord tagged with the key version.
///
/// Hints:
/// - Find the key where `active == true`
/// - Use AES-256-GCM style encryption (you can use ring's AES-GCM)
/// - Generate a fresh nonce for each encryption
pub fn encrypt_with_active_key(keys: &[VersionedKey], plaintext: &[u8]) -> EncryptedRecord {
    todo!("Encrypt with the current active key")
}

/// Exercise 3: Decrypt data using the correct key version.
///
/// Look up the key version from the EncryptedRecord, find the matching key,
/// and decrypt.
///
/// Hints:
/// - Find the key with matching `version` from the record
/// - Use the nonce from the record
/// - The key may be inactive (retired) — that's OK for decryption
pub fn decrypt_with_version(keys: &[VersionedKey], record: &EncryptedRecord) -> Vec<u8> {
    todo!("Decrypt using the correct key version")
}

/// Exercise 4: Rotate keys — deactivate the current key and create a new one.
///
/// This simulates a key rotation event:
/// 1. Set `active = false` on all current keys
/// 2. Create a new key with `active = true`
/// 3. Return the updated key list
///
/// Hints:
/// - Iterate through keys, set `active = false`
/// - Create new key with highest version + 1
pub fn rotate_keys(keys: &mut Vec<VersionedKey>, current_timestamp: u64, validity_seconds: u64) {
    todo!("Rotate keys: deactivate old, create new")
}

/// Exercise 5: Find expired keys.
///
/// Return the versions of all keys whose `expires_at` is before the current timestamp.
pub fn find_expired_keys(keys: &[VersionedKey], current_timestamp: u64) -> Vec<u32> {
    todo!("Find all expired key versions")
}

/// Exercise 6: Demonstrate that rotation limits breach impact.
///
/// Simulate: encrypt 10 records across 3 key versions (rotate after every 3 records).
/// Show that only records encrypted with the compromised key are affected.
///
/// Returns (total_records, records_per_version: Vec<(version, count)>)
pub fn demonstrate_breach_limit(
    keys: &[VersionedKey],
    record_count: usize,
) -> (usize, Vec<(u32, usize)>) {
    todo!("Demonstrate how key rotation limits breach impact")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_keys() -> Vec<VersionedKey> {
        vec![
            VersionedKey {
                version: 1,
                key: vec![0xAA; 32],
                created_at: 1000,
                expires_at: 2000,
                active: false,
            },
            VersionedKey {
                version: 2,
                key: vec![0xBB; 32],
                created_at: 2000,
                expires_at: 3000,
                active: true,
            },
        ]
    }

    #[test]
    fn test_create_new_key_version() {
        let key = create_new_key(5, 1000, 90 * 86400);
        assert_eq!(key.version, 6);
    }

    #[test]
    fn test_create_new_key_length() {
        let key = create_new_key(0, 1000, 86400);
        assert_eq!(key.key.len(), 32);
    }

    #[test]
    fn test_create_new_key_active() {
        let key = create_new_key(0, 1000, 86400);
        assert!(key.active);
    }

    #[test]
    fn test_create_new_key_expiry() {
        let key = create_new_key(0, 1000, 86400);
        assert_eq!(key.expires_at, 1000 + 86400);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let keys = sample_keys();
        let plaintext = b"Hello, key rotation!";
        let record = encrypt_with_active_key(&keys, plaintext);
        let decrypted = decrypt_with_version(&keys, &record);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_uses_active_key() {
        let keys = sample_keys();
        let record = encrypt_with_active_key(&keys, b"test");
        assert_eq!(record.key_version, 2, "Should use active key version 2");
    }

    #[test]
    fn test_decrypt_with_old_version() {
        // Encrypt with v1 key directly, then decrypt
        let keys = sample_keys();
        let record = EncryptedRecord {
            key_version: 1,
            ciphertext: Vec::new(), // placeholder
            nonce: Vec::new(),
        };
        // Just verify the key can be found (full encryption test would need real crypto)
        let key = keys.iter().find(|k| k.version == record.key_version);
        assert!(key.is_some());
    }

    #[test]
    fn test_rotate_keys() {
        let mut keys = sample_keys();
        rotate_keys(&mut keys, 3000, 86400);
        // Old keys should be inactive
        for k in &keys[..2] {
            assert!(!k.active, "Old key v{} should be inactive", k.version);
        }
        // New key should be active
        let new_key = keys.last().unwrap();
        assert!(new_key.active);
        assert_eq!(new_key.version, 3);
    }

    #[test]
    fn test_find_expired_keys() {
        let keys = sample_keys();
        let expired = find_expired_keys(&keys, 2500);
        assert_eq!(expired, vec![1]); // Key v1 expired (expires_at=2000 < 2500)
    }

    #[test]
    fn test_find_expired_keys_none() {
        let keys = sample_keys();
        let expired = find_expired_keys(&keys, 500);
        assert!(expired.is_empty());
    }
}
