//! # Lesson 05: Secure Database Backups
//!
//! ## The Backup Problem
//!
//! Database backups are the most overlooked security risk. Your live database might
//! have encryption at rest, TLS connections, and row-level security — but if your
//! backup is an unencrypted SQL dump on a shared drive, none of that matters.
//!
//! ## Real-World Breaches via Backups
//!
//! - **2019 Capital One**: 100M records stolen from unencrypted S3 backups
//! - **2017 Uber**: 57M records from unencrypted S3 backup, paid $100K ransom
//! - **2015 Anthem**: 80M records from unencrypted database backup
//!
//! ## Backup Security Checklist
//!
//! ```text
//! [ ] Encrypt backup before writing to storage
//! [ ] Use a separate key from the database encryption key
//! [ ] Store encryption key in a KMS, NOT alongside the backup
//! [ ] Verify backup integrity (HMAC/signature)
//! [ ] Test restore procedure regularly
//! [ ] Securely delete old backups
//! [ ] Log all backup/restore operations
//! [ ] Restrict backup access to authorized personnel only
//! ```
//!
//! ## Architecture
//!
//! ```text
//! Database  →  Dump  →  Encrypt(AES-256-GCM)  →  HMAC(SHA-256)  →  Store
//!
//! Restore:
//! Storage  →  Verify HMAC  →  Decrypt  →  Import
//! ```

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use sha2::{Digest, Sha256};
use hmac::{Hmac, Mac};

type HmacSha256 = Hmac<Sha256>;

/// Encrypt a backup using AES-256-GCM.
///
/// Returns the encrypted backup blob with the nonce prepended.
///
/// Exercise: Encrypt the backup data.
///
/// Hints:
/// - Generate a fresh 12-byte nonce
/// - Encrypt with AES-256-GCM
/// - Return `nonce || ciphertext || tag`
pub fn encrypt_backup(key: &Key<Aes256Gcm>, backup_data: &[u8]) -> Vec<u8> {
    todo!("Encrypt backup data with AES-256-GCM")
}

/// Decrypt a backup.
///
/// Exercise: Extract nonce and decrypt.
///
/// Hints:
/// - First 12 bytes = nonce
/// - Rest = ciphertext + tag
pub fn decrypt_backup(key: &Key<Aes256Gcm>, encrypted_backup: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
    todo!("Decrypt backup data")
}

/// Compute an HMAC-SHA256 over the encrypted backup for integrity verification.
///
/// This ensures the backup was not tampered with in storage. The HMAC key should
/// be different from the encryption key.
///
/// Exercise: Compute HMAC-SHA256 of the data using the given key.
///
/// Hints:
/// - Create `HmacSha256::new_from_slice(hmac_key)`
/// - Feed in the data with `.update(data)`
/// - Finalize with `.finalize().into_bytes()`
pub fn compute_backup_hmac(hmac_key: &[u8], data: &[u8]) -> Vec<u8> {
    todo!("Compute HMAC-SHA256 over backup data")
}

/// Verify the HMAC of a backup.
///
/// Returns true if the HMAC matches (backup is intact), false otherwise.
/// Uses constant-time comparison to prevent timing attacks.
///
/// Exercise: Recompute the HMAC and compare.
///
/// Hints:
/// - Compute HMAC with the same key
/// - Use `hmac::Mac::verify_slice` for constant-time comparison
pub fn verify_backup_hmac(hmac_key: &[u8], data: &[u8], expected_hmac: &[u8]) -> bool {
    todo!("Verify backup HMAC using constant-time comparison")
}

/// Create a complete encrypted, integrity-protected backup package.
///
/// Returns (encrypted_data, hmac_hex).
///
/// Exercise: Encrypt the data, then compute HMAC over the encrypted data.
///
/// Hints:
/// - Encrypt with `encrypt_backup`
/// - Compute HMAC over the encrypted bytes
/// - Encode HMAC as hex string
pub fn create_secure_backup(
    encryption_key: &Key<Aes256Gcm>,
    hmac_key: &[u8],
    data: &[u8],
) -> (Vec<u8>, String) {
    todo!("Create encrypted + HMAC-protected backup")
}

/// Restore a backup: verify integrity, then decrypt.
///
/// Returns the plaintext data if both checks pass.
///
/// Exercise: Verify HMAC first, then decrypt.
///
/// Hints:
/// - Call `verify_backup_hmac` — return error if it fails
/// - Call `decrypt_backup` if HMAC is valid
pub fn restore_secure_backup(
    encryption_key: &Key<Aes256Gcm>,
    hmac_key: &[u8],
    encrypted_data: &[u8],
    expected_hmac_hex: &str,
) -> Result<Vec<u8>, String> {
    todo!("Verify HMAC then decrypt backup")
}

/// Demonstrate the full backup security pipeline.
///
/// Exercise: Create a backup, verify it, then show that tampering is detected.
///
/// Hints:
/// - Create a secure backup
/// - Restore it successfully
/// - Tamper with the encrypted data
/// - Show that restore fails on tampered data
pub fn demonstrate_backup_security() -> (bool, bool) {
    todo!("Return (restore_works, tampering_detected)")
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
