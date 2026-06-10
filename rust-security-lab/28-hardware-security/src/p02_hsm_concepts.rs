//! # Lesson 02: HSM Concepts — Hardware Security Module
//!
//! ## What is an HSM?
//!
//! A Hardware Security Module (HSM) is a dedicated cryptographic device that:
//! - **Generates and stores keys** inside tamper-resistant hardware
//! - **Performs cryptographic operations** (sign, encrypt, decrypt) without exposing keys
//! - **Resists physical tampering** — zeroizes keys if intrusion detected
//! - **Provides high throughput** — dedicated crypto accelerators
//!
//! ## HSM vs TPM
//!
//! | Property | HSM | TPM |
//! |----------|-----|-----|
//! | Primary use | Key management, PKI, payments | Platform integrity |
//! | Key capacity | Millions | 24-150 |
//! | Performance | High (dedicated hardware) | Low (slow bus) |
//! | Form factor | PCIe card, network appliance | Chip on motherboard |
//! | FIPS level | Typically 140-2 Level 3+ | Typically Level 1-2 |
//!
//! ## Key Attributes in HSM
//!
//! Keys in an HSM have attributes that control their usage:
//! - `extractable`: Can the key leave the HSM? (Usually false for high-value keys)
//! - `sign`: Can this key be used for signing?
//! - `encrypt`: Can this key be used for encryption?
//! - `wrap`: Can this key wrap (encrypt) other keys?
//!
//! ## Attack: Key Extraction
//!
//! If an HSM key is marked as extractable, an attacker who compromises the host
//! can request the key to be exported. Defense: mark all high-value keys as
//! non-extractable. Operations happen inside the HSM.
//!
//! ## Attack: PKCS#11 Abuse
//!
//! Many HSMs expose PKCS#11 API. Misconfigurations (e.g., allowing wrap+decrypt
//! with the same key) can let an attacker wrap a key under a decrypt-enabled key,
//! then decrypt the wrapped key to extract it. Defense: enforce strict key policies.

use ring::{aead, digest, rand as ring_rand};
use serde::{Deserialize, Serialize};

/// Key usage attributes that control what operations are permitted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyAttributes {
    /// Can this key be exported from the HSM?
    pub extractable: bool,
    /// Can this key be used for signing/verification?
    pub sign: bool,
    /// Can this key be used for encryption/decryption?
    pub encrypt: bool,
    /// Can this key be used to wrap/unwrap other keys?
    pub wrap: bool,
}

impl Default for KeyAttributes {
    fn default() -> Self {
        Self {
            extractable: false,
            sign: false,
            encrypt: false,
            wrap: false,
        }
    }
}

/// A key handle stored inside the simulated HSM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyHandle {
    pub id: String,
    pub attributes: KeyAttributes,
    /// The raw key material (in a real HSM, this never leaves the device).
    pub material: Vec<u8>,
}

/// Simulated HSM that stores keys and performs crypto operations.
#[derive(Debug, Default)]
pub struct SimulatedHsm {
    /// Keys stored inside the HSM, indexed by ID.
    keys: Vec<KeyHandle>,
}

impl SimulatedHsm {
    /// Exercise 1: Generate a new key inside the HSM.
    ///
    /// Generate `key_len` random bytes using a CSPRNG and store them with
    /// the given attributes.
    ///
    /// Hints:
    /// - Use `ring::rand::SystemRandom::new()` and `ring::rand::secure_random_fill()`
    /// - Create a KeyHandle with a unique ID
    /// - Store it in the keys vector
    /// - Return the key ID
    pub fn generate_key(&mut self, key_len: usize, attributes: KeyAttributes) -> String {
        todo!("Generate a random key inside the HSM")
    }

    /// Exercise 2: Import an external key into the HSM.
    ///
    /// Store the provided key material with the given attributes.
    /// Return the key ID.
    pub fn import_key(&mut self, key_material: Vec<u8>, attributes: KeyAttributes) -> String {
        todo!("Import an external key into the HSM")
    }

    /// Exercise 3: Look up a key by ID and return a reference.
    pub fn get_key(&self, key_id: &str) -> Option<&KeyHandle> {
        todo!("Look up key by ID")
    }

    /// Exercise 4: Sign data using a key inside the HSM.
    ///
    /// The key must have the `sign` attribute set.
    /// Use HMAC-SHA256 for signing.
    ///
    /// Returns the signature, or None if the key doesn't exist or lacks sign permission.
    ///
    /// Hints:
    /// - Check that the key has `sign: true`
    /// - Use `ring::hmac` to compute HMAC-SHA256
    /// - Create key with `hmac::Key::new(hmac::HMAC_SHA256, &key.material)`
    /// - Sign with `hmac::sign(&key, data)`
    pub fn sign(&self, key_id: &str, data: &[u8]) -> Option<Vec<u8>> {
        todo!("Sign data with HSM key (check permissions)")
    }

    /// Exercise 5: Encrypt data using a key inside the HSM.
    ///
    /// The key must have the `encrypt` attribute set.
    /// Use AES-256-GCM via ring.
    ///
    /// Returns (ciphertext || nonce), or None if key lacks permission.
    ///
    /// Hints:
    /// - Check `encrypt: true`
    /// - Use `ring::aead::LessSafeKey` with `aead::AES_256_GCM`
    /// - Generate a random 12-byte nonce
    /// - Seal the data in-place
    pub fn encrypt(&self, key_id: &str, plaintext: &[u8]) -> Option<Vec<u8>> {
        todo!("Encrypt data with HSM key (check permissions)")
    }

    /// Exercise 6: Export a key (only if marked extractable).
    ///
    /// Returns the key material, or None if the key is not extractable.
    pub fn export_key(&self, key_id: &str) -> Option<Vec<u8>> {
        todo!("Export key only if extractable")
    }

    /// Exercise 7: Enforce key policy — validate that requested attributes are safe.
    ///
    /// A key should not be both `wrap` and `extractable` (PKCS#11 abuse vector).
    /// An attacker could wrap the key and extract the wrapped blob, then decrypt it.
    ///
    /// Returns true if the attributes are safe (no wrap+extractable combination).
    pub fn validate_key_policy(attributes: &KeyAttributes) -> bool {
        todo!("Reject unsafe key attribute combinations")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signing_attrs() -> KeyAttributes {
        KeyAttributes {
            sign: true,
            ..Default::default()
        }
    }

    fn encrypt_attrs() -> KeyAttributes {
        KeyAttributes {
            encrypt: true,
            ..Default::default()
        }
    }

    #[test]
    fn test_generate_key() {
        let mut hsm = SimulatedHsm::default();
        let key_id = hsm.generate_key(32, signing_attrs());
        assert!(!key_id.is_empty());
        let key = hsm.get_key(&key_id);
        assert!(key.is_some());
        assert_eq!(key.unwrap().material.len(), 32);
    }

    #[test]
    fn test_import_key() {
        let mut hsm = SimulatedHsm::default();
        let material = vec![0xAB; 32];
        let key_id = hsm.import_key(material.clone(), signing_attrs());
        let key = hsm.get_key(&key_id).unwrap();
        assert_eq!(key.material, material);
    }

    #[test]
    fn test_sign_with_permission() {
        let mut hsm = SimulatedHsm::default();
        let key_id = hsm.generate_key(32, signing_attrs());
        let sig = hsm.sign(&key_id, b"test data");
        assert!(sig.is_some(), "Signing should work with sign permission");
    }

    #[test]
    fn test_sign_without_permission() {
        let mut hsm = SimulatedHsm::default();
        let key_id = hsm.generate_key(32, encrypt_attrs());
        let sig = hsm.sign(&key_id, b"test data");
        assert!(sig.is_none(), "Signing should fail without sign permission");
    }

    #[test]
    fn test_encrypt_with_permission() {
        let mut hsm = SimulatedHsm::default();
        let key_id = hsm.generate_key(32, encrypt_attrs());
        let ct = hsm.encrypt(&key_id, b"secret message");
        assert!(ct.is_some());
    }

    #[test]
    fn test_encrypt_without_permission() {
        let mut hsm = SimulatedHsm::default();
        let key_id = hsm.generate_key(32, signing_attrs());
        let ct = hsm.encrypt(&key_id, b"secret message");
        assert!(ct.is_none(), "Encryption should fail without encrypt permission");
    }

    #[test]
    fn test_export_extractable() {
        let mut hsm = SimulatedHsm::default();
        let attrs = KeyAttributes {
            extractable: true,
            ..signing_attrs()
        };
        let key_id = hsm.generate_key(32, attrs);
        let exported = hsm.export_key(&key_id);
        assert!(exported.is_some(), "Extractable key should be exportable");
    }

    #[test]
    fn test_export_non_extractable() {
        let mut hsm = SimulatedHsm::default();
        let key_id = hsm.generate_key(32, signing_attrs());
        let exported = hsm.export_key(&key_id);
        assert!(exported.is_none(), "Non-extractable key should NOT be exportable");
    }

    #[test]
    fn test_policy_rejects_wrap_plus_extractable() {
        let unsafe_attrs = KeyAttributes {
            wrap: true,
            extractable: true,
            ..Default::default()
        };
        assert!(!SimulatedHsm::validate_key_policy(&unsafe_attrs));
    }

    #[test]
    fn test_policy_allows_safe_combinations() {
        let safe = KeyAttributes {
            sign: true,
            extractable: false,
            wrap: false,
            encrypt: false,
        };
        assert!(SimulatedHsm::validate_key_policy(&safe));
    }
}
