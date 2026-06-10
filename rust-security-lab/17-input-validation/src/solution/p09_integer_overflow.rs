//! # Lesson 09: Integer Overflow (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

pub fn safe_add(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b).ok_or_else(|| {
        format!("Overflow: {} + {} exceeds u64::MAX", a, b)
    })
}

pub fn safe_mul(a: u64, b: u64) -> Result<u64, String> {
    a.checked_mul(b).ok_or_else(|| {
        format!("Overflow: {} * {} exceeds u64::MAX", a, b)
    })
}

pub fn safe_buffer_size(num_items: u64, item_size: u64, header_size: u64) -> Result<u64, String> {
    let data_size = safe_mul(num_items, item_size)?;
    safe_add(data_size, header_size)
}

pub fn safe_to_usize(value: u64) -> Result<usize, String> {
    usize::try_from(value).map_err(|_| {
        format!("Value {} does not fit in usize ({} bits)", value, usize::BITS)
    })
}

pub fn parse_bounded_int(input: &str, min: i64, max: i64) -> Result<i64, String> {
    let value: i64 = input
        .parse()
        .map_err(|e| format!("Failed to parse '{}': {}", input, e))?;

    if value < min || value > max {
        return Err(format!(
            "Value {} is out of range [{}, {}]",
            value, min, max
        ));
    }

    Ok(value)
}

pub fn safe_sum_u32(values: &[u32]) -> Result<u32, String> {
    let mut sum: u32 = 0;
    for &val in values {
        sum = sum.checked_add(val).ok_or_else(|| {
            format!("Overflow: sum {} + {} exceeds u32::MAX", sum, val)
        })?;
    }
    Ok(sum)
}

pub fn safe_index_offset(base: usize, offset: usize, element_size: usize) -> Result<usize, String> {
    let stride = offset
        .checked_mul(element_size)
        .ok_or_else(|| format!("Overflow: offset {} * element_size {}", offset, element_size))?;

    base.checked_add(stride)
        .ok_or_else(|| format!("Overflow: base {} + stride {}", base, stride))
}

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
        // u64::MAX/2 + 1 * 8 wraps to a small value
        let num = u64::MAX / 2 + 1;
        let (vulnerable, safe) = demonstrate_size_overflow(num);
        assert!(vulnerable < num, "Wrapped result should be smaller than input: {} vs {}", vulnerable, num);
        assert!(safe.is_err(), "Safe version should detect overflow");
    }
}
