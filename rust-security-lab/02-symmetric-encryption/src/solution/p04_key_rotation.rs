//! # Lesson 04: Key Rotation (Reference Solution)
//!
//! See the exercise file for full documentation on key rotation strategies.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Nonce},
    Aes256Gcm, Key,
};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type KeyVersion = u64;

/// Tracks which key version was active at what time.
pub struct KeyEpochTracker {
    epochs: HashMap<KeyVersion, u64>,
    current_version: KeyVersion,
}

impl KeyEpochTracker {
    pub fn new() -> Self {
        let mut epochs = HashMap::new();
        let now = Self::now();
        epochs.insert(1, now);
        Self {
            epochs,
            current_version: 1,
        }
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn current_version(&self) -> KeyVersion {
        self.current_version
    }

    pub fn rotate(&mut self) -> KeyVersion {
        self.current_version += 1;
        self.epochs.insert(self.current_version, Self::now());
        self.current_version
    }

    pub fn epoch_start(&self, version: KeyVersion) -> Option<u64> {
        self.epochs.get(&version).copied()
    }
}

/// Versioned key store with rotation and grace period.
pub struct VersionedKeyStore {
    keys: HashMap<KeyVersion, Key<Aes256Gcm>>,
    current_version: KeyVersion,
    grace_versions: Vec<KeyVersion>,
}

impl VersionedKeyStore {
    pub fn new() -> Self {
        let mut keys = HashMap::new();
        let key = Aes256Gcm::generate_key(&mut OsRng);
        keys.insert(1, key);
        Self {
            keys,
            current_version: 1,
            grace_versions: Vec::new(),
        }
    }

    pub fn current_version(&self) -> KeyVersion {
        self.current_version
    }

    pub fn rotate(&mut self) -> KeyVersion {
        // Move current to grace period
        self.grace_versions.push(self.current_version);

        // Keep at most 2 grace versions
        if self.grace_versions.len() > 2 {
            let removed = self.grace_versions.remove(0);
            self.keys.remove(&removed);
        }

        // Generate new key
        self.current_version += 1;
        let key = Aes256Gcm::generate_key(&mut OsRng);
        self.keys.insert(self.current_version, key);

        self.current_version
    }

    pub fn get_key(&self, version: KeyVersion) -> Option<&Key<Aes256Gcm>> {
        self.keys.get(&version)
    }

    pub fn current_key(&self) -> &Key<Aes256Gcm> {
        self.keys.get(&self.current_version).expect("Current key must exist")
    }

    /// Encrypt with current key, prepending version + nonce.
    /// Format: [version: 8 bytes] [nonce: 12 bytes] [ciphertext+tag]
    pub fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        let key = self.current_key();
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext).expect("Encryption failed");

        let mut output = Vec::with_capacity(8 + 12 + ciphertext.len());
        output.extend_from_slice(&self.current_version.to_be_bytes());
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&ciphertext);
        output
    }

    /// Decrypt data encrypted with `encrypt`.
    /// Parses version, looks up key, extracts nonce, decrypts.
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, &'static str> {
        if data.len() < 20 {
            return Err("Data too short: need at least 8 (version) + 12 (nonce) bytes");
        }

        let version = u64::from_be_bytes(data[..8].try_into().unwrap());
        let nonce = Nonce::<Aes256Gcm>::from_slice(&data[8..20]);
        let ciphertext = &data[20..];

        let key = self.get_key(version).ok_or("Unknown key version")?;
        let cipher = Aes256Gcm::new(key);
        cipher.decrypt(nonce, ciphertext).map_err(|_| "Decryption failed (bad key, nonce, or tampered data)")
    }
}

/// Check if the key should be rotated based on encryption count.
pub fn should_rotate_by_count(encryption_count: u64, max_encryptions: u64) -> bool {
    encryption_count >= max_encryptions
}

/// Check if the key should be rotated based on time elapsed.
pub fn should_rotate_by_time(epoch_start: u64, max_age: Duration) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    now.saturating_sub(epoch_start) >= max_age.as_secs()
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

        store.rotate();

        let decrypted = store.decrypt(&encrypted).expect("Grace period decryption failed");
        assert_eq!(decrypted, plaintext);

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

        assert!(!should_rotate_by_time(now, one_day));
        assert!(should_rotate_by_time(now - 172800, one_day));
        assert!(should_rotate_by_time(now - 7200, one_hour));
    }
}
