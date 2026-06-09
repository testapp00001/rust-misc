// ============================================================================
// Problem: House Robber (LeetCode #198)
// ============================================================================
// You are a robber planning to rob houses along a street. Each house has
// a certain amount of money. Adjacent houses have connected security systems.
// Given an array of money, return the maximum amount you can rob without
// robbing two adjacent houses.
//
// Example:
//   Input:  [1, 2, 3, 1]
//   Output: 4  (rob house 1 and 3: 1 + 3 = 4)
//
// ============================================================================
// APPROACH: Dynamic Programming (O(n) time, O(1) space)
// ============================================================================
//
// For each house, we have two choices:
// 1. Rob it: profit = money[i] + dp[i-2]
// 2. Skip it: profit = dp[i-1]
//
// dp[i] = max(money[i] + dp[i-2], dp[i-1])
//
// We only need the last two values, so use two variables.
// ============================================================================

pub fn rob(nums: &[i32]) -> i32 {
    if nums.is_empty() {
        return 0;
    }

    let mut prev2 = 0; // dp[i-2]
    let mut prev1 = 0; // dp[i-1]

    for &num in nums {
        let current = prev1.max(num + prev2);
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
        assert_eq!(rob(&[1, 2, 3, 1]), 4);
    }

    #[test]
    fn test_alternating() {
        assert_eq!(rob(&[2, 7, 9, 3, 1]), 12);
    }

    #[test]
    fn test_single() {
        assert_eq!(rob(&[5]), 5);
    }

    #[test]
    fn test_empty() {
        assert_eq!(rob(&[]), 0);
    }

    #[test]
    fn test_two() {
        assert_eq!(rob(&[1, 2]), 2);
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
        //     let _ = rob(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}