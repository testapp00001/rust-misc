// ============================================================================
// Problem: Single Number (LeetCode #136)
// ============================================================================
// Given a non-empty array of integers where every element appears twice
// except for one, find the single element.
//
// Example:
//   Input:  [2,2,1]
//   Output: 1
//
// ============================================================================
// APPROACH: XOR (O(n) time, O(1) space)
// ============================================================================
//
// XOR properties:
// - a ^ a = 0
// - a ^ 0 = a
// - XOR is commutative and associative
//
// XOR all elements: pairs cancel out, leaving the single element.
// ============================================================================

pub fn single_number(nums: &[i32]) -> i32 {
    nums.iter().fold(0, |acc, &x| acc ^ x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(single_number(&[2, 2, 1]), 1);
    }

    #[test]
    fn test_longer() {
        assert_eq!(single_number(&[4, 1, 2, 1, 2]), 4);
    }

    #[test]
    fn test_single() {
        assert_eq!(single_number(&[1]), 1);
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
        //     let _ = single_number(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}