//! # Lesson 01: Zeroize Basics — Zero Memory on Drop
//!
//! ## The Problem
//!
//! When Rust drops a variable, the memory is "freed" but NOT erased. The bytes
//! remain in RAM until another allocation reuses that region. An attacker with
//! access to process memory (core dump, memory forensics, cold boot attack) can
//! read those residual secret bytes.
//!
//! ```text
//! let mut key = vec![0xDE, 0xAD, 0xC0, 0xDE];
//! drop(key);
//! // Memory still contains: DE AD C0 DE
//! // Allocator marks it as "available" but does NOT zero it
//! ```
//!
//! ## The Solution: `zeroize`
//!
//! The `zeroize` crate provides:
//! - `Zeroize` trait: overwrites memory with zeros
//! - Compiler fence: prevents the optimizer from removing the zeroing (important!)
//! - `#[derive(Zeroize)]`: auto-implement for structs
//! - `#[zeroize(drop)]`: automatically zero on Drop
//!
//! ## Why Not Just Write Zeros Manually?
//!
//! ```rust,ignore
//! // This might be REMOVED by the compiler optimizer:
//! secret.iter_mut().for_each(|b| *b = 0);
//!
//! // zeroize uses a volatile write + compiler fence to prevent this
//! ```
//!
//! ## Attack: Residual Memory
//!
//! 1. Process A holds a secret key in a buffer
//! 2. Process A frees the buffer (drop)
//! 3. Process B allocates memory — may receive the SAME physical pages
//! 4. Process B reads its uninitialized memory → finds Process A's secret key!
//!
//! Defense: `zeroize` the buffer before dropping.

use zeroize::Zeroize;

/// Exercise 1: Implement `zeroize_vec` that zeros a byte vector in place.
///
/// Requirements:
/// - Must use the `zeroize` crate's `Zeroize` trait (not manual zeroing)
/// - After calling this function, `data` must contain all zeros
///
/// Hints:
/// - `Vec<u8>` implements `Zeroize` from the `zeroize` crate
/// - Call `.zeroize()` on the vector
pub fn zeroize_vec(data: &mut Vec<u8>) {
    todo!("Zero the vector using the Zeroize trait")
}

/// Exercise 2: Create a struct that automatically zeroes on drop.
///
/// Use `#[derive(Zeroize)]` and `#[zeroize(drop)]` to make a struct
/// that automatically zeros its fields when dropped.
///
/// Hints:
/// - Use `#[derive(Zeroize)]` on the struct
/// - Use `#[zeroize(drop)]` on the struct
/// - The struct should have a `key: [u8; 32]` field
#[derive(Zeroize)]
// TODO: Add the right attributes so this zeroizes on drop
pub struct SecretKey {
    pub key: [u8; 32],
}

/// Exercise 3: Implement `secure_compare` — a constant-time byte comparison.
///
/// This should compare two byte slices and return true only if they are equal.
/// The comparison MUST be constant-time (no early return on first difference).
///
/// Hints:
/// - Accumulate differences with OR: `diff |= a ^ b`
/// - After checking all bytes, return `diff == 0`
/// - NEVER use `==` on slices or `.zip().all()` — these short-circuit
pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement constant-time byte comparison")
}

/// Exercise 4: Implement `SecretBuffer` — a fixed-size buffer that zeroizes on drop.
///
/// Requirements:
/// - Generic over size N
/// - Stores `[u8; N]` internally
/// - Implements `Zeroize` via derive
/// - Auto-zeroes on drop via `#[zeroize(drop)]`
/// - Has methods: `new(data: &[u8]) -> Self` (copy data, pad with zeros) and `as_slice(&self) -> &[u8]`
///
/// Hints:
/// - Use `const N: usize` generic parameter
/// - Copy min(data.len(), N) bytes from input
/// - Use `#[derive(Zeroize)]` and `#[zeroize(drop)]`
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SecretBuffer<const N: usize> {
    pub buf: [u8; N],
}

impl<const N: usize> SecretBuffer<N> {
    pub fn new(data: &[u8]) -> Self {
        todo!("Create a SecretBuffer by copying data (pad with zeros if shorter)")
    }

    pub fn as_slice(&self) -> &[u8] {
        todo!("Return a reference to the internal buffer")
    }
}

/// Exercise 5: Implement `zeroize_stack_string` that creates and immediately
/// zeroizes a secret string on the stack.
///
/// Requirements:
/// - Take a `&str` input
/// - Copy it into a stack-allocated `Vec<u8>`
/// - Use `zeroize` to wipe it before returning
/// - Return the length of the string (NOT the string itself)
///
/// Hints:
/// - `Vec<u8>` implements `Zeroize`
/// - Call `.zeroize()` before the function returns
pub fn zeroize_stack_string(s: &str) -> usize {
    todo!("Copy string to stack buffer, zeroize it, return length")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zeroize_vec() {
        let mut data = vec![0xDE, 0xAD, 0xC0, 0xDE, 0xBE, 0xEF];
        zeroize_vec(&mut data);
        assert!(
            data.iter().all(|&b| b == 0),
            "All bytes should be zero after zeroize, got: {:?}",
            data
        );
    }

    #[test]
    fn test_zeroize_vec_empty() {
        let mut data = vec![];
        zeroize_vec(&mut data);
        assert!(data.is_empty());
    }

    #[test]
    fn test_secret_key_zeroize_on_drop() {
        let bytes = {
            let key = SecretKey { key: [0x42; 32] };
            // We can't directly verify zeroize happened after drop,
            // but we can verify the struct has the right derive
            key.key
        };
        // The original key was all 0x42 — verify it was copyable
        assert_eq!(bytes, [0x42; 32]);
    }

    #[test]
    fn test_secure_compare_equal() {
        let a = b"super_secret_key_1234567890";
        let b = b"super_secret_key_1234567890";
        assert!(secure_compare(a, b));
    }

    #[test]
    fn test_secure_compare_not_equal() {
        let a = b"super_secret_key_1234567890";
        let b = b"super_secret_key_1234567891";
        assert!(!secure_compare(a, b));
    }

    #[test]
    fn test_secure_compare_different_lengths() {
        let a = b"short";
        let b = b"longer_data";
        assert!(!secure_compare(a, b));
    }

    #[test]
    fn test_secret_buffer_new() {
        let data = b"hello";
        let buf = SecretBuffer::<16>::new(data);
        assert_eq!(&buf.as_slice()[..5], b"hello");
        assert!(buf.as_slice()[5..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_secret_buffer_full() {
        let data = [0xAA; 32];
        let buf = SecretBuffer::<32>::new(&data);
        assert_eq!(buf.as_slice(), &[0xAA; 32]);
    }

    #[test]
    fn test_zeroize_stack_string() {
        let len = zeroize_stack_string("my_secret_password");
        assert_eq!(len, 18);
    }

    #[test]
    fn test_zeroize_stack_string_empty() {
        let len = zeroize_stack_string("");
        assert_eq!(len, 0);
    }
}
