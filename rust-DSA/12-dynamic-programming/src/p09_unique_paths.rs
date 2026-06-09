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
    todo!("Implement unique_paths")
}

pub fn unique_paths_math(m: i32, n: i32) -> i32 {
    todo!("Implement unique_paths_math")
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
        //     let _ = unique_paths(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}