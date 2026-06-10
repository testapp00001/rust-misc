//! # Lesson 03: PBKDF2 — Password-Based Key Derivation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;

/// Derive a key from a password using PBKDF2-HMAC-SHA256.
///
/// Security notes:
/// - `iterations` should be >= 600,000 for PBKDF2-HMAC-SHA256 (OWASP 2023)
/// - `salt` should be >= 16 bytes, randomly generated
/// - `key_length` can be up to 2^32 - 1 bytes, but 32 is typical for AES-256
pub fn derive_key(password: &[u8], salt: &[u8], iterations: u32, key_length: usize) -> Vec<u8> {
    let iter_count = NonZeroU32::new(iterations).expect("iterations must be > 0");
    let mut output = vec![0u8; key_length];
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iter_count,
        salt,
        password,
        &mut output,
    );
    output
}

/// Verify a password against a previously derived key.
///
/// This is constant-time — it doesn't reveal which byte differs.
/// Internally, `ring::pbkdf2::verify` recomputes the derivation and
/// uses constant-time comparison.
pub fn verify_password(password: &[u8], salt: &[u8], iterations: u32, expected_key: &[u8]) -> bool {
    let iter_count = NonZeroU32::new(iterations).expect("iterations must be > 0");
    pbkdf2::verify(
        pbkdf2::PBKDF2_HMAC_SHA256,
        iter_count,
        salt,
        password,
        expected_key,
    )
    .is_ok()
}

/// Benchmark PBKDF2 at different iteration counts.
///
/// Returns (elapsed_ns_low_iters, elapsed_ns_high_iters).
/// The ratio should roughly match the iteration ratio (100x in this case).
pub fn benchmark_iterations(password: &[u8], salt: &[u8]) -> (u128, u128) {
    let start = std::time::Instant::now();
    let _key1 = derive_key(password, salt, 1_000, 32);
    let elapsed_low = start.elapsed().as_nanos();

    let start = std::time::Instant::now();
    let _key2 = derive_key(password, salt, 100_000, 32);
    let elapsed_high = start.elapsed().as_nanos();

    (elapsed_low, elapsed_high)
}

/// Derive a key and package it with metadata for storage.
///
/// In production, you'd store this in a database with columns:
/// `user_id`, `derived_key`, `salt`, `iterations`, `algorithm`.
///
/// The salt MUST be unique per user. Reusing salts allows attackers to
/// precompute keys for common passwords.
pub fn derive_and_package(
    password: &[u8],
    iterations: u32,
    key_length: usize,
) -> (Vec<u8>, Vec<u8>, u32, String) {
    let rng = SystemRandom::new();
    let mut salt = vec![0u8; 16];
    rng.fill(&mut salt).expect("Failed to generate salt");

    let key = derive_key(password, &salt, iterations, key_length);
    let algorithm = String::from("PBKDF2-HMAC-SHA256");

    (key, salt, iterations, algorithm)
}

/// Verify a password using stored metadata.
///
/// This is the pattern used by authentication systems:
/// 1. User provides password at login
/// 2. Look up stored (key, salt, iterations) from database
/// 3. Re-derive key from provided password + stored salt + stored iterations
/// 4. Compare with stored key using constant-time comparison
pub fn verify_from_package(
    password: &[u8],
    stored_key: &[u8],
    stored_salt: &[u8],
    stored_iterations: u32,
) -> bool {
    verify_password(password, stored_salt, stored_iterations, stored_key)
}

/// Demonstrate salt importance.
///
/// Same password + different salts = completely different derived keys.
/// This is why per-user salts are essential — without them, attackers can
/// check all users against a single precomputed table (rainbow table).
pub fn demonstrate_salt_importance(password: &[u8], iterations: u32) -> (Vec<u8>, Vec<u8>) {
    let salt1 = b"salt_for_user_1!!";
    let salt2 = b"salt_for_user_2!!";
    let key1 = derive_key(password, salt1, iterations, 32);
    let key2 = derive_key(password, salt2, iterations, 32);
    (key1, key2)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ITERATIONS: u32 = 10_000;

    #[test]
    fn test_derive_key_length() {
        let key = derive_key(b"password123", b"fixedsalt12345678", TEST_ITERATIONS, 32);
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_derive_key_deterministic() {
        let key1 = derive_key(b"password123", b"fixedsalt12345678", TEST_ITERATIONS, 32);
        let key2 = derive_key(b"password123", b"fixedsalt12345678", TEST_ITERATIONS, 32);
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let key1 = derive_key(b"password1", b"fixedsalt12345678", TEST_ITERATIONS, 32);
        let key2 = derive_key(b"password2", b"fixedsalt12345678", TEST_ITERATIONS, 32);
        assert_ne!(key1, key2);
    }

    #[test]
    fn test_verify_correct_password() {
        let salt = b"fixedsalt12345678";
        let key = derive_key(b"correct_password", salt, TEST_ITERATIONS, 32);
        assert!(verify_password(b"correct_password", salt, TEST_ITERATIONS, &key));
    }

    #[test]
    fn test_verify_wrong_password() {
        let salt = b"fixedsalt12345678";
        let key = derive_key(b"correct_password", salt, TEST_ITERATIONS, 32);
        assert!(!verify_password(b"wrong_password", salt, TEST_ITERATIONS, &key));
    }

    #[test]
    fn test_derive_and_package() {
        let (key, salt, iters, algo) = derive_and_package(b"mypassword", TEST_ITERATIONS, 32);
        assert_eq!(key.len(), 32);
        assert_eq!(salt.len(), 16);
        assert_eq!(iters, TEST_ITERATIONS);
        assert_eq!(algo, "PBKDF2-HMAC-SHA256");
    }

    #[test]
    fn test_verify_from_package() {
        let (key, salt, iters, _) = derive_and_package(b"mypassword", TEST_ITERATIONS, 32);
        assert!(verify_from_package(b"mypassword", &key, &salt, iters));
        assert!(!verify_from_package(b"wrongpassword", &key, &salt, iters));
    }

    #[test]
    fn test_salt_importance() {
        let (key1, key2) = demonstrate_salt_importance(b"samepassword", TEST_ITERATIONS);
        assert_ne!(key1, key2, "Same password with different salts must produce different keys");
    }
}
