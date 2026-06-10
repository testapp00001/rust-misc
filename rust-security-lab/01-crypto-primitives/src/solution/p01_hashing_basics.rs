//! # Lesson 01: Hashing Basics — SHA-256 and BLAKE3 (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;

/// Compute SHA-256 hash of input data.
///
/// `ring::digest::digest` creates a one-shot hash. For streaming/incremental
/// hashing, use `ring::digest::Context` instead (see `sha256_streaming`).
pub fn sha256(data: &[u8]) -> Vec<u8> {
    digest::digest(&digest::SHA256, data).as_ref().to_vec()
}

/// Compute SHA-256 and return as lowercase hex string.
///
/// Each byte becomes exactly 2 hex characters. A 32-byte hash → 64 hex chars.
pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(sha256(data))
}

/// Compute BLAKE3 hash of input data.
///
/// BLAKE3 is ~10x faster than SHA-256 and supports parallelism.
/// Use it when you need high throughput and don't require NIST standardization.
pub fn blake3_hash(data: &[u8]) -> Vec<u8> {
    blake3::Hasher::new()
        .update(data)
        .finalize()
        .as_bytes()
        .to_vec()
}

/// Verify data integrity using constant-time comparison.
///
/// ⚠️ NEVER use `==` for hash comparison — it's vulnerable to timing attacks.
/// An attacker can measure response time to guess the hash byte-by-byte.
///
/// `ring::constant_time::verify_slices_are_equal` runs in constant time
/// regardless of where the first difference occurs.
pub fn verify_integrity(data: &[u8], expected_hash: &[u8]) -> bool {
    let computed = sha256(data);
    ring::constant_time::verify_slices_are_equal(&computed, expected_hash).is_ok()
}

/// Hash data in chunks (streaming/incremental hashing).
///
/// This is essential for large files that don't fit in memory.
/// `ring::digest::Context` accumulates data across multiple `.update()` calls.
///
/// The final hash is identical to hashing all data at once:
/// `sha256_streaming([a, b, c]) == sha256(a || b || c)`
pub fn sha256_streaming(chunks: &[&[u8]]) -> Vec<u8> {
    let mut context = digest::Context::new(&digest::SHA256);
    for chunk in chunks {
        context.update(chunk);
    }
    context.finish().as_ref().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let h2 = sha256(b"hellp");
        assert_ne!(h1, h2);
        let diff_bits: u32 = h1.iter().zip(h2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
        assert!(diff_bits > 50, "Avalanche effect: {} bits differ (expected ~128)", diff_bits);
    }

    #[test]
    fn test_blake3_hash() {
        let hash = blake3_hash(b"hello");
        assert_eq!(hash.len(), 32);
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
    #[ignore]
    fn bench_hashing() {
        let data = vec![0u8; 1_000_000];
        let _hash = sha256(&data);
    }
}
