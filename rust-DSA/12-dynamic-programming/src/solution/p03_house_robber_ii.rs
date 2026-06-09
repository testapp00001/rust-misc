// ============================================================================
// Problem: House Robber II (LeetCode #213)
// ============================================================================
// All houses are arranged in a circle. Return the maximum amount you can
// rob without robbing two adjacent houses.
//
// Example:
//   Input:  [2, 3, 2]
//   Output: 3  (rob house 2 only)
//
// ============================================================================
// APPROACH: Dynamic Programming (O(n) time, O(1) space)
// ============================================================================
//
// Since houses are in a circle, house 0 and house n-1 are adjacent.
// Solution: Run House Robber I twice:
// 1. On houses[0..n-1] (exclude last house).
// 2. On houses[1..n] (exclude first house).
// Return the maximum of the two.
// ============================================================================

pub fn rob(nums: &[i32]) -> i32 {
    if nums.len() == 1 {
        return nums[0];
    }
    if nums.is_empty() {
        return 0;
    }

    rob_linear(&nums[..nums.len() - 1]).max(rob_linear(&nums[1..]))
}

fn rob_linear(nums: &[i32]) -> i32 {
    let mut prev2 = 0;
    let mut prev1 = 0;

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
        assert_eq!(rob(&[2, 3, 2]), 3);
    }

    #[test]
    fn test_four() {
        assert_eq!(rob(&[1, 2, 3, 1]), 4);
    }

    #[test]
    fn test_single() {
        assert_eq!(rob(&[5]), 5);
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