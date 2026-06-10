//! # Lesson 01: File Encryption with AES-256-GCM (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    Aes256Gcm, Key, Nonce,
};

/// Generate a random AES-256 encryption key.
///
/// `Aes256Gcm::generate_key` uses the OS CSPRNG (via `OsRng`) to produce
/// 32 bytes of cryptographic randomness suitable for AES-256.
pub fn generate_file_key() -> Key<Aes256Gcm> {
    Aes256Gcm::generate_key(&mut OsRng)
}

/// Encrypt file contents using AES-256-GCM.
///
/// Format: `[nonce (12 bytes)] [ciphertext + auth tag (16 bytes)]`
///
/// The nonce is prepended so the decryptor can extract it. Each encryption
/// uses a fresh random nonce, ensuring that encrypting the same plaintext
/// twice produces different ciphertext.
pub fn encrypt_file(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext).expect("Encryption failed");
    let mut output = nonce.to_vec();
    output.extend_from_slice(&ciphertext);
    output
}

/// Decrypt file contents using AES-256-GCM.
///
/// Extracts the nonce from the first 12 bytes, then decrypts the remainder.
/// The authentication tag is verified automatically — any tampering causes
/// an error.
pub fn decrypt_file(
    key: &Key<Aes256Gcm>,
    encrypted_data: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    if encrypted_data.len() < 12 {
        return Err(aes_gcm::Error);
    }
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(key);
    cipher.decrypt(nonce, ciphertext)
}

/// Encrypt a file with associated authenticated data (AAD).
///
/// AAD is authenticated but NOT encrypted. Use it for metadata that must
/// be readable but protected from tampering (e.g., filenames, permissions).
///
/// The `Payload` struct bundles the message and AAD for the AEAD operation.
pub fn encrypt_file_with_aad(
    key: &Key<Aes256Gcm>,
    plaintext: &[u8],
    aad: &[u8],
) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, Payload { msg: plaintext, aad })
        .expect("Encryption with AAD failed");
    let mut output = nonce.to_vec();
    output.extend_from_slice(&ciphertext);
    output
}

/// Decrypt a file that was encrypted with AAD.
///
/// The AAD must match exactly — any difference causes tag verification to fail.
pub fn decrypt_file_with_aad(
    key: &Key<Aes256Gcm>,
    encrypted_data: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    if encrypted_data.len() < 12 {
        return Err(aes_gcm::Error);
    }
    let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(key);
    cipher.decrypt(nonce, Payload { msg: ciphertext, aad })
}

/// Encrypt a file and return the result as base64.
///
/// Base64 encoding makes binary encrypted data safe for text-based storage
/// (JSON, XML, email, etc.).
pub fn encrypt_file_to_base64(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> String {
    use base64::Engine;
    let encrypted = encrypt_file(key, plaintext);
    base64::engine::general_purpose::STANDARD.encode(&encrypted)
}

/// Decrypt a file from a base64-encoded string.
pub fn decrypt_file_from_base64(
    key: &Key<Aes256Gcm>,
    base64_data: &str,
) -> Result<Vec<u8>, aes_gcm::Error> {
    use base64::Engine;
    let encrypted = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|_| aes_gcm::Error)?;
    decrypt_file(key, &encrypted)
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
