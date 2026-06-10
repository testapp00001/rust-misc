//! # Lesson 01: Argon2id Password Hashing
//!
//! ## The Recommended Algorithm
//!
//! Argon2id is the winner of the Password Hashing Competition (PHC) and the
//! OWASP-recommended algorithm for password hashing. It comes in three variants:
//!
//! - **Argon2d**: Maximizes resistance to GPU cracking. Data-dependent memory access.
//! - **Argon2i**: Resistance to side-channel attacks. Data-independent memory access.
//! - **Argon2id**: Hybrid -- starts with Argon2i (side-channel resistant), then
//!   switches to Argon2d (GPU resistant). **This is the one you want.**
//!
//! ## Why Argon2id?
//!
//! Argon2id is **memory-hard**: it requires a large amount of RAM to compute.
//! GPUs have limited memory per core, and ASICs cannot cheaply replicate large
//! SRAM banks. This makes brute-force attacks vastly more expensive:
//!
//! | Algorithm | Hash Rate (GPU cluster) | Cost to crack "P@ssw0rd!" |
//! |-----------|------------------------|--------------------------|
//! | SHA-256   | ~50 billion/sec        | < 1 second               |
//! | bcrypt    | ~100 thousand/sec      | ~3 hours                  |
//! | Argon2id  | ~1 thousand/sec        | ~130 days                 |
//!
//! ## Parameters
//!
//! Argon2id has three tuning parameters:
//! - **Memory (m)**: Kilobytes of RAM used. Higher = more GPU resistance.
//!   OWASP minimum: 19456 KB (19 MB).
//! - **Iterations (t)**: Number of passes over memory. Higher = more CPU time.
//!   OWASP minimum: 2.
//! - **Parallelism (p)**: Number of threads. Higher = faster on multi-core CPUs.
//!   Typically 1-4.
//!
//! ## Attack Scenario
//!
//! An attacker breaches your database and steals all password hashes. With
//! SHA-256 hashes, they can try billions of passwords per second on a GPU cluster.
//! With Argon2id (19 MB memory per hash), each attempt requires allocating and
//! filling 19 MB of RAM. A GPU with 8 GB of VRAM can only run ~400 concurrent
//! attempts instead of millions.
//!
//! ## Rust Crates
//!
//! The `argon2` crate is a pure-Rust implementation of Argon2id maintained by the
//! RustCrypto project. It supports all three variants and provides a safe, ergonomic
//! API.

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, SaltString};
use argon2::{Argon2, Algorithm, Version, Params};

/// Exercise 1: Hash a password using Argon2id with default parameters.
///
/// Use `SaltString::generate(&mut OsRng)` for a random salt.
/// Use `Argon2::default()` which uses Argon2id with reasonable defaults.
/// The output is a PHC-formatted string like:
/// `$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>`
///
/// Hints:
/// - Create a salt with `SaltString::generate(&mut OsRng)`
/// - Get an `Argon2` instance with `Argon2::default()`
/// - Call `argon2.hash_password(password.as_bytes(), &salt)?`
/// - Convert the result to string with `.to_string()`
pub fn hash_password_argon2id(password: &str) -> Result<String, String> {
    todo!("Implement Argon2id password hashing")
}

/// Exercise 2: Verify a password against an Argon2id hash.
///
/// The hash string is in PHC format. Parse it, then verify the password.
///
/// Hints:
/// - Parse the hash: `PasswordHash::new(&hash_str)?`
/// - Verify: `Argon2::default().verify_password(password.as_bytes(), &parsed_hash)?`
/// - Return `Ok(true)` if valid, `Ok(false)` if invalid, `Err` on parse failure
pub fn verify_password_argon2id(password: &str, hash_str: &str) -> Result<bool, String> {
    todo!("Implement Argon2id password verification")
}

/// Exercise 3: Hash a password with custom Argon2id parameters.
///
/// Use the specified memory (in KB), iterations, and parallelism.
///
/// Hints:
/// - Build params: `Params::new(m_cost, t_cost, p_cost, None)?`
/// - Create Argon2: `Argon2::new(Algorithm::Argon2id, Version::V0x13, params)`
/// - Generate salt and hash as before
pub fn hash_password_custom_params(
    password: &str,
    memory_kb: u32,
    iterations: u32,
    parallelism: u32,
) -> Result<String, String> {
    todo!("Implement Argon2id with custom parameters")
}

/// Exercise 4: Extract the parameters from a PHC-formatted Argon2id hash string.
///
/// Return (algorithm, version, memory_kb, iterations, parallelism).
///
/// Hints:
/// - Parse with `PasswordHash::new(&hash_str)?`
/// - Access params: `hash.algorithm()`, `hash.version()`
/// - Access params: `hash.params().get_decimal("m")`, etc.
pub fn parse_hash_params(hash_str: &str) -> Result<(String, u32, u32, u32, u32), String> {
    todo!("Implement hash parameter extraction")
}

/// Exercise 5: Hash with explicit pepper (application-wide secret).
///
/// Combine the password with a pepper before hashing.
/// The pepper is NOT stored in the hash -- it must be provided at verification time.
///
/// Hints:
/// - Concatenate: `format!("{}:{}", pepper, password)`
/// - Hash the combined string with Argon2id
pub fn hash_with_pepper(password: &str, pepper: &str) -> Result<String, String> {
    todo!("Implement peppered Argon2id hashing")
}

/// Exercise 6: Verify a peppered password.
///
/// Hints:
/// - Recombine: `format!("{}:{}", pepper, password)`
/// - Verify using `verify_password_argon2id`
pub fn verify_with_pepper(password: &str, pepper: &str, hash_str: &str) -> Result<bool, String> {
    todo!("Implement peppered password verification")
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
