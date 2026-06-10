//! # Lesson 03: Miri for Undefined Behavior Detection
//!
//! ## The Problem
//!
//! Unsafe Rust can introduce **undefined behavior (UB)** that the compiler
//! cannot detect. UB means the compiler makes assumptions about your code that
//! may not hold, leading to:
//!
//! - Reading uninitialized memory
//! - Use-after-free
//! - Data races
//! - Violating aliasing rules
//! - Out-of-bounds pointer arithmetic
//!
//! ```rust,ignore
//! // This compiles without warnings but is UB:
//! let mut x: i32 = 0;
//! let ptr = &mut x as *mut i32;
//! let ptr2 = ptr;  // two mutable pointers to same data
//! unsafe {
//!     *ptr = 1;
//!     *ptr2 = 2;  // UB: violates exclusivity of mutable reference
//! }
//! ```
//!
//! ## Miri
//!
//! Miri is an interpreter for Rust's Mid-level Intermediate Representation (MIR).
//! It detects UB at runtime:
//!
//! ```bash
//! cargo +nightly miri test    # Run tests under Miri
//! cargo +nightly miri run     # Run binary under Miri
//! ```
//!
//! ## What Miri Detects
//!
//! | Category | Example |
//! |----------|---------|
//! | Use-after-free | Dangling pointer dereference |
//! | Out-of-bounds | Pointer arithmetic past allocation |
//! | Uninitialized memory | Reading `MaybeUninit` without init |
//! | Data races | Concurrent unsynchronized access |
//! | Invalid values | `bool` that is not 0 or 1 |
//! | Alignment violations | Unaligned pointer dereference |
//!
//! ## Exercise
//!
//! Implement safe wrappers that demonstrate the patterns Miri would catch,
//! and provide safe alternatives that avoid UB.

/// Exercise 1: Safe initialization pattern (avoiding uninitialized memory).
///
/// A common UB source is reading uninitialized memory. In crypto code, this
/// could leak stack contents as "random" bytes.
///
/// Requirements:
/// - Create a `Vec<u8>` of size `n`, initialized to zeros
/// - Return the zeroed vector
/// - NEVER use `MaybeUninit` or `set_len` tricks
pub fn safe_zeroed_buffer(n: usize) -> Vec<u8> {
    todo!("Create a properly initialized zeroed buffer")
}

/// Exercise 2: Safe pointer arithmetic with bounds checking.
///
/// Pointer arithmetic that goes past the allocation boundary is UB.
///
/// Requirements:
/// - Return a slice starting at `offset` with `len` bytes
/// - Return `None` if the range `[offset, offset+len)` is out of bounds
/// - Use safe slicing, NOT raw pointer arithmetic
pub fn safe_subslice(data: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    todo!("Return subslice safely with bounds checking")
}

/// Exercise 3: Safe concurrent access pattern.
///
/// Data races are UB. Even with `unsafe`, concurrent mutable access to the
/// same data without synchronization is undefined behavior.
///
/// Requirements:
/// - Take two byte slices and return their XOR
/// - This is a safe, single-threaded operation
/// - If slices have different lengths, XOR up to the shorter length
/// - Demonstrate that the safe version has no data race possibility
pub fn safe_xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    todo!("XOR two byte slices safely")
}

/// Exercise 4: Validate that a byte is a valid ASCII alphanumeric character.
///
/// Using `char::from_u32_unchecked` with an invalid value is UB.
///
/// Requirements:
/// - Return `true` if `b` is an ASCII alphanumeric character (a-z, A-Z, 0-9)
/// - Use safe comparisons, NOT `unsafe { char::from_u32_unchecked(b as u32) }`
pub fn is_ascii_alphanumeric(b: u8) -> bool {
    todo!("Check ASCII alphanumeric without unsafe conversion")
}

/// Exercise 5: Safe integer-to-bytes conversion.
///
/// Requirements:
/// - Convert a `u32` to its big-endian byte representation
/// - Return a `[u8; 4]`
/// - Use `u32::to_be_bytes()` -- NOT `unsafe { std::mem::transmute }`
pub fn safe_u32_to_be_bytes(value: u32) -> [u8; 4] {
    todo!("Convert u32 to bytes safely")
}

/// Exercise 6: Safe bytes-to-integer conversion with validation.
///
/// Requirements:
/// - Convert a byte slice to a `u64` (big-endian)
/// - Return `Err` if the slice is not exactly 8 bytes
/// - Use `u64::from_be_bytes()` with a fixed-size array
pub fn safe_bytes_to_u64_be(bytes: &[u8]) -> Result<u64, String> {
    todo!("Convert bytes to u64 safely with length validation")
}

/// Exercise 7: Detect potential UB in a simulated unsafe operation.
///
/// This function simulates checking whether a pointer operation would be valid.
///
/// Requirements:
/// - Given a buffer length and an offset, check if accessing `offset..offset+4`
///   is within bounds
/// - Return `true` if the 4-byte read at `offset` is valid
/// - Return `false` if it would be out-of-bounds (potential UB)
pub fn is_safe_4byte_read(buffer_len: usize, offset: usize) -> bool {
    todo!("Check if a 4-byte read at offset would be within bounds")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_zeroed_buffer() {
        let buf = safe_zeroed_buffer(16);
        assert_eq!(buf.len(), 16);
        assert!(buf.iter().all(|&b| b == 0), "Buffer should be all zeros");
    }

    #[test]
    fn test_safe_zeroed_buffer_empty() {
        let buf = safe_zeroed_buffer(0);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_safe_subslice_valid() {
        let data = b"hello world";
        let sub = safe_subslice(data, 6, 5);
        assert_eq!(sub, Some(b"world".as_slice()));
    }

    #[test]
    fn test_safe_subslice_out_of_bounds() {
        let data = b"hello";
        assert_eq!(safe_subslice(data, 3, 5), None);
    }

    #[test]
    fn test_safe_subslice_offset_at_end() {
        let data = b"hello";
        assert_eq!(safe_subslice(data, 5, 1), None);
    }

    #[test]
    fn test_safe_xor_same_length() {
        let a = [0xFF, 0x00, 0xAA];
        let b = [0xFF, 0xFF, 0x55];
        assert_eq!(safe_xor(&a, &b), vec![0x00, 0xFF, 0xFF]);
    }

    #[test]
    fn test_safe_xor_different_lengths() {
        let a = [0xFF, 0x00];
        let b = [0xFF, 0xFF, 0xFF];
        assert_eq!(safe_xor(&a, &b), vec![0x00, 0xFF]);
    }

    #[test]
    fn test_safe_xor_empty() {
        assert_eq!(safe_xor(&[], &[]), vec![]);
    }

    #[test]
    fn test_is_ascii_alphanumeric_valid() {
        assert!(is_ascii_alphanumeric(b'a'));
        assert!(is_ascii_alphanumeric(b'Z'));
        assert!(is_ascii_alphanumeric(b'5'));
    }

    #[test]
    fn test_is_ascii_alphanumeric_invalid() {
        assert!(!is_ascii_alphanumeric(b'!'));
        assert!(!is_ascii_alphanumeric(b' '));
        assert!(!is_ascii_alphanumeric(0xFF));
    }

    #[test]
    fn test_safe_u32_to_be_bytes() {
        assert_eq!(safe_u32_to_be_bytes(0x01020304), [0x01, 0x02, 0x03, 0x04]);
    }

    #[test]
    fn test_safe_u32_to_be_bytes_zero() {
        assert_eq!(safe_u32_to_be_bytes(0), [0, 0, 0, 0]);
    }

    #[test]
    fn test_safe_bytes_to_u64_be_valid() {
        let bytes: [u8; 8] = 0x0102030405060708u64.to_be_bytes();
        assert_eq!(safe_bytes_to_u64_be(&bytes).unwrap(), 0x0102030405060708);
    }

    #[test]
    fn test_safe_bytes_to_u64_be_wrong_length() {
        assert!(safe_bytes_to_u64_be(&[1, 2, 3]).is_err());
    }

    #[test]
    fn test_is_safe_4byte_read_valid() {
        assert!(is_safe_4byte_read(10, 0));
        assert!(is_safe_4byte_read(10, 6));
    }

    #[test]
    fn test_is_safe_4byte_read_out_of_bounds() {
        assert!(!is_safe_4byte_read(10, 8));
        assert!(!is_safe_4byte_read(4, 1));
    }

    #[test]
    fn test_is_safe_4byte_read_exact_boundary() {
        assert!(!is_safe_4byte_read(4, 4));
        assert!(is_safe_4byte_read(4, 0));
    }
}
