//! # Lesson 03: PBKDF2 — Password-Based Key Derivation
//!
//! ## What Is PBKDF2?
//!
//! PBKDF2 (Password-Based Key Derivation Function 2) derives a cryptographic key from
//! a password. Unlike HKDF (which assumes high-entropy input), PBKDF2 is designed for
//! low-entropy passwords by being intentionally **slow**.
//!
//! How it works:
//! ```text
//! DK = PBKDF2(password, salt, iterations, dkLen)
//!
//! Internally:
//! U1 = HMAC(password, salt || 0x00000001)
//! U2 = HMAC(password, U1)
//! U3 = HMAC(password, U2)
//! ... repeated `iterations` times
//! DK = U1 XOR U2 XOR U3 XOR ... XOR Uc
//! ```
//!
//! The high iteration count (100,000+) makes brute-force attacks expensive.
//!
//! ## Attack Scenario: Fast Password Cracking
//!
//! If you use SHA-256(password) to derive a key, an attacker can try billions of
//! passwords per second on a GPU. With PBKDF2 at 600,000 iterations, each guess
//! costs ~100ms instead of ~1ns — a 100-million-fold slowdown.
//!
//! | Method | Speed (guesses/sec on GPU) |
//! |--------|---------------------------|
//! | SHA-256 | ~10 billion |
//! | PBKDF2 (1000 iter) | ~10 million |
//! | PBKDF2 (600,000 iter) | ~1,600 |
//! | Argon2id | ~1,000 (memory-bound) |
//!
//! ## PBKDF2 vs Argon2id
//!
//! PBKDF2 is CPU-bound. Argon2id is both CPU-bound AND memory-hard, making it harder
//! to accelerate with GPUs or ASICs. For new systems, prefer Argon2id. PBKDF2 is still
//! widely used and acceptable, especially in constrained environments.
//!
//! ## Security Notes
//!
//! - **Always use a random salt** (>= 16 bytes) — prevents rainbow tables
//! - **Use >= 600,000 iterations** (OWASP 2023 recommendation for PBKDF2-SHA256)
//! - **Never store plaintext passwords** — only store the derived key + salt

use ring::pbkdf2;

/// Exercise 1: Derive a key from a password using PBKDF2-HMAC-SHA256.
///
/// Use `ring::pbkdf2::derive` with the specified number of iterations.
///
/// Hints:
/// - Algorithm: `ring::pbkdf2::PBKDF2_HMAC_SHA256`
/// - `ring::pbkdf2::derive(algorithm, iterations, salt, password, &mut output)`
/// - `iterations` is a `std::num::NonZeroU32`
pub fn derive_key(password: &[u8], salt: &[u8], iterations: u32, key_length: usize) -> Vec<u8> {
    todo!("Derive a key from a password using PBKDF2")
}

/// Exercise 2: Verify a password against a previously derived key.
///
/// Use `ring::pbkdf2::verify` to check if a password produces the expected key.
///
/// Hints:
/// - `ring::pbkdf2::verify(algorithm, iterations, salt, password, expected_key)`
/// - Returns `Ok(())` on match, `Err` on mismatch
pub fn verify_password(password: &[u8], salt: &[u8], iterations: u32, expected_key: &[u8]) -> bool {
    todo!("Verify password against derived key")
}

/// Exercise 3: Demonstrate why high iteration counts matter.
///
/// This function times the PBKDF2 derivation at two different iteration counts
/// and returns (time_low_iters_ns, time_high_iters_ns).
///
/// Hints:
/// - Use `std::time::Instant::now()` for timing
/// - Use a consistent password and salt
/// - Low: 1000 iterations, High: 100,000 iterations
/// - Return elapsed nanoseconds for each
pub fn benchmark_iterations(password: &[u8], salt: &[u8]) -> (u128, u128) {
    todo!("Benchmark PBKDF2 at different iteration counts")
}

/// Exercise 4: Derive a key and store it with metadata.
///
/// Return a struct-like tuple: (derived_key, salt, iterations, algorithm_name)
/// This simulates what you'd store in a database for password verification.
pub fn derive_and_package(
    password: &[u8],
    iterations: u32,
    key_length: usize,
) -> (Vec<u8>, Vec<u8>, u32, String) {
    todo!("Derive key and package with metadata for storage")
}

/// Exercise 5: Re-derive a key from stored metadata.
///
/// Given the package from `derive_and_package`, verify a password.
pub fn verify_from_package(
    password: &[u8],
    stored_key: &[u8],
    stored_salt: &[u8],
    stored_iterations: u32,
) -> bool {
    todo!("Verify password using stored metadata")
}

/// Exercise 6: Demonstrate salt importance.
///
/// Derive keys from the same password with two different salts.
/// They must produce different keys.
///
/// Returns (key_with_salt_1, key_with_salt_2)
pub fn demonstrate_salt_importance(password: &[u8], iterations: u32) -> (Vec<u8>, Vec<u8>) {
    todo!("Show that different salts produce different keys from the same password")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ITERATIONS: u32 = 10_000; // Low for fast tests; use 600,000+ in production

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
