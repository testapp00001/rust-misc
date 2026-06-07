// ============================================================================
// Problem: Longest Palindromic Substring (LeetCode #5)
// ============================================================================
// Given a string `s`, return the longest palindromic substring.
//
// Example:
//   Input:  "babad"
//   Output: "bab" or "aba"
//
// ============================================================================
// APPROACH: Expand Around Center (O(n²) time, O(1) space)
// ============================================================================
//
// For each character (and each pair of characters), expand outward:
// 1. Odd-length palindromes: center at i.
// 2. Even-length palindromes: center between i and i+1.
//
// Track the maximum palindrome found.
//
// Alternative: Manacher's algorithm (O(n) time).
// ============================================================================

pub fn longest_palindrome(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() {
        return String::new();
    }

    let mut start = 0;
    let mut max_len = 1;

    for i in 0..chars.len() {
        // Odd-length palindrome
        let len1 = expand(&chars, i as i32, i as i32);
        // Even-length palindrome
        let len2 = expand(&chars, i as i32, i as i32 + 1);

        let len = len1.max(len2);
        if len > max_len {
            max_len = len;
            start = i - (len - 1) / 2;
        }
    }

    chars[start..start + max_len].iter().collect()
}

fn expand(chars: &[char], mut left: i32, mut right: i32) -> usize {
    while left >= 0 && (right as usize) < chars.len() && chars[left as usize] == chars[right as usize] {
        left -= 1;
        right += 1;
    }
    (right - left - 1) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let result = longest_palindrome("babad");
        assert!(result == "bab" || result == "aba");
    }

    #[test]
    fn test_even() {
        assert_eq!(longest_palindrome("cbbd"), "bb");
    }

    #[test]
    fn test_single() {
        assert_eq!(longest_palindrome("a"), "a");
    }

    #[test]
    fn test_all_same() {
        assert_eq!(longest_palindrome("aaaa"), "aaaa");
    }

    #[test]
    fn test_empty() {
        assert_eq!(longest_palindrome(""), "");
    }
}
