//! # Lesson 10: Side-Channel Attack Defense
//!
//! ## The Problem
//!
//! Side-channel attacks exploit PHYSICAL properties of computation — not the
//! algorithm's mathematical weakness, but its implementation's physical behavior.
//!
//! ```text
//! Types of side-channels:
//! ├── Timing:        How long does the operation take?
//! ├── Cache:         Which cache lines were accessed?
//! ├── Power:         How much power did the computation use?
//! ├── Electromagnetic: What EM radiation was emitted?
//! ├── Acoustic:      What sounds did the hardware make?
//! └── Branch:        Which branch paths were taken?
//! ```
//!
//! ## Cache-Timing Attack
//!
//! CPUs have L1/L2/L3 caches. Accessing cached data is ~100x faster than
//! accessing main memory. If a crypto operation uses a lookup table indexed
//! by a secret value, the cache access pattern reveals the secret.
//!
//! ```text
//! // Vulnerable AES S-box lookup:
//! let byte = sbox[secret_byte as usize];  // Cache line reveals secret_byte!
//!
//! // Attacker flushes cache, runs encryption, then probes which lines are cached
//! ```
//!
//! ## Branch Prediction Attack
//!
//! Modern CPUs speculatively execute branches. If a branch depends on a secret,
//! the speculative execution leaves traces in the branch predictor that an
//! attacker can detect (Spectre-style attacks).
//!
//! ```text
//! // Vulnerable:
//! if secret_bit == 1 {
//!     do_expensive_operation();  // Branch predictor learns the secret!
//! }
//! ```
//!
//! ## Defense Principles
//!
//! 1. **No secret-dependent branches**: Use bitwise operations instead of `if`
//! 2. **No secret-dependent memory access**: Use constant-time table lookups
//! 3. **No secret-dependent loop bounds**: Always iterate the same number of times
//! 4. **Use hardware-specific countermeasures**: AES-NI, etc.
//! 5. **Masking**: Split secrets into random shares

/// Exercise 1: Implement `constant_time_select` — select between two values
/// without branching.
///
/// Returns `a` if `condition` is true, `b` if false.
/// Must NOT use `if` or `match` on the condition.
///
/// Hints:
/// - Convert bool to mask: `let mask = (condition as u8).wrapping_neg();`
///   (true → 0xFF, false → 0x00)
/// - Apply: `(a & mask) | (b & !mask)`
pub fn ct_select(condition: bool, a: u8, b: u8) -> u8 {
    todo!("Select without branching")
}

/// Exercise 2: Implement `ct_eq` — constant-time equality check for byte slices.
///
/// Must check ALL bytes regardless of where differences occur.
///
/// Hints:
/// - Accumulate XOR differences with OR
/// - `diff |= a ^ b` for each byte pair
/// - Return `diff == 0`
pub fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    todo!("Constant-time equality check")
}

/// Exercise 3: Implement `ct_memcmp` — constant-time comparison returning
/// an ordering (-1, 0, 1).
///
/// Returns:
/// - -1 if a < b
/// - 0 if a == b
/// - 1 if a > b
///
/// Must be constant-time for slices of equal length.
///
/// Hints:
/// - For each byte pair (a_i, b_i):
///   - If a_i < b_i and no result set yet → result = -1
///   - If a_i > b_i and no result set yet → result = 1
/// - Use bitwise operations to avoid branching
pub fn ct_memcmp(a: &[u8], b: &[u8]) -> i32 {
    todo!("Constant-time comparison returning ordering")
}

/// Exercise 4: Implement `ct_min` and `ct_max` — constant-time min/max.
///
/// Must NOT branch on the comparison result.
///
/// Hints:
/// - `ct_min(a, b) = ct_select(a < b, a, b)`
/// - `ct_max(a, b) = ct_select(a < b, b, a)`
pub fn ct_min(a: u8, b: u8) -> u8 {
    todo!("Constant-time minimum")
}

pub fn ct_max(a: u8, b: u8) -> u8 {
    todo!("Constant-time maximum")
}

/// Exercise 5: Implement `ct_is_zero` — constant-time check if a byte is zero.
///
/// Returns 1 if the byte is zero, 0 otherwise.
/// Must NOT use `if`, `match`, or any branching.
///
/// Hints:
/// - A byte is zero iff all bits are zero
/// - `!byte` is 0xFF only when byte is 0
/// - Use arithmetic: `((byte as u16).wrapping_sub(1) >> 8) as u8 & 1`
///   - If byte=0: 0-1 = 0xFFFF, >>8 = 0xFF, &1 = 1
///   - If byte!=0: n-1 < 0xFF, >>8 = 0, &1 = 0
pub fn ct_is_zero(byte: u8) -> u8 {
    todo!("Constant-time zero check")
}

/// Exercise 6: Implement `ct_and` — constant-time logical AND for byte slices.
///
/// Computes element-wise AND of two equal-length slices.
/// Returns None if lengths differ.
///
/// Hints:
/// - Check lengths, return None if different
/// - Use `a[i] & b[i]` for each element
pub fn ct_and(a: &[u8], b: &[u8]) -> Option<Vec<u8>> {
    todo!("Constant-time element-wise AND")
}

/// Exercise 7: Implement `dummy_operations` — perform a fixed number of
/// operations regardless of input, to hide the actual workload.
///
/// Requirements:
/// - Always perform exactly `N` iterations (even if fewer are "needed")
/// - Use `ct_select` to conditionally include/exclude results
/// - Return the actual result, not the dummy computations
///
/// Hints:
/// - Loop N times
/// - Use ct_select to only accumulate real results
/// - Dummy iterations do the same work but discard the result
pub fn constant_time_loop(data: &[u8], target: u8) -> bool {
    todo!("Search for target in data with constant-time loop")
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
