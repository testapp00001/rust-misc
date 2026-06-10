//! # Lesson 03: Miri Basics (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Create a properly initialized zeroed buffer.
pub fn safe_zeroed_buffer(n: usize) -> Vec<u8> {
    vec![0u8; n]
}

/// Return subslice safely with bounds checking.
pub fn safe_subslice(data: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
    let end = offset.checked_add(len)?;
    if end <= data.len() {
        Some(&data[offset..end])
    } else {
        None
    }
}

/// XOR two byte slices safely, up to the shorter length.
pub fn safe_xor(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

/// Check ASCII alphanumeric without unsafe conversion.
pub fn is_ascii_alphanumeric(b: u8) -> bool {
    b.is_ascii_alphanumeric()
}

/// Convert u32 to bytes safely.
pub fn safe_u32_to_be_bytes(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

/// Convert bytes to u64 safely with length validation.
pub fn safe_bytes_to_u64_be(bytes: &[u8]) -> Result<u64, String> {
    if bytes.len() != 8 {
        return Err(format!("Expected 8 bytes, got {}", bytes.len()));
    }
    let arr = [bytes[0], bytes[1], bytes[2], bytes[3],
               bytes[4], bytes[5], bytes[6], bytes[7]];
    Ok(u64::from_be_bytes(arr))
}

/// Check if a 4-byte read at offset would be within bounds.
pub fn is_safe_4byte_read(buffer_len: usize, offset: usize) -> bool {
    offset.checked_add(4).map_or(false, |end| end <= buffer_len)
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
