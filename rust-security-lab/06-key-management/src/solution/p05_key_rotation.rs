//! # Lesson 05: Key Rotation Strategy (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

/// A versioned encryption key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedKey {
    pub version: u32,
    pub key: Vec<u8>,
    pub created_at: u64,
    pub expires_at: u64,
    pub active: bool,
}

/// An encrypted record tagged with the key version used.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedRecord {
    pub key_version: u32,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Create a new versioned key with version = current_version + 1.
///
/// Uses `ring::SystemRandom` for cryptographic key generation.
pub fn create_new_key(current_version: u32, current_timestamp: u64, validity_seconds: u64) -> VersionedKey {
    let rng = SystemRandom::new();
    let mut key = vec![0u8; 32];
    rng.fill(&mut key).expect("Failed to generate random key");

    VersionedKey {
        version: current_version + 1,
        key,
        created_at: current_timestamp,
        expires_at: current_timestamp + validity_seconds,
        active: true,
    }
}

/// Encrypt data with the current active key.
///
/// Uses XOR-based encryption for simplicity (demonstrates the pattern).
/// In production, use ring's AES-256-GCM.
pub fn encrypt_with_active_key(keys: &[VersionedKey], plaintext: &[u8]) -> EncryptedRecord {
    let active = keys.iter().find(|k| k.active).expect("No active key found");

    let rng = SystemRandom::new();
    let mut nonce = vec![0u8; 12];
    rng.fill(&mut nonce).expect("Failed to generate nonce");

    // XOR-based encryption with key + nonce for demonstration
    let ciphertext: Vec<u8> = plaintext
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            let key_byte = active.key[i % active.key.len()];
            let nonce_byte = nonce[i % nonce.len()];
            b ^ key_byte ^ nonce_byte
        })
        .collect();

    EncryptedRecord {
        key_version: active.version,
        ciphertext,
        nonce,
    }
}

/// Decrypt data using the correct key version.
///
/// Looks up the key by version from the record, then decrypts.
pub fn decrypt_with_version(keys: &[VersionedKey], record: &EncryptedRecord) -> Vec<u8> {
    let key = keys
        .iter()
        .find(|k| k.version == record.key_version)
        .expect("Key version not found");

    // Reverse the XOR encryption
    record
        .ciphertext
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            let key_byte = key.key[i % key.key.len()];
            let nonce_byte = record.nonce[i % record.nonce.len()];
            b ^ key_byte ^ nonce_byte
        })
        .collect()
}

/// Rotate keys — deactivate all current keys and create a new one.
///
/// 1. Set `active = false` on all existing keys
/// 2. Create a new key with highest version + 1
/// 3. Push the new key onto the list
pub fn rotate_keys(keys: &mut Vec<VersionedKey>, current_timestamp: u64, validity_seconds: u64) {
    for key in keys.iter_mut() {
        key.active = false;
    }
    let max_version = keys.iter().map(|k| k.version).max().unwrap_or(0);
    let new_key = create_new_key(max_version, current_timestamp, validity_seconds);
    keys.push(new_key);
}

/// Find all expired keys.
///
/// A key is expired if its `expires_at` is before the current timestamp.
pub fn find_expired_keys(keys: &[VersionedKey], current_timestamp: u64) -> Vec<u32> {
    keys.iter()
        .filter(|k| k.expires_at < current_timestamp)
        .map(|k| k.version)
        .collect()
}

/// Demonstrate that key rotation limits breach impact.
///
/// Simulates encrypting `record_count` records across multiple key versions,
/// rotating after every `record_count / 3` records.
///
/// Returns (total_records, records_per_version)
pub fn demonstrate_breach_limit(
    keys: &[VersionedKey],
    record_count: usize,
) -> (usize, Vec<(u32, usize)>) {
    let mut working_keys = keys.to_vec();
    let records_per_rotation = record_count / 3;
    let mut version_counts: Vec<(u32, usize)> = Vec::new();

    let mut encrypted_count = 0;
    while encrypted_count < record_count {
        let batch_size = std::cmp::min(records_per_rotation, record_count - encrypted_count);
        let active_version = working_keys.iter().find(|k| k.active).unwrap().version;
        version_counts.push((active_version, batch_size));
        encrypted_count += batch_size;

        if encrypted_count < record_count {
            rotate_keys(&mut working_keys, encrypted_count as u64, 86400);
        }
    }

    (record_count, version_counts)
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
        let keys = sample_keys();
        let record = EncryptedRecord {
            key_version: 1,
            ciphertext: Vec::new(),
            nonce: Vec::new(),
        };
        let key = keys.iter().find(|k| k.version == record.key_version);
        assert!(key.is_some());
    }

    #[test]
    fn test_rotate_keys() {
        let mut keys = sample_keys();
        rotate_keys(&mut keys, 3000, 86400);
        for k in &keys[..2] {
            assert!(!k.active, "Old key v{} should be inactive", k.version);
        }
        let new_key = keys.last().unwrap();
        assert!(new_key.active);
        assert_eq!(new_key.version, 3);
    }

    #[test]
    fn test_find_expired_keys() {
        let keys = sample_keys();
        let expired = find_expired_keys(&keys, 2500);
        assert_eq!(expired, vec![1]);
    }

    #[test]
    fn test_find_expired_keys_none() {
        let keys = sample_keys();
        let expired = find_expired_keys(&keys, 500);
        assert!(expired.is_empty());
    }
}
