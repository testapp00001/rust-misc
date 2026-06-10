//! # Lesson 09: Tamper Detection and Response
//!
//! ## What is Tamper Detection?
//!
//! Hardware security modules and secure elements detect physical intrusion attempts:
//! - **Tamper mesh**: A wire mesh around the chip. Cutting it triggers zeroization.
//! - **Light sensors**: Detect if the chip package is opened (light exposure).
//! - **Voltage monitors**: Detect abnormal power supply (fault injection).
//! - **Temperature sensors**: Detect extreme temperatures (cold boot attacks).
//! - **Active zeroization**: Immediately erase keys when tamper is detected.
//!
//! ## Zeroize on Tamper
//!
//! When tamper is detected, the device must:
//! 1. Overwrite all key material with zeros
//! 2. Overwrite again with 0xFF
//! 3. Overwrite again with random data
//! 4. Verify the overwrite succeeded
//!
//! This multi-pass approach defeats data remanence (residual magnetic charge).
//!
//! ## Cold Boot Attack
//!
//! An attacker freezes DRAM chips to retain data for seconds after power removal,
//! then reads the memory. Defense: zeroize keys in registers before they reach DRAM,
//! or use SRAM-based key storage that loses data instantly on power loss.
//!
//! ## Fault Injection Attack
//!
//! An attacker manipulates voltage or clock to cause computation errors,
//! potentially leaking key material (Differential Fault Analysis).
//! Defense: detect voltage/clock anomalies and zeroize.

use zeroize::Zeroize;
use serde::{Deserialize, Serialize};

/// Security state of a device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityState {
    /// Normal operation.
    Normal,
    /// Tamper detected — keys zeroized.
    TamperDetected,
    /// Device is permanently disabled after multiple tamper events.
    Disabled,
}

/// A key store that supports tamper detection and zeroization.
#[derive(Debug, Clone)]
pub struct SecureKeyStore {
    /// Stored keys (simulated — real HSM keeps these in SRAM).
    keys: Vec<SecureKey>,
    /// Current security state.
    pub state: SecurityState,
    /// Number of tamper events detected.
    pub tamper_count: u32,
    /// Maximum tamper events before permanent disable.
    pub max_tamper_events: u32,
}

/// A key with automatic zeroization on drop.
#[derive(Debug, Clone, Zeroize)]
#[zeroize(drop)]
pub struct SecureKey {
    pub id: String,
    pub material: Vec<u8>,
}

impl SecureKeyStore {
    /// Exercise 1: Create a new secure key store.
    ///
    /// Initialize in Normal state with the given tamper threshold.
    pub fn new(max_tamper_events: u32) -> Self {
        todo!("Create secure key store with tamper threshold")
    }

    /// Exercise 2: Store a key in the secure store.
    ///
    /// Returns false if the device is disabled.
    pub fn store_key(&mut self, id: &str, material: &[u8]) -> bool {
        todo!("Store key (reject if device is disabled)")
    }

    /// Exercise 3: Retrieve a key from the store.
    ///
    /// Returns None if the device is disabled or the key doesn't exist.
    pub fn get_key(&self, id: &str) -> Option<&[u8]> {
        todo!("Retrieve key (return None if disabled)")
    }

    /// Exercise 4: Detect a tamper event and respond.
    ///
    /// When tamper is detected:
    /// 1. Increment tamper_count
    /// 2. Zeroize ALL keys
    /// 3. If tamper_count >= max_tamper_events, set state to Disabled
    /// 4. Otherwise, set state to TamperDetected
    ///
    /// Hints:
    /// - Clear all key material (set to zeros)
    /// - Use zeroize pattern: material.iter_mut().for_each(|b| *b = 0)
    pub fn tamper_detected(&mut self) {
        todo!("Handle tamper event: zeroize keys, update state")
    }

    /// Exercise 5: Attempt to recover from tamper state.
    ///
    /// Recovery is only possible if:
    /// - Current state is TamperDetected (not Disabled)
    /// - A valid recovery code is provided (SHA-256 of the recovery secret)
    ///
    /// If recovery succeeds, set state to Normal and clear tamper count.
    /// Returns true if recovery succeeded.
    pub fn recover(&mut self, recovery_code: &[u8]) -> bool {
        todo!("Recover from tamper state with valid recovery code")
    }

    /// Exercise 6: Perform a secure self-test.
    ///
    /// Verify that:
    /// 1. Key material is actually zero (if in TamperDetected state)
    /// 2. Internal consistency checks pass
    ///
    /// Returns true if the self-test passes.
    pub fn self_test(&self) -> bool {
        todo!("Perform integrity self-test on the key store")
    }

    /// Get the number of stored keys.
    pub fn key_count(&self) -> usize {
        self.keys.len()
    }
}

/// Exercise 7: Secure memory wipe.
///
/// Perform a multi-pass wipe of a memory region:
/// 1. Fill with zeros
/// 2. Fill with 0xFF
/// 3. Fill with random data
/// 4. Fill with zeros again
///
/// This defeats data remanence in DRAM.
pub fn secure_wipe(data: &mut [u8]) {
    todo!("Multi-pass secure memory wipe")
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
        // Key should be zeroized
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
        store.tamper_detected(); // reaches max
        assert_eq!(store.state, SecurityState::Disabled);
        assert!(!store.store_key("new", b"data"));
        assert!(store.get_key("new").is_none());
    }

    #[test]
    fn test_recovery_from_tamper() {
        use ring::digest;
        let mut store = SecureKeyStore::new(3);
        store.tamper_detected();
        assert_eq!(store.state, SecurityState::TamperDetected);

        let recovery_secret = b"admin-recovery-secret";
        let recovery_code = digest::digest(&digest::SHA256, recovery_secret).as_ref().to_vec();

        assert!(store.recover(&recovery_code));
        assert_eq!(store.state, SecurityState::Normal);
        assert_eq!(store.tamper_count, 0);
    }

    #[test]
    fn test_cannot_recover_when_disabled() {
        let mut store = SecureKeyStore::new(1);
        store.tamper_detected(); // disabled
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
