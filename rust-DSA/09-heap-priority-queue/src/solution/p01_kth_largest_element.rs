// ============================================================================
// Problem: Kth Largest Element in an Array (LeetCode #215)
// ============================================================================
// Given an integer array `nums` and an integer `k`, return the kth largest
// element (not the kth distinct element).
//
// Example:
//   Input:  nums = [3,2,1,5,6,4], k = 2
//   Output: 5
//
// ============================================================================
// APPROACH 1: Min Heap of size k (O(n log k) time, O(k) space)
// ============================================================================
//
// Maintain a min heap of size k:
// 1. For each element, push it onto the heap.
// 2. If heap size > k, pop the minimum.
// 3. At the end, the heap top is the kth largest.
//
// APPROACH 2: Quickselect (O(n) average, O(n²) worst, O(1) space)
//
// Rust-specific tips:
// - Use `std::collections::BinaryHeap` (it's a max heap by default).
// - For a min heap, negate values or use `std::cmp::Reverse`.
// ============================================================================

use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub fn find_kth_largest(nums: Vec<i32>, k: i32) -> i32 {
    let mut heap: BinaryHeap<Reverse<i32>> = BinaryHeap::new();

    for num in nums {
        heap.push(Reverse(num));
        if heap.len() > k as usize {
            heap.pop();
        }
    }

    heap.peek().unwrap().0
}

// Alternative: Sort approach (O(n log n))
pub fn find_kth_largest_sort(mut nums: Vec<i32>, k: i32) -> i32 {
    nums.sort_unstable();
    nums[nums.len() - k as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(find_kth_largest(vec![3, 2, 1, 5, 6, 4], 2), 5);
    }

    #[test]
    fn test_third() {
        assert_eq!(find_kth_largest(vec![3, 2, 3, 1, 2, 4, 5, 5, 6], 4), 4);
    }

    #[test]
    fn test_single() {
        assert_eq!(find_kth_largest(vec![1], 1), 1);
    }

    #[test]
    fn test_sort_approach() {
        assert_eq!(find_kth_largest_sort(vec![3, 2, 1, 5, 6, 4], 2), 5);
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
        //     let _ = find_kth_largest(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}