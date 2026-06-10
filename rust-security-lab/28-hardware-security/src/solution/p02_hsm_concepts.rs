//! # Lesson 02: HSM Concepts — Hardware Security Module (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::{aead, hmac, rand as ring_rand};
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};

/// Key usage attributes that control what operations are permitted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyAttributes {
    pub extractable: bool,
    pub sign: bool,
    pub encrypt: bool,
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
    pub material: Vec<u8>,
}

/// Simulated HSM that stores keys and performs crypto operations.
#[derive(Debug, Default)]
pub struct SimulatedHsm {
    keys: Vec<KeyHandle>,
    key_counter: u64,
}

impl SimulatedHsm {
    /// Generate a new key inside the HSM with random material.
    pub fn generate_key(&mut self, key_len: usize, attributes: KeyAttributes) -> String {
        let rng = ring_rand::SystemRandom::new();
        let mut material = vec![0u8; key_len];
        rng.fill(&mut material).unwrap();

        self.key_counter += 1;
        let id = format!("hsm-key-{}", self.key_counter);

        self.keys.push(KeyHandle {
            id: id.clone(),
            attributes,
            material,
        });
        id
    }

    /// Import an external key into the HSM.
    pub fn import_key(&mut self, key_material: Vec<u8>, attributes: KeyAttributes) -> String {
        self.key_counter += 1;
        let id = format!("hsm-key-{}", self.key_counter);

        self.keys.push(KeyHandle {
            id: id.clone(),
            attributes,
            material: key_material,
        });
        id
    }

    /// Look up a key by ID.
    pub fn get_key(&self, key_id: &str) -> Option<&KeyHandle> {
        self.keys.iter().find(|k| k.id == key_id)
    }

    /// Sign data with HMAC-SHA256 (requires sign permission).
    pub fn sign(&self, key_id: &str, data: &[u8]) -> Option<Vec<u8>> {
        let key = self.get_key(key_id)?;
        if !key.attributes.sign {
            return None;
        }
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &key.material);
        Some(hmac::sign(&hmac_key, data).as_ref().to_vec())
    }

    /// Encrypt data with AES-256-GCM (requires encrypt permission).
    pub fn encrypt(&self, key_id: &str, plaintext: &[u8]) -> Option<Vec<u8>> {
        let key = self.get_key(key_id)?;
        if !key.attributes.encrypt {
            return None;
        }
        // AES-256-GCM requires a 32-byte key
        if key.material.len() != 32 {
            return None;
        }

        let sealing_key = aead::LessSafeKey::new(
            aead::UnboundKey::new(&aead::AES_256_GCM, &key.material).ok()?,
        );

        let rng = ring_rand::SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes).unwrap();
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        let mut in_out = plaintext.to_vec();
        sealing_key
            .seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut in_out)
            .ok()?;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&in_out);
        Some(result)
    }

    /// Export a key (only if marked extractable).
    pub fn export_key(&self, key_id: &str) -> Option<Vec<u8>> {
        let key = self.get_key(key_id)?;
        if !key.attributes.extractable {
            return None;
        }
        Some(key.material.clone())
    }

    /// Validate key policy — reject wrap+extractable combination (PKCS#11 abuse).
    pub fn validate_key_policy(attributes: &KeyAttributes) -> bool {
        !(attributes.wrap && attributes.extractable)
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
