// ============================================================================
// Problem: Unique Paths (LeetCode #62)
// ============================================================================
// A robot is at the top-left corner of an m x n grid. It can only move
// right or down. How many unique paths exist to reach the bottom-right?
//
// Example:
//   m = 3, n = 7 → Output: 28
//
// ============================================================================
// APPROACH: Dynamic Programming (O(m*n) time, O(n) space)
// ============================================================================
//
// dp[i][j] = dp[i-1][j] + dp[i][j-1]
// Base case: dp[0][j] = 1, dp[i][0] = 1
//
// Space optimization: use a single row.
// ============================================================================

pub fn unique_paths(m: i32, n: i32) -> i32 {
    let m = m as usize;
    let n = n as usize;
    let mut dp = vec![1; n];

    for _ in 1..m {
        for j in 1..n {
            dp[j] += dp[j - 1];
        }
    }

    dp[n - 1]
}

// Alternative: Math approach using combinations
// C(m+n-2, m-1) = (m+n-2)! / ((m-1)! * (n-1)!)
pub fn unique_paths_math(m: i32, n: i32) -> i32 {
    let m = m as i64;
    let n = n as i64;
    let mut result: i64 = 1;

    for i in 0..m.min(n) - 1 {
        result = result * (m + n - 2 - i) / (i + 1);
    }

    result as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(unique_paths(3, 7), 28);
    }

    #[test]
    fn test_square() {
        assert_eq!(unique_paths(3, 3), 6);
    }

    #[test]
    fn test_single_row() {
        assert_eq!(unique_paths(1, 5), 1);
    }

    #[test]
    fn test_single_col() {
        assert_eq!(unique_paths(5, 1), 1);
    }

    #[test]
    fn test_math_approach() {
        assert_eq!(unique_paths_math(3, 7), 28);
    }
}
