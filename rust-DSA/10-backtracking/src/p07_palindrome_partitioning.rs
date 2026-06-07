// ============================================================================
// Problem: Palindrome Partitioning (LeetCode #131)
// ============================================================================
// Given a string `s`, partition `s` such that every substring is a
// palindrome. Return all possible palindrome partitionings.
//
// Example:
//   Input:  "aab"
//   Output: [["a","a","b"],["aa","b"]]
//
// ============================================================================
// APPROACH: Backtracking (O(n * 2^n) time, O(n) space)
// ============================================================================
//
// At each position, try all possible palindrome prefixes:
// 1. If s[start..=end] is a palindrome, add it to current partition.
// 2. Recurse on the remaining string.
// 3. When start reaches the end, we have a valid partition.
// ============================================================================

pub fn partition(s: &str) -> Vec<Vec<String>> {
    let chars: Vec<char> = s.chars().collect();
    let mut result = Vec::new();
    let mut current = Vec::new();
    backtrack(&chars, 0, &mut current, &mut result);
    result
}

fn backtrack(
    chars: &[char],
    start: usize,
    current: &mut Vec<String>,
    result: &mut Vec<Vec<String>>,
) {
    if start == chars.len() {
        result.push(current.clone());
        return;
    }

    for end in start..chars.len() {
        if is_palindrome(chars, start, end) {
            let s: String = chars[start..=end].iter().collect();
            current.push(s);
            backtrack(chars, end + 1, current, result);
            current.pop();
        }
    }
}

fn is_palindrome(chars: &[char], mut left: usize, mut right: usize) -> bool {
    while left < right {
        if chars[left] != chars[right] {
            return false;
        }
        left += 1;
        right -= 1;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let result = partition("aab");
        assert_eq!(result.len(), 2);
        assert!(result.contains(&vec!["a".to_string(), "a".to_string(), "b".to_string()]));
        assert!(result.contains(&vec!["aa".to_string(), "b".to_string()]));
    }

    #[test]
    fn test_single() {
        assert_eq!(partition("a"), vec![vec!["a"]]);
    }

    #[test]
    fn test_all_palindrome() {
        let result = partition("aba");
        assert!(result.contains(&vec!["a".to_string(), "b".to_string(), "a".to_string()]));
        assert!(result.contains(&vec!["aba".to_string()]));
    }
}
