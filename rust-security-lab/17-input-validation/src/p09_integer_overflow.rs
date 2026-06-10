//! # Lesson 09: Integer Overflow
//!
//! ## The Problem
//!
//! Integer overflow occurs when an arithmetic operation produces a result
//! that exceeds the range of the data type. In many languages (C, C++,
//! older Rust), this wraps silently or causes undefined behavior.
//!
//! In security contexts, integer overflow leads to:
//!
//! 1. **Buffer overflows**: A calculated buffer size wraps to a small value,
//!    then a large copy overflows the buffer
//! 2. **Authentication bypass**: A balance check wraps negative to positive
//! 3. **Allocation failures**: A size calculation wraps, allocating a tiny
//!    buffer for a large operation
//!
//! ## Classic Example: Buffer Size Calculation
//!
//! ```ignore
//! // C code (vulnerable)
//! int total_size = num_items * item_size;  // OVERFLOW if num_items is large!
//! char *buffer = malloc(total_size);       // Small allocation
//! memcpy(buffer, src, num_items * item_size); // Buffer overflow!
//! ```
//!
//! ## Rust's Approach
//!
//! Rust detects integer overflow in debug mode (panics) but wraps silently
//! in release mode. For security-critical code, you must use explicit
//! checked arithmetic:
//!
//! ```ignore
//! let result = a.checked_add(b).ok_or("overflow")?;
//! let result = a.checked_mul(b).ok_or("overflow")?;
//! ```
//!
//! ## Defense
//!
//! 1. Use checked arithmetic (`checked_add`, `checked_mul`, etc.)
//! 2. Validate inputs before arithmetic operations
//! 3. Use `saturating_*` methods where wrapping to MAX is acceptable
//! 4. Use `try_from` for fallible type conversions

/// Safe addition that returns an error on overflow.
///
/// Use `u64::checked_add` to detect overflow and return an error
/// instead of wrapping.
pub fn safe_add(a: u64, b: u64) -> Result<u64, String> {
    todo!("Implement safe addition with overflow detection")
}

/// Safe multiplication that returns an error on overflow.
pub fn safe_mul(a: u64, b: u64) -> Result<u64, String> {
    todo!("Implement safe multiplication with overflow detection")
}

/// Calculate buffer size safely: num_items * item_size + header_size.
///
/// This is the pattern seen in network protocol parsers, file format
/// readers, and any code that calculates memory allocation sizes.
///
/// Returns Ok(size) if all operations succeed without overflow,
/// Err(message) if any operation would overflow.
pub fn safe_buffer_size(num_items: u64, item_size: u64, header_size: u64) -> Result<u64, String> {
    todo!("Calculate buffer size with overflow protection")
}

/// Safely convert a u64 to usize, handling platform differences.
///
/// On 32-bit systems, a u64 value > 4GB cannot fit in a usize.
/// Use `usize::try_from()` to detect this.
pub fn safe_to_usize(value: u64) -> Result<usize, String> {
    todo!("Convert u64 to usize safely")
}

/// Parse a string to an integer with overflow detection.
///
/// Unlike `str::parse::<i64>()`, this also checks that the parsed value
/// is within the given range [min, max].
///
/// Returns Ok(value) if parsing succeeds and the value is in range,
/// Err(message) otherwise.
pub fn parse_bounded_int(input: &str, min: i64, max: i64) -> Result<i64, String> {
    todo!("Parse string to bounded integer")
}

/// Calculate the sum of a vector of u32 values with overflow detection.
///
/// Returns Ok(sum) if no overflow occurs, Err(message) if the sum
/// would exceed u32::MAX.
pub fn safe_sum_u32(values: &[u32]) -> Result<u32, String> {
    todo!("Sum a vector of u32 with overflow detection")
}

/// Calculate index offset: base_index + offset * element_size.
///
/// This pattern is common in array indexing with stride calculations.
/// Must check each operation for overflow.
///
/// Returns Ok(index) if safe, Err(message) on overflow.
pub fn safe_index_offset(base: usize, offset: usize, element_size: usize) -> Result<usize, String> {
    todo!("Calculate safe index offset")
}

/// Simulate a vulnerable allocation size calculation.
///
/// This demonstrates how integer overflow leads to buffer overflow:
///
/// 1. Read `num_items` (attacker-controlled)
/// 2. Calculate `total = num_items * 8`
/// 3. If total overflows, a tiny buffer is allocated
/// 4. Copying num_items * 8 bytes overflows the buffer
///
/// Returns the vulnerable (wrapped) size and the safe (checked) result.
/// This is for educational purposes -- show how wrapping produces a
/// dangerously small value.
pub fn demonstrate_size_overflow(num_items: u64) -> (u64, Result<u64, String>) {
    let vulnerable = num_items.wrapping_mul(8);
    let safe = safe_mul(num_items, 8);
    (vulnerable, safe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_add_normal() {
        assert_eq!(safe_add(10, 20).unwrap(), 30);
    }

    #[test]
    fn test_safe_add_overflow() {
        assert!(safe_add(u64::MAX, 1).is_err());
    }

    #[test]
    fn test_safe_mul_normal() {
        assert_eq!(safe_mul(100, 200).unwrap(), 20000);
    }

    #[test]
    fn test_safe_mul_overflow() {
        assert!(safe_mul(u64::MAX, 2).is_err());
    }

    #[test]
    fn test_buffer_size_normal() {
        assert_eq!(safe_buffer_size(10, 100, 8).unwrap(), 1008);
    }

    #[test]
    fn test_buffer_size_overflow() {
        // u64::MAX / 8 items * 8 + header = overflow
        assert!(safe_buffer_size(u64::MAX / 4, 8, 0).is_err());
    }

    #[test]
    fn test_safe_to_usize_normal() {
        assert_eq!(safe_to_usize(42).unwrap(), 42);
    }

    #[test]
    fn test_parse_bounded_int_valid() {
        assert_eq!(parse_bounded_int("42", 0, 100).unwrap(), 42);
    }

    #[test]
    fn test_parse_bounded_int_out_of_range() {
        assert!(parse_bounded_int("200", 0, 100).is_err());
    }

    #[test]
    fn test_parse_bounded_int_not_a_number() {
        assert!(parse_bounded_int("abc", 0, 100).is_err());
    }

    #[test]
    fn test_safe_sum_normal() {
        assert_eq!(safe_sum_u32(&[100, 200, 300]).unwrap(), 600);
    }

    #[test]
    fn test_safe_sum_overflow() {
        assert!(safe_sum_u32(&[u32::MAX, 1]).is_err());
    }

    #[test]
    fn test_safe_index_offset_normal() {
        assert_eq!(safe_index_offset(100, 5, 4).unwrap(), 120);
    }

    #[test]
    fn test_safe_index_offset_overflow() {
        assert!(safe_index_offset(usize::MAX, 1, 1).is_err());
    }

    #[test]
    fn test_demonstrate_size_overflow() {
        // When num_items is very large, wrapping_mul produces a small number
        let num = u64::MAX / 2 + 1;
        let (vulnerable, safe) = demonstrate_size_overflow(num);
        // The vulnerable result wraps to a small value
        assert!(vulnerable < num, "Wrapped result should be smaller than input");
        // The safe result is an error
        assert!(safe.is_err(), "Safe version should detect overflow");
    }
}
