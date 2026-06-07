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
    heap: BinaryHeap<Reverse<i32>>,
    k: usize,
}

impl KthLargest {
    pub fn new(k: i32, nums: Vec<i32>) -> Self {
        let mut kth = KthLargest {
            heap: BinaryHeap::new(),
            k: k as usize,
        };
        for num in nums {
            kth.add(num);
        }
        kth
    }

    pub fn add(&mut self, val: i32) -> i32 {
        self.heap.push(Reverse(val));
        if self.heap.len() > self.k {
            self.heap.pop();
        }
        self.heap.peek().unwrap().0
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
}
