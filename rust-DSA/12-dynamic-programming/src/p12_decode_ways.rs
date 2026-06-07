// ============================================================================
// Problem: Decode Ways (LeetCode #91)
// ============================================================================
// A message containing letters A-Z is encoded as numbers: 'A' → "1", ...,
// 'Z' → "26". Given a string of digits, return the number of ways to decode.
//
// Example:
//   "12" → 2  ("AB" or "L")
//   "226" → 3  ("BZ", "VF", "BBF")
//
// ============================================================================
// APPROACH: Dynamic Programming (O(n) time, O(1) space)
// ============================================================================
//
// dp[i] = number of ways to decode s[0..i]
//
// For each position:
// 1. If s[i] != '0': dp[i] += dp[i-1] (single digit decode)
// 2. If s[i-1..i] forms 10-26: dp[i] += dp[i-2] (two digit decode)
//
// We only need the last two values, so use two variables.
// ============================================================================

pub fn num_decodings(s: &str) -> i32 {
    let bytes = s.as_bytes();
    if bytes.is_empty() || bytes[0] == b'0' {
        return 0;
    }

    let mut prev2 = 1; // dp[i-2]
    let mut prev1 = 1; // dp[i-1]

    for i in 1..bytes.len() {
        let mut current = 0;

        // Single digit decode
        if bytes[i] != b'0' {
            current += prev1;
        }

        // Two digit decode
        let two_digit = (bytes[i - 1] - b'0') * 10 + (bytes[i] - b'0');
        if two_digit >= 10 && two_digit <= 26 {
            current += prev2;
        }

        prev2 = prev1;
        prev1 = current;
    }

    prev1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(num_decodings("12"), 2);
    }

    #[test]
    fn test_three() {
        assert_eq!(num_decodings("226"), 3);
    }

    #[test]
    fn test_zero() {
        assert_eq!(num_decodings("10"), 1);
    }

    #[test]
    fn test_leading_zero() {
        assert_eq!(num_decodings("06"), 0);
    }

    #[test]
    fn test_single() {
        assert_eq!(num_decodings("1"), 1);
    }

    #[test]
    fn test_empty() {
        assert_eq!(num_decodings(""), 0);
    }
}
