//! # Lesson 05: Constant-Time Comparison
//!
//! ## The Timing Attack
//!
//! When comparing two byte slices with `==`, most implementations return `false`
//! as soon as they find the first differing byte. This means:
//!
//! ```text
//! Comparing "abcdef" with "aXXXXX" → fails at byte 1 (fast)
//! Comparing "abcdef" with "abcdeX" → fails at byte 5 (slower)
//! Comparing "abcdef" with "abcdef" → succeeds (slowest)
//! ```
//!
//! An attacker can measure the time it takes to get a response and guess the
//! correct value byte by byte. This is a **timing side-channel attack**.
//!
//! ## Real-World Impact
//!
//! - **HMAC verification**: Attacker guesses the authentication tag byte by byte
//! - **Password comparison**: Attacker guesses the password character by character
//! - **API key validation**: Attacker recovers secret tokens
//!
//! ## The Fix: Constant-Time Comparison
//!
//! A constant-time comparison function always takes the same amount of time,
//! regardless of where (or whether) the inputs differ. It does this by:
//!
//! 1. Always examining ALL bytes (no early return)
//! 2. Accumulating differences in a register
//! 3. Returning based on the accumulated result
//!
//! ## Rust's `ring` crate
//!
//! `ring::constant_time::verify_slices_are_equal` provides a constant-time
//! comparison that is resistant to compiler optimizations (it uses volatile
//! operations internally).
//!
//! ## Key Takeaway
//!
//! ALWAYS use constant-time comparison for security-sensitive values:
//! HMACs, passwords, tokens, keys, nonces, etc.

/// Exercise 1: Implement constant-time byte comparison.
///
/// Compare two byte slices and return true only if they are equal.
/// The comparison must take the same time regardless of where the
/// first difference occurs (or if there is no difference).
///
/// Hints:
/// - First check lengths — if different, still do work (don't short-circuit)
/// - XOR each pair of bytes: equal bytes → 0, different bytes → non-zero
/// - OR all XOR results together
/// - Return true only if the final OR is 0
/// - NEVER use `if a != b { return false; }` — that's early exit!
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement constant-time comparison")
}

/// Exercise 2: Use ring's constant-time comparison.
///
/// Hints:
/// - Use `ring::constant_time::verify_slices_are_equal(a, b)`
/// - It returns `Result<(), Unspecified>` — convert to bool
pub fn ring_constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    todo!("Use ring's constant-time comparison")
}

/// Exercise 3: Demonstrate why `==` is dangerous for security.
///
/// This function uses naive comparison (the INSECURE way).
/// It exists purely for educational purposes — NEVER use this for real security.
///
/// Hints:
/// - Just use `a == b` (the standard slice comparison)
/// - This is vulnerable to timing attacks
pub fn insecure_compare(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement naive (insecure) comparison for demonstration")
}

/// Exercise 4: Constant-time comparison for hex strings.
///
/// Compare two hex-encoded strings in constant time.
///
/// Hints:
/// - Convert both hex strings to bytes first
/// - Then use constant-time byte comparison
/// - Handle invalid hex gracefully (return false)
pub fn constant_time_hex_eq(a: &str, b: &str) -> bool {
    todo!("Implement constant-time hex string comparison")
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
        // Both should give the same result, just different timing characteristics
        assert!(insecure_compare(b"hello", b"hello"));
        assert!(!insecure_compare(b"hello", b"world"));
    }

    #[test]
    fn test_constant_time_hex_eq_valid() {
        let hex1 = "deadbeef";
        let hex2 = "deadbeef";
        assert!(constant_time_hex_eq(hex1, hex2));
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
        // Typical use case: comparing 32-byte HMAC tags
        let hash1 = vec![0xAAu8; 32];
        let hash2 = vec![0xAAu8; 32];
        let hash3 = vec![0xABu8; 32];
        assert!(constant_time_eq(&hash1, &hash2));
        assert!(!constant_time_eq(&hash1, &hash3));
    }
}
