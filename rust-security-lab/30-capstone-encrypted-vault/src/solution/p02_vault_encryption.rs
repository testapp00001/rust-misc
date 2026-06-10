//! # Lesson 02: Vault Encryption (Reference Solution)
//!
//! See the exercise file for full documentation on AES-256-GCM encryption.

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rand::RngCore;

/// The nonce size for AES-256-GCM (96 bits = 12 bytes).
pub const NONCE_SIZE: usize = 12;

/// The authentication tag size for AES-256-GCM (128 bits = 16 bytes).
pub const TAG_SIZE: usize = 16;

/// Generate a random 12-byte nonce for AES-256-GCM.
pub fn generate_nonce() -> [u8; NONCE_SIZE] {
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    nonce_bytes
}

/// Encrypt plaintext bytes using AES-256-GCM.
/// Returns nonce || ciphertext || tag.
pub fn encrypt_vault(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt vault data that was encrypted with `encrypt_vault`.
pub fn decrypt_vault(key: &[u8; 32], encrypted: &[u8]) -> Result<Vec<u8>, String> {
    if encrypted.len() < NONCE_SIZE + TAG_SIZE {
        return Err("Encrypted data too short".to_string());
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Encrypt and base64-encode.
pub fn encrypt_to_base64(key: &[u8; 32], plaintext: &[u8]) -> Result<String, String> {
    let encrypted = encrypt_vault(key, plaintext)?;
    Ok(BASE64.encode(&encrypted))
}

/// Base64-decode and decrypt.
pub fn decrypt_from_base64(key: &[u8; 32], encoded: &str) -> Result<Vec<u8>, String> {
    let encrypted = BASE64
        .decode(encoded)
        .map_err(|e| format!("Base64 decode failed: {}", e))?;
    decrypt_vault(key, &encrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0x42u8; 32]
    }

    #[test]
    fn test_generate_nonce_unique() {
        let n1 = generate_nonce();
        let n2 = generate_nonce();
        assert_ne!(n1, n2, "Nonces should be unique");
        assert_eq!(n1.len(), NONCE_SIZE);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"my secret vault data";
        let encrypted = encrypt_vault(&key, plaintext).unwrap();
        let decrypted = decrypt_vault(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_different_nonces() {
        let key = test_key();
        let plaintext = b"same data";
        let e1 = encrypt_vault(&key, plaintext).unwrap();
        let e2 = encrypt_vault(&key, plaintext).unwrap();
        assert_ne!(e1, e2);
    }

    #[test]
    fn test_decrypt_wrong_key() {
        let key1 = test_key();
        let key2 = [0x99u8; 32];
        let encrypted = encrypt_vault(&key1, b"secret").unwrap();
        assert!(decrypt_vault(&key2, &encrypted).is_err());
    }

    #[test]
    fn test_decrypt_tampered_data() {
        let key = test_key();
        let mut encrypted = encrypt_vault(&key, b"secret").unwrap();
        if encrypted.len() > NONCE_SIZE + 1 {
            encrypted[NONCE_SIZE + 1] ^= 0xFF;
        }
        assert!(decrypt_vault(&key, &encrypted).is_err());
    }

    #[test]
    fn test_base64_roundtrip() {
        let key = test_key();
        let plaintext = b"vault entry data here";
        let encoded = encrypt_to_base64(&key, plaintext).unwrap();
        let decrypted = decrypt_from_base64(&key, &encoded).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_empty_data() {
        let key = test_key();
        let encrypted = encrypt_vault(&key, b"").unwrap();
        let decrypted = decrypt_vault(&key, &encrypted).unwrap();
        assert_eq!(decrypted, b"");
    }
}
