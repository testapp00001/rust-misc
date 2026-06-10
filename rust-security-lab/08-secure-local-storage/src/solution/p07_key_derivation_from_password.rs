//! # Lesson 07: Key Derivation from Passwords (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2, Algorithm, Params, Version,
};

/// Generate a random salt for key derivation.
///
/// Salt values must be unique per password. They prevent rainbow table
/// attacks and ensure that two users with the same password get different
/// derived keys. The `SaltString::generate` function uses the OS CSPRNG.
pub fn generate_salt() -> SaltString {
    SaltString::generate(&mut OsRng)
}

/// Derive an encryption key from a password using Argon2id.
///
/// Parameters chosen for a balance of security and usability:
/// - 64MB memory: Makes GPU attacks expensive (each guess needs 64MB)
/// - 3 iterations: Moderate computational cost
/// - 1 thread: Single-threaded (can increase for parallel-capable systems)
/// - 32 bytes output: Exactly one AES-256 key
pub fn derive_key(password: &str, salt: &SaltString) -> Result<[u8; 32], String> {
    let params = Params::new(65536, 3, 1, Some(32))
        .map_err(|e| format!("Invalid params: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt.as_str().as_bytes(), &mut output)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    Ok(output)
}

/// Derive a key with custom Argon2id parameters.
///
/// Allows tuning the security/performance tradeoff:
/// - Higher memory = more secure (GPU-resistant) but slower
/// - Higher iterations = more secure but slower
/// - Higher parallelism = faster on multi-core (doesn't affect security much)
pub fn derive_key_custom(
    password: &str,
    salt: &[u8],
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<[u8; 32], String> {
    let params = Params::new(memory_kib, iterations, parallelism, Some(32))
        .map_err(|e| format!("Invalid params: {}", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    Ok(output)
}

/// Hash a password for storage (not for encryption).
///
/// Uses Argon2id's built-in encoding format which includes all parameters:
/// `$argon2id$v=19$m=65536,t=3,p=1$c2FsdA$hash`
///
/// This format is self-contained — the verifier can extract the salt
/// and parameters from the hash string itself.
pub fn hash_password_for_storage(password: &str) -> Result<String, String> {
    let salt = generate_salt();
    let params = Params::new(65536, 3, 1, None)
        .map_err(|e| format!("Invalid params: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Password hashing failed: {}", e))?;

    Ok(hash.to_string())
}

/// Verify a password against a stored hash.
///
/// The stored hash contains the salt and parameters, so we can
/// re-derive the hash and compare. This is the standard login flow.
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(stored_hash)
        .map_err(|e| format!("Invalid hash format: {}", e))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Derive multiple independent keys from a single password.
///
/// Each key gets a unique salt, ensuring independence. This is useful
/// when you need separate keys for different purposes (e.g., encryption
/// and authentication) but only have one password.
pub fn derive_multiple_keys(password: &str, count: usize) -> Vec<(SaltString, [u8; 32])> {
    (0..count)
        .map(|_| {
            let salt = generate_salt();
            let key = derive_key(password, &salt).expect("Key derivation failed");
            (salt, key)
        })
        .collect()
}

/// Benchmark key derivation with given parameters.
///
/// Returns the elapsed time in milliseconds. This helps users choose
/// appropriate parameters for their hardware — the goal is typically
/// 100-500ms for interactive use, or 1-5s for high-security scenarios.
pub fn benchmark_key_derivation(
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> u128 {
    let start = std::time::Instant::now();
    let salt = b"benchmark_salt_16";
    let _ = derive_key_custom("benchmark_password", salt, memory_kib, iterations, parallelism);
    start.elapsed().as_millis()
}

/// Create a key derivation "envelope" with all parameters.
///
/// Stores everything needed to re-derive the key from the password:
/// algorithm, version, parameters, and salt. The password is NOT stored.
pub fn create_key_envelope(
    password: &str,
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<String, String> {
    let salt = generate_salt();
    let key = derive_key_custom(
        password,
        salt.as_str().as_bytes(),
        memory_kib,
        iterations,
        parallelism,
    )?;

    let envelope = serde_json::json!({
        "algorithm": "argon2id",
        "version": 19,
        "memory_kib": memory_kib,
        "iterations": iterations,
        "parallelism": parallelism,
        "salt": salt.as_str(),
        "key_hex": hex::encode(key),
    });

    serde_json::to_string_pretty(&envelope)
        .map_err(|e| format!("Envelope serialization failed: {}", e))
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
