//! # Lesson 01: Hashing Basics — SHA-256 and BLAKE3
//!
//! ## What is Cryptographic Hashing?
//!
//! A cryptographic hash function takes arbitrary-length input and produces a fixed-length
//! "fingerprint" (digest). It must be:
//! - **Deterministic**: Same input always gives same output
//! - **Pre-image resistant**: Can't reverse the hash to find the input
//! - **Collision resistant**: Can't find two different inputs with the same hash
//! - **Avalanche effect**: 1-bit change in input → ~50% of output bits change
//!
//! ## SHA-256 vs BLAKE3
//!
//! | Property | SHA-256 | BLAKE3 |
//! |----------|---------|--------|
//! | Standard | NIST FIPS 180-4 | BLAKE3 spec |
//! | Speed | ~400 MB/s | ~4 GB/s (10x faster) |
//! | Security | 128-bit collision | 128-bit collision |
//! | Parallelizable | No | Yes |
//! | Use case | Standards compliance, TLS | High-performance hashing |
//!
//! ## ⚠️ Common Mistakes
//!
//! 1. **Hashing passwords with SHA-256** — Use Argon2id instead (Module 07)
//! 2. **Using MD5/SHA-1** — Both are broken (collisions found)
//! 3. **Thinking hashing is encryption** — Hashing is ONE-WAY
//!
//! ## 🔴 Attack Demo: Hash Collision (conceptual)
//!
//! MD5 and SHA-1 have known collision attacks. Two different files can produce the same hash.
//! This is why we use SHA-256 or BLAKE3 — no practical collisions exist yet.

use ring::digest;

/// Exercise 1: Compute SHA-256 hash of input data.
///
/// The output should be a 32-byte digest.
///
/// Hints:
/// - Use `ring::digest::digest(&digest::SHA256, data)`
/// - The result is `digest::Digest` — call `.as_ref()` to get `&[u8]`
pub fn sha256(data: &[u8]) -> Vec<u8> {
    todo!("Implement SHA-256 hashing using ring::digest")
}

/// Exercise 2: Compute SHA-256 and return as hex string.
///
/// Hints:
/// - First compute the hash with `sha256()`
/// - Then convert each byte to its 2-digit hex representation
/// - Format: "a1b2c3d4..." (lowercase)
pub fn sha256_hex(data: &[u8]) -> String {
    todo!("Implement SHA-256 → hex string conversion")
}

/// Exercise 3: Compute BLAKE3 hash of input data.
///
/// Hints:
/// - Use `blake3::Hasher::new()`
/// - Call `.update(data)` then `.finalize()`
/// - `.finalize()` returns a `blake3::Hash` — use `.as_bytes()` to get `&[u8; 32]`
pub fn blake3_hash(data: &[u8]) -> Vec<u8> {
    todo!("Implement BLAKE3 hashing")
}

/// Exercise 4: Verify data integrity by comparing hash.
///
/// Given data and an expected hash, return true if they match.
///
/// Hints:
/// - Compute SHA-256 of data
/// - Compare with expected hash using constant-time comparison
/// - DO NOT use `==` on slices (vulnerable to timing attacks)
/// - Use `ring::constant_time::verify_slices_are_equal`
pub fn verify_integrity(data: &[u8], expected_hash: &[u8]) -> bool {
    todo!("Implement integrity verification with constant-time comparison")
}

/// Exercise 5: Hash a file in chunks (streaming hash).
///
/// For large files, we can't load everything into memory. Hash in chunks.
///
/// Hints:
/// - Use `ring::digest::Context` for incremental hashing
/// - Create with `Context::new(&digest::SHA256)`
/// - Feed chunks with `.update(chunk)`
/// - Finalize with `.finish()` → `Digest`
pub fn sha256_streaming(chunks: &[&[u8]]) -> Vec<u8> {
    todo!("Implement streaming SHA-256 hash")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known SHA-256 hash values for testing
    const EMPTY_SHA256: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const HELLO_SHA256: &str = "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824";

    #[test]
    fn test_sha256_empty() {
        let hash = sha256(b"");
        assert_eq!(hash.len(), 32, "SHA-256 digest should be 32 bytes");
    }

    #[test]
    fn test_sha256_hello() {
        let hash = sha256(b"hello");
        let hex = hex::encode(&hash);
        assert_eq!(hex, HELLO_SHA256);
    }

    #[test]
    fn test_sha256_hex() {
        let hex = sha256_hex(b"");
        assert_eq!(hex, EMPTY_SHA256);
    }

    #[test]
    fn test_sha256_hex_hello() {
        let hex = sha256_hex(b"hello");
        assert_eq!(hex, HELLO_SHA256);
    }

    #[test]
    fn test_sha256_deterministic() {
        let h1 = sha256(b"test data");
        let h2 = sha256(b"test data");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_sha256_avalanche() {
        let h1 = sha256(b"hello");
        let h2 = sha256(b"hellp"); // 1 bit difference
        assert_ne!(h1, h2, "Hashes should differ for different inputs");
        // Count differing bits — should be ~50%
        let diff_bits: u32 = h1.iter().zip(h2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
        assert!(diff_bits > 50, "Avalanche effect: {} bits differ (expected ~128)", diff_bits);
    }

    #[test]
    fn test_blake3_hash() {
        let hash = blake3_hash(b"hello");
        assert_eq!(hash.len(), 32, "BLAKE3 digest should be 32 bytes");
        // Known BLAKE3 hash of "hello"
        let hex = hex::encode(&hash);
        assert_eq!(hex, "ea8f163db39692f695e1c34b617e00e4a6ecf650a2b9a80c396e45e5e8f7e0c8");
    }

    #[test]
    fn test_blake3_deterministic() {
        let h1 = blake3_hash(b"test");
        let h2 = blake3_hash(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_verify_integrity_valid() {
        let data = b"important data";
        let hash = sha256(data);
        assert!(verify_integrity(data, &hash));
    }

    #[test]
    fn test_verify_integrity_tampered() {
        let data = b"important data";
        let hash = sha256(data);
        assert!(!verify_integrity(b"tampered data", &hash));
    }

    #[test]
    fn test_streaming_matches_single() {
        let data = b"this is a longer piece of data that we hash in chunks";
        let chunks: Vec<&[u8]> = data.chunks(10).collect();
        let streaming_hash = sha256_streaming(&chunks);
        let single_hash = sha256(data);
        assert_eq!(streaming_hash, single_hash);
    }

    #[test]
    #[ignore] // Run with: cargo test --features solution -- --ignored
    fn bench_hashing() {
        // Benchmark placeholder — implement if you want to measure performance
        let data = vec![0u8; 1_000_000]; // 1MB
        let _hash = sha256(&data);
    }
}
