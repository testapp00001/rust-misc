//! # Lesson 06: Timing Attack on Password Comparison
//!
//! ## The Attack
//!
//! When comparing a computed hash with a stored hash, naive string comparison
//! short-circuits on the first differing byte:
//!
//! ```ignore
//! if computed_hash == stored_hash { ... }
//! ```
//!
//! If the first byte differs, the comparison returns immediately.
//! If the first byte matches but the second differs, it takes slightly longer.
//! An attacker who can measure response times can guess the hash byte-by-byte:
//!
//! 1. Send password attempt
//! 2. Measure response time
//! 3. Try all 256 values for byte 0, find the one that takes longest
//! 4. Now byte 0 is correct; try all 256 values for byte 1
//! 5. Repeat for all 32 bytes of the hash
//!
//! Total attempts: 256 * 32 = 8,192 instead of 2^256. This is devastating.
//!
//! ## Real-World Impact
//!
//! Timing attacks have been demonstrated against:
//! - SSH servers (OpenSSH had this bug)
//! - Web application login forms
//! - HMAC verification in APIs
//! - JWT token validation
//!
//! Even over a network with jitter, statistical analysis over thousands of
//! requests can extract timing differences of microseconds.
//!
//! ## The Fix: Constant-Time Comparison
//!
//! A constant-time comparison function examines ALL bytes before returning,
//! taking the same amount of time regardless of where the first difference
//! occurs:
//!
//! ```ignore
//! fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
//!     if a.len() != b.len() { return false; }
//!     let mut result = 0u8;
//!     for (x, y) in a.iter().zip(b.iter()) {
//!         result |= x ^ y;  // XOR: 0 if equal, non-zero if different
//!     }
//!     result == 0  // Only check at the end
//! }
//! ```
//!
//! The key insight: `result |= x ^ y` always executes regardless of whether
//! the bytes match. There is no branch to short-circuit on.

use std::time::{Duration, Instant};

/// Exercise 1: Implement a VULNERABLE string comparison (for demonstration only).
///
/// This uses the standard `==` operator which short-circuits.
/// DO NOT use this in production!
///
/// Hints:
/// - Just use `a == b` (intentionally vulnerable)
pub fn vulnerable_compare(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement vulnerable comparison (for demo)")
}

/// Exercise 2: Implement a constant-time comparison function.
///
/// Must examine all bytes before returning. No early exit.
///
/// Hints:
/// - If lengths differ, return false (but still do work to avoid length oracle)
/// - XOR each byte pair, OR the results together
/// - Return whether the accumulator is zero
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement constant-time comparison")
}

/// Exercise 3: Demonstrate that the vulnerable comparison leaks timing info.
///
/// Compare a target hash with two guesses:
/// - guess1: first byte is correct, rest is wrong
/// - guess2: first byte is wrong
///
/// Measure the time for each comparison. The correct-first-byte guess should
/// take measurably longer with the vulnerable comparison.
///
/// Returns (time_for_guess1_ns, time_for_guess2_ns).
///
/// Hints:
/// - Create a 32-byte target
/// - Create guess1 with matching first byte
/// - Create guess2 with different first byte
/// - Time each comparison many iterations (e.g., 100,000)
/// - Use `Instant::now()` and `.elapsed()`
pub fn timing_leak_demo(target: &[u8], iterations: u32) -> (u128, u128) {
    todo!("Implement timing leak demonstration")
}

/// Exercise 4: Demonstrate that constant-time comparison has no timing leak.
///
/// Same as `timing_leak_demo` but using `constant_time_compare`.
/// The two timings should be approximately equal.
///
/// Returns (time_for_guess1_ns, time_for_guess2_ns).
pub fn constant_time_demo(target: &[u8], iterations: u32) -> (u128, u128) {
    todo!("Implement constant-time comparison demo")
}

/// Exercise 5: Implement a hash comparison that uses constant-time comparison.
///
/// Compute SHA-256 of the input, then compare with the expected hash
/// using constant-time comparison.
///
/// Hints:
/// - Use `ring::digest` to compute SHA-256
/// - Use `constant_time_compare` for comparison
pub fn secure_hash_verify(input: &[u8], expected_hash: &[u8]) -> bool {
    todo!("Implement secure hash verification")
}

/// Exercise 6: Implement a "double HMAC" comparison technique.
///
/// An alternative defense: HMAC a random key with both values, then compare.
/// Even if the HMAC comparison had a timing leak (it shouldn't), the attacker
/// learns nothing about the original values.
///
/// Hints:
/// - Generate a random HMAC key
/// - HMAC the key with value_a
/// - HMAC the key with value_b
/// - Compare the two HMACs with constant-time comparison
pub fn double_hmac_compare(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement double-HMAC comparison")
}

/// Exercise 7: Create a hash comparison that also avoids length oracles.
///
/// Even returning false immediately when lengths differ can leak information.
/// This function should take constant time regardless of whether the lengths match.
///
/// Hints:
/// - Always iterate over min(a.len(), b.len()) bytes
/// - Accumulate XOR results
/// - Also check length equality at the end
pub fn no_length_oracle_compare(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement length-oracle-resistant comparison")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_equal() {
        assert!(constant_time_compare(b"hello", b"hello"));
    }

    #[test]
    fn test_constant_time_not_equal() {
        assert!(!constant_time_compare(b"hello", b"world"));
    }

    #[test]
    fn test_constant_time_different_lengths() {
        assert!(!constant_time_compare(b"hello", b"hi"));
    }

    #[test]
    fn test_constant_time_empty() {
        assert!(constant_time_compare(b"", b""));
    }

    #[test]
    fn test_secure_hash_verify_correct() {
        let input = b"test data";
        let hash = ring::digest::digest(&ring::digest::SHA256, input);
        assert!(secure_hash_verify(input, hash.as_ref()));
    }

    #[test]
    fn test_secure_hash_verify_wrong() {
        let hash = ring::digest::digest(&ring::digest::SHA256, b"correct");
        assert!(!secure_hash_verify(b"wrong", hash.as_ref()));
    }

    #[test]
    fn test_double_hmac_equal() {
        assert!(double_hmac_compare(b"secret", b"secret"));
    }

    #[test]
    fn test_double_hmac_not_equal() {
        assert!(!double_hmac_compare(b"secret", b"different"));
    }

    #[test]
    fn test_no_length_oracle_equal() {
        assert!(no_length_oracle_compare(b"same", b"same"));
    }

    #[test]
    fn test_no_length_oracle_different() {
        assert!(!no_length_oracle_compare(b"abc", b"xyz"));
    }

    #[test]
    fn test_no_length_oracle_different_lengths() {
        assert!(!no_length_oracle_compare(b"short", b"longer value"));
    }

    #[test]
    fn test_vulnerable_compare_works() {
        assert!(vulnerable_compare(b"test", b"test"));
        assert!(!vulnerable_compare(b"test", b"fail"));
    }
}
