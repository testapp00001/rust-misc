//! # Lesson 02: Vault Encryption
//!
//! ## Encrypting Vault Data with AES-256-GCM
//!
//! Once you have a 32-byte encryption key (from Lesson 01), you can encrypt the
//! vault's contents. We use AES-256-GCM -- an authenticated encryption algorithm
//! that provides both confidentiality and integrity in a single operation.
//!
//! ## AES-256-GCM
//!
//! AES-256-GCM (Galois/Counter Mode) is the industry standard for symmetric
//! authenticated encryption:
//!
//! - **AES-256**: The block cipher with a 256-bit key. Brute-forcing requires
//!   2^256 operations -- physically impossible.
//! - **GCM mode**: Combines CTR (counter mode) encryption with GHASH
//!   authentication. Produces ciphertext + an authentication tag.
//! - **Nonce**: A 96-bit number that must NEVER be reused with the same key.
//!   We generate it randomly for each encryption.
//!
//! ## Output Format
//!
//! The encrypted vault is stored as:
//! ```text
//! base64(nonce || ciphertext || auth_tag)
//! ```
//!
//! The nonce is 12 bytes, the auth tag is 16 bytes, and the ciphertext is the
//! same length as the plaintext.
//!
//! ## Why Authenticated Encryption?
//!
//! Without authentication, an attacker could modify the ciphertext (bit-flipping
//! attack) and the decryption would produce corrupted but valid-looking plaintext.
//! AES-256-GCM's auth tag prevents this: any modification to the ciphertext or
//! nonce causes verification to fail, and no plaintext is released.
//!
//! ## Attack Scenario
//!
//! An attacker intercepts the encrypted vault file. Without the key, they cannot:
//! - Decrypt the contents (AES-256 confidentiality)
//! - Modify the contents without detection (GCM authentication)
//! - Determine anything about the plaintext (semantic security, due to random nonce)

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rand::RngCore;

/// The nonce size for AES-256-GCM (96 bits = 12 bytes).
pub const NONCE_SIZE: usize = 12;

/// The authentication tag size for AES-256-GCM (128 bits = 16 bytes).
pub const TAG_SIZE: usize = 16;

/// Exercise 1: Generate a random 12-byte nonce for AES-256-GCM.
///
/// CRITICAL: Never reuse a nonce with the same key. Random generation is
/// safe for the volumes a password vault handles (birthday bound is ~2^48).
///
/// Hints:
/// - `let mut nonce_bytes = [0u8; NONCE_SIZE];`
/// - `rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);`
pub fn generate_nonce() -> [u8; NONCE_SIZE] {
    todo!("Generate a random nonce")
}

/// Exercise 2: Encrypt plaintext bytes using AES-256-GCM.
///
/// Returns the nonce prepended to the ciphertext: `nonce || ciphertext || tag`
///
/// Hints:
/// - Create cipher: `Aes256Gcm::new(key.into())`
/// - Create nonce: `Nonce::from_slice(&nonce_bytes)`
/// - Encrypt: `cipher.encrypt(nonce, plaintext).map_err(...)?`
/// - Prepend nonce to ciphertext: `nonce_bytes.iter().chain(ciphertext.iter()).copied().collect()`
pub fn encrypt_vault(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Encrypt data with AES-256-GCM")
}

/// Exercise 3: Decrypt vault data that was encrypted with `encrypt_vault`.
///
/// Input format: `nonce || ciphertext || tag`
///
/// Hints:
/// - Split: `let (nonce_bytes, ciphertext) = input.split_at(NONCE_SIZE);`
/// - Create cipher and nonce as in encrypt
/// - Decrypt: `cipher.decrypt(nonce, ciphertext).map_err(...)?`
pub fn decrypt_vault(key: &[u8; 32], encrypted: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Decrypt AES-256-GCM encrypted data")
}

/// Exercise 4: Encrypt and base64-encode (for JSON storage).
///
/// Hints:
/// - Call `encrypt_vault`
/// - Encode: `BASE64.encode(&encrypted)`
pub fn encrypt_to_base64(key: &[u8; 32], plaintext: &[u8]) -> Result<String, String> {
    todo!("Encrypt and base64-encode")
}

/// Exercise 5: Base64-decrypt and decrypt.
///
/// Hints:
/// - Decode: `BASE64.decode(encoded).map_err(...)?`
/// - Call `decrypt_vault`
pub fn decrypt_from_base64(key: &[u8; 32], encoded: &str) -> Result<Vec<u8>, String> {
    todo!("Base64-decode and decrypt")
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
        // Different nonces -> different ciphertext
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
        // Flip a bit in the ciphertext
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
