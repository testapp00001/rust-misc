// ============================================================================
// Problem: Missing Number (LeetCode #268)
// ============================================================================
// Given an array containing n distinct numbers from [0, n], find the missing one.
//
// Example:
//   Input:  [3,0,1]
//   Output: 2
//
// ============================================================================
// APPROACH: XOR or Math (O(n) time, O(1) space)
// ============================================================================
//
// XOR approach: XOR all indices and all values. The missing number remains.
// Math approach: expected_sum - actual_sum = missing number.
// ============================================================================



pub fn missing_number(nums: &[i32]) -> i32 {
    todo!("Implement missing_number")
}

pub fn missing_number_math(nums: &[i32]) -> i32 {
    todo!("Implement missing_number_math")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(missing_number(&[3, 0, 1]), 2);
    }

    #[test]
    fn test_longer() {
        assert_eq!(missing_number(&[9, 6, 4, 2, 3, 5, 7, 0, 1]), 8);
    }

    #[test]
    fn test_zero() {
        assert_eq!(missing_number(&[1]), 0);
    }

    #[test]
    fn test_math_approach() {
        assert_eq!(missing_number_math(&[3, 0, 1]), 2);
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
        //     let _ = missing_number(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}