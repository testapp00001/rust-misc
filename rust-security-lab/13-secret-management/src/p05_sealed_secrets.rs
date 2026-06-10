//! # Lesson 05: Sealed Secrets — Encrypting Secrets for Storage
//!
//! ## The Problem
//!
//! Kubernetes Secrets are base64-encoded, not encrypted. Anyone with access to
//! etcd (the Kubernetes backing store) or the API server can read them in plaintext.
//!
//! ```yaml
//! # This is NOT secure — it's just base64!
//! apiVersion: v1
//! kind: Secret
//! metadata:
//!   name: db-credentials
//! data:
//!   password: c3VwZXJfc2VjcmV0  # "super_secret" in base64
//! ```
//!
//! ## Sealed Secrets Pattern
//!
//! "Sealed secrets" encrypt secrets so they can be safely stored in version control.
//! Only the cluster (holding the private key) can decrypt them.
//!
//! ```text
//! Developer                          Cluster
//! ─────────                          ───────
//! 1. Encrypt secret with cluster's public key
//! 2. Commit sealed secret to git
//!                                    3. Pull sealed secret from git
//!                                    4. Decrypt with cluster's private key
//!                                    5. Create Kubernetes Secret
//! ```
//!
//! ## This Exercise
//!
//! We'll implement a sealed secrets system using AES-256-GCM encryption.
//! Secrets are encrypted with a shared key and can only be decrypted by
//! holders of that key.
//!
//! ## Attack: Secret at Rest
//!
//! Without encryption at rest:
//! - etcd dump contains all secrets in plaintext
//! - Backup files contain secrets
//! - Memory dumps of API server expose secrets
//! - Developers with cluster access can read all secrets
//!
//! ## Defense
//!
//! 1. Encrypt secrets before storing (envelope encryption)
//! 2. Use a KMS (Key Management Service) for the encryption key
//! 3. Rotate encryption keys periodically
//! 4. Implement sealed secrets for GitOps workflows

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// A sealed (encrypted) secret that can be safely stored in version control.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedSecret {
    /// The name of the secret.
    pub name: String,
    /// Base64-encoded ciphertext.
    pub ciphertext: String,
    /// Base64-encoded nonce (must be unique per encryption).
    pub nonce: String,
    /// The encryption algorithm used.
    pub algorithm: String,
}

/// Metadata about a sealed secret (without the actual secret value).
#[derive(Debug, Clone)]
pub struct SecretInfo {
    pub name: String,
    pub algorithm: String,
    pub ciphertext_len: usize,
}

/// Exercise 1: Seal (encrypt) a secret value.
///
/// Given a plaintext secret, encrypt it using AES-256-GCM with the provided key.
/// Return a SealedSecret containing the ciphertext and nonce, both base64-encoded.
///
/// The nonce should be 12 bytes (96 bits), randomly generated.
///
/// Hints:
/// - Create an `Aes256Gcm` instance from the key
/// - Generate a 12-byte random nonce
/// - Encrypt the plaintext with `cipher.encrypt(nonce, plaintext)`
/// - Base64-encode both the ciphertext and nonce
/// - Return a SealedSecret with algorithm "AES-256-GCM"
pub fn seal_secret(name: &str, plaintext: &[u8], key: &[u8; 32]) -> Result<SealedSecret, String> {
    todo!("Encrypt a secret using AES-256-GCM")
}

/// Exercise 2: Unseal (decrypt) a sealed secret.
///
/// Given a SealedSecret, decrypt it using AES-256-GCM with the provided key.
/// Return the plaintext bytes.
///
/// Hints:
/// - Base64-decode the ciphertext and nonce
/// - Create an `Aes256Gcm` instance from the key
/// - Decrypt with `cipher.decrypt(nonce, ciphertext)`
/// - Return the plaintext bytes
pub fn unseal_secret(sealed: &SealedSecret, key: &[u8; 32]) -> Result<Vec<u8>, String> {
    todo!("Decrypt a sealed secret")
}

/// Exercise 3: Generate a new encryption key.
///
/// Return a cryptographically secure random 256-bit (32-byte) key.
///
/// Hints:
/// - Create a `[0u8; 32]` array
/// - Fill it with random bytes using `rand::RngCore::fill_bytes`
/// - Return the key
pub fn generate_key() -> [u8; 32] {
    todo!("Generate a random 256-bit encryption key")
}

/// Exercise 4: Seal multiple secrets with the same key.
///
/// Given a list of (name, value) pairs, seal each one and return a Vec of SealedSecrets.
///
/// Hints:
/// - Iterate over the pairs
/// - Call `seal_secret` for each
/// - Collect the results, propagating errors
pub fn seal_many(secrets: &[(&str, &[u8])], key: &[u8; 32]) -> Result<Vec<SealedSecret>, String> {
    todo!("Seal multiple secrets at once")
}

/// Exercise 5: Get metadata about a sealed secret without decrypting it.
///
/// Return a SecretInfo with the name, algorithm, and ciphertext length.
///
/// Hints:
/// - Base64-decode the ciphertext to get its raw length
/// - Or just use the base64 string length as an approximation
/// - Create and return a SecretInfo
pub fn get_secret_info(sealed: &SealedSecret) -> Result<SecretInfo, String> {
    todo!("Get metadata about a sealed secret")
}

/// Exercise 6: Serialize a sealed secret to JSON.
///
/// Convert a SealedSecret to a JSON string for storage in a file or configmap.
///
/// Hints:
/// - Use `serde_json::to_string_pretty`
pub fn sealed_to_json(sealed: &SealedSecret) -> Result<String, String> {
    todo!("Serialize a sealed secret to JSON")
}

/// Exercise 7: Deserialize a sealed secret from JSON.
///
/// Parse a JSON string back into a SealedSecret.
///
/// Hints:
/// - Use `serde_json::from_str`
pub fn sealed_from_json(json: &str) -> Result<SealedSecret, String> {
    todo!("Deserialize a sealed secret from JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
        ]
    }

    #[test]
    fn test_seal_and_unseal_roundtrip() {
        let key = test_key();
        let plaintext = b"super_secret_password";
        let sealed = seal_secret("db_password", plaintext, &key).unwrap();

        let decrypted = unseal_secret(&sealed, &key).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_seal_different_nonces() {
        let key = test_key();
        let s1 = seal_secret("key", b"secret1", &key).unwrap();
        let s2 = seal_secret("key", b"secret1", &key).unwrap();
        // Different nonces means different ciphertext
        assert_ne!(s1.nonce, s2.nonce);
        assert_ne!(s1.ciphertext, s2.ciphertext);
    }

    #[test]
    fn test_unseal_wrong_key() {
        let key1 = test_key();
        let mut key2 = test_key();
        key2[0] ^= 0xff; // Flip bits to make a different key

        let sealed = seal_secret("key", b"secret", &key1).unwrap();
        let result = unseal_secret(&sealed, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_key_random() {
        let k1 = generate_key();
        let k2 = generate_key();
        assert_ne!(k1, k2, "Generated keys should be random");
        assert_eq!(k1.len(), 32);
    }

    #[test]
    fn test_seal_many() {
        let key = test_key();
        let secrets = vec![
            ("db_pass", b"password123" as &[u8]),
            ("api_key", b"sk-abcdef123456" as &[u8]),
        ];
        let sealed = seal_many(&secrets, &key).unwrap();
        assert_eq!(sealed.len(), 2);
        assert_eq!(sealed[0].name, "db_pass");
        assert_eq!(sealed[1].name, "api_key");
    }

    #[test]
    fn test_sealed_to_json_and_back() {
        let key = test_key();
        let sealed = seal_secret("test", b"value", &key).unwrap();
        let json = sealed_to_json(&sealed).unwrap();
        let parsed = sealed_from_json(&json).unwrap();

        assert_eq!(parsed.name, sealed.name);
        assert_eq!(parsed.ciphertext, sealed.ciphertext);
        assert_eq!(parsed.nonce, sealed.nonce);

        // Verify it still decrypts
        let decrypted = unseal_secret(&parsed, &key).unwrap();
        assert_eq!(decrypted, b"value");
    }

    #[test]
    fn test_get_secret_info() {
        let key = test_key();
        let sealed = seal_secret("my_secret", b"hello world", &key).unwrap();
        let info = get_secret_info(&sealed).unwrap();
        assert_eq!(info.name, "my_secret");
        assert_eq!(info.algorithm, "AES-256-GCM");
        assert!(info.ciphertext_len > 0);
    }
}
