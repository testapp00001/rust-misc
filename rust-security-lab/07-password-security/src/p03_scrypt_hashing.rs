//! # Lesson 03: scrypt Password Hashing
//!
//! ## Memory-Hard Alternative
//!
//! scrypt was designed by Colin Percival in 2009 for the Tarsnap backup service.
//! It was one of the first password hashing functions to be explicitly memory-hard,
//! meaning it requires large amounts of RAM to compute efficiently.
//!
//! ## How scrypt Works
//!
//! scrypt uses PBKDF2-SHA256 as a building block and adds a large memory workspace:
//! 1. Expands the password into a large vector using Salsa20/8 core
//! 2. Pseudorandomly accesses elements of this vector (ROMix algorithm)
//! 3. The vector must be held in RAM -- you cannot trade memory for time
//!
//! ## Parameters
//!
//! scrypt has three main parameters (expressed as powers of 2):
//! - **N (CPU/memory cost)**: Must be a power of 2. N=2^14 = 16384 is a common default.
//!   Memory usage is approximately `128 * N * p` bytes.
//! - **r (block size)**: Affects memory and CPU. Typically 8.
//! - **p (parallelization)**: Number of parallel threads. Typically 1.
//!
//! | Parameters        | Memory Usage | Approx Time |
//! |-------------------|-------------|-------------|
//! | N=2^10, r=8, p=1  | ~1 MB       | ~1 ms       |
//! | N=2^14, r=8, p=1  | ~16 MB      | ~50 ms      |
//! | N=2^15, r=8, p=1  | ~32 MB      | ~100 ms     |
//! | N=2^20, r=8, p=1  | ~1 GB       | ~3 seconds  |
//!
//! ## scrypt vs Argon2id
//!
//! | Property          | scrypt              | Argon2id             |
//! |-------------------|---------------------|----------------------|
//! | Memory-hard       | Yes                 | Yes                  |
//! | Side-channel safe | No (cache-timing)   | Yes (Argon2i phase)  |
//! | GPU resistance    | Good                | Excellent            |
//! | ASIC resistance   | Good                | Excellent            |
//! | Standard          | RFC 7914            | PHC winner, RFC 9106 |
//! | Recommended by    | Some legacy systems | OWASP, NIST          |
//!
//! Argon2id is generally preferred over scrypt for new systems, but scrypt
//! is still a solid choice and is widely deployed (used by many cryptocurrency
//! wallets, for example).
//!
//! ## Attack Scenario
//!
//! scrypt at N=2^14, r=8, p=1 uses ~16 MB per hash. On a GPU with 8 GB VRAM,
//! an attacker can run ~500 concurrent attempts. Combined with the CPU cost of
//! the Salsa20/8 mixing, this limits throughput to roughly 1,000-5,000 hashes
//! per second on a high-end GPU -- far better than bcrypt but worse than Argon2id.

use scrypt::{scrypt, Params};

/// Exercise 1: Hash a password with scrypt using default parameters.
///
/// Hints:
/// - Create params: `Params::new(log_n, r, p, len).map_err(...)?`
///   where log_n=14, r=8, p=1, len=64
/// - Create a random salt (16 bytes): use `rand::Rng` to fill a `[u8; 16]`
/// - Call `scrypt(password.as_bytes(), &salt, &params, &mut output)?`
/// - Return (salt, hash) both as `Vec<u8>`
pub fn hash_password_scrypt(password: &str) -> Result<(Vec<u8>, Vec<u8>), String> {
    todo!("Implement scrypt password hashing")
}

/// Exercise 2: Verify a password against a scrypt hash.
///
/// Recompute the hash with the same salt and parameters, then compare.
///
/// Hints:
/// - Use the same params as hashing (log_n=14, r=8, p=1, len=64)
/// - Recompute: `scrypt(password.as_bytes(), &salt, &params, &mut output)?`
/// - Compare using constant-time comparison (ring::constant_time)
pub fn verify_password_scrypt(password: &str, salt: &[u8], hash: &[u8]) -> Result<bool, String> {
    todo!("Implement scrypt password verification")
}

/// Exercise 3: Encode a scrypt hash as a single portable string.
///
/// Format: `scrypt:<log_n>:<r>:<p>:<hex_salt>:<hex_hash>`
///
/// Hints:
/// - Use `hex::encode` for salt and hash
/// - Format with `format!`
pub fn encode_scrypt_hash(salt: &[u8], hash: &[u8], log_n: u8, r: u32, p: u32) -> String {
    todo!("Implement scrypt hash encoding")
}

/// Exercise 4: Decode a scrypt hash string back to its components.
///
/// Parse the format from `encode_scrypt_hash`.
///
/// Returns (log_n, r, p, salt_bytes, hash_bytes).
///
/// Hints:
/// - Split by ':'
/// - Parse each field
/// - Use `hex::decode` for salt and hash
pub fn decode_scrypt_hash(encoded: &str) -> Result<(u8, u32, u32, Vec<u8>, Vec<u8>), String> {
    todo!("Implement scrypt hash decoding")
}

/// Exercise 5: Estimate memory usage for given scrypt parameters.
///
/// Memory formula: approximately `128 * N * p` bytes, where N = 2^log_n.
///
/// Hints:
/// - N = 2u64.pow(log_n as u32)
/// - memory = 128 * N * (p as u64)
pub fn estimate_memory_usage(log_n: u8, r: u32, p: u32) -> u64 {
    todo!("Implement memory usage estimation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let (salt, hash) = hash_password_scrypt("mypassword").unwrap();
        assert!(verify_password_scrypt("mypassword", &salt, &hash).unwrap());
    }

    #[test]
    fn test_verify_wrong_password() {
        let (salt, hash) = hash_password_scrypt("mypassword").unwrap();
        assert!(!verify_password_scrypt("wrongpassword", &salt, &hash).unwrap());
    }

    #[test]
    fn test_unique_salts() {
        let (salt1, _) = hash_password_scrypt("same").unwrap();
        let (salt2, _) = hash_password_scrypt("same").unwrap();
        assert_ne!(salt1, salt2, "Each hash should use a unique salt");
    }

    #[test]
    fn test_hash_length() {
        let (_, hash) = hash_password_scrypt("test").unwrap();
        assert_eq!(hash.len(), 64, "Hash should be 64 bytes (as configured)");
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let (salt, hash) = hash_password_scrypt("test").unwrap();
        let encoded = encode_scrypt_hash(&salt, &hash, 14, 8, 1);
        let (log_n, r, p, decoded_salt, decoded_hash) = decode_scrypt_hash(&encoded).unwrap();
        assert_eq!(log_n, 14);
        assert_eq!(r, 8);
        assert_eq!(p, 1);
        assert_eq!(decoded_salt, salt);
        assert_eq!(decoded_hash, hash);
    }

    #[test]
    fn test_encode_format() {
        let encoded = encode_scrypt_hash(&[1, 2, 3], &[4, 5, 6], 10, 8, 1);
        assert!(encoded.starts_with("scrypt:10:8:1:"));
    }

    #[test]
    fn test_decode_invalid() {
        assert!(decode_scrypt_hash("invalid").is_err());
        assert!(decode_scrypt_hash("scrypt:abc:8:1:aabb:aabb").is_err());
    }

    #[test]
    fn test_memory_estimation() {
        // N=2^14=16384, p=1: 128 * 16384 * 1 = 2,097,152 bytes = ~2 MB
        let mem = estimate_memory_usage(14, 8, 1);
        assert_eq!(mem, 128 * 16384 * 1);

        // N=2^14, p=2: should double
        let mem2 = estimate_memory_usage(14, 8, 2);
        assert_eq!(mem2, mem * 2);
    }
}
