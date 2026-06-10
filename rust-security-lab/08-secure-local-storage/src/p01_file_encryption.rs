//! # Lesson 01: File Encryption with AES-256-GCM
//!
//! ## What is File-Level Encryption?
//!
//! File-level encryption encrypts individual files rather than entire disk volumes.
//! Each file is encrypted with its own key (or key+nonce combination), allowing
//! fine-grained access control.
//!
//! ## AES-256-GCM for Files
//!
//! AES-256-GCM provides **authenticated encryption**:
//! - **Confidentiality**: File contents are unreadable without the key
//! - **Integrity**: Any tampering with the encrypted file is detected
//! - **Authenticity**: The file is verified to have been encrypted by someone with the key
//!
//! ## File Format
//!
//! We use a simple format: `[nonce (12 bytes)] [ciphertext + tag]`
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │  Nonce (12 bytes)  │  Ciphertext + Auth Tag (16B)   │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! The nonce is prepended so the decryptor can extract it. The 16-byte authentication
//! tag is appended by AES-GCM automatically.
//!
//! ## Attack Scenario: Unencrypted Files
//!
//! If you store sensitive data unencrypted on disk:
//! 1. An attacker with physical access can read the disk directly
//! 2. Backup systems may store copies in less-secure locations
//! 3. File recovery tools can restore "deleted" files
//! 4. Memory dumps and swap files may contain plaintext fragments
//!
//! ## Security Considerations
//!
//! - **Nonce reuse** with the same key is catastrophic — it reveals XOR of two plaintexts
//! - **Key management** is the hard part — where do you store the key?
//! - **Large files** should be encrypted in chunks (streaming) to avoid memory issues
//! - **Metadata** (filename, size, timestamps) is NOT encrypted — consider this separately

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};

/// Exercise 1: Generate a random AES-256 encryption key.
///
/// This key will be used for all file encryption/decryption operations.
///
/// # Hints
/// - Use `Aes256Gcm::generate_key(&mut OsRng)` for a cryptographically secure key
/// - The key is 32 bytes (256 bits)
/// - In production, you'd store this key in a keychain or HSM
pub fn generate_file_key() -> Key<Aes256Gcm> {
    todo!("Generate a random AES-256 key for file encryption")
}

/// Exercise 2: Encrypt file contents using AES-256-GCM.
///
/// Returns a byte vector containing: `[nonce (12 bytes)] [ciphertext + tag]`
///
/// # Arguments
/// * `key` - The AES-256 encryption key
/// * `plaintext` - The file contents to encrypt
///
/// # Hints
/// - Create cipher: `Aes256Gcm::new(key)`
/// - Generate a fresh nonce: `Aes256Gcm::generate_nonce(&mut OsRng)`
/// - Encrypt: `cipher.encrypt(&nonce, plaintext)`
/// - Prepend the nonce bytes to the ciphertext
/// - The nonce must be saved — without it, the file cannot be decrypted
pub fn encrypt_file(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    todo!("Encrypt file contents with AES-256-GCM, prepending the nonce")
}

/// Exercise 3: Decrypt file contents using AES-256-GCM.
///
/// Expects input format: `[nonce (12 bytes)] [ciphertext + tag]`
///
/// # Arguments
/// * `key` - The AES-256 decryption key (same key used for encryption)
/// * `encrypted_data` - The encrypted file contents with prepended nonce
///
/// # Hints
/// - Extract the first 12 bytes as the nonce
/// - The rest is the ciphertext (includes the 16-byte auth tag)
/// - Create cipher and call `cipher.decrypt(&nonce, ciphertext)`
/// - Returns `Result<Vec<u8>, aes_gcm::Error>` — propagation with `?`
pub fn decrypt_file(
    key: &Key<Aes256Gcm>,
    encrypted_data: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    todo!("Extract nonce and decrypt file contents")
}

/// Exercise 4: Encrypt a file with associated data (AAD).
///
/// AAD (Additional Authenticated Data) is data that is authenticated but NOT encrypted.
/// This is useful for file metadata (filename, permissions) that must be readable
/// but protected from tampering.
///
/// # Arguments
/// * `key` - The AES-256 encryption key
/// * `plaintext` - The file contents to encrypt
/// * `aad` - Associated data (authenticated but not encrypted)
///
/// # Hints
/// - Use `cipher.encrypt(&nonce, Payload { msg: plaintext, aad })` for AAD
/// - The `Payload` type is from `aes_gcm::aead`
/// - The AAD must be provided during decryption to verify the tag
pub fn encrypt_file_with_aad(
    key: &Key<Aes256Gcm>,
    plaintext: &[u8],
    aad: &[u8],
) -> Vec<u8> {
    todo!("Encrypt file contents with associated authenticated data")
}

/// Exercise 5: Decrypt a file that was encrypted with AAD.
///
/// The AAD must match exactly what was used during encryption, or decryption fails.
///
/// # Hints
/// - Use `cipher.decrypt(&nonce, Payload { msg: ciphertext, aad })` for AAD
/// - Returns `Result<Vec<u8>, aes_gcm::Error>`
pub fn decrypt_file_with_aad(
    key: &Key<Aes256Gcm>,
    encrypted_data: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    todo!("Decrypt file contents with associated authenticated data")
}

/// Exercise 6: Encrypt a file and return the result as base64.
///
/// For easier storage and transmission, encode the encrypted bytes as base64.
///
/// # Hints
/// - Encrypt the file normally
/// - Encode with `base64::engine::general_purpose::STANDARD.encode(&encrypted)`
pub fn encrypt_file_to_base64(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> String {
    todo!("Encrypt file and encode as base64 string")
}

/// Exercise 7: Decrypt a file from a base64-encoded string.
///
/// # Hints
/// - Decode base64: `base64::engine::general_purpose::STANDARD.decode(&encoded)`
/// - Then decrypt the resulting bytes normally
pub fn decrypt_file_from_base64(
    key: &Key<Aes256Gcm>,
    base64_data: &str,
) -> Result<Vec<u8>, aes_gcm::Error> {
    todo!("Decode base64 and decrypt file contents")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_key() {
        let key = generate_file_key();
        assert_eq!(key.len(), 32, "AES-256 key must be 32 bytes");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_file_key();
        let plaintext = b"This is secret file content that needs protection.";
        let encrypted = encrypt_file(&key, plaintext);

        // Encrypted data should be larger (nonce + ciphertext + tag)
        assert!(encrypted.len() > plaintext.len());

        let decrypted = decrypt_file(&key, &encrypted).expect("Decryption should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_different_each_time() {
        let key = generate_file_key();
        let plaintext = b"Same content";

        let encrypted1 = encrypt_file(&key, plaintext);
        let encrypted2 = encrypt_file(&key, plaintext);

        // Different nonces → different ciphertext
        assert_ne!(encrypted1, encrypted2, "Same plaintext should produce different ciphertext");

        // Both should decrypt to the same plaintext
        let dec1 = decrypt_file(&key, &encrypted1).unwrap();
        let dec2 = decrypt_file(&key, &encrypted2).unwrap();
        assert_eq!(dec1, dec2);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = generate_file_key();
        let key2 = generate_file_key();
        let encrypted = encrypt_file(&key1, b"secret");

        let result = decrypt_file(&key2, &encrypted);
        assert!(result.is_err(), "Wrong key must fail decryption");
    }

    #[test]
    fn test_tampered_data_fails() {
        let key = generate_file_key();
        let mut encrypted = encrypt_file(&key, b"integrity test");

        // Tamper with the ciphertext (skip the 12-byte nonce)
        if encrypted.len() > 15 {
            encrypted[15] ^= 0xff;
        }

        let result = decrypt_file(&key, &encrypted);
        assert!(result.is_err(), "Tampered ciphertext must be detected");
    }

    #[test]
    fn test_aad_roundtrip() {
        let key = generate_file_key();
        let plaintext = b"file contents";
        let aad = b"metadata: secret.txt";

        let encrypted = encrypt_file_with_aad(&key, plaintext, aad);
        let decrypted = decrypt_file_with_aad(&key, &encrypted, aad).expect("AAD decrypt failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_wrong_aad_fails() {
        let key = generate_file_key();
        let plaintext = b"file contents";
        let aad = b"metadata: secret.txt";

        let encrypted = encrypt_file_with_aad(&key, plaintext, aad);

        let wrong_aad = b"metadata: tampered.txt";
        let result = decrypt_file_with_aad(&key, &encrypted, wrong_aad);
        assert!(result.is_err(), "Wrong AAD must fail decryption");
    }

    #[test]
    fn test_base64_roundtrip() {
        let key = generate_file_key();
        let plaintext = b"base64 test data";

        let encoded = encrypt_file_to_base64(&key, plaintext);
        let decrypted = decrypt_file_from_base64(&key, &encoded).expect("Base64 decrypt failed");
        assert_eq!(decrypted, plaintext);
    }
}
