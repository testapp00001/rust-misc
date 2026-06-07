// ============================================================================
// Problem: Subsets II (LeetCode #90)
// ============================================================================
// Given an integer array `nums` that may contain duplicates, return all
// possible subsets (the power set). The solution set must not contain
// duplicate subsets.
//
// Example:
//   Input:  [1, 2, 2]
//   Output: [[], [1], [1,2], [1,2,2], [2], [2,2]]
//
// ============================================================================
// APPROACH: Backtracking with Sorting (O(n * 2^n) time, O(n) space)
// ============================================================================
//
// Sort the array first. When backtracking:
// - Skip duplicates at the same level (i > start && nums[i] == nums[i-1]).
// - This ensures each subset is generated only once.
// ============================================================================

pub fn subsets_with_dup(mut nums: Vec<i32>) -> Vec<Vec<i32>> {
    nums.sort();
    let mut result = Vec::new();
    let mut current = Vec::new();
    backtrack(&nums, 0, &mut current, &mut result);
    result
}

fn backtrack(nums: &[i32], start: usize, current: &mut Vec<i32>, result: &mut Vec<Vec<i32>>) {
    result.push(current.clone());

    for i in start..nums.len() {
        // Skip duplicates at the same level
        if i > start && nums[i] == nums[i - 1] {
            continue;
        }
        current.push(nums[i]);
        backtrack(nums, i + 1, current, result);
        current.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut v: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
        v.sort();
        v
    }

    #[test]
    fn test_basic() {
        let result = sorted(subsets_with_dup(vec![1, 2, 2]));
        let expected = sorted(vec![
            vec![], vec![1], vec![1, 2], vec![1, 2, 2], vec![2], vec![2, 2],
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_all_same() {
        let result = sorted(subsets_with_dup(vec![2, 2, 2]));
        let expected = sorted(vec![
            vec![], vec![2], vec![2, 2], vec![2, 2, 2],
        ]);
        assert_eq!(result, expected);
    }
}
