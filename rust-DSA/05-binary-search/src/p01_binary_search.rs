// ============================================================================
// Problem: Binary Search (LeetCode #704)
// ============================================================================
// Given a sorted array of integers `nums` and an integer `target`, return
// the index of `target` if it exists, otherwise return -1.
//
// Example:
//   Input:  nums = [-1,0,3,5,9,12], target = 9
//   Output: 4
//
// ============================================================================
// APPROACH: Binary Search (O(log n) time, O(1) space)
// ============================================================================
//
// Classic binary search:
// 1. Set left=0, right=n-1.
// 2. While left <= right:
//    - mid = left + (right - left) / 2 (avoids overflow).
//    - If nums[mid] == target → found.
//    - If nums[mid] < target → search right half (left = mid + 1).
//    - If nums[mid] > target → search left half (right = mid - 1).
//
// Rust-specific tips:
// - Use `mid = left + (right - left) / 2` to avoid integer overflow.
// - `.wrapping_add()` and `.wrapping_sub()` for overflow-safe arithmetic.
// ============================================================================

pub fn binary_search(nums: &[i32], target: i32) -> i32 {
    let mut left = 0;
    let mut right = nums.len();

    while left < right {
        let mid = left + (right - left) / 2;
        match nums[mid].cmp(&target) {
            std::cmp::Ordering::Equal => return mid as i32,
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid,
        }
    }

    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_found() {
        assert_eq!(binary_search(&[-1, 0, 3, 5, 9, 12], 9), 4);
    }

    #[test]
    fn test_not_found() {
        assert_eq!(binary_search(&[-1, 0, 3, 5, 9, 12], 2), -1);
    }

    #[test]
    fn test_first() {
        assert_eq!(binary_search(&[1, 2, 3], 1), 0);
    }

    #[test]
    fn test_last() {
        assert_eq!(binary_search(&[1, 2, 3], 3), 2);
    }

    #[test]
    fn test_empty() {
        assert_eq!(binary_search(&[], 5), -1);
    }
}
