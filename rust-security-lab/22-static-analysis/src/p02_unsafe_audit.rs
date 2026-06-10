//! # Lesson 02: Unsafe Code Audit with cargo-geiger
//!
//! ## The Problem
//!
//! `unsafe` code bypasses Rust's safety guarantees. Every `unsafe` block is a
//! potential location for undefined behavior, memory corruption, or data races.
//! The more `unsafe` you have, the more manual auditing you need.
//!
//! ```rust,ignore
//! // This compiles and runs, but is UB:
//! unsafe {
//!     let ptr = 0x1 as *const u8;
//!     let val = *ptr;  // dangling pointer dereference!
//! }
//! ```
//!
//! ## cargo-geiger
//!
//! `cargo-geiger` counts `unsafe` usage across your entire dependency tree:
//!
//! ```text
//! Crate           | safe | unsafe | unsafe %
//! ----------------|------|--------|--------
//! ring            |  120 |     45 |   27.3%
//! sha2            |   80 |      2 |    2.4%
//! zeroize         |   15 |      8 |   34.8%
//! serde           |  200 |     10 |    4.8%
//! ```
//!
//! ## Attack: Unsafe Code Exploits
//!
//! The `ring` crate uses `unsafe` for FFI calls to BoringSSL. If there is a bug
//! in the unsafe boundary, an attacker might:
//! - Read uninitialized memory (information leak)
//! - Write past buffer boundaries (code execution)
//! - Trigger data races (corruption)
//!
//! ## Defense: Audit, Minimize, Verify
//!
//! 1. Run `cargo-geiger` to identify all unsafe usage
//! 2. For each `unsafe` block: is it actually necessary?
//! 3. For each `unsafe` block: does it maintain all safety invariants?
//! 4. Use Miri to verify unsafe code (Lesson 03)
//!
//! ## Exercise
//!
//! Implement safe wrappers around operations that commonly use `unsafe`,
//! and identify where `unsafe` is justified vs. where safe alternatives exist.

/// Exercise 1: Implement a safe wrapper for pointer-based operations.
///
/// Many developers reach for `unsafe` when they need to work with raw pointers.
/// But most pointer operations have safe alternatives.
///
/// Requirements:
/// - Find the first occurrence of `needle` in `haystack`
/// - Return `Some(index)` if found, `None` if not
/// - Use safe Rust iterators, NOT raw pointer arithmetic
pub fn safe_find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    todo!("Find needle in haystack using safe iterator methods")
}

/// Exercise 2: Implement safe unchecked math with explicit bounds checking.
///
/// Some developers use `unsafe { std::hint::unreachable_unchecked() }` for
/// performance. Show you can achieve the same safety guarantees without `unsafe`.
///
/// Requirements:
/// - If `value` is in range [0, max), return `Some(value)`
/// - If `value >= max`, return `None`
/// - NO `unsafe` code
pub fn safe_range_check(value: usize, max: usize) -> Option<usize> {
    todo!("Check bounds without unsafe")
}

/// Exercise 3: Safe transmute alternative using `TryFrom`.
///
/// `unsafe { std::mem::transmute(value) }` is a common source of UB.
/// Safe alternatives exist for most transmute use cases.
///
/// Requirements:
/// - Convert a `u8` to a `bool` safely
/// - Return `Ok(true)` for 1, `Ok(false)` for 0
/// - Return `Err(String)` for any other value
/// - NO `unsafe` code, NO `transmute`
pub fn safe_u8_to_bool(value: u8) -> Result<bool, String> {
    todo!("Convert u8 to bool safely without transmute")
}

/// Exercise 4: Safe byte reinterpretation.
///
/// Converting between byte slices and typed slices via `unsafe` pointer casts
/// is a common source of alignment and padding UB.
///
/// Requirements:
/// - Convert a `&[u8]` to a `u32` (little-endian) safely
/// - Return `Err` if the slice is too short (< 4 bytes)
/// - Use `u32::from_le_bytes()` -- NO `unsafe` pointer casts
pub fn safe_bytes_to_u32_le(bytes: &[u8]) -> Result<u32, String> {
    todo!("Convert bytes to u32 safely using from_le_bytes")
}

/// Exercise 5: Implement a safe version of `get_unchecked`.
///
/// Requirements:
/// - Return `Some(&data[index])` if index is in bounds
/// - Return `None` if out of bounds
/// - Must use bounds checking -- NO `unsafe` code
/// - Demonstrate that bounds checking is the correct alternative to unchecked access
pub fn safe_unchecked_access(data: &[u8], index: usize) -> Option<u8> {
    todo!("Safe alternative to get_unchecked")
}

/// Exercise 6: Audit report: count unsafe blocks in a code snippet.
///
/// This function simulates what cargo-geiger does -- count `unsafe` occurrences.
///
/// Requirements:
/// - Count occurrences of the literal string "unsafe" in `code_snippet`
/// - This is a simple substring count (not a full parser)
/// - Return the count
pub fn count_unsafe_occurrences(code_snippet: &str) -> usize {
    todo!("Count occurrences of 'unsafe' in code snippet")
}

/// Exercise 7: Classify an `unsafe` block as necessary or unnecessary.
///
/// Requirements:
/// - If `reason` contains "ffi" or "extern", return "necessary" (FFI requires unsafe)
/// - If `reason` contains "performance" or "unchecked", return "review" (may be avoidable)
/// - Otherwise return "audit" (needs manual review)
/// - Case-insensitive matching
pub fn classify_unsafe_reason(reason: &str) -> &'static str {
    todo!("Classify whether an unsafe block is justified")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_find_byte_found() {
        let data = b"hello world";
        assert_eq!(safe_find_byte(data, b'w'), Some(6));
    }

    #[test]
    fn test_safe_find_byte_not_found() {
        let data = b"hello world";
        assert_eq!(safe_find_byte(data, b'z'), None);
    }

    #[test]
    fn test_safe_find_byte_first_occurrence() {
        let data = b"banana";
        assert_eq!(safe_find_byte(data, b'a'), Some(1));
    }

    #[test]
    fn test_safe_range_check_in_range() {
        assert_eq!(safe_range_check(5, 10), Some(5));
    }

    #[test]
    fn test_safe_range_check_out_of_range() {
        assert_eq!(safe_range_check(10, 10), None);
    }

    #[test]
    fn test_safe_range_check_zero() {
        assert_eq!(safe_range_check(0, 10), Some(0));
    }

    #[test]
    fn test_safe_u8_to_bool_valid() {
        assert_eq!(safe_u8_to_bool(0).unwrap(), false);
        assert_eq!(safe_u8_to_bool(1).unwrap(), true);
    }

    #[test]
    fn test_safe_u8_to_bool_invalid() {
        assert!(safe_u8_to_bool(2).is_err());
        assert!(safe_u8_to_bool(255).is_err());
    }

    #[test]
    fn test_safe_bytes_to_u32_le_valid() {
        let bytes: [u8; 4] = 0x04030201u32.to_le_bytes();
        assert_eq!(safe_bytes_to_u32_le(&bytes).unwrap(), 0x04030201);
    }

    #[test]
    fn test_safe_bytes_to_u32_le_too_short() {
        assert!(safe_bytes_to_u32_le(&[1, 2, 3]).is_err());
    }

    #[test]
    fn test_safe_unchecked_access_in_bounds() {
        assert_eq!(safe_unchecked_access(&[10, 20, 30], 1), Some(20));
    }

    #[test]
    fn test_safe_unchecked_access_out_of_bounds() {
        assert_eq!(safe_unchecked_access(&[10, 20, 30], 5), None);
    }

    #[test]
    fn test_count_unsafe_occurrences() {
        let code = r#"
            unsafe { do_thing() }
            let x = safe_function();
            unsafe { do_other() }
        "#;
        assert_eq!(count_unsafe_occurrences(code), 2);
    }

    #[test]
    fn test_count_unsafe_occurrences_none() {
        assert_eq!(count_unsafe_occurrences("let x = 42;"), 0);
    }

    #[test]
    fn test_classify_unsafe_reason_ffi() {
        assert_eq!(classify_unsafe_reason("FFI call to C library"), "necessary");
        assert_eq!(classify_unsafe_reason("extern C function"), "necessary");
    }

    #[test]
    fn test_classify_unsafe_reason_performance() {
        assert_eq!(classify_unsafe_reason("Performance optimization"), "review");
        assert_eq!(classify_unsafe_reason("Unchecked indexing for speed"), "review");
    }

    #[test]
    fn test_classify_unsafe_reason_unknown() {
        assert_eq!(classify_unsafe_reason("legacy code"), "audit");
    }
}
