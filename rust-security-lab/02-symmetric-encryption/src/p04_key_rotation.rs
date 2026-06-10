//! # Lesson 04: Key Rotation
//!
//! ## Why Rotate Keys?
//!
//! Key rotation limits the blast radius of a key compromise. If an attacker obtains a key,
//! they can only decrypt data encrypted during that key's active period — not your entire
//! history.
//!
//! ## Key Rotation Strategies
//!
//! | Strategy | Description | Use Case |
//! |----------|-------------|----------|
//! | Time-based | Rotate every N hours/days | TLS certificates, API keys |
//! | Volume-based | Rotate after N encryptions | Database encryption |
//! | Event-based | Rotate on compromise detection | Incident response |
//! | Hybrid | Rotate on whichever threshold hits first | Best practice |
//!
//! ## Key Hierarchy
//!
//! In practice, you don't directly rotate the data-encryption key (DEK). Instead:
//!
//! ```text
//! Key Encryption Key (KEK)  ← rotated rarely, stored in HSM/KMS
//!       │
//!       ├── DEK-1 (epoch 1) ← encrypts data for period 1
//!       ├── DEK-2 (epoch 2) ← encrypts data for period 2
//!       └── DEK-3 (epoch 3) ← encrypts data for period 3
//! ```
//!
//! The KEK encrypts DEKs. Rotating the KEK re-encrypts all DEKs, not the actual data.
//!
//! ## Attack Scenario: No Key Rotation
//!
//! If you never rotate keys:
//! 1. A single key compromise exposes ALL data ever encrypted
//! 2. The key has been in use longer, increasing the attack window
//! 3. More ciphertext is available for cryptanalysis
//! 4. Compliance violations (PCI-DSS requires 90-day rotation)
//!
//! ## Defense
//!
//! - Use a key hierarchy (KEK + DEKs)
//! - Rotate keys on a schedule with safety margins
//! - Maintain a "grace period" where old keys can still decrypt
//! - Securely destroy old keys after the grace period
//! - Log all key operations for audit

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// A key version identifier. Each key in the system has a unique version.
pub type KeyVersion = u64;

/// Exercise 1: Implement a key epoch tracker.
///
/// A key epoch represents a time period during which a specific key is active.
/// This struct tracks which key version was active at what time.
///
/// Implement the methods below to manage key epochs.
pub struct KeyEpochTracker {
    /// Map from KeyVersion to the timestamp (seconds since UNIX epoch) when it became active.
    epochs: HashMap<KeyVersion, u64>,
    /// The currently active key version.
    current_version: KeyVersion,
}

impl KeyEpochTracker {
    /// Create a new tracker with the first key version starting now.
    pub fn new() -> Self {
        todo!("Create a new KeyEpochTracker with version 1 and current timestamp")
    }

    /// Get the current timestamp in seconds since UNIX epoch.
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get the current active key version.
    pub fn current_version(&self) -> KeyVersion {
        todo!("Return the current key version")
    }

    /// Rotate to a new key version. Returns the new version number.
    ///
    /// The new version should be current_version + 1.
    /// Record the current timestamp for the new epoch.
    pub fn rotate(&mut self) -> KeyVersion {
        todo!("Increment version and record the epoch timestamp")
    }

    /// Get the timestamp when a specific key version became active.
    /// Returns None if the version doesn't exist.
    pub fn epoch_start(&self, version: KeyVersion) -> Option<u64> {
        todo!("Look up the epoch start time for the given version")
    }
}

/// Exercise 2: Implement a versioned key store with rotation.
///
/// This struct holds multiple key versions and can encrypt/decrypt with
/// any version. On rotation, a new key is generated and stored.
pub struct VersionedKeyStore {
    /// Map from KeyVersion to the actual AES key.
    keys: HashMap<KeyVersion, Key<Aes256Gcm>>,
    /// The currently active key version.
    current_version: KeyVersion,
    /// Grace period versions (old keys still accepted for decryption).
    grace_versions: Vec<KeyVersion>,
}

impl VersionedKeyStore {
    /// Create a new store with one initial key.
    pub fn new() -> Self {
        todo!("Generate an initial key and set version to 1")
    }

    /// Get the current active key version.
    pub fn current_version(&self) -> KeyVersion {
        todo!("Return current version")
    }

    /// Rotate to a new key.
    ///
    /// - Generate a new AES-256 key
    /// - Move the current version into grace_versions
    /// - Update current_version to the new version
    /// - If there are more than 2 grace versions, remove the oldest
    pub fn rotate(&mut self) -> KeyVersion {
        todo!("Generate new key, update versions and grace period")
    }

    /// Get a key by version. Returns None if the version doesn't exist.
    ///
    /// This is used during decryption when the ciphertext specifies which
    /// key version was used.
    pub fn get_key(&self, version: KeyVersion) -> Option<&Key<Aes256Gcm>> {
        todo!("Look up the key for the given version")
    }

    /// Get the current active key.
    pub fn current_key(&self) -> &Key<Aes256Gcm> {
        todo!("Return a reference to the current key")
    }

    /// Encrypt with the current key, prepending the key version to the output.
    ///
    /// Output format: [version: 8 bytes (big-endian u64)] [nonce: 12 bytes] [ciphertext+tag]
    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        todo!("Encrypt with current key and prepend version + nonce")
    }

    /// Decrypt data that was encrypted with `encrypt`.
    ///
    /// Parse the version from the first 8 bytes, look up the key,
    /// extract the nonce, and decrypt.
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        todo!("Parse version, find key, extract nonce, decrypt")
    }
}

/// Exercise 3: Implement key rotation based on encryption count.
///
/// Returns true if the key should be rotated based on the number of
/// encryptions performed with the current key.
///
/// The threshold is the maximum number of encryptions per key.
pub fn should_rotate_by_count(encryption_count: u64, max_encryptions: u64) -> bool {
    todo!("Check if encryption count exceeds the threshold")
}

/// Exercise 4: Implement key rotation based on time.
///
/// Returns true if the key should be rotated based on how long it has been active.
///
/// `epoch_start` is the timestamp when the key became active.
/// `max_age` is the maximum allowed key lifetime.
pub fn should_rotate_by_time(epoch_start: u64, max_age: Duration) -> bool {
    todo!("Check if the key has exceeded its maximum age")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epoch_tracker_initial_version() {
        let tracker = KeyEpochTracker::new();
        assert_eq!(tracker.current_version(), 1);
    }

    #[test]
    fn test_epoch_tracker_rotation() {
        let mut tracker = KeyEpochTracker::new();
        let v2 = tracker.rotate();
        assert_eq!(v2, 2);
        assert_eq!(tracker.current_version(), 2);
    }

    #[test]
    fn test_epoch_tracker_epoch_start() {
        let mut tracker = KeyEpochTracker::new();
        let start_v1 = tracker.epoch_start(1).unwrap();
        tracker.rotate();
        let start_v2 = tracker.epoch_start(2).unwrap();
        assert!(start_v2 >= start_v1, "V2 epoch should be at or after V1");
        assert!(tracker.epoch_start(999).is_none(), "Non-existent version returns None");
    }

    #[test]
    fn test_versioned_store_initial() {
        let store = VersionedKeyStore::new();
        assert_eq!(store.current_version(), 1);
        assert!(store.get_key(1).is_some());
    }

    #[test]
    fn test_versioned_store_rotation() {
        let mut store = VersionedKeyStore::new();
        store.rotate();
        assert_eq!(store.current_version(), 2);
        // Old key should still be available (grace period)
        assert!(store.get_key(1).is_some());
        assert!(store.get_key(2).is_some());
    }

    #[test]
    fn test_versioned_store_encrypt_decrypt_roundtrip() {
        let store = VersionedKeyStore::new();
        let plaintext = b"key rotation test";
        let encrypted = store.encrypt(plaintext);
        let decrypted = store.decrypt(&encrypted).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_versioned_store_decrypt_after_rotation() {
        let mut store = VersionedKeyStore::new();
        let plaintext = b"encrypted with v1";
        let encrypted = store.encrypt(plaintext);

        // Rotate to new key
        store.rotate();

        // Should still decrypt with old key (grace period)
        let decrypted = store.decrypt(&encrypted).expect("Grace period decryption failed");
        assert_eq!(decrypted, plaintext);

        // New encryption should use v2
        let new_encrypted = store.encrypt(b"encrypted with v2");
        let new_decrypted = store.decrypt(&new_encrypted).expect("V2 decryption failed");
        assert_eq!(new_decrypted, b"encrypted with v2");
    }

    #[test]
    fn test_should_rotate_by_count() {
        assert!(!should_rotate_by_count(100, 1000));
        assert!(should_rotate_by_count(1000, 1000));
        assert!(should_rotate_by_count(1001, 1000));
    }

    #[test]
    fn test_should_rotate_by_time() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let one_hour = Duration::from_secs(3600);
        let one_day = Duration::from_secs(86400);

        // Key started now — not expired for 1 day max
        assert!(!should_rotate_by_time(now, one_day));
        // Key started 2 days ago — expired for 1 day max
        assert!(should_rotate_by_time(now - 172800, one_day));
        // Key started 2 hours ago — expired for 1 hour max
        assert!(should_rotate_by_time(now - 7200, one_hour));
    }
}
