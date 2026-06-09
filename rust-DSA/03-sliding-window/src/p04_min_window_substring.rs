// ============================================================================
// Problem: Minimum Window Substring (LeetCode #76)
// ============================================================================
// Given two strings `s` and `t`, return the minimum window substring of `s`
// such that every character in `t` (including duplicates) is included in
// the window. If there is no such substring, return "".
//
// Example:
//   Input:  s = "ADOBECODEBANC", t = "ABC"
//   Output: "BANC"
//
// ============================================================================
// APPROACH: Sliding Window + HashMap (O(n) time, O(m) space)
// ============================================================================
//
// 1. Count character frequencies in `t`.
// 2. Use a sliding window on `s`:
//    - Expand right to include more characters.
//    - When the window contains all characters of `t`, try to shrink left.
//    - Track the minimum valid window.
// 3. Use a "formed" counter to track how many characters have their
//    required frequency in the window.
// ============================================================================


use std::collections::HashMap;

pub fn min_window(s: &str, t: &str) -> String {
    todo!("Implement min_window")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(min_window("ADOBECODEBANC", "ABC"), "BANC");
    }

    #[test]
    fn test_no_solution() {
        assert_eq!(min_window("a", "aa"), "");
    }

    #[test]
    fn test_entire_string() {
        assert_eq!(min_window("a", "a"), "a");
    }

    #[test]
    fn test_duplicate_chars() {
        assert_eq!(min_window("aa", "aa"), "aa");
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
        //     let _ = min_window(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}