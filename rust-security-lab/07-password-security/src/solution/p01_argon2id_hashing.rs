//! # Lesson 01: Argon2id Password Hashing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Argon2, Algorithm, Version, Params};

/// Hash a password using Argon2id with default parameters.
pub fn hash_password_argon2id(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Hashing failed: {}", e))?;
    Ok(hash.to_string())
}

/// Verify a password against an Argon2id hash.
pub fn verify_password_argon2id(password: &str, hash_str: &str) -> Result<bool, String> {
    let parsed_hash = PasswordHash::new(hash_str)
        .map_err(|e| format!("Failed to parse hash: {}", e))?;
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(format!("Verification error: {}", e)),
    }
}

/// Hash a password with custom Argon2id parameters.
pub fn hash_password_custom_params(
    password: &str,
    memory_kb: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<String, String> {
    let params = Params::new(memory_kb, iterations, parallelism, None)
        .map_err(|e| format!("Invalid params: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Hashing failed: {}", e))?;
    Ok(hash.to_string())
}

/// Extract the parameters from a PHC-formatted Argon2id hash string.
pub fn parse_hash_params(hash_str: &str) -> Result<(String, u32, u32, u32, u32), String> {
    let hash = PasswordHash::new(hash_str)
        .map_err(|e| format!("Failed to parse hash: {}", e))?;

    let algorithm = hash.algorithm.to_string();
    let version: u32 = hash.version.unwrap_or(0).into();
    let m: u32 = hash.params.get_decimal("m").unwrap_or(0).into();
    let t: u32 = hash.params.get_decimal("t").unwrap_or(0).into();
    let p: u32 = hash.params.get_decimal("p").unwrap_or(0).into();

    Ok((algorithm, version, m, t, p))
}

/// Hash with explicit pepper (application-wide secret).
pub fn hash_with_pepper(password: &str, pepper: &str) -> Result<String, String> {
    let combined = format!("{}:{}", pepper, password);
    hash_password_argon2id(&combined)
}

/// Verify a peppered password.
pub fn verify_with_pepper(password: &str, pepper: &str, hash_str: &str) -> Result<bool, String> {
    let combined = format!("{}:{}", pepper, password);
    verify_password_argon2id(&combined, hash_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_format() {
        let hash = hash_password_argon2id("correcthorsebatterystaple").unwrap();
        assert!(hash.starts_with("$argon2id$"), "Hash should use Argon2id algorithm");
        assert!(hash.contains("$v=19$"), "Hash should use version 19");
    }

    #[test]
    fn test_verify_correct_password() {
        let hash = hash_password_argon2id("mypassword").unwrap();
        assert!(verify_password_argon2id("mypassword", &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let hash = hash_password_argon2id("mypassword").unwrap();
        assert!(!verify_password_argon2id("wrongpassword", &hash).unwrap());
    }

    #[test]
    fn test_hash_unique_each_time() {
        let h1 = hash_password_argon2id("samepassword").unwrap();
        let h2 = hash_password_argon2id("samepassword").unwrap();
        assert_ne!(h1, h2, "Each hash should have a unique salt");
    }

    #[test]
    fn test_custom_params() {
        let hash = hash_password_custom_params("test", 4096, 1, 1).unwrap();
        assert!(hash.starts_with("$argon2id$"));
        assert!(hash.contains("m=4096"), "Should use custom memory");
        assert!(hash.contains("t=1"), "Should use custom iterations");
    }

    #[test]
    fn test_parse_hash_params() {
        let hash = hash_password_custom_params("test", 8192, 3, 2).unwrap();
        let (algo, version, mem, iters, par) = parse_hash_params(&hash).unwrap();
        assert_eq!(algo, "argon2id");
        assert_eq!(version, 0x13);
        assert_eq!(mem, 8192);
        assert_eq!(iters, 3);
        assert_eq!(par, 2);
    }

    #[test]
    fn test_peppered_hash_and_verify() {
        let pepper = "my-application-secret-pepper";
        let hash = hash_with_pepper("password123", pepper).unwrap();
        assert!(verify_with_pepper("password123", pepper, &hash).unwrap());
        assert!(!verify_with_pepper("password123", "wrong-pepper", &hash).unwrap());
        assert!(!verify_with_pepper("wrong-password", pepper, &hash).unwrap());
    }
}
