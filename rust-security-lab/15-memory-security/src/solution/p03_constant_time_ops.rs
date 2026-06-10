//! # Lesson 03: Constant-Time Operations (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Constant-time byte slice comparison — no early exit.
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

/// Constant-time conditional select — returns `a` if true, `b` if false.
pub fn conditional_select(condition: bool, a: u8, b: u8) -> u8 {
    let mask = (condition as u8).wrapping_neg(); // true→0xFF, false→0x00
    (a & mask) | (b & !mask)
}

/// Constant-time check if all bytes are zero.
pub fn all_zeros(data: &[u8]) -> bool {
    let mut acc: u8 = 0;
    for &byte in data {
        acc |= byte;
    }
    acc == 0
}

/// Constant-time table lookup — accesses all entries to hide the index.
pub fn constant_time_lookup(table: &[u8; 16], index: u8) -> u8 {
    let mut result: u8 = 0;
    for i in 0..16u8 {
        let mask = ((i ^ index) == 0) as u8; // 1 if i == index, else 0
        // Expand mask to all-ones or all-zeros for selection
        let mask = mask.wrapping_neg();
        result |= table[i as usize] & mask;
    }
    result
}

/// Constant-time conditional memory copy.
pub fn conditional_copy(condition: bool, dst: &mut [u8], src: &[u8]) {
    let len = dst.len().min(src.len());
    for i in 0..len {
        dst[i] = conditional_select(condition, src[i], dst[i]);
    }
}

/// Constant-time byte search — checks all bytes regardless of match position.
pub fn constant_time_contains(data: &[u8], target: u8) -> bool {
    let mut found: u8 = 0;
    for &byte in data {
        // byte ^ target == 0 iff byte == target
        // (!byte.wrapping_sub(target)) wraps to non-zero only when equal
        // Simpler: use XOR-based check
        let is_match = ((byte ^ target) == 0) as u8;
        found |= is_match;
    }
    found != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq_equal() {
        assert!(constant_time_eq(b"hello", b"hello"));
    }

    #[test]
    fn test_constant_time_eq_different() {
        assert!(!constant_time_eq(b"hello", b"world"));
    }

    #[test]
    fn test_constant_time_eq_different_lengths() {
        assert!(!constant_time_eq(b"short", b"longer data"));
    }

    #[test]
    fn test_constant_time_eq_empty() {
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn test_conditional_select_true() {
        assert_eq!(conditional_select(true, 0xAA, 0xBB), 0xAA);
    }

    #[test]
    fn test_conditional_select_false() {
        assert_eq!(conditional_select(false, 0xAA, 0xBB), 0xBB);
    }

    #[test]
    fn test_all_zeros_true() {
        assert!(all_zeros(&[0, 0, 0, 0, 0]));
    }

    #[test]
    fn test_all_zeros_false() {
        assert!(!all_zeros(&[0, 0, 1, 0, 0]));
    }

    #[test]
    fn test_all_zeros_empty() {
        assert!(all_zeros(&[]));
    }

    #[test]
    fn test_constant_time_lookup() {
        let table: [u8; 16] = [10, 20, 30, 40, 50, 60, 70, 80,
                                 90, 100, 110, 120, 130, 140, 150, 160];
        for i in 0..16 {
            assert_eq!(constant_time_lookup(&table, i), table[i as usize]);
        }
    }

    #[test]
    fn test_conditional_copy_true() {
        let src = b"secret";
        let mut dst = [0u8; 6];
        conditional_copy(true, &mut dst, src);
        assert_eq!(&dst, src);
    }

    #[test]
    fn test_conditional_copy_false() {
        let src = b"secret";
        let mut dst = [0xAA; 6];
        conditional_copy(false, &mut dst, src);
        assert_eq!(&dst, &[0xAA; 6]);
    }

    #[test]
    fn test_constant_time_contains_found() {
        assert!(constant_time_contains(b"abcdef", b'd'));
    }

    #[test]
    fn test_constant_time_contains_not_found() {
        assert!(!constant_time_contains(b"abcdef", b'z'));
    }

    #[test]
    fn test_constant_time_contains_empty() {
        assert!(!constant_time_contains(b"", b'a'));
    }
}
