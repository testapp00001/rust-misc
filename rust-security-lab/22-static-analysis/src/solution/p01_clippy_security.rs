//! # Lesson 01: Clippy Security Lints (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use sha2::{Sha256, Digest};

/// Read a file safely, propagating errors instead of panicking.
pub fn safe_read_file(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", path, e))
}

/// Safe addition using checked arithmetic.
pub fn safe_add(a: u64, b: u64) -> Option<u64> {
    a.checked_add(b)
}

/// Safe subtraction using checked arithmetic.
pub fn safe_sub(a: u64, b: u64) -> Option<u64> {
    a.checked_sub(b)
}

/// Safe multiplication using checked arithmetic.
pub fn safe_mul(a: u64, b: u64) -> Option<u64> {
    a.checked_mul(b)
}

/// Safe slice indexing that returns None on out-of-bounds.
pub fn safe_get(data: &[u8], index: usize) -> Option<u8> {
    data.get(index).copied()
}

/// Hash input with SHA-256 and return hex string.
pub fn safe_hash_hex(input: &[u8]) -> String {
    let hash = Sha256::digest(input);
    hash.iter().map(|byte| format!("{:02x}", byte)).collect()
}

/// Parse a u32 from a string safely.
pub fn safe_parse_u32(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|e| format!("Failed to parse '{}': {}", s, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_read_file_not_found() {
        let result = safe_read_file("/nonexistent/path/file.txt");
        assert!(result.is_err(), "Should return Err for missing file");
    }

    #[test]
    fn test_safe_add_normal() {
        assert_eq!(safe_add(100, 200), Some(300));
    }

    #[test]
    fn test_safe_add_overflow() {
        assert_eq!(
            safe_add(u64::MAX, 1),
            None,
            "Should return None on overflow"
        );
    }

    #[test]
    fn test_safe_sub_normal() {
        assert_eq!(safe_sub(200, 100), Some(100));
    }

    #[test]
    fn test_safe_sub_underflow() {
        assert_eq!(
            safe_sub(100, 200),
            None,
            "Should return None on underflow"
        );
    }

    #[test]
    fn test_safe_mul_overflow() {
        assert_eq!(
            safe_mul(u64::MAX, 2),
            None,
            "Should return None on overflow"
        );
    }

    #[test]
    fn test_safe_get_in_bounds() {
        let data = vec![10, 20, 30];
        assert_eq!(safe_get(&data, 1), Some(20));
    }

    #[test]
    fn test_safe_get_out_of_bounds() {
        let data = vec![10, 20, 30];
        assert_eq!(safe_get(&data, 5), None);
    }

    #[test]
    fn test_safe_hash_hex_deterministic() {
        let hash1 = safe_hash_hex(b"hello");
        let hash2 = safe_hash_hex(b"hello");
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64, "SHA-256 hex should be 64 chars");
    }

    #[test]
    fn test_safe_hash_hex_different_inputs() {
        assert_ne!(safe_hash_hex(b"hello"), safe_hash_hex(b"world"));
    }

    #[test]
    fn test_safe_parse_u32_valid() {
        assert_eq!(safe_parse_u32("42").unwrap(), 42);
    }

    #[test]
    fn test_safe_parse_u32_invalid() {
        assert!(safe_parse_u32("not_a_number").is_err());
    }

    #[test]
    fn test_safe_parse_u32_overflow() {
        assert!(safe_parse_u32("99999999999").is_err());
    }
}
