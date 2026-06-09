// ============================================================================
// Problem: Multiply Strings (LeetCode #43)
// ============================================================================
// Given two non-negative integers as strings, return their product as a string.
// You must not use built-in BigInteger library.
//
// Example:
//   "123" * "456" = "56088"
//
// ============================================================================
// APPROACH: Grade School Multiplication (O(m*n) time, O(m+n) space)
// ============================================================================
//
// Use an array to store intermediate results:
// 1. For each digit pair (i, j), add num1[i] * num2[j] to result[i+j+1].
// 2. Handle carries.
// ============================================================================



pub fn multiply(num1: &str, num2: &str) -> String {
    todo!("Implement multiply")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(multiply("123", "456"), "56088");
    }

    #[test]
    fn test_zeros() {
        assert_eq!(multiply("0", "0"), "0");
    }

    #[test]
    fn test_single() {
        assert_eq!(multiply("5", "3"), "15");
    }

    #[test]
    fn test_large() {
        assert_eq!(multiply("999", "999"), "998001");
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
        //     let _ = multiply(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}