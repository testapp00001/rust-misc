// ============================================================================
// Problem: Hand of Straights (LeetCode #846)
// ============================================================================
// Given an array of integers `hand` and an integer `groupSize`, return true
// if the hand can be rearranged into groups of `groupSize` consecutive cards.
//
// Example:
//   hand = [1,2,3,6,2,3,4,7,8], groupSize = 3
//   Output: true  ([[1,2,3],[2,3,4],[6,7,8]])
//
// ============================================================================
// APPROACH: Greedy with HashMap (O(n log n) time, O(n) space)
// ============================================================================
//
// 1. Count card frequencies.
// 2. Sort unique cards.
// 3. For each card (in order), try to form a group starting from it.
// 4. If we can't form a complete group → return false.
// ============================================================================


use std::collections::HashMap;

pub fn is_n_straight_hand(hand: &[i32], group_size: i32) -> bool {
    todo!("Implement is_n_straight_hand")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert!(is_n_straight_hand(&[1, 2, 3, 6, 2, 3, 4, 7, 8], 3));
    }

    #[test]
    fn test_impossible() {
        assert!(!is_n_straight_hand(&[1, 2, 3, 4, 5], 4));
    }

    #[test]
    fn test_single_group() {
        assert!(is_n_straight_hand(&[1, 2, 3], 3));
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
        //     let _ = is_n_straight_hand(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}