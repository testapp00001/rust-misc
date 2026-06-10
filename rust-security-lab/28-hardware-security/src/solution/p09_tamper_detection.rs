//! # Lesson 09: Tamper Detection and Response (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use ring::rand::SecureRandom;
use zeroize::Zeroize;
use serde::{Deserialize, Serialize};

/// Security state of a device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityState {
    Normal,
    TamperDetected,
    Disabled,
}

/// A key with automatic zeroization on drop.
#[derive(Debug, Clone, Zeroize)]
#[zeroize(drop)]
pub struct SecureKey {
    pub id: String,
    pub material: Vec<u8>,
}

/// A key store that supports tamper detection and zeroization.
#[derive(Debug, Clone)]
pub struct SecureKeyStore {
    keys: Vec<SecureKey>,
    pub state: SecurityState,
    pub tamper_count: u32,
    pub max_tamper_events: u32,
}

impl SecureKeyStore {
    /// Create a new secure key store with tamper threshold.
    pub fn new(max_tamper_events: u32) -> Self {
        Self {
            keys: Vec::new(),
            state: SecurityState::Normal,
            tamper_count: 0,
            max_tamper_events,
        }
    }

    /// Store a key (reject if device is disabled).
    pub fn store_key(&mut self, id: &str, material: &[u8]) -> bool {
        if self.state == SecurityState::Disabled {
            return false;
        }
        self.keys.push(SecureKey {
            id: id.to_string(),
            material: material.to_vec(),
        });
        true
    }

    /// Retrieve a key (return None if disabled).
    pub fn get_key(&self, id: &str) -> Option<&[u8]> {
        if self.state == SecurityState::Disabled {
            return None;
        }
        self.keys.iter().find(|k| k.id == id).map(|k| k.material.as_slice())
    }

    /// Handle tamper event: zeroize all keys, update state.
    pub fn tamper_detected(&mut self) {
        self.tamper_count += 1;

        // Zeroize all keys
        for key in &mut self.keys {
            key.material.iter_mut().for_each(|b| *b = 0);
        }

        if self.tamper_count >= self.max_tamper_events {
            self.state = SecurityState::Disabled;
        } else {
            self.state = SecurityState::TamperDetected;
        }
    }

    /// Recover from tamper state with valid recovery code.
    pub fn recover(&mut self, recovery_code: &[u8]) -> bool {
        if self.state != SecurityState::TamperDetected {
            return false;
        }

        // Recovery code must be SHA-256 of "admin-recovery-secret"
        let expected = digest::digest(&digest::SHA256, b"admin-recovery-secret")
            .as_ref()
            .to_vec();

        if ring::constant_time::verify_slices_are_equal(recovery_code, &expected).is_ok() {
            self.state = SecurityState::Normal;
            self.tamper_count = 0;
            true
        } else {
            false
        }
    }

    /// Perform integrity self-test.
    pub fn self_test(&self) -> bool {
        if self.state == SecurityState::TamperDetected {
            // Verify all keys are zeroed
            for key in &self.keys {
                if !key.material.iter().all(|&b| b == 0) {
                    return false;
                }
            }
        }
        true
    }

    pub fn key_count(&self) -> usize {
        self.keys.len()
    }
}

/// Multi-pass secure memory wipe: zeros -> 0xFF -> random -> zeros.
pub fn secure_wipe(data: &mut [u8]) {
    // Pass 1: zeros
    data.iter_mut().for_each(|b| *b = 0x00);
    // Pass 2: all ones
    data.iter_mut().for_each(|b| *b = 0xFF);
    // Pass 3: random data
    let rng = ring::rand::SystemRandom::new();
    let mut random = vec![0u8; data.len()];
    rng.fill(&mut random).unwrap();
    data.copy_from_slice(&random);
    // Pass 4: zeros again
    data.iter_mut().for_each(|b| *b = 0x00);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_store_creation() {
        let store = SecureKeyStore::new(3);
        assert_eq!(store.state, SecurityState::Normal);
        assert_eq!(store.tamper_count, 0);
    }

    #[test]
    fn test_store_and_retrieve_key() {
        let mut store = SecureKeyStore::new(3);
        store.store_key("key1", b"secret material");
        let key = store.get_key("key1").unwrap();
        assert_eq!(key, b"secret material");
    }

    #[test]
    fn test_tamper_zeroizes_keys() {
        let mut store = SecureKeyStore::new(3);
        store.store_key("key1", b"secret material");
        store.tamper_detected();

        assert_eq!(store.state, SecurityState::TamperDetected);
        let key = store.get_key("key1").unwrap();
        assert!(key.iter().all(|&b| b == 0), "Key should be zeroized after tamper");
    }

    #[test]
    fn test_repeated_tamper_disables_device() {
        let mut store = SecureKeyStore::new(2);
        store.store_key("key1", b"secret");
        store.tamper_detected();
        assert_eq!(store.state, SecurityState::TamperDetected);

        store.tamper_detected();
        assert_eq!(store.state, SecurityState::Disabled);
    }

    #[test]
    fn test_disabled_device_rejects_operations() {
        let mut store = SecureKeyStore::new(1);
        store.tamper_detected();
        assert_eq!(store.state, SecurityState::Disabled);
        assert!(!store.store_key("new", b"data"));
        assert!(store.get_key("new").is_none());
    }

    #[test]
    fn test_recovery_from_tamper() {
        let mut store = SecureKeyStore::new(3);
        store.tamper_detected();
        assert_eq!(store.state, SecurityState::TamperDetected);

        let recovery_code = digest::digest(&digest::SHA256, b"admin-recovery-secret")
            .as_ref()
            .to_vec();

        assert!(store.recover(&recovery_code));
        assert_eq!(store.state, SecurityState::Normal);
        assert_eq!(store.tamper_count, 0);
    }

    #[test]
    fn test_cannot_recover_when_disabled() {
        let mut store = SecureKeyStore::new(1);
        store.tamper_detected();
        assert!(!store.recover(b"any code"));
    }

    #[test]
    fn test_secure_wipe() {
        let mut data = vec![0xAB; 64];
        secure_wipe(&mut data);
        assert!(data.iter().all(|&b| b == 0), "After secure wipe, data should be zeros");
    }

    #[test]
    fn test_self_test_passes_in_normal_state() {
        let store = SecureKeyStore::new(3);
        assert!(store.self_test());
    }

    #[test]
    fn test_self_test_passes_after_tamper_with_zeroized_keys() {
        let mut store = SecureKeyStore::new(3);
        store.store_key("key1", b"data");
        store.tamper_detected();
        assert!(store.self_test(), "Self-test should pass after proper zeroization");
    }
}
