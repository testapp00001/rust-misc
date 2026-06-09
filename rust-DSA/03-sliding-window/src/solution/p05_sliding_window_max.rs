// ============================================================================
// Problem: Sliding Window Maximum (LeetCode #239)
// ============================================================================
// Given an array `nums` and a sliding window of size `k` moving from left
// to right, return the max value in each window position.
//
// Example:
//   Input:  nums = [1,3,-1,-3,5,3,6,7], k = 3
//   Output: [3,3,5,5,6,7]
//
// ============================================================================
// APPROACH: Monotonic Deque (O(n) time, O(k) space)
// ============================================================================
//
// Use a deque that stores indices. The deque maintains elements in
// decreasing order of their values:
//
// 1. Remove indices outside the current window from the front.
// 2. Remove indices with smaller values from the back (they're useless).
// 3. Add the current index to the back.
// 4. The front of the deque is always the maximum.
//
// Why this works: If nums[j] > nums[i] and j > i, then nums[i] will
// never be the maximum in any future window that includes j.
// ============================================================================

use std::collections::VecDeque;

pub fn max_sliding_window(nums: Vec<i32>, k: usize) -> Vec<i32> {
    let mut deque: VecDeque<usize> = VecDeque::new();
    let mut result = Vec::new();

    for i in 0..nums.len() {
        // Remove indices outside the window
        while !deque.is_empty() && deque.front().unwrap() + k <= i {
            deque.pop_front();
        }

        // Remove indices with smaller values
        while !deque.is_empty() && nums[*deque.back().unwrap()] <= nums[i] {
            deque.pop_back();
        }

        deque.push_back(i);

        // Record the maximum once we have a full window
        if i >= k - 1 {
            result.push(nums[*deque.front().unwrap()]);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            max_sliding_window(vec![1, 3, -1, -3, 5, 3, 6, 7], 3),
            vec![3, 3, 5, 5, 6, 7]
        );
    }

    #[test]
    fn test_k_equals_length() {
        assert_eq!(max_sliding_window(vec![1, 2, 3], 3), vec![3]);
    }

    #[test]
    fn test_k_equals_one() {
        assert_eq!(max_sliding_window(vec![1, 2, 3], 1), vec![1, 2, 3]);
    }

    #[test]
    fn test_decreasing() {
        assert_eq!(max_sliding_window(vec![5, 4, 3, 2, 1], 2), vec![5, 4, 3, 2]);
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
        //     let _ = max_sliding_window(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}