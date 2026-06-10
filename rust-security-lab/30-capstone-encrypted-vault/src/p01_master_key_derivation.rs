//! # Lesson 01: Master Key Derivation
//!
//! ## The Foundation of Your Vault
//!
//! Every password manager starts with the same problem: a human remembers one
//! passphrase, but the system needs a cryptographic key. This lesson teaches you
//! to derive a strong encryption key from a weak human passphrase using Argon2id
//! and HKDF.
//!
//! ## Two-Stage Derivation
//!
//! Real password managers use a two-stage approach:
//!
//! 1. **Argon2id** (password stretching): Turns the passphrase into a 32-byte
//!    "master secret". This is intentionally slow -- it makes brute-force expensive.
//!    Argon2id is memory-hard: each attempt requires ~19 MB of RAM, making GPU
//!    attacks impractical.
//!
//! 2. **HKDF** (key derivation): Takes the master secret and derives separate
//!    keys for different purposes (encryption, authentication, backup). This is
//!    fast and deterministic -- same input always produces the same output.
//!
//! ## Why Two Stages?
//!
//! - Argon2id is slow by design. You only want to run it once per session.
//! - HKDF is fast. You can derive as many purpose-specific keys as needed.
//! - If one key is compromised, the others remain safe (key separation).
//!
//! ## Salt Storage
//!
//! The Argon2id salt is stored alongside the vault (it is not secret). Its purpose
//! is to ensure that two users with the same password get different keys, and to
//! prevent precomputed rainbow table attacks.
//!
//! ## Parameters
//!
//! For this module we use reduced parameters for fast tests:
//! - Memory: 4096 KB (4 MB) -- production would use 19456+ KB
//! - Iterations: 3
//! - Parallelism: 1
//!
//! ## Attack Scenario
//!
//! An attacker steals the vault file (which includes the salt). They must now
//! brute-force the passphrase through Argon2id. With 4 MB memory per attempt,
//! a GPU with 8 GB VRAM can only run ~2000 concurrent attempts. At 3 iterations,
//! each attempt takes ~100ms. Trying all 8-character passwords would take
//! centuries even with a large cluster.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHasher, SaltString};
use argon2::{Argon2, Algorithm, Params, Version};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

/// A container for the derived key material.
/// In a real system, these bytes would be wrapped in a `Secret` type.
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

/// Exercise 1: Derive a 32-byte master secret from a passphrase using Argon2id.
///
/// Use the provided salt bytes (not a SaltString) with the raw Argon2 API.
///
/// Hints:
/// - Create params: `Params::new(4096, 3, 1, None).unwrap()`
/// - Create Argon2: `Argon2::new(Algorithm::Argon2id, Version::V0x13, params)`
/// - Allocate a 32-byte output buffer
/// - Call `argon2.hash_password_into(passphrase.as_bytes(), salt, &mut output)`
/// - Return the output bytes
pub fn derive_master_secret(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], String> {
    todo!("Derive a 32-byte master secret using Argon2id")
}

/// Exercise 2: Generate a random 32-byte salt for Argon2id.
///
/// Hints:
/// - Allocate a 32-byte array: `let mut salt = [0u8; 32];`
/// - Fill with random bytes: `rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut salt);`
/// - Return the salt
pub fn generate_salt() -> [u8; 32] {
    todo!("Generate a random 32-byte salt")
}

/// Exercise 3: Derive multiple purpose-specific keys from a master secret using HKDF.
///
/// The info parameter is used to derive different keys from the same master secret.
/// For the vault, we derive three keys: encryption, auth, and backup.
///
/// Hints:
/// - Create HKDF: `let hk = Hkdf::<Sha256>::new(Some(salt), master_secret);`
/// - Expand into output: `hk.expand(info, &mut output).unwrap();`
/// - Return the 32-byte output
pub fn derive_key_from_master(master_secret: &[u8], salt: &[u8], info: &[u8]) -> Result<[u8; 32], String> {
    todo!("Derive a purpose-specific key using HKDF")
}

/// Exercise 4: Derive all vault keys from a passphrase.
///
/// This combines exercises 1-3 into the full key derivation pipeline:
/// 1. Generate a salt (or use the provided one)
/// 2. Derive the master secret with Argon2id
/// 3. Derive three purpose-specific keys with HKDF
///
/// Hints:
/// - Use `derive_master_secret` for step 2
/// - Use `derive_key_from_master` three times with different info strings
/// - Use info strings: b"vault-encryption", b"vault-auth", b"vault-backup"
pub fn derive_vault_keys(passphrase: &str, salt: &[u8]) -> Result<DerivedKeys, String> {
    todo!("Derive all vault keys from a passphrase")
}

/// Exercise 5: Verify that a passphrase produces the expected master secret.
///
/// This is used during vault unlock to verify the user entered the correct
/// passphrase without storing the passphrase itself.
///
/// Hints:
/// - Derive the master secret from the passphrase and salt
/// - Compare with the expected value using constant-time comparison
/// - Use a simple XOR-based constant-time compare:
///   ```rust
///   let mut diff = 0u8;
///   for (a, b) in derived.iter().zip(expected_master.iter()) {
///       diff |= a ^ b;
///   }
///   Ok(diff == 0)
///   ```
pub fn verify_passphrase(passphrase: &str, salt: &[u8], expected_master: &[u8; 32]) -> Result<bool, String> {
    todo!("Verify a passphrase against the expected master secret")
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
