//! # Lesson 03: Vault Decryption and Integrity Verification (Reference Solution)
//!
//! See the exercise file for full documentation on authenticated decryption with AAD.

use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Payload};
use rand::RngCore;

pub const NONCE_SIZE: usize = 12;

/// Decrypt vault data and verify integrity.
pub fn decrypt_and_verify(key: &[u8; 32], encrypted: &[u8]) -> Result<Vec<u8>, String> {
    if encrypted.len() < NONCE_SIZE + 16 {
        return Err("Decryption failed".to_string());
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "Decryption failed".to_string())
}

/// Encrypt with Associated Authenticated Data (AAD).
pub fn encrypt_with_aad(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let payload = Payload {
        msg: plaintext,
        aad,
    };
    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt with Associated Authenticated Data (AAD).
pub fn decrypt_with_aad(key: &[u8; 32], encrypted: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
    if encrypted.len() < NONCE_SIZE + 16 {
        return Err("Decryption failed".to_string());
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_SIZE);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    let payload = Payload {
        msg: ciphertext,
        aad,
    };
    cipher
        .decrypt(nonce, payload)
        .map_err(|_| "Decryption failed".to_string())
}

/// Encrypt with a header as AAD.
/// Format: header_len (4 bytes, big-endian) || header || nonce || ciphertext || tag
pub fn encrypt_with_header(key: &[u8; 32], plaintext: &[u8], header: &[u8]) -> Result<Vec<u8>, String> {
    let header_len = (header.len() as u32).to_be_bytes();
    let encrypted = encrypt_with_aad(key, plaintext, header)?;
    let mut result = Vec::with_capacity(4 + header.len() + encrypted.len());
    result.extend_from_slice(&header_len);
    result.extend_from_slice(header);
    result.extend_from_slice(&encrypted);
    Ok(result)
}

/// Decrypt vault data with header verification.
/// Returns (plaintext, header).
pub fn decrypt_with_header(key: &[u8; 32], data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), String> {
    if data.len() < 4 {
        return Err("Data too short".to_string());
    }
    let header_len = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + header_len {
        return Err("Data too short for header".to_string());
    }
    let header = &data[4..4 + header_len];
    let encrypted = &data[4 + header_len..];
    let plaintext = decrypt_with_aad(key, encrypted, header)?;
    Ok((plaintext, header.to_vec()))
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
        let mut nonce = [0u8; NONCE_SIZE];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
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
        let short_data = vec![0u8; 5];
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
        data[5] = b'X';
        assert!(decrypt_with_header(&key, &data).is_err());
    }
}
