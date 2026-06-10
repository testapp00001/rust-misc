//! # Lesson 08: Checksums vs Cryptographic Hashes
//!
//! ## What is a Checksum?
//!
//! A checksum is a small value computed from data to detect **accidental**
//! corruption (e.g., network errors, disk failures, transmission glitches).
//!
//! ## Checksums vs Cryptographic Hashes
//!
//! | Property | Checksum (CRC32) | Cryptographic Hash (SHA-256) |
//! |----------|------------------|------------------------------|
//! | Purpose | Detect accidents | Detect tampering |
//! | Speed | Very fast | Slower |
//! | Collision resistance | Weak (trivially breakable) | Strong (computationally infeasible) |
//! | Output size | 32 bits | 256 bits |
//! | Security | None | High |
//! | Use case | File transfer, storage | Digital signatures, integrity |
//!
//! ## CRC32
//!
//! CRC32 (Cyclic Redundancy Check, 32-bit) is the most common checksum.
//! It's used in:
//! - ZIP/GZIP files
//! - Network packets (Ethernet, Wi-Fi)
//! - File systems (ZFS, Btrfs)
//! - PNG images
//!
//! CRC32 is FAST but NOT secure. An attacker can easily craft data with any
//! desired CRC32 value.
//!
//! ## When to Use What
//!
//! - **Checksums**: Detecting accidental corruption in storage/network
//!   - File download verification (alongside HTTPS)
//!   - Storage integrity checks
//!   - Network packet validation
//!
//! - **Cryptographic hashes**: Detecting intentional tampering
//!   - Digital signatures
//!   - Software distribution verification
//!   - Blockchain
//!   - Password storage (with salt + KDF)
//!
//! ## Key Takeaway
//!
//! Checksums protect against accidents. Cryptographic hashes protect against adversaries.
//! Never use a checksum where you need security.

/// Exercise 1: Compute CRC32 checksum.
///
/// Hints:
/// - CRC32 is not in the standard crate list, so we'll implement a simple version
/// - Use the lookup table approach for CRC32
/// - Or compute it manually with bit operations
///
/// For this exercise, implement a basic CRC32 using the standard polynomial 0xEDB88320.
pub fn crc32(data: &[u8]) -> u32 {
    todo!("Implement CRC32 checksum")
}

/// Exercise 2: Verify data using CRC32 checksum.
///
/// Returns true if the CRC32 of data matches the expected checksum.
///
/// Hints:
/// - Compute CRC32 of the data
/// - Compare with expected value
pub fn verify_crc32(data: &[u8], expected: u32) -> bool {
    todo!("Verify CRC32 checksum")
}

/// Exercise 3: Compute CRC32 and return as hex string.
///
/// Hints:
/// - Compute CRC32
/// - Format as 8-character hex string (zero-padded)
pub fn crc32_hex(data: &[u8]) -> String {
    todo!("Compute CRC32 and return as hex string")
}

/// Exercise 4: Demonstrate that CRC32 is NOT collision-resistant.
///
/// Find a different input that has the same CRC32 as the given input.
/// This shows CRC32 is trivially breakable.
///
/// Hints:
/// - This is a demonstration function
/// - Use a brute-force search: try incrementing values until CRC32 matches
/// - Just try appending different single bytes to a base string
pub fn find_crc32_collision(target_crc: u32) -> Vec<u8> {
    todo!("Find a CRC32 collision by brute force")
}

/// Exercise 5: Compare CRC32 vs SHA-256 for collision resistance.
///
/// Returns (crc32_value, sha256_value) for the given data.
///
/// Hints:
/// - Compute CRC32 (u32) and SHA-256 (Vec<u8>)
/// - Return both for comparison
pub fn compare_checksum_vs_hash(data: &[u8]) -> (u32, Vec<u8>) {
    todo!("Compute both CRC32 and SHA-256 for comparison")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known CRC32 test vectors
    const CRC32_EMPTY: u32 = 0x00000000;
    const CRC32_ABC: u32 = 0x352441C2;
    const CRC32_HELLO: u32 = 0x3610a686;

    #[test]
    fn test_crc32_empty() {
        assert_eq!(crc32(b""), CRC32_EMPTY);
    }

    #[test]
    fn test_crc32_abc() {
        assert_eq!(crc32(b"abc"), CRC32_ABC);
    }

    #[test]
    fn test_crc32_hello() {
        assert_eq!(crc32(b"hello"), CRC32_HELLO);
    }

    #[test]
    fn test_verify_crc32_valid() {
        assert!(verify_crc32(b"abc", CRC32_ABC));
    }

    #[test]
    fn test_verify_crc32_corrupted() {
        assert!(!verify_crc32(b"abd", CRC32_ABC));
    }

    #[test]
    fn test_crc32_hex() {
        let hex = crc32_hex(b"abc");
        assert_eq!(hex, "352441c2");
        assert_eq!(hex.len(), 8, "CRC32 hex should be 8 characters");
    }

    #[test]
    fn test_crc32_deterministic() {
        let c1 = crc32(b"test data");
        let c2 = crc32(b"test data");
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_crc32_detects_corruption() {
        let original = b"Important file content";
        let checksum = crc32(original);

        // Corrupt one byte
        let mut corrupted = original.to_vec();
        corrupted[5] ^= 0xFF;
        assert_ne!(crc32(&corrupted), checksum, "CRC32 should detect corruption");
    }

    #[test]
    fn test_crc32_collision_trivial() {
        // CRC32 has only 2^32 possible values — collisions are easy to find
        let target = crc32(b"target");
        let collision = find_crc32_collision(target);
        assert_ne!(collision, b"target", "Collision should be a different input");
        assert_eq!(crc32(&collision), target, "Collision should have same CRC32");
    }

    #[test]
    fn test_compare_checksum_vs_hash() {
        let data = b"test data";
        let (crc, sha) = compare_checksum_vs_hash(data);
        assert_eq!(crc, crc32(data));
        assert_eq!(sha.len(), 32);
        // SHA-256 should match ring's output
        let expected_sha = ring::digest::digest(&ring::digest::SHA256, data);
        assert_eq!(sha, expected_sha.as_ref().to_vec());
    }
}
