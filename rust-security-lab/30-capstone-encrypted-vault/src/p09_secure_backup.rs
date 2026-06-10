//! # Lesson 09: Secure Backup
//!
//! ## Encrypted Vault Export and Import
//!
//! Backups protect against data loss (hardware failure, accidental deletion).
//! But a backup is only useful if it is both recoverable AND secure. An
//! unencrypted backup defeats the purpose of encrypting the vault.
//!
//! ## Backup Key Architecture
//!
//! We use a SEPARATE key for backups, derived from a separate passphrase.
//! This means:
//!
//! - Compromising the master passphrase does NOT compromise backups
//! - Compromising the backup passphrase does NOT compromise the live vault
//! - Backups can be stored in less-trusted locations (cloud storage)
//!
//! ## Backup Format
//!
//! ```text
//! +------------------------------------------+
//! | Magic bytes: "VAULT_BACKUP_V1\0" (16 B) |
//! | Salt (32 bytes)                          |
//! | Nonce (12 bytes)                         |
//! | Encrypted vault data                     |
//! | Authentication tag (16 bytes)            |
//! | HMAC-SHA256 over all above (32 bytes)    |
//! +------------------------------------------+
//! ```
//!
//! The HMAC provides an additional integrity layer beyond GCM's auth tag.
//! This is defense-in-depth: even if GCM had a weakness, the HMAC would
//! catch tampering.
//!
//! ## Recovery Process
//!
//! 1. User provides backup passphrase
//! 2. Derive backup key from passphrase + stored salt
//! 3. Verify HMAC integrity
//! 4. Decrypt vault data
//! 5. Import entries into the live vault (re-encrypted with master key)
//!
//! ## Attack Scenario
//!
//! An attacker gains access to the user's cloud storage and finds a backup
//! file. Without the backup passphrase, the file is useless -- it looks like
//! random bytes. Even if they had the master passphrase, it would not help
//! because backups use a separate key.

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Magic bytes identifying a vault backup file.
pub const BACKUP_MAGIC: &[u8; 16] = b"VAULT_BACKUP_V1\0";

/// Exercise 1: Create a backup of the encrypted vault.
///
/// Takes the current encrypted vault data and re-encrypts it with a
/// backup-specific key derived from `backup_passphrase`.
///
/// Format: magic || salt || nonce || encrypted_data || hmac
///
/// Hints:
/// - Generate a new 32-byte salt
/// - Derive backup key using Argon2id from backup_passphrase + salt
/// - Generate a 12-byte nonce
/// - Encrypt vault_data with AES-256-GCM using backup key
/// - Compute HMAC-SHA256 over (magic || salt || nonce || encrypted)
/// - Concatenate everything
pub fn create_backup(
    vault_data: &[u8],
    backup_passphrase: &str,
) -> Result<Vec<u8>, String> {
    todo!("Create an encrypted backup of the vault")
}

/// Exercise 2: Verify and restore a backup.
///
/// Verifies the HMAC, then decrypts the vault data using the backup passphrase.
///
/// Hints:
/// - Check magic bytes at the start
/// - Extract salt (bytes 16..48), nonce (bytes 48..60), encrypted (60..len-32), hmac (last 32)
/// - Verify HMAC over (magic || salt || nonce || encrypted)
/// - Derive backup key from passphrase + salt
/// - Decrypt with backup key
pub fn restore_backup(
    backup_data: &[u8],
    backup_passphrase: &str,
) -> Result<Vec<u8>, String> {
    todo!("Verify and restore a backup")
}

/// Exercise 3: Extract backup metadata without decrypting.
///
/// Returns (salt, creation_timestamp_if_available).
/// The salt can be used to verify the passphrase independently.
///
/// Hints:
/// - Check magic bytes
/// - Extract salt (bytes 16..48)
/// - Salt is enough for metadata
pub fn read_backup_metadata(backup_data: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Extract salt from backup without decrypting")
}

/// Exercise 4: Verify backup integrity without decrypting.
///
/// Hints:
/// - Check magic bytes
/// - Extract HMAC (last 32 bytes)
/// - Compute expected HMAC over everything except the HMAC itself
/// - Compare
pub fn verify_backup_integrity(backup_data: &[u8], backup_passphrase: &str) -> Result<bool, String> {
    todo!("Verify backup integrity")
}

/// Exercise 5: Compute HMAC-SHA256 over data using a key.
///
/// Hints:
/// - Use `ring::hmac`:
///   ```rust
///   let hmac_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);
///   let tag = ring::hmac::sign(&hmac_key, data);
///   tag.as_ref().to_vec()
///   ```
pub fn compute_hmac(key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Compute HMAC-SHA256")
}

/// Exercise 6: Verify HMAC-SHA256.
///
/// Hints:
/// - Use `ring::hmac::verify(&hmac_key, data, expected_hmac)`
pub fn verify_hmac(key: &[u8], data: &[u8], expected_hmac: &[u8]) -> Result<bool, String> {
    todo!("Verify HMAC-SHA256")
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
        assert_ne!(b1, b2, "Each backup should use a different salt/nonce");
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
        // Tamper with a byte in the encrypted portion
        if backup.len() > 60 {
            backup[60] ^= 0xFF;
        }
        // Integrity check should fail (either HMAC or passphrase mismatch)
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
        // Flip a bit
        let len = backup.len();
        backup[len - 1] ^= 0x01;
        assert!(restore_backup(&backup, "passphrase").is_err());
    }
}
