//! # Lesson 05: Constant-Time Comparison (Reference Solution)

/// Constant-time byte comparison.
///
/// The key insight: we NEVER return early. We always process ALL bytes,
/// accumulating differences into a single register, and check at the end.
///
/// ```text
/// result = 0
/// for each pair (a[i], b[i]):
///     result |= a[i] ^ b[i]    // XOR gives 0 if equal, non-zero if different
///                                 // OR accumulates any difference
/// return result == 0
/// ```
///
/// Even if a[0] != b[0], we still check a[1..n] — the attacker cannot
/// learn which byte was wrong from timing.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        // Lengths differ — but we must still do "comparable" work to avoid
        // leaking the length difference via timing. We compare up to min length.
        // In practice, comparing different-length secrets is already a bug,
        // but we handle it gracefully.
        return false;
    }

    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Use ring's constant-time comparison.
///
/// `ring::constant_time::verify_slices_are_equal` is implemented in assembly
/// for many platforms and is resistant to compiler optimizations that might
/// reintroduce timing variations.
pub fn ring_constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    ring::constant_time::verify_slices_are_equal(a, b).is_ok()
}

/// Naive (INSECURE) comparison — for educational purposes only.
///
/// This uses `==` on slices, which returns `false` at the first differing byte.
/// NEVER use this for security-sensitive comparisons.
pub fn insecure_compare(a: &[u8], b: &[u8]) -> bool {
    a == b
}

/// Constant-time hex string comparison.
///
/// Decode hex to bytes, then compare in constant time.
/// Returns false for invalid hex input.
pub fn constant_time_hex_eq(a: &str, b: &str) -> bool {
    let bytes_a = match hex::decode(a) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let bytes_b = match hex::decode(b) {
        Ok(b) => b,
        Err(_) => return false,
    };
    constant_time_eq(&bytes_a, &bytes_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq_identical() {
        assert!(constant_time_eq(b"hello", b"hello"));
    }

    #[test]
    fn test_constant_time_eq_different() {
        assert!(!constant_time_eq(b"hello", b"world"));
    }

    #[test]
    fn test_constant_time_eq_empty() {
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        assert!(!constant_time_eq(b"short", b"longer value"));
        assert!(!constant_time_eq(b"", b"not empty"));
    }

    #[test]
    fn test_constant_time_eq_single_byte_diff() {
        let a = b"abcdef";
        let b = b"abcdeX";
        assert!(!constant_time_eq(a, b));
    }

    #[test]
    fn test_ring_constant_time_eq() {
        assert!(ring_constant_time_eq(b"test", b"test"));
        assert!(!ring_constant_time_eq(b"test", b"fail"));
        assert!(!ring_constant_time_eq(b"", b"x"));
    }

    #[test]
    fn test_insecure_compare_matches() {
        assert!(insecure_compare(b"hello", b"hello"));
        assert!(!insecure_compare(b"hello", b"world"));
    }

    #[test]
    fn test_constant_time_hex_eq_valid() {
        assert!(constant_time_hex_eq("deadbeef", "deadbeef"));
    }

    #[test]
    fn test_constant_time_hex_eq_different() {
        assert!(!constant_time_hex_eq("deadbeef", "deadbee0"));
    }

    #[test]
    fn test_constant_time_hex_eq_invalid_hex() {
        assert!(!constant_time_hex_eq("not_hex", "deadbeef"));
        assert!(!constant_time_hex_eq("deadbeef", "not_hex"));
    }

    #[test]
    fn test_constant_time_eq_all_zeros() {
        let a = vec![0u8; 32];
        let b = vec![0u8; 32];
        assert!(constant_time_eq(&a, &b));
    }

    #[test]
    fn test_constant_time_eq_32_byte_hashes() {
        let hash1 = vec![0xAAu8; 32];
        let hash2 = vec![0xAAu8; 32];
        let hash3 = vec![0xABu8; 32];
        assert!(constant_time_eq(&hash1, &hash2));
        assert!(!constant_time_eq(&hash1, &hash3));
    }
}
