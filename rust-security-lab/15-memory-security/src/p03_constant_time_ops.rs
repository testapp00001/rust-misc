//! # Lesson 03: Constant-Time Operations — Preventing Timing Side-Channels
//!
//! ## The Problem
//!
//! CPUs are not constant-time machines. Operations like comparison, branching,
//! and memory access take variable time depending on the data. An attacker who
//! can measure execution time can infer secret-dependent information.
//!
//! ```text
//! // Vulnerable comparison:
//! fn compare(a: &[u8], b: &[u8]) -> bool {
//!     for (x, y) in a.iter().zip(b.iter()) {
//!         if x != y { return false; }  // EARLY EXIT leaks info!
//!     }
//!     true
//! }
//!
//! // Attacker measures:
//! // "A___" → fails at byte 0 → fast
//! // "B___" → fails at byte 0 → fast
//! // "Xaxx" → fails at byte 1 → slightly slower
//! // "Xayy" → fails at byte 2 → even slower
//! // → attacker recovers the secret byte by byte
//! ```
//!
//! ## Defense: Constant-Time Primitives
//!
//! 1. **Comparison**: Always check ALL bytes, accumulate XOR differences
//! 2. **Branching**: Avoid `if secret_value { ... }` — use conditional moves
//! 3. **Memory access**: Avoid `table[secret_index]` — access ALL entries
//!
//! ## Attack: Remote Timing Attack on HMAC
//!
//! Researchers demonstrated extracting HMAC keys over the network by measuring
//! response time differences as small as 20 microseconds. Even "fast" network
//! connections have enough timing resolution for these attacks.
//!
//! Defense: Use `ring`, `constant_time_eq`, or `subtle` crate for all
//! security-sensitive comparisons.

/// Exercise 1: Implement constant-time comparison of two byte slices.
///
/// Requirements:
/// - Return `true` only if slices are equal
/// - Must process ALL bytes regardless of where differences occur
/// - Must NOT short-circuit on first difference
///
/// Hints:
/// - Initialize `result: u8 = 0`
/// - XOR each pair of bytes and OR into result: `result |= a ^ b`
/// - After the loop, return `result == 0`
/// - Handle different lengths: return false immediately (length is not secret)
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    todo!("Implement constant-time byte slice comparison")
}

/// Exercise 2: Implement constant-time conditional select.
///
/// Returns `a` if `condition` is true, `b` if false.
/// The selection must be constant-time (no branching on the condition).
///
/// Hints:
/// - Convert bool to mask: `let mask = -(condition as i8) as u8;`
///   (true → 0xFF, false → 0x00)
/// - Result: `(a & mask) | (b & !mask)`
pub fn conditional_select(condition: bool, a: u8, b: u8) -> u8 {
    todo!("Select a or b without branching")
}

/// Exercise 3: Implement constant-time check if all bytes are zero.
///
/// Requirements:
/// - Return `true` only if every byte in the slice is 0
/// - Must be constant-time (no early exit)
///
/// Hints:
/// - Accumulate with OR: `acc |= byte`
/// - Return `acc == 0`
pub fn all_zeros(data: &[u8]) -> bool {
    todo!("Check if all bytes are zero in constant time")
}

/// Exercise 4: Implement constant-time lookup in a small table.
///
/// Given a table of 16 bytes and an index (0-15), return `table[index]`
/// without any secret-dependent memory access pattern.
///
/// Requirements:
/// - The INDEX is the secret (attacker shouldn't learn which index was accessed)
/// - Must access ALL table entries every time
/// - Use bitwise selection to pick the result
///
/// Hints:
/// - Iterate over all 16 entries
/// - For each entry i, check if i == index (constant-time)
/// - Use conditional_select to accumulate the result
pub fn constant_time_lookup(table: &[u8; 16], index: u8) -> u8 {
    todo!("Lookup table[index] without leaking the index")
}

/// Exercise 5: Implement constant-time conditional memory copy.
///
/// If `condition` is true, copy `src` into `dst`. If false, do nothing.
/// The operation must be constant-time — `dst` must be written to regardless
/// (either copied data or unchanged data, selected byte-by-byte).
///
/// Hints:
/// - For each byte pair, use `conditional_select(condition, src[i], dst[i])`
/// - Write the result back to `dst[i]`
pub fn conditional_copy(condition: bool, dst: &mut [u8], src: &[u8]) {
    todo!("Conditionally copy src to dst in constant time")
}

/// Exercise 6: Implement constant-time byte search.
///
/// Check if a target byte exists in the slice. Return true if found.
/// The result should leak nothing about WHERE the byte was found (or not found).
///
/// Hints:
/// - Accumulate with OR: `found |= (byte == target) as u8` — but make the
///   comparison constant-time using XOR
/// - `let matches = (byte ^ target) == 0;` — this is just comparison
/// - `found |= matches as u8;` — accumulate
/// - Return `found != 0`
pub fn constant_time_contains(data: &[u8], target: u8) -> bool {
    todo!("Check if target byte exists in data (constant time)")
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
