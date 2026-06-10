//! # Lesson 09: Key Backup — Encrypted Key Backup
//!
//! ## Why Backup Keys?
//!
//! If a user loses their device, they lose their identity key and all session keys.
//! Without a backup, they must re-verify all contacts. An encrypted key backup
//! allows recovery without exposing keys to the server.
//!
//! ## Backup Architecture
//!
//! ```
//! User's password → PBKDF2/Argon2 → backup_key
//! backup_key + random_salt → AES-256-GCM → encrypted_key_material
//!
//! Server stores: { salt, nonce, encrypted_blob }
//! Server NEVER sees the plaintext keys.
//! ```
//!
//! ## Attack: Server Compromise
//!
//! If the server is compromised, the attacker gets the encrypted backup.
//! Without the user's password, they cannot decrypt it (assuming strong KDF).
//! **Defense**: Use a strong KDF (Argon2id) with high work factor.
//!
//! ## Attack: Weak Password
//!
//! If the user's password is weak, an attacker can brute-force the KDF.
//! **Defense**: Enforce minimum password strength; use Argon2id with
//! high memory and time costs.
//!
//! ## Attack: Key Rotation Without Backup Update
//!
//! If keys are rotated but the backup isn't updated, recovery gives stale keys.
//! **Defense**: Update the backup on every key rotation.

use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}, Nonce, Key};
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Encrypted key backup stored on the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBackup {
    /// Salt for key derivation (16 bytes).
    pub salt: Vec<u8>,
    /// Nonce for AES-256-GCM (12 bytes).
    pub nonce: Vec<u8>,
    /// Encrypted key material (includes auth tag).
    pub encrypted_blob: Vec<u8>,
    /// Version number for backup format.
    pub version: u32,
}

/// Decrypted key material recovered from backup.
#[derive(Debug, Clone, Zeroize)]
#[zeroize(drop)]
pub struct KeyMaterial {
    /// The identity key (32 bytes).
    pub identity_key: Vec<u8>,
    /// The signed prekey (32 bytes).
    pub signed_prekey: Vec<u8>,
    /// Session keys for active conversations.
    pub session_keys: Vec<Vec<u8>>,
}

/// Exercise 1: Derive a backup encryption key from a password and salt.
///
/// Uses HKDF-SHA256 with:
/// - IKM: SHA-256(password || salt)  (simplified — real systems use Argon2)
/// - Info: "key_backup_v1"
///
/// Returns a 32-byte key suitable for AES-256-GCM.
///
/// Hints:
/// - Concatenate password bytes and salt
/// - Hash with SHA-256
/// - Feed into HKDF with info="key_backup_v1"
/// - Extract 32 bytes
pub fn derive_backup_key(password: &[u8], salt: &[u8]) -> [u8; 32] {
    todo!("Derive backup encryption key from password and salt")
}

/// Exercise 2: Encrypt key material for backup.
///
/// 1. Generate a random 16-byte salt
/// 2. Generate a random 12-byte nonce
/// 3. Derive the backup key from password + salt
/// 4. Serialize key material to JSON
/// 5. Encrypt with AES-256-GCM
///
/// Hints:
/// - Use `rand::random::<[u8; 16]>()` for salt
/// - Serialize KeyMaterial with serde_json
/// - Encrypt the JSON bytes
pub fn create_backup(password: &[u8], material: &KeyMaterial) -> KeyBackup {
    todo!("Encrypt key material for backup")
}

/// Exercise 3: Decrypt a key backup.
///
/// 1. Derive the backup key from password + backup.salt
/// 2. Decrypt the encrypted blob with AES-256-GCM
/// 3. Deserialize the JSON to KeyMaterial
///
/// Returns None if decryption fails (wrong password or corrupted backup).
///
/// Hints:
/// - Derive key with derive_backup_key
/// - Try to decrypt — if auth tag fails, return None
/// - Try to deserialize — if JSON is invalid, return None
pub fn restore_backup(password: &[u8], backup: &KeyBackup) -> Option<KeyMaterial> {
    todo!("Decrypt and restore key material from backup")
}

/// Exercise 4: Verify a backup is valid without decrypting.
///
/// Checks structural validity: salt length, nonce length, version.
/// Does NOT verify the password (that requires decryption).
///
/// Hints:
/// - Check salt.len() == 16
/// - Check nonce.len() == 12
/// - Check version == 1
/// - Check encrypted_blob.len() > 16 (at least auth tag)
pub fn verify_backup_structure(backup: &KeyBackup) -> bool {
    todo!("Verify backup structure without decrypting")
}

/// Exercise 5: Re-encrypt a backup with a new password.
///
/// Decrypts with the old password, then re-encrypts with the new password.
/// Returns None if the old password is wrong.
///
/// Hints:
/// - Restore with old password
/// - If successful, create new backup with new password
pub fn change_backup_password(
    old_password: &[u8],
    new_password: &[u8],
    backup: &KeyBackup,
) -> Option<KeyBackup> {
    todo!("Re-encrypt backup with a new password")
}

/// Exercise 6: Add a new session key to an existing backup.
///
/// Decrypts the backup, adds the session key, re-encrypts.
/// Returns None if the password is wrong.
pub fn add_session_key(
    password: &[u8],
    backup: &KeyBackup,
    new_session_key: Vec<u8>,
) -> Option<KeyBackup> {
    todo!("Add a session key to an existing backup")
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
        backup.salt = vec![0; 8]; // wrong length
        assert!(!verify_backup_structure(&backup));
    }

    #[test]
    fn test_change_backup_password() {
        let material = test_material();
        let backup = create_backup(b"old_pass", &material);
        let new_backup = change_backup_password(b"old_pass", b"new_pass", &backup).unwrap();
        // Old password should no longer work
        assert!(restore_backup(b"old_pass", &new_backup).is_none());
        // New password should work
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
