// ============================================================================
// Problem: Word Break (LeetCode #139)
// ============================================================================
// Given a string `s` and a dictionary of strings `word_dict`, return true
// if `s` can be segmented into a space-separated sequence of dictionary words.
//
// Example:
//   s = "leetcode", word_dict = ["leet", "code"]
//   Output: true  ("leet code")
//
// ============================================================================
// APPROACH: Dynamic Programming (O(n² * m) time, O(n) space)
// ============================================================================
//
// dp[i] = true if s[0..i] can be segmented
//
// dp[i] = true if there exists j where:
//   - dp[j] is true, AND
//   - s[j..i] is in word_dict
// ============================================================================

use std::collections::HashSet;

pub fn word_break(s: &str, word_dict: &[&str]) -> bool {
    let word_set: HashSet<&str> = word_dict.iter().copied().collect();
    let n = s.len();
    let mut dp = vec![false; n + 1];
    dp[0] = true;

    for i in 1..=n {
        for j in 0..i {
            if dp[j] && word_set.contains(&s[j..i]) {
                dp[i] = true;
                break;
            }
        }
    }

    dp[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert!(word_break("leetcode", &["leet", "code"]));
    }

    #[test]
    fn test_multiple() {
        assert!(word_break("applepenapple", &["apple", "pen"]));
    }

    #[test]
    fn test_cannot_break() {
        assert!(!word_break("catsandog", &["cats", "dog", "sand", "and", "cat"]));
    }

    #[test]
    fn test_single() {
        assert!(word_break("a", &["a"]));
    }

    #[test]
    fn test_empty() {
        assert!(word_break("", &["a"]));
    }
}
