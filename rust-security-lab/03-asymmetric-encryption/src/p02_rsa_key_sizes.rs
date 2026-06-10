//! # Lesson 02: RSA Key Sizes — Security vs Performance
//!
//! ## Key Size and Security Level
//!
//! RSA security depends on the difficulty of factoring N = p*q. The best known
//! algorithm (General Number Field Sieve) has sub-exponential complexity.
//!
//! | Key Size | Security Level | Comparable ECC |
//! |----------|---------------|----------------|
//! | 1024-bit | ~80-bit       | DO NOT USE     |
//! | 2048-bit | ~112-bit      | 224-bit curve  |
//! | 3072-bit | ~128-bit      | 256-bit curve  |
//! | 4096-bit | ~140-bit      | 280-bit curve  |
//!
//! ## Why Not Always Use the Biggest Key?
//!
//! - **Generation time**: 4096-bit keys take 10-100x longer to generate than 2048-bit
//! - **Encryption/Decryption**: Slower with larger keys (O(n^2) for multiplication)
//! - **Bandwidth**: Signatures and ciphertexts are larger
//! - **Diminishing returns**: Going from 2048 to 4096 adds ~28 bits of security
//!
//! ## Current Recommendations (2024)
//!
//! - **Minimum**: 2048-bit (NIST recommendation through 2030)
//! - **Recommended**: 3072-bit (for data that must stay secret beyond 2030)
//! - **Maximum practical**: 4096-bit (very slow key generation)
//! - **Better alternative**: Use ECC (256-bit P-256 ≈ 3072-bit RSA)
//!
//! ## Attack Demo: Small Key Factorization
//!
//! RSA-512 was factored in 1999. RSA-768 was factored in 2009.
//! Any key under 1024 bits can be factored by a determined attacker.

use rsa::{RsaPrivateKey, RsaPublicKey, Oaep};
use rand::rngs::OsRng;
use std::time::Instant;

/// Exercise 1: Generate RSA keys of different sizes and measure generation time.
///
/// Returns a vector of (key_size_bits, generation_time_ms) pairs.
///
/// Hints:
/// - Use `Instant::now()` and `.elapsed()` for timing
/// - Generate keys for sizes: 1024, 2048, 3072, 4096
/// - Convert elapsed time to milliseconds: `.as_millis()`
pub fn benchmark_key_generation() -> Vec<(usize, u128)> {
    todo!("Benchmark RSA key generation at different sizes")
}

/// Exercise 2: Benchmark encrypt/decrypt operations at different key sizes.
///
/// Returns a vector of (key_size_bits, encrypt_time_us, decrypt_time_us) in microseconds.
///
/// Hints:
/// - Generate a keypair at each size
/// - Encrypt a fixed message (e.g., 32 bytes)
/// - Measure both encrypt and decrypt times in microseconds
pub fn benchmark_encrypt_decrypt() -> Vec<(usize, u128, u128)> {
    todo!("Benchmark RSA encrypt/decrypt at different key sizes")
}

/// Exercise 3: Determine the maximum plaintext size for RSA-OAEP at a given key size.
///
/// RSA-OAEP with SHA-256 can encrypt at most: key_size_bytes - 2 * hash_size - 2
/// For SHA-256 (32-byte hash): max = key_bytes - 66
///
/// Hints:
/// - key_size_bytes = key_bits / 8
/// - SHA-256 produces 32-byte hashes
/// - Formula: key_bytes - 2 * 32 - 2
pub fn max_plaintext_size(key_bits: usize) -> usize {
    todo!("Calculate maximum plaintext size for RSA-OAEP with SHA-256")
}

/// Exercise 4: Check if a key size meets minimum security requirements.
///
/// NIST recommends minimum 2048-bit RSA keys through 2030.
/// Keys under 2048 bits are considered insecure.
///
/// Hints:
/// - Return `true` if key_bits >= 2048
pub fn is_secure_key_size(key_bits: usize) -> bool {
    todo!("Check if RSA key size meets minimum security requirements")
}

/// Exercise 5: Compare RSA key size to equivalent ECC key size.
///
/// Approximate equivalences:
/// - RSA 2048 ≈ ECC 224
/// - RSA 3072 ≈ ECC 256
/// - RSA 7680 ≈ ECC 384
/// - RSA 15360 ≈ ECC 521
///
/// Return the equivalent ECC key size, or 0 if the RSA size is below minimum.
pub fn equivalent_ecc_key_size(rsa_bits: usize) -> usize {
    todo!("Map RSA key size to equivalent ECC key size")
}

/// Exercise 6: Encrypt a message that's too large for RSA directly (demonstrating the limit).
///
/// Try to encrypt a message larger than max_plaintext_size. Should panic or return error.
///
/// Hints:
/// - Generate a 2048-bit key
/// - Try to encrypt a message of 200 bytes (exceeds the ~190 byte limit)
/// - The rsa crate will return an error
pub fn encrypt_oversized_message(key_bits: usize, message: &[u8]) -> Result<Vec<u8>, String> {
    todo!("Attempt to encrypt a message exceeding RSA-OAEP capacity")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_plaintext_2048() {
        // RSA-2048 with OAEP SHA-256: 256 - 66 = 190 bytes
        assert_eq!(max_plaintext_size(2048), 190);
    }

    #[test]
    fn test_max_plaintext_4096() {
        // RSA-4096 with OAEP SHA-256: 512 - 66 = 446 bytes
        assert_eq!(max_plaintext_size(4096), 446);
    }

    #[test]
    fn test_is_secure_key_size() {
        assert!(!is_secure_key_size(1024), "1024-bit is insecure");
        assert!(is_secure_key_size(2048), "2048-bit is minimum secure");
        assert!(is_secure_key_size(3072), "3072-bit is secure");
        assert!(is_secure_key_size(4096), "4096-bit is secure");
    }

    #[test]
    fn test_equivalent_ecc_sizes() {
        assert_eq!(equivalent_ecc_key_size(2048), 224);
        assert_eq!(equivalent_ecc_key_size(3072), 256);
        assert_eq!(equivalent_ecc_key_size(7680), 384);
        assert_eq!(equivalent_ecc_key_size(15360), 521);
    }

    #[test]
    fn test_insecure_key_returns_zero_ecc() {
        assert_eq!(equivalent_ecc_key_size(512), 0, "512-bit RSA is insecure");
        assert_eq!(equivalent_ecc_key_size(1024), 0, "1024-bit RSA is insecure");
    }

    #[test]
    fn test_oversized_message_fails() {
        let result = encrypt_oversized_message(2048, &vec![0u8; 200]);
        assert!(result.is_err(), "Encrypting oversized message should fail");
    }

    #[test]
    fn test_max_size_just_fits() {
        let bits = 2048;
        let max = max_plaintext_size(bits);
        let result = encrypt_oversized_message(bits, &vec![0u8; max]);
        assert!(result.is_ok(), "Message at max size should encrypt successfully");
    }

    #[test]
    #[ignore] // Slow test — run with: cargo test --features solution -- --ignored
    fn test_key_generation_timing() {
        let benchmarks = benchmark_key_generation();
        assert_eq!(benchmarks.len(), 4);
        // Larger keys should take longer
        assert!(benchmarks[1].1 >= benchmarks[0].1,
            "2048-bit should take at least as long as 1024-bit");
    }
}
