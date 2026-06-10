//! # Lesson 04: Encryption at Rest
//!
//! ## What Is Encryption at Rest?
//!
//! Encryption at rest protects data stored on disk — database files, tablespaces,
//! WAL logs, and temp files. If an attacker gains access to the raw disk (stolen
//! server, cloud storage breach, discarded hard drive), they see only ciphertext.
//!
//! ## Layers of Encryption at Rest
//!
//! ```text
//! Layer 1: Full-Disk Encryption (FDE)
//!   - OS-level: LUKS, BitLocker, FileVault
//!   - Protects against physical theft
//!   - Does NOT protect while the system is running (disk is decrypted)
//!
//! Layer 2: Database-Level Encryption (TDE)
//!   - Transparent Data Encryption (SQL Server, Oracle, PostgreSQL pgcrypto)
//!   - Encrypts data files, logs, backups automatically
//!   - Key managed by DBA or external KMS
//!
//! Layer 3: Application-Level Encryption
//!   - Application encrypts before sending to DB
//!   - Most granular control (per-field, per-record)
//!   - Covered in p03_field_level_encryption
//! ```
//!
//! ## Key Hierarchy
//!
//! ```text
//! Master Key (in KMS/HSM)
//!   └── Database Encryption Key (DEK)
//!         └── Tablespace Key
//!               └── Page-level encryption
//!
//! The master key never leaves the KMS. The DEK encrypts the actual data.
//! Rotating the master key re-encrypts only the DEK, not the entire database.
//! ```
//!
//! ## This Module
//!
//! We simulate a key hierarchy and demonstrate encrypting/decrypting data pages.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use sha2::{Digest, Sha256};

/// Simulate a Master Key stored in a Key Management System (KMS).
///
/// In production, this would be in an HSM or cloud KMS (AWS KMS, HashiCorp Vault).
/// Here we derive it deterministically from a passphrase for reproducibility.
///
/// Exercise: Derive a 32-byte master key from a passphrase using SHA-256.
///
/// Hints:
/// - Hash the passphrase with SHA-256
/// - Return the 32-byte hash as `Key<Aes256Gcm>`
pub fn derive_master_key(passphrase: &str) -> Key<Aes256Gcm> {
    todo!("Derive master key from passphrase using SHA-256")
}

/// Derive a Database Encryption Key (DEK) from the master key.
///
/// In a real system, the master key encrypts the DEK, and the DEK encrypts data.
/// Here we derive the DEK deterministically from the master key using a context label.
///
/// Exercise: Hash `master_key_bytes || ":" || context` to produce the DEK.
///
/// Hints:
/// - Use SHA-256 on `master_key.as_slice() || b":" || context.as_bytes()`
/// - Return as `Key<Aes256Gcm>`
pub fn derive_dek(master_key: &Key<Aes256Gcm>, context: &str) -> Key<Aes256Gcm> {
    todo!("Derive a DEK from master key and context label")
}

/// Encrypt a "data page" (simulated database page).
///
/// A database page is typically 4KB-16KB. We encrypt the entire page as one unit.
/// The nonce is prepended to the output.
///
/// Exercise: Encrypt the page data with the DEK.
///
/// Hints:
/// - Generate a 12-byte random nonce
/// - Encrypt with AES-256-GCM
/// - Return `nonce || ciphertext || tag`
pub fn encrypt_page(dek: &Key<Aes256Gcm>, page_data: &[u8]) -> Vec<u8> {
    todo!("Encrypt a database page with AES-256-GCM")
}

/// Decrypt a "data page".
///
/// Exercise: Extract the nonce and decrypt.
///
/// Hints:
/// - First 12 bytes = nonce
/// - Rest = ciphertext + tag
pub fn decrypt_page(dek: &Key<Aes256Gcm>, encrypted_page: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
    todo!("Decrypt a database page")
}

/// Simulate key rotation: re-encrypt data with a new DEK.
///
/// Key rotation is critical for long-term security. The master key encrypts the
/// DEK, so rotating the master key only requires re-encrypting the DEK, not the
/// entire database. But rotating the DEK requires re-encrypting all pages.
///
/// Exercise: Decrypt with old DEK, encrypt with new DEK.
///
/// Hints:
/// - Decrypt the page with `old_dek`
/// - Encrypt the result with `new_dek`
pub fn rotate_page_encryption(
    old_dek: &Key<Aes256Gcm>,
    new_dek: &Key<Aes256Gcm>,
    encrypted_page: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    todo!("Re-encrypt a page with a new DEK")
}

/// Demonstrate the full encryption-at-rest pipeline.
///
/// Exercise: Show the complete flow:
/// 1. Derive master key from passphrase
/// 2. Derive DEK from master key
/// 3. Encrypt a page
/// 4. Decrypt the page
/// 5. Rotate the DEK and re-encrypt
///
/// Hints:
/// - Return (encrypted_hex, decrypted_matches_original, rotation_works)
pub fn demonstrate_encryption_at_rest() -> (String, bool, bool) {
    todo!("Demonstrate full encryption-at-rest pipeline")
}

/// Simulate what an attacker sees when they steal the raw database files.
///
/// Exercise: Encrypt sample data and return the raw encrypted bytes.
/// The caller can verify these bytes are not readable as plaintext.
///
/// Hints:
/// - Use a known passphrase and sample data
/// - Encrypt and return the encrypted bytes
pub fn simulate_disk_theft(sample_data: &[u8]) -> Vec<u8> {
    todo!("Encrypt sample data as an attacker would find on disk")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_key_derivation() {
        let key = derive_master_key("my_secret_passphrase");
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_master_key_deterministic() {
        let k1 = derive_master_key("passphrase");
        let k2 = derive_master_key("passphrase");
        assert_eq!(k1.as_slice(), k2.as_slice());
    }

    #[test]
    fn test_dek_derivation_varies_by_context() {
        let master = derive_master_key("passphrase");
        let dek1 = derive_dek(&master, "users_table");
        let dek2 = derive_dek(&master, "orders_table");
        assert_ne!(dek1.as_slice(), dek2.as_slice(), "Different contexts must produce different DEKs");
    }

    #[test]
    fn test_page_encrypt_decrypt() {
        let master = derive_master_key("passphrase");
        let dek = derive_dek(&master, "test_table");
        let page = b"This is a simulated 4KB database page with row data...";

        let encrypted = encrypt_page(&dek, page);
        assert_ne!(encrypted[..page.len().min(encrypted.len())], page[..page.len().min(encrypted.len())]);

        let decrypted = decrypt_page(&dek, &encrypted).expect("Decryption failed");
        assert_eq!(decrypted, page);
    }

    #[test]
    fn test_page_wrong_dek_fails() {
        let master = derive_master_key("passphrase");
        let dek1 = derive_dek(&master, "table_a");
        let dek2 = derive_dek(&master, "table_b");
        let encrypted = encrypt_page(&dek1, b"secret data");

        assert!(decrypt_page(&dek2, &encrypted).is_err());
    }

    #[test]
    fn test_key_rotation() {
        let master = derive_master_key("passphrase");
        let old_dek = derive_dek(&master, "table_v1");
        let new_dek = derive_dek(&master, "table_v2");

        let page = b"Important data that needs re-encryption";
        let encrypted_old = encrypt_page(&old_dek, page);

        let encrypted_new = rotate_page_encryption(&old_dek, &new_dek, &encrypted_old)
            .expect("Rotation failed");

        let decrypted = decrypt_page(&new_dek, &encrypted_new).expect("Decryption after rotation failed");
        assert_eq!(decrypted, page);
    }

    #[test]
    fn test_disk_theft_returns_ciphertext() {
        let data = b"SSN: 123-45-6789, CC: 4111-1111-1111-1111";
        let stolen = simulate_disk_theft(data);
        // The stolen data should not contain the original plaintext
        let stolen_str = String::from_utf8_lossy(&stolen);
        assert!(!stolen_str.contains("123-45-6789"), "Encrypted disk data must not contain plaintext SSN");
    }

    #[test]
    fn test_demonstrate_pipeline() {
        let (encrypted_hex, decrypt_ok, rotation_ok) = demonstrate_encryption_at_rest();
        assert!(!encrypted_hex.is_empty());
        assert!(decrypt_ok, "Decryption must succeed");
        assert!(rotation_ok, "Rotation must succeed");
    }
}
