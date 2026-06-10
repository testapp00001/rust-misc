//! # Lesson 07: Key Rotation
//!
//! ## Changing the Master Password
//!
//! Key rotation is the process of re-encrypting the vault with a new master key.
//! This is needed when:
//!
//! - The user changes their master password
//! - The user suspects their passphrase may have been compromised
//! - Periodic rotation as a security best practice
//!
//! ## The Rotation Process
//!
//! ```text
//! 1. Verify old passphrase (unlock vault)
//! 2. Decrypt vault with old key
//! 3. Derive new key from new passphrase (new salt)
//! 4. Re-encrypt vault with new key
//! 5. Store new salt + encrypted vault
//! 6. Zeroize old key material
//! ```
//!
//! ## Critical Security Requirement
//!
//! The old key MUST be zeroized after rotation. If an attacker had the old key
//! but not the new one, they should not be able to decrypt the vault after
//! rotation.
//!
//! ## Atomicity
//!
//! In a real system, the rotation must be atomic -- if the process crashes
//! between decrypt and re-encrypt, the vault should not be lost. This is
//! typically handled by writing the new vault to a temp file, then atomically
//! renaming it.
//!
//! ## Attack Scenario
//!
//! An attacker obtains the user's old passphrase (e.g., from a keylogger).
//! The user rotates to a new passphrase. Even with the old passphrase, the
//! attacker cannot decrypt the vault because it has been re-encrypted with
//! a new key derived from the new passphrase and a new salt.

use zeroize::Zeroize;

/// Metadata stored alongside the encrypted vault.
#[derive(Debug, Clone)]
pub struct VaultMetadata {
    /// Salt for Argon2id key derivation
    pub salt: Vec<u8>,
    /// Argon2id memory parameter (KB)
    pub memory_kb: u32,
    /// Argon2id iterations parameter
    pub iterations: u32,
    /// Argon2id parallelism parameter
    pub parallelism: u32,
    /// Number of key rotations performed
    pub rotation_count: u32,
    /// ISO 8601 timestamp of last rotation
    pub last_rotation: String,
}

/// The result of a key rotation operation.
#[derive(Debug)]
pub struct RotationResult {
    /// New metadata (with new salt and updated rotation count)
    pub new_metadata: VaultMetadata,
    /// New encrypted vault data
    pub new_encrypted_vault: Vec<u8>,
    /// Old key material (should be zeroized after use)
    pub old_key_material: Vec<u8>,
}

/// Exercise 1: Derive a new key set from a new passphrase.
///
/// Returns (new_salt, new_encryption_key).
///
/// Hints:
/// - Generate new salt (random 32 bytes)
/// - Use Argon2id to derive master secret from new passphrase + new salt
/// - Use HKDF to derive encryption key from master secret
pub fn derive_new_key(
    new_passphrase: &str,
    memory_kb: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<(Vec<u8>, [u8; 32]), String> {
    todo!("Derive a new key from a new passphrase")
}

/// Exercise 2: Rotate the vault key.
///
/// Takes the old encrypted vault, decrypts with old key, re-encrypts with new key.
/// Returns the RotationResult.
///
/// Hints:
/// - Decrypt vault with old key
/// - Generate new salt
/// - Derive new key from new passphrase + new salt
/// - Encrypt vault with new key
/// - Build new metadata with incremented rotation_count
/// - Zeroize old_key_material
pub fn rotate_key(
    old_encrypted_vault: &[u8],
    old_key: &[u8; 32],
    new_passphrase: &str,
    old_metadata: &VaultMetadata,
) -> Result<RotationResult, String> {
    todo!("Rotate the vault encryption key")
}

/// Exercise 3: Verify that rotation was successful by decrypting with the new key.
///
/// Hints:
/// - Decrypt the new encrypted vault with the new key
/// - Return true if decryption succeeds
pub fn verify_rotation(
    new_encrypted_vault: &[u8],
    new_key: &[u8; 32],
) -> Result<bool, String> {
    todo!("Verify that key rotation was successful")
}

/// Exercise 4: Build a VaultMetadata from parameters.
pub fn build_metadata(
    salt: Vec<u8>,
    memory_kb: u32,
    iterations: u32,
    parallelism: u32,
    rotation_count: u32,
) -> VaultMetadata {
    VaultMetadata {
        salt,
        memory_kb,
        iterations,
        parallelism,
        rotation_count,
        last_rotation: chrono::Utc::now().to_rfc3339(),
    }
}

/// Exercise 5: Zeroize key material in a RotationResult.
///
/// Hints:
/// - `result.old_key_material.zeroize()`
pub fn zeroize_rotation_result(result: &mut RotationResult) {
    todo!("Zeroize old key material")
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::{Algorithm, Argon2, Params, Version};
    use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
    use aes_gcm::aead::Aead;
    use rand::RngCore;

    fn derive_test_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
        let params = Params::new(4096, 3, 1, None).unwrap();
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut output = [0u8; 32];
        argon2.hash_password_into(passphrase.as_bytes(), salt, &mut output).unwrap();
        output
    }

    fn encrypt_test_data(key: &[u8; 32], data: &[u8]) -> Vec<u8> {
        let cipher = Aes256Gcm::new(key.into());
        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, data).unwrap();
        nonce_bytes.iter().chain(ciphertext.iter()).copied().collect()
    }

    #[test]
    fn test_derive_new_key() {
        let (salt, key) = derive_new_key("newpassword", 4096, 3, 1).unwrap();
        assert_eq!(salt.len(), 32);
        // Key should not be all zeros
        assert!(key.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_derive_new_key_deterministic() {
        // We can't test exact equality because salt is random,
        // but we can test that it works
        let (_, key1) = derive_new_key("password", 4096, 3, 1).unwrap();
        let (_, key2) = derive_new_key("password", 4096, 3, 1).unwrap();
        assert_ne!(key1, key2, "Different salts should produce different keys");
    }

    #[test]
    fn test_rotate_key() {
        let old_salt = vec![1u8; 32];
        let old_key = derive_test_key("oldpass", &old_salt);
        let plaintext = b"my vault data";
        let old_encrypted = encrypt_test_data(&old_key, plaintext);

        let old_metadata = build_metadata(old_salt, 4096, 3, 1, 0);
        let result = rotate_key(&old_encrypted, &old_key, "newpass", &old_metadata).unwrap();

        // Verify with new key
        assert!(verify_rotation(&result.new_encrypted_vault, &result.old_key_material).is_err()
            || {
                // The old_key_material in RotationResult should be zeroed by now
                // Let's verify with the actual new key derivation
                let new_key = derive_test_key("newpass", &result.new_metadata.salt);
                verify_rotation(&result.new_encrypted_vault, &new_key).unwrap()
            });
    }

    #[test]
    fn test_rotation_increments_count() {
        let old_salt = vec![2u8; 32];
        let old_key = derive_test_key("pass", &old_salt);
        let old_encrypted = encrypt_test_data(&old_key, b"data");
        let old_metadata = build_metadata(old_salt, 4096, 3, 1, 5);
        let result = rotate_key(&old_encrypted, &old_key, "newpass", &old_metadata).unwrap();
        assert_eq!(result.new_metadata.rotation_count, 6);
    }

    #[test]
    fn test_verify_rotation_success() {
        let key = [0x42u8; 32];
        let encrypted = encrypt_test_data(&key, b"vault");
        assert!(verify_rotation(&encrypted, &key).unwrap());
    }

    #[test]
    fn test_verify_rotation_wrong_key() {
        let key = [0x42u8; 32];
        let wrong_key = [0x99u8; 32];
        let encrypted = encrypt_test_data(&key, b"vault");
        assert!(!verify_rotation(&encrypted, &wrong_key).unwrap());
    }

    #[test]
    fn test_zeroize_rotation_result() {
        let mut result = RotationResult {
            new_metadata: build_metadata(vec![0u8; 32], 4096, 3, 1, 0),
            new_encrypted_vault: vec![],
            old_key_material: vec![0xAB; 32],
        };
        zeroize_rotation_result(&mut result);
        assert!(result.old_key_material.iter().all(|&b| b == 0));
    }
}
