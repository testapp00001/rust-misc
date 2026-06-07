// ============================================================================
// Problem: Sum of Two Integers (LeetCode #371)
// ============================================================================
// Given two integers a and b, return the sum without using + or -.
//
// Example:
//   a = 1, b = 2 → 3
//
// ============================================================================
// APPROACH: Bit Manipulation (O(1) time, O(1) space)
// ============================================================================
//
// Use XOR for sum without carry, AND for carry:
// - sum = a ^ b (XOR gives sum without carry)
// - carry = (a & b) << 1 (AND gives carry bits, shifted left)
// - Repeat until carry is 0
//
// For negative numbers in Rust, use wrapping operations.
// ============================================================================

pub fn get_sum(a: i32, b: i32) -> i32 {
    let mut a = a;
    let mut b = b;

    while b != 0 {
        let carry = (a & b) << 1;
        a = a ^ b;
        b = carry;
    }

    a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(get_sum(1, 2), 3);
    }

    #[test]
    fn test_negative() {
        assert_eq!(get_sum(-1, 1), 0);
    }

    #[test]
    fn test_both_negative() {
        assert_eq!(get_sum(-1, -2), -3);
    }

    #[test]
    fn test_zero() {
        assert_eq!(get_sum(0, 0), 0);
        assert_eq!(get_sum(5, 0), 5);
    }
}
