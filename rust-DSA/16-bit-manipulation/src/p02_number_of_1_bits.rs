// ============================================================================
// Problem: Number of 1 Bits (LeetCode #191)
// ============================================================================
// Write a function that takes an unsigned integer and returns the number of
// '1' bits it has (Hamming weight).
//
// Example:
//   Input:  00000000000000000000000000001011
//   Output: 3
//
// ============================================================================
// APPROACH: Brian Kernighan's Algorithm (O(k) time where k = number of 1s)
// ============================================================================
//
// n & (n-1) clears the lowest set bit.
// Count how many times we can do this until n becomes 0.
// ============================================================================



pub fn hammut_weight(n: u32) -> i32 {
    todo!("Implement hammut_weight")
}

pub fn hammut_weight_simple(n: u32) -> i32 {
    todo!("Implement hammut_weight_simple")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(hammut_weight(0b00000000000000000000000000001011), 3);
    }

    #[test]
    fn test_power_of_two() {
        assert_eq!(hammut_weight(0b00000000000000000000000010000000), 1);
    }

    #[test]
    fn test_all_ones() {
        assert_eq!(hammut_weight(0b11111111111111111111111111111111), 32);
    }

    #[test]
    fn test_zero() {
        assert_eq!(hammut_weight(0), 0);
    }

    #[test]
    fn test_simple_approach() {
        assert_eq!(hammut_weight_simple(0b00000000000000000000000000001011), 3);
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
        //     let _ = hammut_weight(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}