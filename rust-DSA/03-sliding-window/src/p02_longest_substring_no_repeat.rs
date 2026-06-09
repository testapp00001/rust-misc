// ============================================================================
// Problem: Longest Substring Without Repeating Characters (LeetCode #3)
// ============================================================================
// Given a string `s`, find the length of the longest substring without
// repeating characters.
//
// Example:
//   Input:  "abcabcbb"
//   Output: 3  (the answer is "abc")
//
// ============================================================================
// APPROACH: Sliding Window + HashSet (O(n) time, O(min(m,n)) space)
// ============================================================================
//
// Use a sliding window with a HashSet to track characters in the window:
// 1. Expand the window by moving the right pointer.
// 2. If a character is already in the set, shrink from the left until
//    the duplicate is removed.
// 3. Track the maximum window size.
//
// Alternative: Use a HashMap to store the last index of each character.
// This allows jumping the left pointer directly.
// ============================================================================


use std::collections::HashSet;
    use std::collections::HashMap;

pub fn length_of_longest_substring(s: &str) -> i32 {
    todo!("Implement length_of_longest_substring")
}

pub fn length_of_longest_substring_map(s: &str) -> i32 {
    todo!("Implement length_of_longest_substring_map")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(length_of_longest_substring("abcabcbb"), 3);
    }

    #[test]
    fn test_all_same() {
        assert_eq!(length_of_longest_substring("bbbbb"), 1);
    }

    #[test]
    fn test_mixed() {
        assert_eq!(length_of_longest_substring("pwwkew"), 3);
    }

    #[test]
    fn test_empty() {
        assert_eq!(length_of_longest_substring(""), 0);
    }

    #[test]
    fn test_single() {
        assert_eq!(length_of_longest_substring(" "), 1);
    }

    #[test]
    fn test_map_approach() {
        assert_eq!(length_of_longest_substring_map("abcabcbb"), 3);
        assert_eq!(length_of_longest_substring_map("bbbbb"), 1);
        assert_eq!(length_of_longest_substring_map("pwwkew"), 3);
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
        //     let _ = length_of_longest_substring(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}