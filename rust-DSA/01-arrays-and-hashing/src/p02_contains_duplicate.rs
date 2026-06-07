// ============================================================================
// Problem: Contains Duplicate (LeetCode #217)
// ============================================================================
// Given an integer array `nums`, return `true` if any value appears at least
// twice, and `false` if every element is distinct.
//
// Example:
//   Input:  [1, 2, 3, 1]
//   Output: true
//
//   Input:  [1, 2, 3, 4]
//   Output: false
//
// ============================================================================
// APPROACH: HashSet (O(n) time, O(n) space)
// ============================================================================
//
// Use a HashSet to track seen elements:
// - Iterate through the array.
// - If the current element is already in the set → return true.
// - Otherwise, insert it.
// - If we finish the loop → return false.
//
// Alternative: Sort first and check adjacent elements — O(n log n) time, O(1) space.
//
// Rust-specific tips:
// - `HashSet::insert()` returns `false` if the element already exists!
//   This is more idiomatic than checking `.contains()` first.
// ============================================================================

use std::collections::HashSet;

pub fn contains_duplicate(nums: Vec<i32>) -> bool {
    let mut seen = HashSet::new();
    for num in nums {
        if !seen.insert(num) {
            return true;
        }
    }
    false
}

// Alternative: sort-based approach
pub fn contains_duplicate_sort(mut nums: Vec<i32>) -> bool {
    nums.sort();
    nums.windows(2).any(|w| w[0] == w[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_duplicate() {
        assert!(contains_duplicate(vec![1, 2, 3, 1]));
    }

    #[test]
    fn test_no_duplicate() {
        assert!(!contains_duplicate(vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_empty() {
        assert!(!contains_duplicate(vec![]));
    }

    #[test]
    fn test_single_element() {
        assert!(!contains_duplicate(vec![1]));
    }

    #[test]
    fn test_all_same() {
        assert!(contains_duplicate(vec![1, 1, 1, 1]));
    }

    #[test]
    fn test_sort_approach() {
        assert!(contains_duplicate_sort(vec![1, 2, 3, 1]));
        assert!(!contains_duplicate_sort(vec![1, 2, 3, 4]));
    }
}
