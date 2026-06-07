// ============================================================================
// Problem: Next Greater Element (LeetCode #496, #503)
// ============================================================================
// Given an array, find the next greater element for each element.
// The next greater element is the first element to the right that is greater.
// If none exists, return -1.
//
// Example:
//   Input:  [4,1,2], [1,3,4,2]
//   Output: [-1,3,-1,-1]  (for each element in first array, find next greater in second)
//
// ============================================================================
// APPROACH: Monotonic Stack (O(n) time, O(n) space)
// ============================================================================
//
// Process array from right to left:
// 1. Maintain a stack of elements in decreasing order.
// 2. For each element, pop elements smaller than it from the stack.
// 3. The next greater element is the stack top (or -1 if empty).
// 4. Push current element onto the stack.
// ============================================================================

use std::collections::HashMap;

// Next Greater Element I (find next greater in a second array)
pub fn next_greater_element(nums1: &[i32], nums2: &[i32]) -> Vec<i32> {
    let mut next_greater: HashMap<i32, i32> = HashMap::new();
    let mut stack: Vec<i32> = Vec::new();

    // Process nums2 from right to left
    for &num in nums2.iter().rev() {
        while !stack.is_empty() && *stack.last().unwrap() <= num {
            stack.pop();
        }
        next_greater.insert(num, stack.last().copied().unwrap_or(-1));
        stack.push(num);
    }

    nums1.iter().map(|&num| next_greater[&num]).collect()
}

// Next Greater Element II (circular array)
pub fn next_greater_element_circular(nums: &[i32]) -> Vec<i32> {
    let n = nums.len();
    let mut result = vec![-1; n];
    let mut stack: Vec<usize> = Vec::new();

    // Process twice for circular behavior
    for i in 0..2 * n {
        while !stack.is_empty() && nums[i % n] > nums[*stack.last().unwrap()] {
            result[stack.pop().unwrap()] = nums[i % n];
        }
        if i < n {
            stack.push(i);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(next_greater_element(&[4, 1, 2], &[1, 3, 4, 2]), vec![-1, 3, -1]);
    }

    #[test]
    fn test_all_greater() {
        assert_eq!(next_greater_element(&[2, 4], &[1, 2, 3, 4]), vec![3, -1]);
    }

    #[test]
    fn test_circular() {
        assert_eq!(
            next_greater_element_circular(&[1, 2, 1]),
            vec![2, -1, 2]
        );
    }

    #[test]
    fn test_circular_all() {
        assert_eq!(
            next_greater_element_circular(&[1, 2, 3, 4, 3]),
            vec![2, 3, 4, -1, 4]
        );
    }
}
