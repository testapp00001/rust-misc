// ============================================================================
// Problem: Longest Consecutive Sequence (LeetCode #128)
// ============================================================================
// Given an unsorted array of integers `nums`, return the length of the
// longest consecutive elements sequence.
//
// You must write an algorithm that runs in O(n) time.
//
// Example:
//   Input:  [100, 4, 200, 1, 3, 2]
//   Output: 4  (the sequence [1, 2, 3, 4])
//
// ============================================================================
// APPROACH: HashSet (O(n) time, O(n) space)
// ============================================================================
//
// Key insight: A number `n` is the START of a consecutive sequence if
// `n - 1` does NOT exist in the set. This lets us avoid counting the
// same sequence multiple times.
//
// Steps:
// 1. Insert all numbers into a HashSet.
// 2. For each number, check if it's the start of a sequence (n-1 not in set).
// 3. If yes, count how long the sequence is (n+1, n+2, ...).
// 4. Track the maximum length.
//
// Why O(n): Each number is visited at most twice (once in the outer loop,
// once while extending the sequence).
// ============================================================================

use std::collections::HashSet;

pub fn longest_consecutive(nums: Vec<i32>) -> i32 {
    let set: HashSet<i32> = nums.into_iter().collect();
    let mut max_len = 0;

    for &num in &set {
        // Only start counting if this is the beginning of a sequence
        if !set.contains(&(num - 1)) {
            let mut current = num;
            let mut len = 1;
            while set.contains(&(current + 1)) {
                current += 1;
                len += 1;
            }
            max_len = max_len.max(len);
        }
    }

    max_len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(longest_consecutive(vec![100, 4, 200, 1, 3, 2]), 4);
    }

    #[test]
    fn test_unsorted() {
        assert_eq!(longest_consecutive(vec![0, 3, 7, 2, 5, 8, 4, 6, 0, 1]), 9);
    }

    #[test]
    fn test_empty() {
        assert_eq!(longest_consecutive(vec![]), 0);
    }

    #[test]
    fn test_single() {
        assert_eq!(longest_consecutive(vec![1]), 1);
    }

    #[test]
    fn test_duplicates() {
        assert_eq!(longest_consecutive(vec![1, 2, 0, 1]), 3);
    }

    #[test]
    fn test_no_consecutive() {
        assert_eq!(longest_consecutive(vec![10, 20, 30]), 1);
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
        //     let _ = longest_consecutive(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}