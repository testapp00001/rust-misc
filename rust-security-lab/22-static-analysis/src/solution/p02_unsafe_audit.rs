//! # Lesson 02: Unsafe Code Audit (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Find first occurrence of needle in haystack using safe iterators.
pub fn safe_find_byte(haystack: &[u8], needle: u8) -> Option<usize> {
    haystack.iter().position(|&b| b == needle)
}

/// Check bounds without unsafe.
pub fn safe_range_check(value: usize, max: usize) -> Option<usize> {
    if value < max {
        Some(value)
    } else {
        None
    }
}

/// Convert u8 to bool safely without transmute.
pub fn safe_u8_to_bool(value: u8) -> Result<bool, String> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(format!("Invalid boolean value: {} (expected 0 or 1)", value)),
    }
}

/// Convert bytes to u32 safely using from_le_bytes.
pub fn safe_bytes_to_u32_le(bytes: &[u8]) -> Result<u32, String> {
    if bytes.len() < 4 {
        return Err(format!("Expected 4 bytes, got {}", bytes.len()));
    }
    let arr = [bytes[0], bytes[1], bytes[2], bytes[3]];
    Ok(u32::from_le_bytes(arr))
}

/// Safe alternative to get_unchecked with bounds checking.
pub fn safe_unchecked_access(data: &[u8], index: usize) -> Option<u8> {
    data.get(index).copied()
}

/// Count occurrences of 'unsafe' in a code snippet.
pub fn count_unsafe_occurrences(code_snippet: &str) -> usize {
    code_snippet.matches("unsafe").count()
}

/// Classify whether an unsafe block is justified.
pub fn classify_unsafe_reason(reason: &str) -> &'static str {
    let lower = reason.to_lowercase();
    if lower.contains("ffi") || lower.contains("extern") {
        "necessary"
    } else if lower.contains("performance") || lower.contains("unchecked") {
        "review"
    } else {
        "audit"
    }
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
