// ============================================================================
// Problem: 3Sum (LeetCode #15)
// ============================================================================
// Given an integer array `nums`, return all the triplets [nums[i], nums[j],
// nums[k]] such that i != j, i != k, j != k, and nums[i] + nums[j] + nums[k] == 0.
//
// The solution set must not contain duplicate triplets.
//
// Example:
//   Input:  [-1, 0, 1, 2, -1, -4]
//   Output: [[-1, -1, 2], [-1, 0, 1]]
//
// ============================================================================
// APPROACH: Sort + Two Pointers (O(n²) time, O(1) space)
// ============================================================================
//
// 1. Sort the array.
// 2. For each element nums[i] (as the first element of the triplet):
//    - Use two pointers (left = i+1, right = n-1) to find pairs that
//      sum to -nums[i].
//    - Skip duplicate values to avoid duplicate triplets.
// 3. Skip duplicate values of nums[i] to avoid duplicate triplets.
//
// Why O(n²): The outer loop is O(n), and each two-pointer scan is O(n).
// ============================================================================

pub fn three_sum(mut nums: Vec<i32>) -> Vec<Vec<i32>> {
    nums.sort();
    let mut result = Vec::new();
    let n = nums.len();

    for i in 0..n {
        // Skip duplicate first elements
        if i > 0 && nums[i] == nums[i - 1] {
            continue;
        }

        // Two pointers for the remaining elements
        let mut left = i + 1;
        let mut right = n - 1;

        while left < right {
            let sum = nums[i] + nums[left] + nums[right];
            match sum.cmp(&0) {
                std::cmp::Ordering::Less => left += 1,
                std::cmp::Ordering::Greater => right -= 1,
                std::cmp::Ordering::Equal => {
                    result.push(vec![nums[i], nums[left], nums[right]]);
                    // Skip duplicates
                    while left < right && nums[left] == nums[left + 1] {
                        left += 1;
                    }
                    while left < right && nums[right] == nums[right - 1] {
                        right -= 1;
                    }
                    left += 1;
                    right -= 1;
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut result = three_sum(vec![-1, 0, 1, 2, -1, -4]);
        result.sort();
        assert_eq!(result, vec![vec![-1, -1, 2], vec![-1, 0, 1]]);
    }

    #[test]
    fn test_no_solution() {
        assert_eq!(three_sum(vec![0, 1, 1]), Vec::<Vec<i32>>::new());
    }

    #[test]
    fn test_all_zeros() {
        assert_eq!(three_sum(vec![0, 0, 0]), vec![vec![0, 0, 0]]);
    }

    #[test]
    fn test_empty() {
        assert_eq!(three_sum(vec![]), Vec::<Vec<i32>>::new());
    }

    #[test]
    fn test_two_elements() {
        assert_eq!(three_sum(vec![0, 0]), Vec::<Vec<i32>>::new());
    }
}
