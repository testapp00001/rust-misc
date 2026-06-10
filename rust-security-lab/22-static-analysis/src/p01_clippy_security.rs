//! # Lesson 01: Clippy Security Lints
//!
//! ## The Problem
//!
//! Clippy is Rust's built-in linter, and it ships with security-relevant lints
//! that are **disabled by default**. Most teams never enable them. This means
//! common security anti-patterns compile without warning:
//!
//! ```rust,ignore
//! // These all compile silently with default Clippy settings:
//! let key = std::fs::read("key.pem").unwrap();           // panics on error
//! let data = parse_input(user_input).expect("parsed");   // panics on error
//! let total = a + b;                                      // integer overflow
//! ```
//!
//! ## Attack: Panic-Induced Denial of Service
//!
//! An attacker sends malformed input that causes `.unwrap()` to panic. If the
//! panic is not caught, the entire process crashes. In a web server, this means
//! denial of service for ALL users. Each `.unwrap()` is a potential crash site
//! that an attacker can target.
//!
//! ## The Defense: Enable Security Lints
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [lints.clippy]
//! unwrap_used = "deny"
//! expect_used = "deny"
//! integer_arithmetic = "deny"
//! ```
//!
//! Or in code:
//! ```rust,ignore
//! #![deny(clippy::unwrap_used)]
//! #![deny(clippy::expect_used)]
//! #![deny(clippy::integer_arithmetic)]
//! ```
//!
//! ## Key Lints
//!
//! | Lint | What it catches | Severity |
//! |------|----------------|----------|
//! | `unwrap_used` | `.unwrap()` calls that panic on `None`/`Err` | High |
//! | `expect_used` | `.expect()` calls that panic on `None`/`Err` | High |
//! | `integer_arithmetic` | `+`, `-`, `*`, `/` on integers (overflow/underflow) | Medium |
//! | `indexing_slicing` | `arr[i]` that can panic on out-of-bounds | Medium |
//!
//! ## Exercise
//!
//! Implement safe alternatives that handle errors properly instead of panicking.

use sha2::{Sha256, Digest};

/// Exercise 1: Safe unwrap replacement for file reading.
///
/// An attacker can cause a panic by deleting the file between the existence
/// check and the read. Instead, propagate the error.
///
/// Requirements:
/// - Read the file at `path` into a `String`
/// - Return `Err(String)` on any failure instead of panicking
/// - NEVER use `.unwrap()` or `.expect()` -- that's the whole point
pub fn safe_read_file(path: &str) -> Result<String, String> {
    todo!("Read file safely, propagating errors instead of panicking")
}

/// Exercise 2: Safe integer arithmetic that returns None on overflow.
///
/// Integer overflow in security code can lead to buffer underallocation,
/// incorrect size calculations, or bypass of bounds checks.
///
/// Requirements:
/// - Add `a` and `b` using checked arithmetic
/// - Return `Some(result)` on success, `None` on overflow
/// - NEVER use bare `+` operator (Clippy `integer_arithmetic` lint)
pub fn safe_add(a: u64, b: u64) -> Option<u64> {
    todo!("Use checked_add instead of + operator")
}

/// Exercise 3: Safe integer subtraction.
///
/// Requirements:
/// - Subtract `b` from `a` using checked arithmetic
/// - Return `Some(result)` on success, `None` on underflow
pub fn safe_sub(a: u64, b: u64) -> Option<u64> {
    todo!("Use checked_sub instead of - operator")
}

/// Exercise 4: Safe integer multiplication.
///
/// Requirements:
/// - Multiply `a` and `b` using checked arithmetic
/// - Return `Some(result)` on success, `None` on overflow
pub fn safe_mul(a: u64, b: u64) -> Option<u64> {
    todo!("Use checked_mul instead of * operator")
}

/// Exercise 5: Safe slice indexing.
///
/// An attacker who controls an index can cause a panic via out-of-bounds access.
///
/// Requirements:
/// - Return `Some(&data[index])` if index is valid
/// - Return `None` if index is out of bounds
/// - NEVER use `data[index]` directly
pub fn safe_get(data: &[u8], index: usize) -> Option<u8> {
    todo!("Use .get() instead of direct indexing")
}

/// Exercise 6: Hash input data safely, returning hex string.
///
/// Requirements:
/// - Hash `input` with SHA-256
/// - Return the hex-encoded hash as a String
/// - Use `format!("{:02x}", byte)` for hex encoding (no unwrap needed)
pub fn safe_hash_hex(input: &[u8]) -> String {
    todo!("Hash input with SHA-256 and return hex string")
}

/// Exercise 7: Parse a u32 from a string safely.
///
/// Requirements:
/// - Parse `s` as a u32
/// - Return `Ok(value)` on success, `Err(String)` on failure
/// - NEVER use `.unwrap()` or `.parse::<u32>().unwrap()`
pub fn safe_parse_u32(s: &str) -> Result<u32, String> {
    todo!("Parse u32 safely, returning error on invalid input")
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
