//! # Lesson 09: Secure Backup (Reference Solution)
//!
//! See the exercise file for full documentation on encrypted vault backup/restore.

use argon2::{Argon2, Algorithm, Params, Version};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;

/// Magic bytes identifying a vault backup file.
pub const BACKUP_MAGIC: &[u8; 16] = b"VAULT_BACKUP_V1\0";

fn derive_backup_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let params = Params::new(4096, 3, 1, None)
        .map_err(|e| format!("Invalid params: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(key)
}

fn derive_hmac_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    use hkdf::Hkdf;
    use sha2::Sha256;
    let master = derive_backup_key(passphrase, salt)?;
    let hk = Hkdf::<Sha256>::new(Some(salt), &master);
    let mut hmac_key = [0u8; 32];
    hk.expand(b"backup-hmac", &mut hmac_key)
        .map_err(|e| format!("HKDF failed: {}", e))?;
    Ok(hmac_key)
}

/// Compute HMAC-SHA256 over data using a key.
pub fn compute_hmac(key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let hmac_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    let tag = ring::hmac::sign(&hmac_key, data);
    Ok(tag.as_ref().to_vec())
}

/// Verify HMAC-SHA256.
pub fn verify_hmac(key: &[u8], data: &[u8], expected_hmac: &[u8]) -> Result<bool, String> {
    let hmac_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
    match ring::hmac::verify(&hmac_key, data, expected_hmac) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Create a backup of the encrypted vault.
/// Format: magic || salt || nonce || encrypted_data || hmac
pub fn create_backup(
    vault_data: &[u8],
    backup_passphrase: &str,
) -> Result<Vec<u8>, String> {
    // Generate salt and derive keys
    let mut salt = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let enc_key = derive_backup_key(backup_passphrase, &salt)?;
    let hmac_key = derive_hmac_key(backup_passphrase, &salt)?;

    // Encrypt
    let cipher = Aes256Gcm::new((&enc_key).into());
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, vault_data)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    // Build data for HMAC: magic || salt || nonce || ciphertext
    let mut data_for_hmac = Vec::new();
    data_for_hmac.extend_from_slice(BACKUP_MAGIC);
    data_for_hmac.extend_from_slice(&salt);
    data_for_hmac.extend_from_slice(&nonce_bytes);
    data_for_hmac.extend_from_slice(&ciphertext);
    let hmac = compute_hmac(&hmac_key, &data_for_hmac)?;

    // Final output: data_for_hmac || hmac
    data_for_hmac.extend_from_slice(&hmac);
    Ok(data_for_hmac)
}

/// Verify and restore a backup.
pub fn restore_backup(
    backup_data: &[u8],
    backup_passphrase: &str,
) -> Result<Vec<u8>, String> {
    // Minimum size: magic(16) + salt(32) + nonce(12) + tag(16) + hmac(32) = 108
    if backup_data.len() < 108 {
        return Err("Backup data too short".to_string());
    }

    // Check magic
    if &backup_data[..16] != BACKUP_MAGIC {
        return Err("Invalid backup magic bytes".to_string());
    }

    // Extract components
    let salt = &backup_data[16..48];
    let nonce_bytes = &backup_data[48..60];
    let encrypted = &backup_data[60..backup_data.len() - 32];
    let stored_hmac = &backup_data[backup_data.len() - 32..];

    // Verify HMAC
    let hmac_key = derive_hmac_key(backup_passphrase, salt)?;
    let data_for_hmac = &backup_data[..backup_data.len() - 32];
    if !verify_hmac(&hmac_key, data_for_hmac, stored_hmac)? {
        return Err("HMAC verification failed -- backup may be tampered".to_string());
    }

    // Decrypt
    let enc_key = derive_backup_key(backup_passphrase, salt)?;
    let cipher = Aes256Gcm::new((&enc_key).into());
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, encrypted)
        .map_err(|_| "Decryption failed -- wrong passphrase or corrupted data".to_string())
}

/// Extract backup metadata (salt) without decrypting.
pub fn read_backup_metadata(backup_data: &[u8]) -> Result<Vec<u8>, String> {
    if backup_data.len() < 48 {
        return Err("Backup data too short".to_string());
    }
    if &backup_data[..16] != BACKUP_MAGIC {
        return Err("Invalid backup magic bytes".to_string());
    }
    Ok(backup_data[16..48].to_vec())
}

/// Verify backup integrity without decrypting.
pub fn verify_backup_integrity(backup_data: &[u8], backup_passphrase: &str) -> Result<bool, String> {
    if backup_data.len() < 108 {
        return Err("Backup data too short".to_string());
    }
    if &backup_data[..16] != BACKUP_MAGIC {
        return Err("Invalid backup magic bytes".to_string());
    }
    let salt = &backup_data[16..48];
    let hmac_key = derive_hmac_key(backup_passphrase, salt)?;
    let data_for_hmac = &backup_data[..backup_data.len() - 32];
    let stored_hmac = &backup_data[backup_data.len() - 32..];
    verify_hmac(&hmac_key, data_for_hmac, stored_hmac)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_vault_data() -> Vec<u8> {
        b"{\"version\":1,\"entries\":[{\"name\":\"Test\",\"username\":\"user\",\"password\":\"pass\"}]}".to_vec()
    }

    #[test]
    fn test_create_and_restore_backup() {
        let vault = test_vault_data();
        let passphrase = "backup-secret-passphrase";
        let backup = create_backup(&vault, passphrase).unwrap();
        let restored = restore_backup(&backup, passphrase).unwrap();
        assert_eq!(restored, vault);
    }

    #[test]
    fn test_backup_wrong_passphrase() {
        let vault = test_vault_data();
        let backup = create_backup(&vault, "correct-passphrase").unwrap();
        assert!(restore_backup(&backup, "wrong-passphrase").is_err());
    }

    #[test]
    fn test_backup_starts_with_magic() {
        let backup = create_backup(b"data", "pass").unwrap();
        assert!(backup.starts_with(BACKUP_MAGIC));
    }

    #[test]
    fn test_backup_different_each_time() {
        let vault = test_vault_data();
        let b1 = create_backup(&vault, "pass").unwrap();
        let b2 = create_backup(&vault, "pass").unwrap();
        assert_ne!(b1, b2);
    }

    #[test]
    fn test_read_backup_metadata() {
        let backup = create_backup(b"data", "pass").unwrap();
        let salt = read_backup_metadata(&backup).unwrap();
        assert_eq!(salt.len(), 32);
    }

    #[test]
    fn test_verify_backup_integrity() {
        let passphrase = "backup-key";
        let backup = create_backup(b"vault data", passphrase).unwrap();
        assert!(verify_backup_integrity(&backup, passphrase).unwrap());
    }

    #[test]
    fn test_verify_backup_integrity_tampered() {
        let passphrase = "backup-key";
        let mut backup = create_backup(b"vault data", passphrase).unwrap();
        if backup.len() > 60 {
            backup[60] ^= 0xFF;
        }
        let result = verify_backup_integrity(&backup, passphrase);
        assert!(result.is_err() || !result.unwrap());
    }

    #[test]
    fn test_compute_and_verify_hmac() {
        let key = b"hmac-secret-key";
        let data = b"data to authenticate";
        let hmac = compute_hmac(key, data).unwrap();
        assert!(verify_hmac(key, data, &hmac).unwrap());
        assert!(!verify_hmac(b"wrong-key", data, &hmac).unwrap());
    }

    #[test]
    fn test_backup_tamper_detection() {
        let vault = test_vault_data();
        let mut backup = create_backup(&vault, "passphrase").unwrap();
        let len = backup.len();
        backup[len - 1] ^= 0x01;
        assert!(restore_backup(&backup, "passphrase").is_err());
    }
}
