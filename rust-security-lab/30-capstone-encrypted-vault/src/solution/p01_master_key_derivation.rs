//! # Lesson 01: Master Key Derivation (Reference Solution)
//!
//! See the exercise file for full documentation on Argon2id + HKDF key derivation.

use argon2::{Argon2, Algorithm, Params, Version};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

/// A container for the derived key material.
#[derive(Clone, Debug)]
pub struct DerivedKeys {
    /// The encryption key for AES-256-GCM (32 bytes).
    pub encryption_key: [u8; 32],
    /// The authentication key for HMAC (32 bytes).
    pub auth_key: [u8; 32],
    /// The backup encryption key (32 bytes).
    pub backup_key: [u8; 32],
}

impl Drop for DerivedKeys {
    fn drop(&mut self) {
        self.encryption_key.zeroize();
        self.auth_key.zeroize();
        self.backup_key.zeroize();
    }
}

/// Derive a 32-byte master secret from a passphrase using Argon2id.
pub fn derive_master_secret(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    let params = Params::new(4096, 3, 1, None)
        .map_err(|e| format!("Invalid Argon2 params: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut output = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut output)
        .map_err(|e| format!("Argon2id derivation failed: {}", e))?;
    Ok(output)
}

/// Generate a random 32-byte salt for Argon2id.
pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut salt);
    salt
}

/// Derive a purpose-specific key from a master secret using HKDF.
pub fn derive_key_from_master(master_secret: &[u8], salt: &[u8], info: &[u8]) -> Result<[u8; 32], String> {
    let hk = Hkdf::<Sha256>::new(Some(salt), master_secret);
    let mut output = [0u8; 32];
    hk.expand(info, &mut output)
        .map_err(|e| format!("HKDF expansion failed: {}", e))?;
    Ok(output)
}

/// Derive all vault keys from a passphrase.
pub fn derive_vault_keys(passphrase: &str, salt: &[u8]) -> Result<DerivedKeys, String> {
    let master = derive_master_secret(passphrase, salt)?;
    let enc_key = derive_key_from_master(&master, salt, b"vault-encryption")?;
    let auth_key = derive_key_from_master(&master, salt, b"vault-auth")?;
    let backup_key = derive_key_from_master(&master, salt, b"vault-backup")?;
    Ok(DerivedKeys {
        encryption_key: enc_key,
        auth_key,
        backup_key,
    })
}

/// Verify that a passphrase produces the expected master secret.
pub fn verify_passphrase(passphrase: &str, salt: &[u8], expected_master: &[u8; 32]) -> Result<bool, String> {
    let derived = derive_master_secret(passphrase, salt)?;
    // Constant-time comparison to prevent timing attacks
    if derived.len() != expected_master.len() {
        return Ok(false);
    }
    let mut diff = 0u8;
    for (a, b) in derived.iter().zip(expected_master.iter()) {
        diff |= a ^ b;
    }
    Ok(diff == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_master_secret_deterministic() {
        let salt = generate_salt();
        let s1 = derive_master_secret("correcthorsebatterystaple", &salt).unwrap();
        let s2 = derive_master_secret("correcthorsebatterystaple", &salt).unwrap();
        assert_eq!(s1, s2, "Same passphrase + salt should produce same secret");
    }

    #[test]
    fn test_derive_master_secret_different_passphrases() {
        let salt = generate_salt();
        let s1 = derive_master_secret("password1", &salt).unwrap();
        let s2 = derive_master_secret("password2", &salt).unwrap();
        assert_ne!(s1, s2, "Different passphrases should produce different secrets");
    }

    #[test]
    fn test_derive_master_secret_different_salts() {
        let s1 = derive_master_secret("password", &generate_salt()).unwrap();
        let s2 = derive_master_secret("password", &generate_salt()).unwrap();
        assert_ne!(s1, s2, "Different salts should produce different secrets");
    }

    #[test]
    fn test_generate_salt_unique() {
        let s1 = generate_salt();
        let s2 = generate_salt();
        assert_ne!(s1, s2, "Each salt should be unique");
        assert_eq!(s1.len(), 32, "Salt should be 32 bytes");
    }

    #[test]
    fn test_derive_key_from_master_different_info() {
        let master = [42u8; 32];
        let salt = [1u8; 32];
        let k1 = derive_key_from_master(&master, &salt, b"encryption").unwrap();
        let k2 = derive_key_from_master(&master, &salt, b"authentication").unwrap();
        assert_ne!(k1, k2, "Different info should produce different keys");
    }

    #[test]
    fn test_derive_vault_keys() {
        let salt = generate_salt();
        let keys = derive_vault_keys("mypassword", &salt).unwrap();
        // All three keys should be different
        assert_ne!(keys.encryption_key, keys.auth_key);
        assert_ne!(keys.encryption_key, keys.backup_key);
        assert_ne!(keys.auth_key, keys.backup_key);
    }

    #[test]
    fn test_verify_passphrase_correct() {
        let salt = generate_salt();
        let master = derive_master_secret("correctpassword", &salt).unwrap();
        assert!(verify_passphrase("correctpassword", &salt, &master).unwrap());
    }

    #[test]
    fn test_verify_passphrase_incorrect() {
        let salt = generate_salt();
        let master = derive_master_secret("correctpassword", &salt).unwrap();
        assert!(!verify_passphrase("wrongpassword", &salt, &master).unwrap());
    }
}
