// ============================================================================
// Problem: Longest Common Subsequence (LeetCode #1143)
// ============================================================================
// Given two strings `text1` and `text2`, return the length of their longest
// common subsequence. If there is no common subsequence, return 0.
//
// Example:
//   text1 = "abcde", text2 = "ace"
//   Output: 3  (the LCS is "ace")
//
// ============================================================================
// APPROACH: 2D Dynamic Programming (O(m*n) time, O(min(m,n)) space)
// ============================================================================
//
// dp[i][j] = length of LCS of text1[0..i] and text2[0..j]
//
// If text1[i-1] == text2[j-1]: dp[i][j] = dp[i-1][j-1] + 1
// Else: dp[i][j] = max(dp[i-1][j], dp[i][j-1])
//
// Space optimization: use two rows instead of full matrix.
// ============================================================================

pub fn longest_common_subsequence(text1: &str, text2: &str) -> i32 {
    let s1: Vec<char> = text1.chars().collect();
    let s2: Vec<char> = text2.chars().collect();
    let m = s1.len();
    let n = s2.len();

    // Space-optimized: use two rows
    let mut prev = vec![0; n + 1];
    let mut curr = vec![0; n + 1];

    for i in 1..=m {
        for j in 1..=n {
            if s1[i - 1] == s2[j - 1] {
                curr[j] = prev[j - 1] + 1;
            } else {
                curr[j] = prev[j].max(curr[j - 1]);
            }
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.fill(0);
    }

    prev[n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(longest_common_subsequence("abcde", "ace"), 3);
    }

    #[test]
    fn test_no_common() {
        assert_eq!(longest_common_subsequence("abc", "def"), 0);
    }

    #[test]
    fn test_same() {
        assert_eq!(longest_common_subsequence("abc", "abc"), 3);
    }

    #[test]
    fn test_empty() {
        assert_eq!(longest_common_subsequence("", "abc"), 0);
    }

    #[test]
    fn test_single() {
        assert_eq!(longest_common_subsequence("a", "a"), 1);
    }


    #[test]
    #[ignore]
    fn bench_performance() {
        // ⏱️  Benchmark test
        // Run: cargo test -p <package> bench_performance -- --ignored --nocapture
        //
        // To use: uncomment and customize the code below with your function
        // and realistic test data.
        //
        // let iterations = 10_000;
        // let input = /* generate your test input here */;
        // let start = std::time::Instant::now();
        // for _ in 0..iterations {
        //     let _ = longest_common_subsequence(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}