// ============================================================================
// Problem: Top K Frequent Elements (LeetCode #347)
// ============================================================================
// Given an integer array `nums` and an integer `k`, return the `k` most
// frequent elements. The answer can be in any order.
//
// Example:
//   Input:  nums = [1,1,1,2,2,3], k = 2
//   Output: [1, 2]
//
// ============================================================================
// APPROACH: HashMap + Bucket Sort (O(n) time, O(n) space)
// ============================================================================
//
// Step 1: Count frequencies with a HashMap — O(n).
// Step 2: Use "bucket sort" — create buckets indexed by frequency.
//   - bucket[i] = list of elements that appear exactly i times.
//   - The maximum frequency is at most n (the array length).
// Step 3: Collect from the highest bucket down until we have k elements.
//
// This is O(n) — better than the O(n log k) heap approach!
//
// Rust-specific tips:
// - Use `vec![vec![]; n + 1]` to create a vector of empty vectors.
// - `.into_iter().rev()` to iterate in reverse.
// - `flat_map` to flatten nested iterators.
// ============================================================================

use std::collections::HashMap;

pub fn top_k_frequent(nums: Vec<i32>, k: usize) -> Vec<i32> {
    let n = nums.len();
    let mut freq: HashMap<i32, usize> = HashMap::new();

    // Count frequencies
    for num in &nums {
        *freq.entry(*num).or_insert(0) += 1;
    }

    // Bucket sort: bucket[i] = numbers appearing exactly i times
    let mut buckets: Vec<Vec<i32>> = vec![vec![]; n + 1];
    for (num, count) in freq {
        buckets[count].push(num);
    }

    // Collect from highest frequency bucket down
    buckets
        .into_iter()
        .rev()
        .flat_map(|bucket| bucket.into_iter())
        .take(k)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut v: Vec<i32>) -> Vec<i32> {
        v.sort();
        v
    }

    #[test]
    fn test_basic() {
        let result = sorted(top_k_frequent(vec![1, 1, 1, 2, 2, 3], 2));
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_single_element() {
        assert_eq!(top_k_frequent(vec![1], 1), vec![1]);
    }

    #[test]
    fn test_k_equals_length() {
        let result = sorted(top_k_frequent(vec![1, 2, 3], 3));
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_all_same_frequency() {
        let result = sorted(top_k_frequent(vec![1, 2, 3, 4], 2));
        assert_eq!(result.len(), 2);
    }
}
