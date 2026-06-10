//! # Lesson 05: Secure Database Backups (Reference Solution)
//!
//! See the exercise file for full documentation.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

/// Encrypt backup data with AES-256-GCM.
///
/// Returns nonce || ciphertext || tag.
pub fn encrypt_backup(key: &Key<Aes256Gcm>, backup_data: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, backup_data)
        .expect("Encryption should not fail");

    let mut output = nonce.to_vec();
    output.extend_from_slice(&ciphertext);
    output
}

/// Decrypt backup data.
pub fn decrypt_backup(key: &Key<Aes256Gcm>, encrypted_backup: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
    if encrypted_backup.len() < 12 {
        return Err(aes_gcm::Error);
    }
    let (nonce_bytes, ciphertext) = encrypted_backup.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(key);
    cipher.decrypt(nonce, ciphertext)
}

/// Compute HMAC-SHA256 over backup data.
pub fn compute_backup_hmac(hmac_key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac =
        <HmacSha256 as Mac>::new_from_slice(hmac_key).expect("HMAC key creation should not fail");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

/// Verify backup HMAC using constant-time comparison.
pub fn verify_backup_hmac(hmac_key: &[u8], data: &[u8], expected_hmac: &[u8]) -> bool {
    let mut mac =
        <HmacSha256 as Mac>::new_from_slice(hmac_key).expect("HMAC key creation should not fail");
    mac.update(data);
    mac.verify_slice(expected_hmac).is_ok()
}

/// Create a complete encrypted + HMAC-protected backup.
pub fn create_secure_backup(
    encryption_key: &Key<Aes256Gcm>,
    hmac_key: &[u8],
    data: &[u8],
) -> (Vec<u8>, String) {
    let encrypted = encrypt_backup(encryption_key, data);
    let hmac = compute_backup_hmac(hmac_key, &encrypted);
    (encrypted, hex::encode(&hmac))
}

/// Restore a backup: verify HMAC, then decrypt.
pub fn restore_secure_backup(
    encryption_key: &Key<Aes256Gcm>,
    hmac_key: &[u8],
    encrypted_data: &[u8],
    expected_hmac_hex: &str,
) -> Result<Vec<u8>, String> {
    let expected_hmac =
        hex::decode(expected_hmac_hex).map_err(|_| "Invalid HMAC hex".to_string())?;

    if !verify_backup_hmac(hmac_key, encrypted_data, &expected_hmac) {
        return Err("HMAC verification failed — backup may be tampered".to_string());
    }

    decrypt_backup(encryption_key, encrypted_data)
        .map_err(|_| "Decryption failed".to_string())
}

/// Demonstrate the full backup security pipeline.
pub fn demonstrate_backup_security() -> (bool, bool) {
    let key_hash = Sha256::digest(b"backup_encryption_key");
    let encryption_key = Key::<Aes256Gcm>::clone_from_slice(&key_hash);
    let hmac_key = Sha256::digest(b"backup_hmac_key").to_vec();

    let data = b"Full database dump: CREATE TABLE users (...); INSERT INTO ...";

    // Create and restore a clean backup
    let (encrypted, hmac_hex) = create_secure_backup(&encryption_key, &hmac_key, data);
    let restored = restore_secure_backup(&encryption_key, &hmac_key, &encrypted, &hmac_hex);
    let restore_works = restored.map_or(false, |d| d == data);

    // Tamper with the encrypted data
    let mut tampered = encrypted.clone();
    if !tampered.is_empty() {
        tampered[20] ^= 0xff;
    }
    let tampered_result =
        restore_secure_backup(&encryption_key, &hmac_key, &tampered, &hmac_hex);
    let tampering_detected = tampered_result.is_err();

    (restore_works, tampering_detected)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Key<Aes256Gcm> {
        let hash = Sha256::digest(b"test_backup_key");
        Key::<Aes256Gcm>::clone_from_slice(&hash)
    }

    fn test_hmac_key() -> Vec<u8> {
        Sha256::digest(b"test_hmac_key").to_vec()
    }

    #[test]
    fn test_backup_encrypt_decrypt() {
        let key = test_key();
        let data = b"CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'alice');";
        let encrypted = encrypt_backup(&key, data);
        let decrypted = decrypt_backup(&key, &encrypted).expect("Decryption failed");
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_backup_encrypted_differs() {
        let key = test_key();
        let data = b"sensitive backup data";
        let encrypted = encrypt_backup(&key, data);
        assert_ne!(&encrypted[..data.len().min(encrypted.len())], &data[..data.len().min(encrypted.len())]);
    }

    #[test]
    fn test_hmac_verification() {
        let hmac_key = test_hmac_key();
        let data = b"backup data to protect";
        let hmac = compute_backup_hmac(&hmac_key, data);

        assert!(verify_backup_hmac(&hmac_key, data, &hmac));
    }

    #[test]
    fn test_hmac_detects_tampering() {
        let hmac_key = test_hmac_key();
        let data = b"backup data";
        let hmac = compute_backup_hmac(&hmac_key, data);

        let mut tampered_data = data.to_vec();
        tampered_data[0] ^= 0xff;
        assert!(!verify_backup_hmac(&hmac_key, &tampered_data, &hmac));
    }

    #[test]
    fn test_create_and_restore_backup() {
        let key = test_key();
        let hmac_key = test_hmac_key();
        let data = b"Full database dump with sensitive data";

        let (encrypted, hmac_hex) = create_secure_backup(&key, &hmac_key, data);
        let restored = restore_secure_backup(&key, &hmac_key, &encrypted, &hmac_hex)
            .expect("Restore failed");
        assert_eq!(restored, data);
    }

    #[test]
    fn test_restore_fails_on_tampered_data() {
        let key = test_key();
        let hmac_key = test_hmac_key();
        let data = b"important backup";

        let (mut encrypted, hmac_hex) = create_secure_backup(&key, &hmac_key, data);
        encrypted[20] ^= 0xff; // tamper

        let result = restore_secure_backup(&key, &hmac_key, &encrypted, &hmac_hex);
        assert!(result.is_err(), "Restore must fail on tampered backup");
    }

    #[test]
    fn test_restore_fails_wrong_key() {
        let key = test_key();
        let hmac_key = test_hmac_key();
        let data = b"backup data";

        let (encrypted, hmac_hex) = create_secure_backup(&key, &hmac_key, data);

        let wrong_key_hash = Sha256::digest(b"wrong_key");
        let wrong_key = Key::<Aes256Gcm>::clone_from_slice(&wrong_key_hash);
        let result = restore_secure_backup(&wrong_key, &hmac_key, &encrypted, &hmac_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_demonstrate_security() {
        let (restore_works, tampering_detected) = demonstrate_backup_security();
        assert!(restore_works, "Clean restore must work");
        assert!(tampering_detected, "Tampering must be detected");
    }
}
