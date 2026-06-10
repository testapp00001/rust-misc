//! # Lesson 04: Kani Formal Verification
//!
//! ## The Problem
//!
//! Testing checks specific inputs. Formal verification proves properties hold
//! for ALL inputs. Consider a bounds check:
//!
//! ```text
//! Testing:   "We tested 1000 values and it never panicked"
//!             (what about value 1001?)
//!
//! Kani:      "For ALL possible u32 values, this function never panics"
//!             (mathematical proof)
//! ```
//!
//! ## Kani
//!
//! Kani uses **model checking** to prove properties about Rust code. It verifies
//! that specified properties hold for ALL possible inputs, not just tested ones.
//!
//! ```rust,ignore
//! #[cfg(kani)]
//! #[kani::proof]
//! fn verify_bounds_check() {
//!     let input: u32 = kani::any();
//!     let result = safe_divide(input, 10);
//!     assert!(result <= input / 10);
//! }
//! ```
//!
//! ## What Kani Proves
//!
//! | Property | Description |
//! |----------|-------------|
//! | No panics | Function never panics on any input |
//! | Bounds safety | Array access is always in bounds |
//! | Overflow safety | Arithmetic never overflows |
//! | Assertion holds | All `assert!` macros pass |
//! | Postconditions | Output satisfies specified constraints |
//!
//! ## Security Application
//!
//! In security code, Kani can prove:
//! - A padding check never has timing side channels
//! - A bounds check is never bypassed
//! - An integer calculation never overflows
//! - A crypto operation always produces correct output size
//!
//! ## Exercise
//!
//! Implement functions whose properties can be verified, and write Kani-style
//! verification harnesses (as regular tests that check the same properties).

/// Exercise 1: Implement a division function that never panics.
///
/// Kani would verify this for all possible u64 inputs.
///
/// Requirements:
/// - Return `Some(a / b)` if `b != 0`
/// - Return `None` if `b == 0`
/// - Must handle ALL u64 values including edge cases
pub fn safe_divide(a: u64, b: u64) -> Option<u64> {
    todo!("Implement safe division that Kani can verify is panic-free")
}

/// Exercise 2: Implement array access that is provably safe.
///
/// Kani can verify that this function never panics.
///
/// Requirements:
/// - Return `Some(data[index])` if `index < data.len()`
/// - Return `None` otherwise
/// - The proof: for all possible `index` values, this function never panics
pub fn provable_array_access(data: &[u8], index: usize) -> Option<u8> {
    todo!("Implement bounds-checked array access")
}

/// Exercise 3: Implement a function with a verifiable postcondition.
///
/// Requirements:
/// - Clamp `value` to the range `[min, max]`
/// - Postcondition: result >= min AND result <= max
/// - If `min > max`, return `min` (degenerate case)
pub fn clamp_value(value: u64, min: u64, max: u64) -> u64 {
    todo!("Implement clamping with verifiable postcondition")
}

/// Exercise 4: Implement checked buffer copy.
///
/// Requirements:
/// - Copy bytes from `src` to a new `Vec<u8>`
/// - If `src.len() > max_len`, return `Err`
/// - Otherwise return `Ok(copy of src)`
/// - Invariant: result length is always <= max_len
pub fn checked_copy(src: &[u8], max_len: usize) -> Result<Vec<u8>, String> {
    todo!("Implement checked buffer copy with length invariant")
}

/// Exercise 5: Implement a function that Kani proves is constant-time.
///
/// Requirements:
/// - Compare two byte slices
/// - Accumulate differences with OR: `diff |= a ^ b`
/// - Return `true` only if all bytes match
/// - The proof: no early return based on data values
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement constant-time comparison")
}

/// Exercise 6: Implement verified modular arithmetic.
///
/// Requirements:
/// - Compute `(a + b) % modulus`
/// - If modulus is 0, return `Err`
/// - Must not overflow (use u128 intermediate or checked arithmetic)
/// - Postcondition: result < modulus (when modulus > 0)
pub fn verified_mod_add(a: u64, b: u64, modulus: u64) -> Result<u64, String> {
    todo!("Implement overflow-safe modular addition")
}

/// Exercise 7: Implement a size validator for security-critical allocations.
///
/// Requirements:
/// - Validate that an allocation of `count * element_size` bytes is safe
/// - Return `Ok(total_bytes)` if the multiplication does not overflow AND
///   total does not exceed `max_allocation`
/// - Return `Err` otherwise
/// - Kani proof: the returned size is always <= max_allocation and does not overflow
pub fn validate_allocation_size(
    count: usize,
    element_size: usize,
    max_allocation: usize,
) -> Result<usize, String> {
    todo!("Validate allocation size with overflow and limit checks")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_divide_normal() {
        assert_eq!(safe_divide(100, 10), Some(10));
    }

    #[test]
    fn test_safe_divide_by_zero() {
        assert_eq!(safe_divide(100, 0), None);
    }

    #[test]
    fn test_safe_divide_large() {
        assert_eq!(safe_divide(u64::MAX, u64::MAX), Some(1));
    }

    #[test]
    fn test_provable_array_access_in_bounds() {
        assert_eq!(provable_array_access(&[10, 20, 30], 1), Some(20));
    }

    #[test]
    fn test_provable_array_access_out_of_bounds() {
        assert_eq!(provable_array_access(&[10, 20, 30], 5), None);
    }

    #[test]
    fn test_provable_array_access_empty() {
        assert_eq!(provable_array_access(&[], 0), None);
    }

    #[test]
    fn test_clamp_value_in_range() {
        assert_eq!(clamp_value(5, 0, 10), 5);
    }

    #[test]
    fn test_clamp_value_above_max() {
        assert_eq!(clamp_value(15, 0, 10), 10);
    }

    #[test]
    fn test_clamp_value_below_min() {
        assert_eq!(clamp_value(0, 5, 10), 5);
    }

    #[test]
    fn test_clamp_value_degenerate() {
        assert_eq!(clamp_value(5, 10, 3), 10, "min > max should return min");
    }

    #[test]
    fn test_checked_copy_valid() {
        let result = checked_copy(b"hello", 10).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_checked_copy_too_large() {
        assert!(checked_copy(b"hello world", 5).is_err());
    }

    #[test]
    fn test_checked_copy_exact_size() {
        let result = checked_copy(b"hello", 5).unwrap();
        assert_eq!(result, b"hello");
    }

    #[test]
    fn test_constant_time_eq_equal() {
        assert!(constant_time_eq(b"secret", b"secret"));
    }

    #[test]
    fn test_constant_time_eq_not_equal() {
        assert!(!constant_time_eq(b"secret", b"Secret"));
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        assert!(!constant_time_eq(b"abc", b"abcd"));
    }

    #[test]
    fn test_verified_mod_add_normal() {
        assert_eq!(verified_mod_add(7, 8, 10).unwrap(), 5);
    }

    #[test]
    fn test_verified_mod_add_no_overflow() {
        // u64::MAX + 1 = 2^64, 2^64 % 100 = 16
        assert_eq!(verified_mod_add(u64::MAX, 1, 100).unwrap(), 16);
    }

    #[test]
    fn test_verified_mod_add_zero_modulus() {
        assert!(verified_mod_add(1, 2, 0).is_err());
    }

    #[test]
    fn test_validate_allocation_size_valid() {
        assert_eq!(validate_allocation_size(100, 32, 4096).unwrap(), 3200);
    }

    #[test]
    fn test_validate_allocation_size_exceeds_max() {
        assert!(validate_allocation_size(1000, 32, 1024).is_err());
    }

    #[test]
    fn test_validate_allocation_size_overflow() {
        assert!(validate_allocation_size(usize::MAX, 2, usize::MAX).is_err());
    }
}
