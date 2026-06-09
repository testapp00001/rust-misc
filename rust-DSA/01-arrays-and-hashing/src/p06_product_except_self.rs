// ============================================================================
// Problem: Product of Array Except Self (LeetCode #238)
// ============================================================================
// Given an integer array `nums`, return an array `answer` such that
// `answer[i]` is the product of all elements of `nums` except `nums[i]`.
//
// You must solve it WITHOUT using division and in O(n) time.
//
// Example:
//   Input:  [1, 2, 3, 4]
//   Output: [24, 12, 8, 6]
//
// ============================================================================
// APPROACH: Prefix and Suffix Products (O(n) time, O(1) extra space)
// ============================================================================
//
// Key insight: answer[i] = (product of all elements before i) *
//                           (product of all elements after i)
//
// Two-pass approach:
// Pass 1 (left to right): Build prefix products in the answer array.
//   answer[i] = product of nums[0] * nums[1] * ... * nums[i-1]
//
// Pass 2 (right to left): Multiply by suffix products.
//   answer[i] *= product of nums[i+1] * nums[i+2] * ... * nums[n-1]
//
// We reuse the answer array for the prefix, then multiply in the suffix.
// This gives O(1) extra space (not counting the output).
//
// Rust-specific tips:
// - Use `.iter()` for the forward pass and `.iter().rev()` for backward.
// - `iter_mut()` lets you modify elements in place.
// ============================================================================



pub fn product_except_self(nums: Vec<i32>) -> Vec<i32> {
    todo!("Implement product_except_self")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(product_except_self(vec![1, 2, 3, 4]), vec![24, 12, 8, 6]);
    }

    #[test]
    fn test_with_zero() {
        assert_eq!(product_except_self(vec![-1, 1, 0, -3, 3]), vec![0, 0, 9, 0, 0]);
    }

    #[test]
    fn test_two_elements() {
        assert_eq!(product_except_self(vec![2, 3]), vec![3, 2]);
    }

    #[test]
    fn test_with_negatives() {
        assert_eq!(product_except_self(vec![-1, 2, -3, 4]), vec![-24, 12, -8, 6]);
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
        //     let _ = product_except_self(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}