//! # Lesson 10: Side-Channel Attack Defense (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Constant-time conditional select — no branching.
pub fn ct_select(condition: bool, a: u8, b: u8) -> u8 {
    let mask = (condition as u8).wrapping_neg(); // true→0xFF, false→0x00
    (a & mask) | (b & !mask)
}

/// Constant-time equality check for byte slices.
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Constant-time comparison returning ordering (-1, 0, 1).
pub fn ct_memcmp(a: &[u8], b: &[u8]) -> i32 {
    if a.len() != b.len() {
        // For different lengths, compare up to the shorter length
        // then use length difference
        let min_len = a.len().min(b.len());
        let mut result: i32 = 0;
        let mut all_eq = true;
        for i in 0..min_len {
            let ai = a[i];
            let bi = b[i];
            // Check if a[i] < b[i] and we haven't set a result yet
            let lt = ((ai < bi) as i32) & ((all_eq) as i32);
            let gt = ((ai > bi) as i32) & ((all_eq) as i32);
            result -= lt;
            result += gt;
            all_eq &= ai == bi;
        }
        if all_eq {
            // All compared bytes were equal; use length difference
            if a.len() < b.len() { -1 }
            else if a.len() > b.len() { 1 }
            else { 0 }
        } else {
            result
        }
    } else {
        let mut result: i32 = 0;
        let mut all_eq = true;
        for i in 0..a.len() {
            let ai = a[i];
            let bi = b[i];
            let lt = ((ai < bi) as i32) & ((all_eq) as i32);
            let gt = ((ai > bi) as i32) & ((all_eq) as i32);
            result -= lt;
            result += gt;
            all_eq &= ai == bi;
        }
        result
    }
}

/// Constant-time minimum.
pub fn ct_min(a: u8, b: u8) -> u8 {
    ct_select(a < b, a, b)
}

/// Constant-time maximum.
pub fn ct_max(a: u8, b: u8) -> u8 {
    ct_select(a < b, b, a)
}

/// Constant-time zero check — returns 1 if zero, 0 otherwise.
pub fn ct_is_zero(byte: u8) -> u8 {
    // If byte=0: 0-1 underflows to 0xFF (all ones), which is truthy
    // If byte!=0: n-1 is in [0,254], which may or may not be truthy
    // Better approach: OR all bits, then negate
    // byte | (byte>>1) | (byte>>2) | ... will be nonzero if any bit is set
    // Simpler: check if byte == 0 by using the property that
    // (byte | byte.wrapping_neg()) has the sign bit set iff byte != 0
    // For u8: byte.wrapping_neg() = 256 - byte (mod 256)
    // byte | (256-byte) has bit 7 set iff byte != 0
    // Actually simplest: branch-free using arithmetic
    // 0 → 1, anything else → 0
    // Use: ((byte | byte.wrapping_neg()) >> 7) & 1 gives 1 iff byte != 0
    // Then flip: 1 - ((byte | byte.wrapping_neg()) >> 7) & 1
    // Or: !((byte | byte.wrapping_neg()) >> 7) & 1
    let nonzero = ((byte | byte.wrapping_neg()) >> 7) & 1;
    1 - nonzero
}

/// Constant-time element-wise AND for byte slices.
pub fn ct_and(a: &[u8], b: &[u8]) -> Option<Vec<u8>> {
    if a.len() != b.len() {
        return None;
    }
    Some(a.iter().zip(b.iter()).map(|(x, y)| x & y).collect())
}

/// Search for target in data with constant-time loop.
pub fn constant_time_loop(data: &[u8], target: u8) -> bool {
    let mut found: u8 = 0;
    for &byte in data {
        found |= ((byte ^ target) == 0) as u8;
    }
    found != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ct_select_true() {
        assert_eq!(ct_select(true, 0xAA, 0xBB), 0xAA);
    }

    #[test]
    fn test_ct_select_false() {
        assert_eq!(ct_select(false, 0xAA, 0xBB), 0xBB);
    }

    #[test]
    fn test_ct_eq_equal() {
        assert!(ct_eq(b"hello", b"hello"));
    }

    #[test]
    fn test_ct_eq_different() {
        assert!(!ct_eq(b"hello", b"world"));
    }

    #[test]
    fn test_ct_eq_different_lengths() {
        assert!(!ct_eq(b"ab", b"abc"));
    }

    #[test]
    fn test_ct_memcmp_equal() {
        assert_eq!(ct_memcmp(b"abc", b"abc"), 0);
    }

    #[test]
    fn test_ct_memcmp_less() {
        assert_eq!(ct_memcmp(b"abc", b"abd"), -1);
    }

    #[test]
    fn test_ct_memcmp_greater() {
        assert_eq!(ct_memcmp(b"abd", b"abc"), 1);
    }

    #[test]
    fn test_ct_min_max() {
        assert_eq!(ct_min(5, 10), 5);
        assert_eq!(ct_max(5, 10), 10);
        assert_eq!(ct_min(10, 5), 5);
        assert_eq!(ct_max(10, 5), 10);
        assert_eq!(ct_min(7, 7), 7);
    }

    #[test]
    fn test_ct_is_zero() {
        assert_eq!(ct_is_zero(0), 1);
        assert_eq!(ct_is_zero(1), 0);
        assert_eq!(ct_is_zero(255), 0);
        assert_eq!(ct_is_zero(128), 0);
    }

    #[test]
    fn test_ct_and_basic() {
        let a = [0xFF, 0x0F, 0xF0, 0x00];
        let b = [0x0F, 0xFF, 0x0F, 0xFF];
        let result = ct_and(&a, &b).unwrap();
        assert_eq!(result, [0x0F, 0x0F, 0x00, 0x00]);
    }

    #[test]
    fn test_ct_and_different_lengths() {
        assert!(ct_and(&[1, 2], &[1, 2, 3]).is_none());
    }

    #[test]
    fn test_constant_time_loop_found() {
        assert!(constant_time_loop(b"abcdefgh", b'd'));
    }

    #[test]
    fn test_constant_time_loop_not_found() {
        assert!(!constant_time_loop(b"abcdefgh", b'z'));
    }

    #[test]
    fn test_constant_time_loop_empty() {
        assert!(!constant_time_loop(b"", b'a'));
    }
}
