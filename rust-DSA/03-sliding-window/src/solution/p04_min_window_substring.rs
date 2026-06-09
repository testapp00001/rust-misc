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
    if s.len() < t.len() {
        return String::new();
    }

    let s_chars: Vec<char> = s.chars().collect();
    let mut t_count: HashMap<char, i32> = HashMap::new();
    for c in t.chars() {
        *t_count.entry(c).or_insert(0) += 1;
    }

    let required = t_count.len();
    let mut formed = 0;
    let mut window_count: HashMap<char, i32> = HashMap::new();
    let mut left = 0;
    let mut best = (usize::MAX, 0, 0); // (length, left, right)

    for right in 0..s_chars.len() {
        // Add character from right
        let c = s_chars[right];
        *window_count.entry(c).or_insert(0) += 1;

        if t_count.get(&c).map_or(false, |&req| window_count[&c] == req) {
            formed += 1;
        }

        // Shrink window from left
        while formed == required {
            // Update best
            if right - left + 1 < best.0 {
                best = (right - left + 1, left, right);
            }

            let left_char = s_chars[left];
            *window_count.entry(left_char).or_insert(0) -= 1;
            if t_count.get(&left_char).map_or(false, |&req| window_count[&left_char] < req) {
                formed -= 1;
            }
            left += 1;
        }
    }

    if best.0 == usize::MAX {
        String::new()
    } else {
        s_chars[best.1..=best.2].iter().collect()
    }
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