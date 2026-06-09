// ============================================================================
// Problem: Counting Bits (LeetCode #338)
// ============================================================================
// Given an integer n, return an array where ans[i] is the number of 1s in
// the binary representation of i.
//
// Example:
//   Input:  n = 5
//   Output: [0,1,1,2,1,2]
//
// ============================================================================
// APPROACH: Dynamic Programming (O(n) time, O(n) space)
// ============================================================================
//
// Key insight: count[i] = count[i >> 1] + (i & 1)
//   - i >> 1 is i divided by 2 (right shift)
//   - i & 1 is the least significant bit
//
// This builds on previously computed values.
// ============================================================================



pub fn count_bits(n: i32) -> Vec<i32> {
    todo!("Implement count_bits")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(count_bits(5), vec![0, 1, 1, 2, 1, 2]);
    }

    #[test]
    fn test_zero() {
        assert_eq!(count_bits(0), vec![0]);
    }

    #[test]
    fn test_one() {
        assert_eq!(count_bits(1), vec![0, 1]);
    }

    #[test]
    fn test_power_of_two() {
        assert_eq!(count_bits(8), vec![0, 1, 1, 2, 1, 2, 2, 3, 1]);
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
        //     let _ = count_bits(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}