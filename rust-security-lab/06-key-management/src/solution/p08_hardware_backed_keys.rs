//! # Lesson 08: Hardware-Backed Key Storage (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

/// A handle to a key stored in the (simulated) HSM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyHandle {
    pub key_id: String,
    pub algorithm: String,
    pub created_at: u64,
    pub can_export: bool,
}

/// Simulated HSM that stores keys securely.
#[derive(Debug)]
pub struct SimulatedHsm {
    keys: Vec<(String, Vec<u8>)>,
    handles: Vec<KeyHandle>,
}

impl SimulatedHsm {
    pub fn new() -> Self {
        SimulatedHsm {
            keys: Vec::new(),
            handles: Vec::new(),
        }
    }
}

/// Generate a key inside the HSM (non-exportable by default).
///
/// The key material is stored internally and never exposed to the caller.
pub fn hsm_generate_key(
    hsm: &mut SimulatedHsm,
    key_id: &str,
    algorithm: &str,
    timestamp: u64,
) -> KeyHandle {
    let rng = SystemRandom::new();
    let mut key = vec![0u8; 32];
    rng.fill(&mut key).expect("Failed to generate key in HSM");

    hsm.keys.push((key_id.to_string(), key));

    let handle = KeyHandle {
        key_id: key_id.to_string(),
        algorithm: algorithm.to_string(),
        created_at: timestamp,
        can_export: false,
    };
    hsm.handles.push(handle.clone());
    handle
}

/// Encrypt using an HSM key (XOR-based for demonstration).
///
/// The key material never leaves the HSM — the operation happens inside.
pub fn hsm_encrypt(hsm: &SimulatedHsm, handle: &KeyHandle, plaintext: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let key = hsm
        .keys
        .iter()
        .find(|(id, _)| id == &handle.key_id)
        .map(|(_, k)| k)
        .expect("Key not found in HSM");

    let rng = SystemRandom::new();
    let mut nonce = vec![0u8; 12];
    rng.fill(&mut nonce).expect("Failed to generate nonce");

    let ciphertext: Vec<u8> = plaintext
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            let key_byte = key[i % key.len()];
            let nonce_byte = nonce[i % nonce.len()];
            b ^ key_byte ^ nonce_byte
        })
        .collect();

    (nonce, ciphertext)
}

/// Decrypt using an HSM key.
pub fn hsm_decrypt(hsm: &SimulatedHsm, handle: &KeyHandle, nonce: &[u8], ciphertext: &[u8]) -> Vec<u8> {
    let key = hsm
        .keys
        .iter()
        .find(|(id, _)| id == &handle.key_id)
        .map(|(_, k)| k)
        .expect("Key not found in HSM");

    ciphertext
        .iter()
        .enumerate()
        .map(|(i, &b)| {
            let key_byte = key[i % key.len()];
            let nonce_byte = nonce[i % nonce.len()];
            b ^ key_byte ^ nonce_byte
        })
        .collect()
}

/// Attempt to export a key from the HSM.
///
/// Fails for non-exportable keys (the default).
pub fn hsm_export_key(hsm: &SimulatedHsm, handle: &KeyHandle) -> Result<Vec<u8>, String> {
    if !handle.can_export {
        return Err(format!("Key {} is non-exportable", handle.key_id));
    }
    hsm.keys
        .iter()
        .find(|(id, _)| id == &handle.key_id)
        .map(|(_, k)| k.clone())
        .ok_or_else(|| format!("Key {} not found", handle.key_id))
}

/// Generate an exportable key (for backup/escrow scenarios).
pub fn hsm_generate_exportable_key(
    hsm: &mut SimulatedHsm,
    key_id: &str,
    algorithm: &str,
    timestamp: u64,
) -> KeyHandle {
    let rng = SystemRandom::new();
    let mut key = vec![0u8; 32];
    rng.fill(&mut key).expect("Failed to generate key in HSM");

    hsm.keys.push((key_id.to_string(), key));

    let handle = KeyHandle {
        key_id: key_id.to_string(),
        algorithm: algorithm.to_string(),
        created_at: timestamp,
        can_export: true,
    };
    hsm.handles.push(handle.clone());
    handle
}

/// List all key handles (public metadata only).
pub fn hsm_list_keys(hsm: &SimulatedHsm) -> Vec<&KeyHandle> {
    hsm.handles.iter().collect()
}

/// Demonstrate HSM key protection.
///
/// Returns (export_failed, encrypt_decrypt_works).
pub fn demonstrate_hsm_protection(hsm: &mut SimulatedHsm, timestamp: u64) -> (bool, bool) {
    let handle = hsm_generate_key(hsm, "protected-key", "AES-256-GCM", timestamp);
    let export_failed = hsm_export_key(hsm, &handle).is_err();

    let plaintext = b"test data";
    let (nonce, ciphertext) = hsm_encrypt(hsm, &handle, plaintext);
    let decrypted = hsm_decrypt(hsm, &handle, &nonce, &ciphertext);
    let encrypt_decrypt_works = decrypted == plaintext;

    (export_failed, encrypt_decrypt_works)
}

/// Delete a key from the HSM (secure erase).
pub fn hsm_delete_key(hsm: &mut SimulatedHsm, key_id: &str) -> Result<(), String> {
    let handle_idx = hsm
        .handles
        .iter()
        .position(|h| h.key_id == key_id)
        .ok_or_else(|| format!("Key handle {} not found", key_id))?;
    hsm.handles.remove(handle_idx);

    let key_idx = hsm
        .keys
        .iter()
        .position(|(id, _)| id == key_id)
        .ok_or_else(|| format!("Key {} not found in storage", key_id))?;
    hsm.keys.remove(key_idx);

    Ok(())
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
        assert!(!handle.can_export);
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
        assert!(result.is_err());
    }

    #[test]
    fn test_exportable_key_succeeds() {
        let mut hsm = setup_hsm();
        let handle = hsm_generate_exportable_key(&mut hsm, "backup-key", "AES-256", 1000);
        let result = hsm_export_key(&hsm, &handle);
        assert!(result.is_ok());
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
        assert!(handles.is_empty());
    }

    #[test]
    fn test_hsm_protection_demonstration() {
        let mut hsm = setup_hsm();
        let (export_failed, encrypt_works) = demonstrate_hsm_protection(&mut hsm, 1000);
        assert!(export_failed);
        assert!(encrypt_works);
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        let mut hsm = setup_hsm();
        let handle = hsm_generate_key(&mut hsm, "test-key", "AES-256", 1000);
        let (_, ct1) = hsm_encrypt(&hsm, &handle, b"same");
        let (_, ct2) = hsm_encrypt(&hsm, &handle, b"same");
        assert_ne!(ct1, ct2);
    }
}
