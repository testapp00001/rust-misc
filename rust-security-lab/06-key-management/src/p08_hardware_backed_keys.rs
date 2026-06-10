//! # Lesson 08: Hardware-Backed Key Storage
//!
//! ## What Are Hardware-Backed Keys?
//!
//! Hardware-backed keys are cryptographic keys that are generated, stored, and used
//! inside a secure hardware element — the key material never leaves the hardware.
//!
//! ```text
//! Software Key:                    Hardware-Backed Key:
//! +-----------+                    +-----------+
//! | Key in RAM| <--- exposed!      | App Code  |
//! +-----------+                    +-----------+
//! | encrypt() |                       | encrypt_request(data)
//! |           |                       v
//! +-----------+                    +-------------------+
//!                                  | Secure Element    |
//!                                  | Key generated HERE|
//!                                  | Key NEVER leaves  |
//!                                  | Returns ciphertext|
//!                                  +-------------------+
//! ```
//!
//! ## Hardware Options
//!
//! | Technology | Platform | API |
//! |------------|----------|-----|
//! | TPM 2.0 | Windows/Linux | TSS |
//! | Secure Enclave | macOS/iOS | Security.framework |
//! | Android Keystore | Android | KeyStore API |
//! | HSM | Server | PKCS#11 |
//! | YubiKey | USB | PIV/OpenPGP |
//!
//! ## Attack Scenario: Memory Dump
//!
//! An attacker with root access can dump process memory and extract software keys.
//! Hardware-backed keys are immune — the key is inside the secure element and cannot
//! be extracted, even with physical access to the device.
//!
//! ## For This Lesson
//!
//! We simulate a hardware security module (HSM) in software. The simulation enforces
//! the critical property: once a key is "generated" in the HSM, you can only REQUEST
//! operations (encrypt/decrypt/sign), never extract the raw key bytes.
//!
//! ## Security Notes
//!
//! - Hardware-backed keys provide the strongest key protection available
//! - Always verify the hardware attestation chain
//! - Hardware failures can cause key loss — ensure backup/recovery strategy
//! - Performance: hardware operations are slower than software (latency: 1-100ms)

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

/// A handle to a key stored in the (simulated) HSM.
/// The handle contains a reference ID, but NOT the key material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyHandle {
    pub key_id: String,
    pub algorithm: String,
    pub created_at: u64,
    pub can_export: bool,  // false = key is non-exportable (true hardware-backed)
}

/// Simulated HSM that stores keys securely.
#[derive(Debug)]
pub struct SimulatedHsm {
    /// Internal key storage — not accessible from outside
    keys: Vec<(String, Vec<u8>)>,
    /// Public handles — these are what callers see
    handles: Vec<KeyHandle>,
}

impl SimulatedHsm {
    /// Create a new simulated HSM.
    pub fn new() -> Self {
        SimulatedHsm {
            keys: Vec::new(),
            handles: Vec::new(),
        }
    }
}

/// Exercise 1: Generate a key inside the HSM.
///
/// Create a 32-byte random key inside the HSM and return a KeyHandle.
/// The key material should be stored internally, NOT in the handle.
///
/// Hints:
/// - Use `ring::SystemRandom` to generate key bytes
/// - Store the key in `self.keys` (the HSM's internal storage)
/// - Create a `KeyHandle` with `can_export: false` for true hardware-backed behavior
pub fn hsm_generate_key(hsm: &mut SimulatedHsm, key_id: &str, algorithm: &str, timestamp: u64) -> KeyHandle {
    todo!("Generate a key inside the HSM")
}

/// Exercise 2: Encrypt data using an HSM key.
///
/// The caller provides a KeyHandle and plaintext. The HSM looks up the
/// actual key internally and performs encryption. The raw key is never exposed.
///
/// For simplicity, use XOR-based encryption with the key (not production-grade,
/// but demonstrates the HSM pattern). In reality, you'd use ring's AES-GCM.
///
/// Hints:
/// - Find the key by key_id in hsm.keys
/// - XOR each plaintext byte with the corresponding key byte (cycling)
/// - Return (nonce, ciphertext)
pub fn hsm_encrypt(hsm: &SimulatedHsm, handle: &KeyHandle, plaintext: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Encrypt using HSM key (key never leaves HSM)")
}

/// Exercise 3: Decrypt data using an HSM key.
///
/// Reverse of hsm_encrypt. Look up the key by handle and decrypt.
///
/// Hints:
/// - XOR ciphertext with the key (same operation as encrypt for XOR)
pub fn hsm_decrypt(hsm: &SimulatedHsm, handle: &KeyHandle, nonce: &[u8], ciphertext: &[u8]) -> Vec<u8> {
    todo!("Decrypt using HSM key")
}

/// Exercise 4: Attempt to export a non-exportable key (should fail).
///
/// If the key was created with `can_export: false`, this should return an error.
/// This simulates the hardware guarantee that key material cannot be extracted.
///
/// Hints:
/// - Check `handle.can_export`
/// - If false, return Err("Key is non-exportable")
/// - If true, return the key bytes
pub fn hsm_export_key(hsm: &SimulatedHsm, handle: &KeyHandle) -> Result<Vec<u8>, String> {
    todo!("Attempt to export a non-exportable key")
}

/// Exercise 5: Generate a key with export enabled (for backup/escrow).
///
/// Some keys need to be exportable (e.g., for backup to another HSM).
/// Create a key with `can_export: true`.
pub fn hsm_generate_exportable_key(
    hsm: &mut SimulatedHsm,
    key_id: &str,
    algorithm: &str,
    timestamp: u64,
) -> KeyHandle {
    todo!("Generate an exportable key in the HSM")
}

/// Exercise 6: List all key handles in the HSM.
///
/// Return the handles (public metadata) without exposing key material.
pub fn hsm_list_keys(hsm: &SimulatedHsm) -> Vec<&KeyHandle> {
    todo!("List key handles without exposing key material")
}

/// Exercise 7: Demonstrate that HSM keys cannot be extracted.
///
/// Generate a non-exportable key, try to export it, verify it fails,
/// then verify encryption/decryption still works through the HSM API.
///
/// Returns (export_failed: bool, encrypt_decrypt_works: bool)
pub fn demonstrate_hsm_protection(hsm: &mut SimulatedHsm, timestamp: u64) -> (bool, bool) {
    todo!("Demonstrate HSM key protection")
}

/// Exercise 8: Delete a key from the HSM (secure erase).
///
/// Remove both the handle and the internal key material.
/// After deletion, encryption with that handle should fail.
pub fn hsm_delete_key(hsm: &mut SimulatedHsm, key_id: &str) -> Result<(), String> {
    todo!("Delete a key from the HSM")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_hsm() -> SimulatedHsm {
        SimulatedHsm::new()
    }

    #[test]
    fn test_generate_key_returns_handle() {
        let mut hsm = setup_hsm();
        let handle = hsm_generate_key(&mut hsm, "test-key", "AES-256", 1000);
        assert_eq!(handle.key_id, "test-key");
        assert!(!handle.can_export, "Default should be non-exportable");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut hsm = setup_hsm();
        let handle = hsm_generate_key(&mut hsm, "test-key", "AES-256", 1000);
        let plaintext = b"Hello, HSM!";
        let (nonce, ciphertext) = hsm_encrypt(&hsm, &handle, plaintext);
        let decrypted = hsm_decrypt(&hsm, &handle, &nonce, &ciphertext);
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_non_exportable_key_fails() {
        let mut hsm = setup_hsm();
        let handle = hsm_generate_key(&mut hsm, "secret", "AES-256", 1000);
        let result = hsm_export_key(&hsm, &handle);
        assert!(result.is_err(), "Non-exportable key should fail to export");
    }

    #[test]
    fn test_exportable_key_succeeds() {
        let mut hsm = setup_hsm();
        let handle = hsm_generate_exportable_key(&mut hsm, "backup-key", "AES-256", 1000);
        let result = hsm_export_key(&hsm, &handle);
        assert!(result.is_ok(), "Exportable key should succeed");
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_list_keys_shows_handles() {
        let mut hsm = setup_hsm();
        hsm_generate_key(&mut hsm, "key-1", "AES-256", 1000);
        hsm_generate_key(&mut hsm, "key-2", "AES-256", 1000);
        let handles = hsm_list_keys(&hsm);
        assert_eq!(handles.len(), 2);
    }

    #[test]
    fn test_delete_key() {
        let mut hsm = setup_hsm();
        hsm_generate_key(&mut hsm, "temp-key", "AES-256", 1000);
        hsm_delete_key(&mut hsm, "temp-key").unwrap();
        let handles = hsm_list_keys(&hsm);
        assert!(handles.is_empty(), "Key should be deleted");
    }

    #[test]
    fn test_hsm_protection_demonstration() {
        let mut hsm = setup_hsm();
        let (export_failed, encrypt_works) = demonstrate_hsm_protection(&mut hsm, 1000);
        assert!(export_failed, "Export of non-exportable key should fail");
        assert!(encrypt_works, "Encrypt/decrypt through HSM API should work");
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let mut hsm = setup_hsm();
        let handle = hsm_generate_key(&mut hsm, "test-key", "AES-256", 1000);
        let (_, ct1) = hsm_encrypt(&hsm, &handle, b"same");
        let (_, ct2) = hsm_encrypt(&hsm, &handle, b"same");
        // XOR with different random nonces should produce different ciphertexts
        assert_ne!(ct1, ct2, "Different encryptions should use different nonces");
    }
}
