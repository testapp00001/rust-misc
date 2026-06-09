// ============================================================================
// Problem: Maximum Subarray (LeetCode #53)
// ============================================================================
// Given an integer array `nums`, find the subarray with the largest sum
// and return its sum.
//
// Example:
//   Input:  [-2,1,-3,4,-1,2,1,-5,4]
//   Output: 6  (subarray [4,-1,2,1])
//
// ============================================================================
// APPROACH: Kadane's Algorithm (O(n) time, O(1) space)
// ============================================================================
//
// Track the current sum and maximum sum:
// 1. For each element, decide: extend current subarray or start new one.
// 2. current_sum = max(num, current_sum + num)
// 3. max_sum = max(max_sum, current_sum)
//
// This is the classic Kadane's algorithm.
// ============================================================================



pub fn max_sub_array(nums: &[i32]) -> i32 {
    todo!("Implement max_sub_array")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            max_sub_array(&[-2, 1, -3, 4, -1, 2, 1, -5, 4]),
            6
        );
    }

    #[test]
    fn test_single() {
        assert_eq!(max_sub_array(&[1]), 1);
    }

    #[test]
    fn test_all_negative() {
        assert_eq!(max_sub_array(&[-1, -2, -3]), -1);
    }

    #[test]
    fn test_all_positive() {
        assert_eq!(max_sub_array(&[1, 2, 3]), 6);
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
        //     let _ = max_sub_array(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}