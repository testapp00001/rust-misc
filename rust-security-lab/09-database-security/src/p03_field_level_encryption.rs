//! # Lesson 03: Field-Level Encryption
//!
//! ## What Is Field-Level Encryption?
//!
//! Instead of relying solely on database-level encryption, field-level encryption
//! encrypts individual sensitive columns (SSN, credit card, medical records) with
//! their own keys before the data reaches the database.
//!
//! ## Why Field-Level Encryption?
//!
//! ```text
//! DATABASE DUMP ATTACK:
//!   1. Attacker exploits SQL injection or backup theft
//!   2. Gets full table contents
//!   3. WITHOUT field encryption: reads SSNs, credit cards in plaintext
//!   4. WITH field encryption: reads ciphertext that is useless without the key
//! ```
//!
//! Even if the database admin is compromised, they see only ciphertext for the
//! encrypted columns. The encryption keys live in a separate key management system.
//!
//! ## Architecture
//!
//! ```text
//! Application  →  Encrypt(SSN)  →  Store ciphertext in DB
//! Application  ←  Decrypt(SSN)  ←  Read ciphertext from DB
//!
//! Key Management System (separate from DB)
//!   - Master key
//!   - Per-field or per-table data keys
//!   - Key rotation support
//! ```
//!
//! ## What We Build
//!
//! A `FieldEncryptor` that uses AES-256-GCM to encrypt/decrypt individual field
//! values, with a key derived from a master secret.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use sha2::{Digest, Sha256};

/// Derive an AES-256 key from a master secret and a field name.
///
/// In production, use HKDF or a dedicated KMS. Here we use SHA-256 for simplicity.
/// The field name acts as "context" to produce different keys per column.
///
/// Exercise: Hash `master_secret + ":" + field_name` to produce a 32-byte key.
///
/// Hints:
/// - Use `Sha256::new()`, feed in `master_secret.as_bytes()` then `b":"` then `field_name.as_bytes()`
/// - Convert the hash output to `Key<Aes256Gcm>` using `Key::from_slice()`
pub fn derive_field_key(master_secret: &[u8], field_name: &str) -> Key<Aes256Gcm> {
    todo!("Derive a per-field AES-256 key from master secret and field name")
}

/// Encrypt a single field value using AES-256-GCM.
///
/// Returns the ciphertext with the nonce prepended (nonce || ciphertext || tag).
/// This is a common pattern: store everything needed for decryption in one blob.
///
/// Exercise: Encrypt the plaintext field value.
///
/// Hints:
/// - Generate a random 12-byte nonce
/// - Encrypt with AES-256-GCM
/// - Prepend the nonce to the ciphertext: `nonce_bytes || ciphertext`
pub fn encrypt_field(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    todo!("Encrypt a field value, returning nonce || ciphertext || tag")
}

/// Decrypt a single field value.
///
/// The input is `nonce || ciphertext || tag` as produced by `encrypt_field`.
///
/// Exercise: Extract the nonce, then decrypt the rest.
///
/// Hints:
/// - First 12 bytes are the nonce
/// - Remaining bytes are ciphertext (includes 16-byte GCM tag at the end)
/// - Decrypt with `Aes256Gcm::new(&key).decrypt(&nonce, ciphertext)`
pub fn decrypt_field(key: &Key<Aes256Gcm>, encrypted: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
    todo!("Extract nonce and decrypt field value")
}

/// A simple record with both plaintext and encrypted fields.
///
/// Demonstrates how sensitive fields are stored as encrypted blobs while
/// non-sensitive fields remain readable.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserRecord {
    pub id: u64,
    pub username: String,
    /// Encrypted: stores base64(nonce || ciphertext || tag)
    pub email_encrypted: String,
    /// Encrypted: stores base64(nonce || ciphertext || tag)
    pub ssn_encrypted: String,
}

/// Encrypt the sensitive fields of a user record.
///
/// Exercise: Encrypt the email and SSN fields, encode as hex, and return
/// a `UserRecord` with the encrypted values.
///
/// Hints:
/// - Derive field keys for "email" and "ssn"
/// - Encrypt each field with `encrypt_field`
/// - Encode the encrypted bytes as hex strings
pub fn encrypt_user_record(
    master_secret: &[u8],
    id: u64,
    username: &str,
    email: &str,
    ssn: &str,
) -> UserRecord {
    todo!("Encrypt email and SSN fields, return UserRecord")
}

/// Decrypt the sensitive fields of a user record.
///
/// Exercise: Decode hex, decrypt, and return (email, ssn) as plaintext strings.
///
/// Hints:
/// - Decode the hex strings to bytes
/// - Derive the same field keys
/// - Decrypt each field
/// - Convert bytes to String
pub fn decrypt_user_record(
    master_secret: &[u8],
    record: &UserRecord,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    todo!("Decrypt email and SSN from UserRecord")
}

/// Demonstrate field-level encryption protecting against database dump.
///
/// Exercise: Create a record, encrypt it, then show that an attacker who
/// sees only the encrypted fields cannot read the SSN without the key.
///
/// Hints:
/// - Create an encrypted record
/// - Show that the encrypted SSN is not equal to the plaintext SSN
/// - Show that decryption with the correct key works
/// - Show that decryption with a wrong key fails
pub fn demonstrate_protection() -> (String, bool) {
    todo!("Return (encrypted_ssn_hex, decryption_with_wrong_key_fails)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_key_derivation() {
        let key1 = derive_field_key(b"master_secret", "email");
        let key2 = derive_field_key(b"master_secret", "ssn");
        assert_ne!(key1.as_slice(), key2.as_slice(), "Different fields must get different keys");
    }

    #[test]
    fn test_field_key_deterministic() {
        let key1 = derive_field_key(b"master", "email");
        let key2 = derive_field_key(b"master", "email");
        assert_eq!(key1.as_slice(), key2.as_slice(), "Same inputs must produce same key");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = derive_field_key(b"secret", "ssn");
        let plaintext = b"123-45-6789";
        let encrypted = encrypt_field(&key, plaintext);
        let decrypted = decrypt_field(&key, &encrypted).expect("Decryption failed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypted_differs_from_plaintext() {
        let key = derive_field_key(b"secret", "ssn");
        let plaintext = b"123-45-6789";
        let encrypted = encrypt_field(&key, plaintext);
        assert_ne!(encrypted, plaintext, "Ciphertext must differ from plaintext");
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = derive_field_key(b"secret1", "ssn");
        let key2 = derive_field_key(b"secret2", "ssn");
        let encrypted = encrypt_field(&key1, b"sensitive");
        assert!(decrypt_field(&key2, &encrypted).is_err(), "Wrong key must fail");
    }

    #[test]
    fn test_user_record_encrypt_decrypt() {
        let master = b"my_master_secret_key_here_12345";
        let record = encrypt_user_record(master, 1, "alice", "alice@example.com", "123-45-6789");

        assert_ne!(record.email_encrypted, "alice@example.com");
        assert_ne!(record.ssn_encrypted, "123-45-6789");

        let (email, ssn) = decrypt_user_record(master, &record).expect("Decryption failed");
        assert_eq!(email, "alice@example.com");
        assert_eq!(ssn, "123-45-6789");
    }

    #[test]
    fn test_user_record_wrong_master_fails() {
        let master = b"correct_master_key_1234567890ab";
        let record = encrypt_user_record(master, 1, "alice", "alice@example.com", "123-45-6789");

        let wrong_master = b"wrong_master_key_1234567890abc";
        let result = decrypt_user_record(wrong_master, &record);
        assert!(result.is_err(), "Wrong master key must fail to decrypt");
    }

    #[test]
    fn test_demonstrate_protection() {
        let (encrypted_ssn, wrong_key_fails) = demonstrate_protection();
        assert!(!encrypted_ssn.is_empty(), "Encrypted SSN should not be empty");
        assert!(wrong_key_fails, "Wrong key decryption should fail");
    }
}
