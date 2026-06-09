// ============================================================================
// Problem: Kth Largest Element in a Stream (LeetCode #703)
// ============================================================================
// Design a class that finds the kth largest element in a stream.
// The kth largest is the kth element in sorted order (not kth distinct).
//
// Example:
//   KthLargest(3, [4,5,8,2]) → add(3)=4, add(5)=5, add(10)=5, add(9)=8
//
// ============================================================================
// APPROACH: Min Heap of size k (O(log k) per add, O(k) space)
// ============================================================================
//
// Maintain a min heap of size k:
// - The top of the min heap is always the kth largest.
// - When adding a new element:
//   - Push it onto the heap.
//   - If heap size > k, pop the minimum.
//
// Rust: Use BinaryHeap with Reverse for min-heap behavior.
// ============================================================================


use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub struct KthLargest {
    // TODO: Define fields
}

impl KthLargest {
    pub fn new(k: i32, nums: Vec<i32>) -> Self {
        todo!("Implement new")
    }

    pub fn add(&mut self, val: i32) -> i32 {
        todo!("Implement add")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut kth = KthLargest::new(3, vec![4, 5, 8, 2]);
        assert_eq!(kth.add(3), 4);
        assert_eq!(kth.add(5), 5);
        assert_eq!(kth.add(10), 5);
        assert_eq!(kth.add(9), 8);
    }

    #[test]
    fn test_single() {
        let mut kth = KthLargest::new(1, vec![]);
        assert_eq!(kth.add(1), 1);
        assert_eq!(kth.add(2), 2);
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
        //     let _ = new(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}