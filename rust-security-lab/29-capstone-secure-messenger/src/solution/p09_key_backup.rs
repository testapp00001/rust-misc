//! # Lesson 09: Key Backup — Encrypted Key Backup (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}, Nonce, Key};
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Encrypted key backup stored on the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBackup {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub encrypted_blob: Vec<u8>,
    pub version: u32,
}

/// Decrypted key material recovered from backup.
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct KeyMaterial {
    pub identity_key: Vec<u8>,
    pub signed_prekey: Vec<u8>,
    pub session_keys: Vec<Vec<u8>>,
}

/// Derive a backup encryption key from a password and salt.
pub fn derive_backup_key(password: &[u8], salt: &[u8]) -> [u8; 32] {
    // SHA-256(password || salt)
    let mut hasher = Sha256::new();
    hasher.update(password);
    hasher.update(salt);
    let prk = hasher.finalize();

    // HKDF with info="key_backup_v1"
    let hk = Hkdf::<Sha256>::from_prk(&prk).expect("HKDF PRK too short");
    let mut key = [0u8; 32];
    hk.expand(b"key_backup_v1", &mut key).expect("HKDF expand failed");
    key
}

/// Encrypt key material for backup.
pub fn create_backup(password: &[u8], material: &KeyMaterial) -> KeyBackup {
    let salt: [u8; 16] = rand::random();
    let nonce_bytes: [u8; 12] = rand::random();

    let backup_key = derive_backup_key(password, &salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&backup_key));
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = serde_json::to_vec(material).expect("serialization failed");
    let encrypted_blob = cipher.encrypt(nonce, plaintext.as_ref())
        .expect("encryption failed");

    KeyBackup {
        salt: salt.to_vec(),
        nonce: nonce_bytes.to_vec(),
        encrypted_blob,
        version: 1,
    }
}

/// Decrypt a key backup. Returns None if decryption fails.
pub fn restore_backup(password: &[u8], backup: &KeyBackup) -> Option<KeyMaterial> {
    let backup_key = derive_backup_key(password, &backup.salt);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&backup_key));
    let nonce = Nonce::from_slice(&backup.nonce);

    let plaintext = cipher.decrypt(nonce, backup.encrypted_blob.as_ref()).ok()?;
    serde_json::from_slice(&plaintext).ok()
}

/// Verify backup structure without decrypting.
pub fn verify_backup_structure(backup: &KeyBackup) -> bool {
    backup.salt.len() == 16
        && backup.nonce.len() == 12
        && backup.version == 1
        && backup.encrypted_blob.len() > 16 // at least auth tag
}

/// Re-encrypt a backup with a new password.
pub fn change_backup_password(
    old_password: &[u8],
    new_password: &[u8],
    backup: &KeyBackup,
) -> Option<KeyBackup> {
    let material = restore_backup(old_password, backup)?;
    Some(create_backup(new_password, &material))
}

/// Add a new session key to an existing backup.
pub fn add_session_key(
    password: &[u8],
    backup: &KeyBackup,
    new_session_key: Vec<u8>,
) -> Option<KeyBackup> {
    let mut material = restore_backup(password, backup)?;
    material.session_keys.push(new_session_key);
    Some(create_backup(password, &material))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_material() -> KeyMaterial {
        KeyMaterial {
            identity_key: vec![0xAA; 32],
            signed_prekey: vec![0xBB; 32],
            session_keys: vec![vec![0xCC; 32], vec![0xDD; 32]],
        }
    }

    #[test]
    fn test_derive_backup_key_deterministic() {
        let key1 = derive_backup_key(b"password123", &[0x01; 16]);
        let key2 = derive_backup_key(b"password123", &[0x01; 16]);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_backup_key_different_passwords() {
        let key1 = derive_backup_key(b"password1", &[0x01; 16]);
        let key2 = derive_backup_key(b"password2", &[0x01; 16]);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_derive_backup_key_different_salts() {
        let key1 = derive_backup_key(b"password", &[0x01; 16]);
        let key2 = derive_backup_key(b"password", &[0x02; 16]);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_backup_restore_roundtrip() {
        let material = test_material();
        let backup = create_backup(b"strong_password!", &material);
        let restored = restore_backup(b"strong_password!", &backup).unwrap();
        assert_eq!(restored.identity_key, material.identity_key);
        assert_eq!(restored.signed_prekey, material.signed_prekey);
        assert_eq!(restored.session_keys, material.session_keys);
    }

    #[test]
    fn test_backup_wrong_password() {
        let material = test_material();
        let backup = create_backup(b"correct_password", &material);
        assert!(restore_backup(b"wrong_password", &backup).is_none());
    }

    #[test]
    fn test_backup_structure_valid() {
        let material = test_material();
        let backup = create_backup(b"password", &material);
        assert!(verify_backup_structure(&backup));
    }

    #[test]
    fn test_backup_structure_invalid_salt() {
        let material = test_material();
        let mut backup = create_backup(b"password", &material);
        backup.salt = vec![0; 8];
        assert!(!verify_backup_structure(&backup));
    }

    #[test]
    fn test_change_backup_password() {
        let material = test_material();
        let backup = create_backup(b"old_pass", &material);
        let new_backup = change_backup_password(b"old_pass", b"new_pass", &backup).unwrap();
        assert!(restore_backup(b"old_pass", &new_backup).is_none());
        let restored = restore_backup(b"new_pass", &new_backup).unwrap();
        assert_eq!(restored.identity_key, material.identity_key);
    }

    #[test]
    fn test_add_session_key() {
        let material = test_material();
        let backup = create_backup(b"password", &material);
        let updated = add_session_key(b"password", &backup, vec![0xEE; 32]).unwrap();
        let restored = restore_backup(b"password", &updated).unwrap();
        assert_eq!(restored.session_keys.len(), 3);
        assert_eq!(restored.session_keys[2], vec![0xEE; 32]);
    }
}
