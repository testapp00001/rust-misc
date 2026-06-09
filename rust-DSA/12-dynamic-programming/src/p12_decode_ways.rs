// ============================================================================
// Problem: Decode Ways (LeetCode #91)
// ============================================================================
// A message containing letters A-Z is encoded as numbers: 'A' → "1", ...,
// 'Z' → "26". Given a string of digits, return the number of ways to decode.
//
// Example:
//   "12" → 2  ("AB" or "L")
//   "226" → 3  ("BZ", "VF", "BBF")
//
// ============================================================================
// APPROACH: Dynamic Programming (O(n) time, O(1) space)
// ============================================================================
//
// dp[i] = number of ways to decode s[0..i]
//
// For each position:
// 1. If s[i] != '0': dp[i] += dp[i-1] (single digit decode)
// 2. If s[i-1..i] forms 10-26: dp[i] += dp[i-2] (two digit decode)
//
// We only need the last two values, so use two variables.
// ============================================================================



pub fn num_decodings(s: &str) -> i32 {
    todo!("Implement num_decodings")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(num_decodings("12"), 2);
    }

    #[test]
    fn test_three() {
        assert_eq!(num_decodings("226"), 3);
    }

    #[test]
    fn test_zero() {
        assert_eq!(num_decodings("10"), 1);
    }

    #[test]
    fn test_leading_zero() {
        assert_eq!(num_decodings("06"), 0);
    }

    #[test]
    fn test_single() {
        assert_eq!(num_decodings("1"), 1);
    }

    #[test]
    fn test_empty() {
        assert_eq!(num_decodings(""), 0);
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
        //     let _ = num_decodings(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}