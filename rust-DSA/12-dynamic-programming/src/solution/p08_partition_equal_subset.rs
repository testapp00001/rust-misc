// ============================================================================
// Problem: Partition Equal Subset Sum (LeetCode #416)
// ============================================================================
// Given a non-empty array of positive integers, determine if the array can
// be partitioned into two subsets with equal sum.
//
// Example:
//   Input:  [1, 5, 11, 5]
//   Output: true  (one subset: [1, 5, 5], other: [11])
//
// ============================================================================
// APPROACH: 0/1 Knapsack DP (O(n * sum/2) time, O(sum/2) space)
// ============================================================================
//
// Key insight: Can we find a subset that sums to total/2?
//
// This is the classic 0/1 knapsack problem:
// dp[j] = true if we can form sum j using some subset
// ============================================================================

pub fn can_partition(nums: &[i32]) -> bool {
    let total: i32 = nums.iter().sum();

    // If total is odd, we can't partition equally
    if total % 2 != 0 {
        return false;
    }

    let target = (total / 2) as usize;
    let mut dp = vec![false; target + 1];
    dp[0] = true;

    for &num in nums {
        let num = num as usize;
        for j in (num..=target).rev() {
            dp[j] = dp[j] || dp[j - num];
        }
    }

    dp[target]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_partition() {
        assert!(can_partition(&[1, 5, 11, 5]));
    }

    #[test]
    fn test_cannot_partition() {
        assert!(!can_partition(&[1, 2, 3, 5]));
    }

    #[test]
    fn test_odd_total() {
        assert!(!can_partition(&[1, 2, 3, 5]));
    }

    #[test]
    fn test_two_equal() {
        assert!(can_partition(&[1, 1]));
    }

    #[test]
    fn test_single() {
        assert!(!can_partition(&[1]));
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
        //     let _ = can_partition(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}