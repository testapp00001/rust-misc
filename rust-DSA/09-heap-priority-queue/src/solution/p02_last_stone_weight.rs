// ============================================================================
// Problem: Last Stone Weight (LeetCode #1046)
// ============================================================================
// You have a collection of stones with integer weights. Each turn:
// - Pick the two heaviest stones and smash them.
// - If they have the same weight, both are destroyed.
// - If different, the lighter is destroyed and the heavier loses the
//   lighter's weight.
//
// Return the weight of the last stone (or 0 if none).
//
// Example:
//   Input:  [2,7,4,1,8,1]
//   Output: 1
//
// ============================================================================
// APPROACH: Max Heap (O(n log n) time, O(n) space)
// ============================================================================
//
// Use a max heap (BinaryHeap in Rust):
// 1. Push all stones onto the heap.
// 2. While heap has >= 2 stones:
//    - Pop the two heaviest.
//    - If different, push the difference.
// 3. Return the remaining stone (or 0).
// ============================================================================

use std::collections::BinaryHeap;

pub fn last_stone_weight(stones: Vec<i32>) -> i32 {
    let mut heap: BinaryHeap<i32> = stones.into_iter().collect();

    while heap.len() >= 2 {
        let y = heap.pop().unwrap();
        let x = heap.pop().unwrap();
        if y != x {
            heap.push(y - x);
        }
    }

    heap.pop().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(last_stone_weight(vec![2, 7, 4, 1, 8, 1]), 1);
    }

    #[test]
    fn test_all_destroyed() {
        assert_eq!(last_stone_weight(vec![1, 1]), 0);
    }

    #[test]
    fn test_single() {
        assert_eq!(last_stone_weight(vec![5]), 5);
    }

    #[test]
    fn test_empty() {
        assert_eq!(last_stone_weight(vec![]), 0);
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
        //     let _ = last_stone_weight(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}