//! # Lesson 07: Key Derivation from Passwords
//!
//! ## The Problem: Passwords Are Not Keys
//!
//! Passwords are human-readable strings with low entropy. An encryption key
//! must be 32 bytes of high-entropy random data. You cannot use a password
//! directly as an encryption key because:
//!
//! 1. **Low entropy**: "MyP@ssw0rd!" has ~40 bits of entropy vs 256 bits needed
//! 2. **Predictable patterns**: Humans choose memorable passwords
//! 3. **Variable length**: Keys must be exactly 32 bytes
//!
//! ## Key Derivation Functions (KDFs)
//!
//! A KDF transforms a password into a cryptographically strong key:
//!
//! ```text
//! key = KDF(password, salt, iterations, memory, parallelism)
//! ```
//!
//! | KDF | Memory-Hard | GPU-Resistant | Recommended |
//! |-----|-------------|---------------|-------------|
//! | PBKDF2 | No | No | Legacy only |
//! | bcrypt | No | Partial | Legacy only |
//! | scrypt | Yes | Yes | Good |
//! | Argon2id | Yes | Yes | Best |
//!
//! ## Argon2id
//!
//! Argon2id is the winner of the Password Hashing Competition (2015).
//! It's "memory-hard" — it requires a lot of RAM, making GPU/ASIC attacks
//! expensive. Argon2id is the recommended variant (hybrid of Argon2i and Argon2d).
//!
//! ## Security Parameters
//!
//! - **Salt**: Random, unique per password. Stored alongside the derived key.
//! - **Memory cost**: How much RAM is used (higher = more secure, slower)
//! - **Time cost**: Number of iterations (higher = more secure, slower)
//! - **Parallelism**: Number of threads (doesn't affect security much)
//!
//! ## Attack Scenario: Brute Force
//!
//! Without a KDF (or with a fast KDF like MD5):
//! 1. Attacker gets the password hash from a database breach
//! 2. Uses GPU to try billions of passwords per second
//! 3. Weak passwords are cracked in seconds
//!
//! With Argon2id:
//! 1. Each guess requires 64MB+ of RAM
//! 2. GPUs have limited memory per core
//! 3. Attack speed drops from billions/sec to thousands/sec

use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2, Params, Version,
};
use rand::Rng;

/// Exercise 1: Generate a random salt for key derivation.
///
/// The salt should be unique for each password. It prevents rainbow table
/// attacks and ensures two users with the same password get different keys.
///
/// # Hints
/// - Use `SaltString::generate(&mut OsRng)` for a cryptographically secure salt
/// - The salt is typically 16 bytes (encoded as base64)
pub fn generate_salt() -> SaltString {
    todo!("Generate a random salt for Argon2id")
}

/// Exercise 2: Derive an encryption key from a password using Argon2id.
///
/// This function should:
/// 1. Use the provided salt
/// 2. Configure Argon2id with reasonable parameters
/// 3. Derive exactly 32 bytes (for AES-256)
///
/// # Arguments
/// * `password` - The user's password
/// * `salt` - Random salt (from `generate_salt()`)
///
/// # Returns
/// A 32-byte key suitable for AES-256
///
/// # Hints
/// - Create `Argon2::new(Algorithm::Argon2id, Version::V0x13, params)`
/// - Use `Params::new(65536, 3, 1, Some(32))` for:
///   - 64MB memory, 3 iterations, 1 thread, 32-byte output
/// - Call `argon2.hash_password_into(password.as_bytes(), salt.as_bytes(), &mut output)`
pub fn derive_key(password: &str, salt: &SaltString) -> Result<[u8; 32], String> {
    todo!("Derive a 32-byte AES-256 key from password using Argon2id")
}

/// Exercise 3: Derive a key with custom Argon2id parameters.
///
/// Allow customization of memory cost, time cost, and parallelism
/// for different security/performance tradeoffs.
///
/// # Arguments
/// * `password` - The user's password
/// * `salt` - Random salt
/// * `memory_kib` - Memory cost in KiB (e.g., 65536 = 64MB)
/// * `iterations` - Time cost (number of passes)
/// * `parallelism` - Number of threads
///
/// # Hints
/// - `Params::new(memory_kib, iterations, parallelism, Some(32))`
pub fn derive_key_custom(
    password: &str,
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<[u8; 32], String> {
    todo!("Derive key with custom Argon2id parameters")
}

/// Exercise 4: Hash a password for storage (not for encryption).
///
/// Password hashes for storage use Argon2id's built-in encoding format,
/// which includes the algorithm, version, salt, and hash in a single string:
///
/// `$argon2id$v=19$m=65536,t=3,p=1$c2FsdA$hash`
///
/// # Hints
/// - Use `Argon2::default()` (or with custom params)
/// - Call `argon2.hash_password(password.as_bytes(), &salt)`
/// - Return `hash.to_string()` which includes the salt
pub fn hash_password_for_storage(password: &str) -> Result<String, String> {
    todo!("Hash a password for storage using Argon2id")
}

/// Exercise 5: Verify a password against a stored hash.
///
/// This is used during login — compare the provided password against
/// the stored hash.
///
/// # Hints
/// - Parse the stored hash: `PasswordHash::new(&stored_hash)`
/// - Verify: `Argon2::default().verify_password(password.as_bytes(), &parsed_hash)`
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, String> {
    todo!("Verify a password against a stored Argon2id hash")
}

/// Exercise 6: Derive multiple keys from a single password.
///
/// Use different salts to derive multiple independent keys from the same
/// password. This is useful when you need separate keys for encryption
/// and authentication.
///
/// # Hints
/// - Generate a unique salt for each key
/// - Call `derive_key()` with each salt
/// - Return a Vec of (salt, key) pairs
pub fn derive_multiple_keys(password: &str, count: usize) -> Vec<(SaltString, [u8; 32])> {
    todo!("Derive multiple independent keys from a single password")
}

/// Exercise 7: Estimate the time required for key derivation.
///
/// Benchmark Argon2id with given parameters and return the elapsed time.
/// This helps users choose appropriate parameters for their hardware.
///
/// # Hints
/// - Use `std::time::Instant::now()` for timing
/// - Run key derivation and measure elapsed time
/// - Return milliseconds
pub fn benchmark_key_derivation(
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> u128 {
    todo!("Benchmark Argon2id key derivation time")
}

/// Exercise 8: Create a key derivation "envelope" with all parameters.
///
/// Store all the information needed to derive a key from a password:
/// the salt, parameters, and algorithm version.
///
/// # Hints
/// - Serialize to JSON for storage
/// - Include: algorithm, version, memory, iterations, parallelism, salt
pub fn create_key_envelope(
    password: &str,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<String, String> {
    todo!("Create a JSON envelope with key derivation parameters and salt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt = generate_salt();
        // Salt should be non-empty base64 string
        assert!(!salt.as_str().is_empty());
        assert!(salt.as_str().len() >= 16, "Salt should be at least 16 chars");
    }

    #[test]
    fn test_derive_key_length() {
        let salt = generate_salt();
        let key = derive_key("test_password", &salt).unwrap();
        assert_eq!(key.len(), 32, "Derived key should be 32 bytes");
    }

    #[test]
    fn test_derive_key_deterministic() {
        let salt = generate_salt();
        let key1 = derive_key("password", &salt).unwrap();
        let key2 = derive_key("password", &salt).unwrap();
        assert_eq!(key1, key2, "Same password + salt should give same key");
    }

    #[test]
    fn test_derive_key_different_salt() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        let key1 = derive_key("password", &salt1).unwrap();
        let key2 = derive_key("password", &salt2).unwrap();
        assert_ne!(key1, key2, "Different salts should give different keys");
    }

    #[test]
    fn test_derive_key_different_password() {
        let salt = generate_salt();
        let key1 = derive_key("password1", &salt).unwrap();
        let key2 = derive_key("password2", &salt).unwrap();
        assert_ne!(key1, key2, "Different passwords should give different keys");
    }

    #[test]
    fn test_derive_key_custom() {
        let salt = b"test_salt_16byte";
        let key = derive_key_custom("password", salt, 65536, 3, 1).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_hash_and_verify_password() {
        let hash = hash_password_for_storage("my_secure_password").unwrap();

        // Should verify correctly
        assert!(verify_password("my_secure_password", &hash).unwrap());

        // Wrong password should fail
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_derive_multiple_keys() {
        let keys = derive_multiple_keys("password", 3);
        assert_eq!(keys.len(), 3);

        // All keys should be different (different salts)
        assert_ne!(keys[0].1, keys[1].1);
        assert_ne!(keys[1].1, keys[2].1);
    }

    #[test]
    fn test_key_derivation_not_too_fast() {
        // Key derivation should take at least a few milliseconds
        // to resist brute force attacks
        let start = std::time::Instant::now();
        let salt = generate_salt();
        let _key = derive_key("test", &salt).unwrap();
        let elapsed = start.elapsed();

        // Should take at least 10ms (otherwise parameters are too weak)
        assert!(
            elapsed.as_millis() >= 10,
            "Key derivation took only {}ms — parameters may be too weak",
            elapsed.as_millis()
        );
    }
}
