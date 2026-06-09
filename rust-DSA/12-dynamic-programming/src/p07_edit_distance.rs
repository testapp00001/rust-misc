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
    todo!("Implement min_distance")
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
        //     let _ = min_distance(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}