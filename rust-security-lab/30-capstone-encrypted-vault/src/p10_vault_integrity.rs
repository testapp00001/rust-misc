//! # Lesson 10: Vault Integrity and Version Control
//!
//! ## The Integration Lesson
//!
//! This capstone lesson integrates everything from the previous 9 lessons into
//! a complete, working vault with integrity verification and version control.
//!
//! ## Vault Integrity
//!
//! Integrity goes beyond encryption. Even with AES-256-GCM's auth tag, we add
//! an additional HMAC layer over the entire vault file structure. This provides:
//!
//! - **Defense in depth**: Two independent integrity checks
//! - **Structural integrity**: HMAC covers metadata, not just data
//! - **Tamper evidence**: Any modification is detectable
//!
//! ## Version Control
//!
//! Every save increments a version counter and records a timestamp. This
//! provides an audit trail: when was the vault last modified? How many times
//! has it been saved? This helps detect unauthorized modifications.
//!
//! ## The Complete Vault
//!
//! ```text
//! VaultFile {
//!     magic: "ENCRYPTED_VAULT_V1\0",
//!     version: u64,
//!     salt: [u8; 32],
//!     nonce: [u8; 12],
//!     encrypted_data: Vec<u8>,    // AES-256-GCM(nonce, vault_json)
//!     hmac: [u8; 32],             // HMAC-SHA256 over all above
//! }
//! ```
//!
//! ## Attack Scenario
//!
//! An attacker modifies the vault file to add a backdoor entry. The HMAC
//! verification fails during the next unlock, alerting the user to tampering.
//! The version counter also reveals if the file was replaced entirely.

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Magic bytes identifying the vault file format.
pub const VAULT_MAGIC: &[u8; 20] = b"ENCRYPTED_VAULT_V1\0\0";

/// A complete vault file with integrity protection.
#[derive(Debug, Clone)]
pub struct VaultFile {
    /// Magic bytes for format identification
    pub magic: [u8; 20],
    /// Version counter (incremented on each save)
    pub version: u64,
    /// Salt for key derivation
    pub salt: Vec<u8>,
    /// Nonce for AES-256-GCM
    pub nonce: Vec<u8>,
    /// Encrypted vault data (includes auth tag)
    pub encrypted_data: Vec<u8>,
    /// HMAC-SHA256 over (magic || version || salt || nonce || encrypted_data)
    pub hmac: Vec<u8>,
}

/// Audit log entry for vault modifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Version number
    pub version: u64,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Action performed
    pub action: String,
}

/// Exercise 1: Create a new vault file from plaintext data.
///
/// 1. Generate salt and nonce
/// 2. Derive encryption key and HMAC key from passphrase + salt
/// 3. Encrypt the plaintext with AES-256-GCM
/// 4. Compute HMAC over the structural data
/// 5. Build the VaultFile
///
/// Hints:
/// - Use Argon2id for key derivation
/// - Use HKDF to derive separate enc_key and hmac_key
/// - Use AES-256-GCM for encryption
/// - Use HMAC-SHA256 for integrity
pub fn create_vault_file(
    plaintext: &[u8],
    passphrase: &str,
) -> Result<VaultFile, String> {
    todo!("Create a complete vault file")
}

/// Exercise 2: Open and verify a vault file.
///
/// 1. Verify HMAC integrity
/// 2. Derive keys from passphrase + salt
/// 3. Decrypt the vault data
/// 4. Return (plaintext, version)
///
/// Hints:
/// - Verify HMAC first (before deriving encryption key)
/// - Derive encryption key from passphrase + salt
/// - Decrypt with AES-256-GCM
pub fn open_vault_file(
    vault_file: &VaultFile,
    passphrase: &str,
) -> Result<(Vec<u8>, u64), String> {
    todo!("Open and verify a vault file")
}

/// Exercise 3: Save/update a vault file (incrementing version).
///
/// 1. Increment version
/// 2. Re-encrypt with new nonce
/// 3. Recompute HMAC
///
/// Hints:
/// - Bump version: `vault_file.version += 1`
/// - Generate new nonce
/// - Encrypt plaintext
/// - Recompute HMAC
pub fn save_vault_file(
    vault_file: &mut VaultFile,
    plaintext: &[u8],
    passphrase: &str,
) -> Result<(), String> {
    todo!("Save/update vault file")
}

/// Exercise 4: Serialize a VaultFile to bytes for storage.
///
/// Format: magic(20) || version(8, big-endian) || salt_len(4) || salt ||
///         nonce_len(4) || nonce || data_len(4) || encrypted_data || hmac(32)
///
/// Hints:
/// - Use `.to_be_bytes()` for version and lengths
/// - Concatenate all parts
pub fn serialize_vault_file(vault_file: &VaultFile) -> Vec<u8> {
    todo!("Serialize vault file to bytes")
}

/// Exercise 5: Deserialize a VaultFile from bytes.
///
/// Hints:
/// - Parse magic (first 20 bytes)
/// - Parse version (next 8 bytes as u64 big-endian)
/// - Parse salt_len, salt, nonce_len, nonce, data_len, encrypted_data, hmac
pub fn deserialize_vault_file(data: &[u8]) -> Result<VaultFile, String> {
    todo!("Deserialize vault file from bytes")
}

/// Exercise 6: Verify only the integrity (HMAC) of a vault file without decrypting.
///
/// Hints:
/// - Derive HMAC key from passphrase + salt
/// - Build the HMAC input: magic || version_be || salt || nonce || encrypted_data
/// - Verify HMAC
pub fn verify_vault_integrity(
    vault_file: &VaultFile,
    passphrase: &str,
) -> Result<bool, String> {
    todo!("Verify vault file integrity without decrypting")
}

/// Exercise 7: Compute HMAC-SHA256.
///
/// Hints:
/// - `let hmac_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);`
/// - `let tag = ring::hmac::sign(&hmac_key, data);`
/// - `Ok(tag.as_ref().to_vec())`
fn compute_hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Compute HMAC-SHA256 using ring::hmac")
}

/// Exercise 8: Verify HMAC-SHA256.
///
/// Hints:
/// - `let hmac_key = ring::hmac::Key::new(ring::hmac::HMAC_SHA256, key);`
/// - `ring::hmac::verify(&hmac_key, data, expected)`
fn verify_hmac_sha256(key: &[u8], data: &[u8], expected: &[u8]) -> Result<bool, String> {
    todo!("Verify HMAC-SHA256 using ring::hmac")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_plaintext() -> Vec<u8> {
        b"{\"version\":1,\"entries\":[{\"name\":\"GitHub\",\"username\":\"user\",\"password\":\"s3cret\"}]}".to_vec()
    }

    #[test]
    fn test_create_and_open_vault() {
        let plaintext = test_plaintext();
        let passphrase = "correcthorsebatterystaple";
        let vault = create_vault_file(&plaintext, passphrase).unwrap();
        let (decrypted, version) = open_vault_file(&vault, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
        assert_eq!(version, 1);
    }

    #[test]
    fn test_wrong_passphrase() {
        let vault = create_vault_file(b"secret", "correct").unwrap();
        assert!(open_vault_file(&vault, "wrong").is_err());
    }

    #[test]
    fn test_save_increments_version() {
        let mut vault = create_vault_file(b"data", "pass").unwrap();
        assert_eq!(vault.version, 1);
        save_vault_file(&mut vault, b"data", "pass").unwrap();
        assert_eq!(vault.version, 2);
        save_vault_file(&mut vault, b"data", "pass").unwrap();
        assert_eq!(vault.version, 3);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let vault = create_vault_file(&test_plaintext(), "passphrase").unwrap();
        let bytes = serialize_vault_file(&vault);
        let restored = deserialize_vault_file(&bytes).unwrap();
        assert_eq!(restored.version, vault.version);
        assert_eq!(restored.salt, vault.salt);
        assert_eq!(restored.encrypted_data, vault.encrypted_data);
    }

    #[test]
    fn test_serialize_deserialize_decrypt() {
        let plaintext = test_plaintext();
        let passphrase = "mypassword";
        let vault = create_vault_file(&plaintext, passphrase).unwrap();
        let bytes = serialize_vault_file(&vault);
        let restored = deserialize_vault_file(&bytes).unwrap();
        let (decrypted, _) = open_vault_file(&restored, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_integrity_tamper_detected() {
        let mut vault = create_vault_file(b"secret", "pass").unwrap();
        // Tamper with encrypted data
        if !vault.encrypted_data.is_empty() {
            vault.encrypted_data[0] ^= 0xFF;
        }
        assert!(!verify_vault_integrity(&vault, "pass").unwrap());
    }

    #[test]
    fn test_verify_integrity_valid() {
        let vault = create_vault_file(b"data", "passphrase").unwrap();
        assert!(verify_vault_integrity(&vault, "passphrase").unwrap());
    }

    #[test]
    fn test_deserialize_invalid_magic() {
        let data = vec![0u8; 100];
        assert!(deserialize_vault_file(&data).is_err());
    }

    #[test]
    fn test_save_preserves_data() {
        let plaintext = test_plaintext();
        let passphrase = "test123";
        let mut vault = create_vault_file(&plaintext, passphrase).unwrap();
        save_vault_file(&mut vault, &plaintext, passphrase).unwrap();
        let (decrypted, version) = open_vault_file(&vault, passphrase).unwrap();
        assert_eq!(decrypted, plaintext);
        assert_eq!(version, 2);
    }
}
