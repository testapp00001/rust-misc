//! # Lesson 08: Checksums vs Cryptographic Hashes (Reference Solution)

/// CRC32 lookup table for polynomial 0xEDB88320 (reflected).
/// Pre-computed for performance.
fn crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for i in 0..256 {
        let mut crc = i as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
        table[i] = crc;
    }
    table
}

/// Compute CRC32 checksum using the standard algorithm.
///
/// Algorithm:
/// 1. Initialize CRC to 0xFFFFFFFF
/// 2. For each byte: crc = table[(crc ^ byte) & 0xFF] ^ (crc >> 8)
/// 3. Final XOR: crc ^= 0xFFFFFFFF
///
/// This is the CRC-32/ISO-HDLC variant (used in ZIP, PNG, etc.)
pub fn crc32(data: &[u8]) -> u32 {
    let table = crc32_table();
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        let index = ((crc ^ byte as u32) & 0xFF) as usize;
        crc = table[index] ^ (crc >> 8);
    }
    crc ^ 0xFFFFFFFF
}

/// Verify data against an expected CRC32 checksum.
pub fn verify_crc32(data: &[u8], expected: u32) -> bool {
    crc32(data) == expected
}

/// Compute CRC32 and return as lowercase hex string (zero-padded to 8 chars).
pub fn crc32_hex(data: &[u8]) -> String {
    format!("{:08x}", crc32(data))
}

/// Find a CRC32 collision by brute force.
///
/// CRC32 has only 2^32 possible outputs (~4 billion). With the birthday paradox,
/// we expect a collision after ~65,000 random inputs. Here we just try appending
/// different bytes until we find a match.
pub fn find_crc32_collision(target_crc: u32) -> Vec<u8> {
    // Try appending different single bytes to "collision_base"
    let base = b"collision_base_";
    for i in 0u32.. {
        let candidate = [base.as_ref(), &i.to_le_bytes()].concat();
        if crc32(&candidate) == target_crc && candidate != b"target" {
            return candidate;
        }
    }
    unreachable!("CRC32 space is small enough that we always find a collision")
}

/// Compute both CRC32 and SHA-256 for comparison.
pub fn compare_checksum_vs_hash(data: &[u8]) -> (u32, Vec<u8>) {
    let crc = crc32(data);
    let sha = ring::digest::digest(&ring::digest::SHA256, data);
    (crc, sha.as_ref().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(hex.len(), 8);
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
        let mut corrupted = original.to_vec();
        corrupted[5] ^= 0xFF;
        assert_ne!(crc32(&corrupted), checksum);
    }

    #[test]
    fn test_crc32_collision_trivial() {
        let target = crc32(b"target");
        let collision = find_crc32_collision(target);
        assert_ne!(collision, b"target");
        assert_eq!(crc32(&collision), target);
    }

    #[test]
    fn test_compare_checksum_vs_hash() {
        let data = b"test data";
        let (crc, sha) = compare_checksum_vs_hash(data);
        assert_eq!(crc, crc32(data));
        assert_eq!(sha.len(), 32);
        let expected_sha = ring::digest::digest(&ring::digest::SHA256, data);
        assert_eq!(sha, expected_sha.as_ref().to_vec());
    }
}
