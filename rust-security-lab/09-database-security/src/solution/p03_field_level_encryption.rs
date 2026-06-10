//! # Lesson 03: Field-Level Encryption (Reference Solution)
//!
//! See the exercise file for full documentation.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use sha2::{Digest, Sha256};

/// Derive an AES-256 key from a master secret and field name.
///
/// Uses SHA-256(master_secret || ":" || field_name) to produce a 32-byte key.
/// Different field names produce different keys from the same master secret.
pub fn derive_field_key(master_secret: &[u8], field_name: &str) -> Key<Aes256Gcm> {
    let mut hasher = Sha256::new();
    hasher.update(master_secret);
    hasher.update(b":");
    hasher.update(field_name.as_bytes());
    let hash = hasher.finalize();
    Key::<Aes256Gcm>::clone_from_slice(&hash)
}

/// Encrypt a field value using AES-256-GCM.
///
/// Returns nonce || ciphertext || tag (the nonce is prepended for storage).
pub fn encrypt_field(key: &Key<Aes256Gcm>, plaintext: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .expect("Encryption should not fail");

    let mut output = nonce.to_vec();
    output.extend_from_slice(&ciphertext);
    output
}

/// Decrypt a field value.
///
/// Input format: nonce (12 bytes) || ciphertext || tag (16 bytes).
pub fn decrypt_field(key: &Key<Aes256Gcm>, encrypted: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
    if encrypted.len() < 12 {
        return Err(aes_gcm::Error);
    }
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(key);
    cipher.decrypt(nonce, ciphertext)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserRecord {
    pub id: u64,
    pub username: String,
    pub email_encrypted: String,
    pub ssn_encrypted: String,
}

/// Encrypt sensitive fields of a user record.
pub fn encrypt_user_record(
    master_secret: &[u8],
    id: u64,
    username: &str,
    email: &str,
    ssn: &str,
) -> UserRecord {
    let email_key = derive_field_key(master_secret, "email");
    let ssn_key = derive_field_key(master_secret, "ssn");

    let email_encrypted = encrypt_field(&email_key, email.as_bytes());
    let ssn_encrypted = encrypt_field(&ssn_key, ssn.as_bytes());

    UserRecord {
        id,
        username: username.to_string(),
        email_encrypted: hex::encode(&email_encrypted),
        ssn_encrypted: hex::encode(&ssn_encrypted),
    }
}

/// Decrypt sensitive fields of a user record.
pub fn decrypt_user_record(
    master_secret: &[u8],
    record: &UserRecord,
) -> Result<(String, String), String> {
    let email_key = derive_field_key(master_secret, "email");
    let ssn_key = derive_field_key(master_secret, "ssn");

    let email_bytes = hex::decode(&record.email_encrypted).map_err(|e| e.to_string())?;
    let ssn_bytes = hex::decode(&record.ssn_encrypted).map_err(|e| e.to_string())?;

    let email_plain = decrypt_field(&email_key, &email_bytes).map_err(|_| "Email decryption failed".to_string())?;
    let ssn_plain = decrypt_field(&ssn_key, &ssn_bytes).map_err(|_| "SSN decryption failed".to_string())?;

    Ok((
        String::from_utf8(email_plain).map_err(|e| e.to_string())?,
        String::from_utf8(ssn_plain).map_err(|e| e.to_string())?,
    ))
}

/// Demonstrate field-level encryption protecting against database dump.
pub fn demonstrate_protection() -> (String, bool) {
    let master = b"my_master_secret_key_here_12345";
    let record = encrypt_user_record(master, 1, "alice", "alice@example.com", "123-45-6789");

    let encrypted_ssn = record.ssn_encrypted.clone();

    let wrong_master = b"wrong_master_secret_key_123456";
    let wrong_key_result = decrypt_user_record(wrong_master, &record);
    let wrong_key_fails = wrong_key_result.is_err();

    (encrypted_ssn, wrong_key_fails)
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
