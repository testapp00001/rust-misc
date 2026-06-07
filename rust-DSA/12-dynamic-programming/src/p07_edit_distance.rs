// ============================================================================
// Problem: Edit Distance (LeetCode #72)
// ============================================================================
// Given two strings `word1` and `word2`, return the minimum number of
// operations (insert, delete, replace) to convert word1 to word2.
//
// Example:
//   word1 = "horse", word2 = "ros"
//   Output: 3  (horse → rorse → rose → ros)
//
// ============================================================================
// APPROACH: 2D Dynamic Programming (O(m*n) time, O(min(m,n)) space)
// ============================================================================
//
// dp[i][j] = min operations to convert word1[0..i] to word2[0..j]
//
// If word1[i-1] == word2[j-1]: dp[i][j] = dp[i-1][j-1]
// Else: dp[i][j] = 1 + min(dp[i-1][j], dp[i][j-1], dp[i-1][j-1])
//   - dp[i-1][j]   → delete from word1
//   - dp[i][j-1]   → insert into word1
//   - dp[i-1][j-1] → replace in word1
// ============================================================================

pub fn min_distance(word1: &str, word2: &str) -> i32 {
    let s1: Vec<char> = word1.chars().collect();
    let s2: Vec<char> = word2.chars().collect();
    let m = s1.len();
    let n = s2.len();

    let mut prev = (0..=n).collect::<Vec<_>>();
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            if s1[i - 1] == s2[j - 1] {
                curr[j] = prev[j - 1];
            } else {
                curr[j] = 1 + prev[j].min(curr[j - 1]).min(prev[j - 1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[n] as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(min_distance("horse", "ros"), 3);
    }

    #[test]
    fn test_same() {
        assert_eq!(min_distance("abc", "abc"), 0);
    }

    #[test]
    fn test_empty() {
        assert_eq!(min_distance("", "abc"), 3);
        assert_eq!(min_distance("abc", ""), 3);
    }

    #[test]
    fn test_single() {
        assert_eq!(min_distance("a", "b"), 1);
    }
}
