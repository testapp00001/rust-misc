//! # Lesson 06: Timing Attack on Password Comparison (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::time::Instant;
use ring::{digest, hmac};

/// A VULNERABLE string comparison (for demonstration only).
pub fn vulnerable_compare(a: &[u8], b: &[u8]) -> bool {
    a == b
}

/// Constant-time comparison function.
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Demonstrate that the vulnerable comparison leaks timing info.
pub fn timing_leak_demo(target: &[u8], iterations: u32) -> (u128, u128) {
    let mut guess1 = vec![0u8; target.len()];
    let mut guess2 = vec![0u8; target.len()];
    if !target.is_empty() {
        guess1[0] = target[0]; // First byte correct
        guess2[0] = target[0].wrapping_add(1); // First byte wrong
    }

    // Warm up
    for _ in 0..1000 {
        let _ = vulnerable_compare(target, &guess1);
        let _ = vulnerable_compare(target, &guess2);
    }

    let start1 = Instant::now();
    for _ in 0..iterations {
        let _ = vulnerable_compare(target, &guess1);
    }
    let time1 = start1.elapsed().as_nanos();

    let start2 = Instant::now();
    for _ in 0..iterations {
        let _ = vulnerable_compare(target, &guess2);
    }
    let time2 = start2.elapsed().as_nanos();

    (time1, time2)
}

/// Demonstrate that constant-time comparison has no timing leak.
pub fn constant_time_demo(target: &[u8], iterations: u32) -> (u128, u128) {
    let mut guess1 = vec![0u8; target.len()];
    let mut guess2 = vec![0u8; target.len()];
    if !target.is_empty() {
        guess1[0] = target[0];
        guess2[0] = target[0].wrapping_add(1);
    }

    // Warm up
    for _ in 0..1000 {
        let _ = constant_time_compare(target, &guess1);
        let _ = constant_time_compare(target, &guess2);
    }

    let start1 = Instant::now();
    for _ in 0..iterations {
        let _ = constant_time_compare(target, &guess1);
    }
    let time1 = start1.elapsed().as_nanos();

    let start2 = Instant::now();
    for _ in 0..iterations {
        let _ = constant_time_compare(target, &guess2);
    }
    let time2 = start2.elapsed().as_nanos();

    (time1, time2)
}

/// Hash comparison that uses constant-time comparison.
pub fn secure_hash_verify(input: &[u8], expected_hash: &[u8]) -> bool {
    let computed = digest::digest(&digest::SHA256, input);
    constant_time_compare(computed.as_ref(), expected_hash)
}

/// Double HMAC comparison technique.
pub fn double_hmac_compare(a: &[u8], b: &[u8]) -> bool {
    let key_bytes: [u8; 32] = rand::random();
    let key = hmac::Key::new(hmac::HMAC_SHA256, &key_bytes);
    let tag_a = hmac::sign(&key, a);
    let tag_b = hmac::sign(&key, b);
    constant_time_compare(tag_a.as_ref(), tag_b.as_ref())
}

/// Hash comparison that also avoids length oracles.
pub fn no_length_oracle_compare(a: &[u8], b: &[u8]) -> bool {
    let min_len = a.len().min(b.len());
    let mut result = 0u8;
    for i in 0..min_len {
        result |= a[i] ^ b[i];
    }
    // Also check that lengths match (but only after doing the work)
    result == 0 && a.len() == b.len()
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
