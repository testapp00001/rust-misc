// ============================================================================
// Problem: Reverse Bits (LeetCode #190)
// ============================================================================
// Reverse the bits of a given 32-bit unsigned integer.
//
// Example:
//   Input:  00000010100101000001111010011100
//   Output: 00111001011110000010100101000000
//
// ============================================================================
// APPROACH: Bit by Bit (O(1) time — always 32 iterations, O(1) space)
// ============================================================================
//
// Process each bit from right to left:
// 1. Take the last bit of n.
// 2. Shift result left and add the bit.
// 3. Shift n right.
// ============================================================================

pub fn reverse_bits(n: u32) -> u32 {
    let mut n = n;
    let mut result = 0;

    for _ in 0..32 {
        result = (result << 1) | (n & 1);
        n >>= 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            reverse_bits(0b00000010100101000001111010011100),
            0b00111001011110000010100101000000
        );
    }

    #[test]
    fn test_all_ones() {
        assert_eq!(
            reverse_bits(0b11111111111111111111111111111111),
            0b11111111111111111111111111111111
        );
    }

    #[test]
    fn test_zero() {
        assert_eq!(reverse_bits(0), 0);
    }

    #[test]
    fn test_one() {
        assert_eq!(reverse_bits(1), 0b10000000000000000000000000000000);
    }
}
