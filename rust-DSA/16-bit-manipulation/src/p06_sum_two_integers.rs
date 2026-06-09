// ============================================================================
// Problem: Sum of Two Integers (LeetCode #371)
// ============================================================================
// Given two integers a and b, return the sum without using + or -.
//
// Example:
//   a = 1, b = 2 → 3
//
// ============================================================================
// APPROACH: Bit Manipulation (O(1) time, O(1) space)
// ============================================================================
//
// Use XOR for sum without carry, AND for carry:
// - sum = a ^ b (XOR gives sum without carry)
// - carry = (a & b) << 1 (AND gives carry bits, shifted left)
// - Repeat until carry is 0
//
// For negative numbers in Rust, use wrapping operations.
// ============================================================================



pub fn get_sum(a: i32, b: i32) -> i32 {
    todo!("Implement get_sum")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(get_sum(1, 2), 3);
    }

    #[test]
    fn test_negative() {
        assert_eq!(get_sum(-1, 1), 0);
    }

    #[test]
    fn test_both_negative() {
        assert_eq!(get_sum(-1, -2), -3);
    }

    #[test]
    fn test_zero() {
        assert_eq!(get_sum(0, 0), 0);
        assert_eq!(get_sum(5, 0), 5);
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
        //     let _ = get_sum(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}