//! # Lesson 03: Vault Decryption and Integrity Verification
//!
//! ## Decrypting the Vault
//!
//! This lesson focuses on the decryption side -- not just recovering plaintext,
//! but properly handling errors and verifying data integrity. In a real password
//! manager, decryption failures must be handled gracefully without leaking
//! information about why they failed.
//!
//! ## Authentication Tag Verification
//!
//! AES-256-GCM produces a 16-byte authentication tag alongside the ciphertext.
//! Before releasing any plaintext, the tag is verified against the ciphertext
//! and associated data (AAD). This catches:
//!
//! - **Bit-flip attacks**: Any modification to ciphertext causes tag mismatch
//! - **Truncation attacks**: Shortened ciphertext produces tag mismatch
//! - **Key mismatch**: Wrong key produces tag mismatch (not a "padding error")
//!
//! ## Associated Authenticated Data (AAD)
//!
//! GCM can optionally authenticate additional data that is NOT encrypted. This
//! is useful for binding the ciphertext to context -- for example, the vault
//! version number or user ID. The AAD is authenticated but sent in the clear.
//!
//! ## Error Handling
//!
//! Decryption can fail for several reasons:
//! - Wrong key (most common -- user mistyped passphrase)
//! - Corrupted ciphertext (storage failure, partial transfer)
//! - Tampered data (active attack)
//!
//! In ALL cases, the error message should be the same: "Decryption failed."
//! Never tell the attacker whether the key was wrong or the data was tampered.

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use rand::RngCore;

pub const NONCE_SIZE: usize = 12;

/// Exercise 1: Decrypt vault data and verify integrity.
///
/// Returns the plaintext only if the authentication tag is valid.
///
/// Hints:
/// - Check input length: must be at least NONCE_SIZE + TAG_SIZE (16)
/// - Split nonce and ciphertext: `input.split_at(NONCE_SIZE)`
/// - Create cipher: `Aes256Gcm::new(key.into())`
/// - Decrypt: `cipher.decrypt(nonce, ciphertext).map_err(|_| "Decryption failed".to_string())?`
pub fn decrypt_and_verify(key: &[u8; 32], encrypted: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Decrypt and verify authentication tag")
}

/// Exercise 2: Encrypt with Associated Authenticated Data (AAD).
///
/// The AAD is authenticated but not encrypted. It can be used to bind
/// ciphertext to a context (e.g., vault version, user ID).
///
/// Hints:
/// - Use `aes_gcm::aead::Payload` for AAD:
///   ```rust
///   let payload = Payload {
///       msg: plaintext,
///       aad: aad,
///   };
///   cipher.encrypt(nonce, payload)
///   ```
/// - Prepend nonce to result
pub fn encrypt_with_aad(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Encrypt with associated authenticated data")
}

/// Exercise 3: Decrypt with Associated Authenticated Data (AAD).
///
/// The AAD must match exactly what was used during encryption, or
/// authentication fails.
///
/// Hints:
/// - Use `Payload { msg: ciphertext, aad }` for decryption
/// - Same split logic as decrypt_and_verify
pub fn decrypt_with_aad(key: &[u8; 32], encrypted: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Decrypt with AAD verification")
}

/// Exercise 4: Encrypt a vault header (version + metadata) alongside the data.
///
/// The header (AAD) is authenticated but stored in plaintext.
/// Format: `header_len (4 bytes, big-endian) || header || nonce || ciphertext || tag`
///
/// Hints:
/// - Encode header length as 4 big-endian bytes
/// - Call `encrypt_with_aad` using the header as AAD
/// - Concatenate: header_len_bytes + header + encrypted
pub fn encrypt_with_header(key: &[u8; 32], plaintext: &[u8], header: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Encrypt with a header as AAD")
}

/// Exercise 5: Decrypt vault data with header verification.
///
/// Hints:
/// - Read first 4 bytes as header length (big-endian u32)
/// - Extract header and encrypted portion
/// - Call `decrypt_with_aad` with the header as AAD
pub fn decrypt_with_header(key: &[u8; 32], data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    todo!("Decrypt and return (plaintext, header)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [0xABu8; 32]
    }

    #[test]
    fn test_decrypt_and_verify_roundtrip() {
        let key = test_key();
        let nonce = {
            let mut n = [0u8; NONCE_SIZE];
            rand::rngs::OsRng.fill_bytes(&mut n);
            n
        };
        let cipher = Aes256Gcm::new((&key).into());
        let nonce_ref = Nonce::from_slice(&nonce);
        let ciphertext = cipher.encrypt(nonce_ref, b"hello vault".as_ref()).unwrap();
        let encrypted: Vec<u8> = nonce.iter().chain(ciphertext.iter()).copied().collect();
        let plaintext = decrypt_and_verify(&key, &encrypted).unwrap();
        assert_eq!(plaintext, b"hello vault");
    }

    #[test]
    fn test_decrypt_and_verify_wrong_key() {
        let key = test_key();
        let mut nonce = [0u8; NONCE_SIZE];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        let cipher = Aes256Gcm::new((&key).into());
        let nonce_ref = Nonce::from_slice(&nonce);
        let ciphertext = cipher.encrypt(nonce_ref, b"data".as_ref()).unwrap();
        let encrypted: Vec<u8> = nonce.iter().chain(ciphertext.iter()).copied().collect();
        let wrong_key = [0xFFu8; 32];
        assert!(decrypt_and_verify(&wrong_key, &encrypted).is_err());
    }

    #[test]
    fn test_decrypt_too_short() {
        let key = test_key();
        let short_data = vec![0u8; 5]; // Less than NONCE_SIZE
        assert!(decrypt_and_verify(&key, &short_data).is_err());
    }

    #[test]
    fn test_encrypt_with_aad_roundtrip() {
        let key = test_key();
        let aad = b"vault-v1";
        let plaintext = b"secret data";
        let encrypted = encrypt_with_aad(&key, plaintext, aad).unwrap();
        let decrypted = decrypt_with_aad(&key, &encrypted, aad).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aad_mismatch() {
        let key = test_key();
        let encrypted = encrypt_with_aad(&key, b"data", b"header-v1").unwrap();
        assert!(decrypt_with_aad(&key, &encrypted, b"header-v2").is_err());
    }

    #[test]
    fn test_encrypt_with_header_roundtrip() {
        let key = test_key();
        let header = b"vault-v2-user123";
        let plaintext = b"my passwords";
        let data = encrypt_with_header(&key, plaintext, header).unwrap();
        let (decrypted, recovered_header) = decrypt_with_header(&key, &data).unwrap();
        assert_eq!(decrypted, plaintext);
        assert_eq!(recovered_header, header);
    }

    #[test]
    fn test_header_tamper_detected() {
        let key = test_key();
        let header = b"vault-v1";
        let mut data = encrypt_with_header(&key, b"data", header).unwrap();
        // Tamper with header
        data[5] = b'X';
        assert!(decrypt_with_header(&key, &data).is_err());
    }
}
