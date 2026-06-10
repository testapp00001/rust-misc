//! # Lesson 07: Key Rotation (Reference Solution)
//!
//! See the exercise file for full documentation on key rotation.

use zeroize::Zeroize;
use argon2::{Argon2, Algorithm, Params, Version};
use hkdf::Hkdf;
use sha2::Sha256;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::Aead;
use rand::RngCore;

/// Metadata stored alongside the encrypted vault.
#[derive(Debug, Clone)]
pub struct VaultMetadata {
    pub salt: Vec<u8>,
    pub memory_kb: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub rotation_count: u32,
    pub last_rotation: String,
}

/// The result of a key rotation operation.
#[derive(Debug)]
pub struct RotationResult {
    pub new_metadata: VaultMetadata,
    pub new_encrypted_vault: Vec<u8>,
    pub old_key_material: Vec<u8>,
}

/// Derive a new key set from a new passphrase.
pub fn derive_new_key(
    new_passphrase: &str,
    memory_kb: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<(Vec<u8>, [u8; 32]), String> {
    let mut salt = vec![0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    let params = Params::new(memory_kb, iterations, parallelism, None)
        .map_err(|e| format!("Invalid params: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut master = [0u8; 32];
    argon2
        .hash_password_into(new_passphrase.as_bytes(), &salt, &mut master)
        .map_err(|e| format!("Argon2id failed: {}", e))?;
    let hk = Hkdf::<Sha256>::new(Some(&salt), &master);
    let mut enc_key = [0u8; 32];
    hk.expand(b"vault-encryption", &mut enc_key)
        .map_err(|e| format!("HKDF failed: {}", e))?;
    Ok((salt, enc_key))
}

/// Rotate the vault key.
pub fn rotate_key(
    old_encrypted_vault: &[u8],
    old_key: &[u8; 32],
    new_passphrase: &str,
    old_metadata: &VaultMetadata,
) -> Result<RotationResult, String> {
    // 1. Decrypt with old key
    let (old_nonce, old_ct) = old_encrypted_vault.split_at(12);
    let old_cipher = Aes256Gcm::new(old_key.into());
    let nonce_ref = Nonce::from_slice(old_nonce);
    let plaintext = old_cipher
        .decrypt(nonce_ref, old_ct)
        .map_err(|e| format!("Decryption with old key failed: {}", e))?;

    // 2. Derive new key
    let (new_salt, new_key) = derive_new_key(
        new_passphrase,
        old_metadata.memory_kb,
        old_metadata.iterations,
        old_metadata.parallelism,
    )?;

    // 3. Re-encrypt with new key
    let new_cipher = Aes256Gcm::new((&new_key).into());
    let mut new_nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut new_nonce_bytes);
    let new_nonce = Nonce::from_slice(&new_nonce_bytes);
    let new_ct = new_cipher
        .encrypt(new_nonce, plaintext.as_ref())
        .map_err(|e| format!("Encryption with new key failed: {}", e))?;
    let mut new_encrypted = Vec::with_capacity(12 + new_ct.len());
    new_encrypted.extend_from_slice(&new_nonce_bytes);
    new_encrypted.extend_from_slice(&new_ct);

    // 4. Build new metadata
    let new_metadata = VaultMetadata {
        salt: new_salt,
        memory_kb: old_metadata.memory_kb,
        iterations: old_metadata.iterations,
        parallelism: old_metadata.parallelism,
        rotation_count: old_metadata.rotation_count + 1,
        last_rotation: chrono::Utc::now().to_rfc3339(),
    };

    Ok(RotationResult {
        new_metadata,
        new_encrypted_vault: new_encrypted,
        old_key_material: old_key.to_vec(),
    })
}

/// Verify that rotation was successful by decrypting with the new key.
pub fn verify_rotation(
    new_encrypted_vault: &[u8],
    new_key: &[u8; 32],
) -> Result<bool, String> {
    let (nonce_bytes, ciphertext) = new_encrypted_vault.split_at(12);
    let cipher = Aes256Gcm::new(new_key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    match cipher.decrypt(nonce, ciphertext) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Build a VaultMetadata from parameters.
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

/// Zeroize key material in a RotationResult.
pub fn zeroize_rotation_result(result: &mut RotationResult) {
    result.old_key_material.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(key.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_derive_new_key_deterministic() {
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
        // derive_new_key uses HKDF with b"vault-encryption", so we must match that
        let (_, _expected_new_key) = derive_new_key("newpass", 4096, 3, 1).unwrap();
        // But the salt is random in derive_new_key, so we can't match exactly.
        // Instead, derive from the same salt that was stored in metadata.
        let new_key = {
            use argon2::{Argon2, Algorithm, Params, Version};
            use hkdf::Hkdf;
            use sha2::Sha256;
            let params = Params::new(4096, 3, 1, None).unwrap();
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            let mut master = [0u8; 32];
            argon2.hash_password_into(b"newpass", &result.new_metadata.salt, &mut master).unwrap();
            let hk = Hkdf::<Sha256>::new(Some(&result.new_metadata.salt), &master);
            let mut enc_key = [0u8; 32];
            hk.expand(b"vault-encryption", &mut enc_key).unwrap();
            enc_key
        };
        assert!(verify_rotation(&result.new_encrypted_vault, &new_key).unwrap());
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
