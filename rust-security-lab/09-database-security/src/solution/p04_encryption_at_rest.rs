//! # Lesson 04: Encryption at Rest (Reference Solution)
//!
//! See the exercise file for full documentation.

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use sha2::{Digest, Sha256};

/// Derive a master key from a passphrase using SHA-256.
pub fn derive_master_key(passphrase: &str) -> Key<Aes256Gcm> {
    let hash = Sha256::digest(passphrase.as_bytes());
    Key::<Aes256Gcm>::clone_from_slice(&hash)
}

/// Derive a Database Encryption Key (DEK) from master key and context.
pub fn derive_dek(master_key: &Key<Aes256Gcm>, context: &str) -> Key<Aes256Gcm> {
    let mut hasher = Sha256::new();
    hasher.update(master_key.as_slice());
    hasher.update(b":");
    hasher.update(context.as_bytes());
    let hash = hasher.finalize();
    Key::<Aes256Gcm>::clone_from_slice(&hash)
}

/// Encrypt a data page with AES-256-GCM.
///
/// Returns nonce || ciphertext || tag.
pub fn encrypt_page(dek: &Key<Aes256Gcm>, page_data: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(dek);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, page_data)
        .expect("Encryption should not fail");

    let mut output = nonce.to_vec();
    output.extend_from_slice(&ciphertext);
    output
}

/// Decrypt a data page.
pub fn decrypt_page(dek: &Key<Aes256Gcm>, encrypted_page: &[u8]) -> Result<Vec<u8>, aes_gcm::Error> {
    if encrypted_page.len() < 12 {
        return Err(aes_gcm::Error);
    }
    let (nonce_bytes, ciphertext) = encrypted_page.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    let cipher = Aes256Gcm::new(dek);
    cipher.decrypt(nonce, ciphertext)
}

/// Re-encrypt a page with a new DEK (key rotation).
pub fn rotate_page_encryption(
    old_dek: &Key<Aes256Gcm>,
    new_dek: &Key<Aes256Gcm>,
    encrypted_page: &[u8],
) -> Result<Vec<u8>, aes_gcm::Error> {
    let plaintext = decrypt_page(old_dek, encrypted_page)?;
    Ok(encrypt_page(new_dek, &plaintext))
}

/// Demonstrate the full encryption-at-rest pipeline.
pub fn demonstrate_encryption_at_rest() -> (String, bool, bool) {
    let master = derive_master_key("production_master_passphrase");
    let dek = derive_dek(&master, "users_table");

    let page_data = b"CREATE TABLE users (id INT, name TEXT); INSERT INTO users VALUES (1, 'alice');";
    let encrypted = encrypt_page(&dek, page_data);
    let encrypted_hex = hex::encode(&encrypted);

    let decrypted = decrypt_page(&dek, &encrypted).expect("Decryption failed");
    let decrypt_ok = decrypted == page_data;

    let new_dek = derive_dek(&master, "users_table_v2");
    let rotated = rotate_page_encryption(&dek, &new_dek, &encrypted).expect("Rotation failed");
    let decrypted_rotated = decrypt_page(&new_dek, &rotated).expect("Decryption after rotation failed");
    let rotation_ok = decrypted_rotated == page_data;

    (encrypted_hex, decrypt_ok, rotation_ok)
}

/// Simulate disk theft — encrypt data as it would appear on disk.
pub fn simulate_disk_theft(sample_data: &[u8]) -> Vec<u8> {
    let master = derive_master_key("production_master_passphrase");
    let dek = derive_dek(&master, "stolen_table");
    encrypt_page(&dek, sample_data)
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
