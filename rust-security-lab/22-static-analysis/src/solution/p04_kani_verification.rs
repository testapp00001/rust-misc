//! # Lesson 04: Kani Verification (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Safe division that never panics.
pub fn safe_divide(a: u64, b: u64) -> Option<u64> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

/// Bounds-checked array access.
pub fn provable_array_access(data: &[u8], index: usize) -> Option<u8> {
    data.get(index).copied()
}

/// Clamp value to range [min, max].
pub fn clamp_value(value: u64, min: u64, max: u64) -> u64 {
    if min > max {
        return min;
    }
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Checked buffer copy with length invariant.
pub fn checked_copy(src: &[u8], max_len: usize) -> Result<Vec<u8>, String> {
    if src.len() > max_len {
        Err(format!(
            "Source length {} exceeds max {}",
            src.len(),
            max_len
        ))
    } else {
        Ok(src.to_vec())
    }
}

/// Constant-time byte comparison.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Overflow-safe modular addition.
pub fn verified_mod_add(a: u64, b: u64, modulus: u64) -> Result<u64, String> {
    if modulus == 0 {
        return Err("Modulus cannot be zero".to_string());
    }
    // Use u128 to avoid overflow
    let sum = (a as u128) + (b as u128);
    Ok((sum % (modulus as u128)) as u64)
}

/// Validate allocation size with overflow and limit checks.
pub fn validate_allocation_size(
    count: usize,
    element_size: usize,
    max_allocation: usize,
) -> Result<usize, String> {
    let total = count
        .checked_mul(element_size)
        .ok_or_else(|| format!("Overflow: {} * {} exceeds usize::MAX", count, element_size))?;

    if total > max_allocation {
        Err(format!(
            "Total {} exceeds max allocation {}",
            total, max_allocation
        ))
    } else {
        Ok(total)
    }
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
