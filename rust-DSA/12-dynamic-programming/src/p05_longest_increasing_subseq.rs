// ============================================================================
// Problem: Longest Increasing Subsequence (LeetCode #300)
// ============================================================================
// Given an integer array `nums`, return the length of the longest strictly
// increasing subsequence.
//
// Example:
//   Input:  [10,9,2,5,3,7,101,18]
//   Output: 4  (the subsequence [2,3,7,101])
//
// ============================================================================
// APPROACH: DP (O(n²) time, O(n) space) or Binary Search (O(n log n) time)
// ============================================================================
//
// DP approach:
// dp[i] = length of LIS ending at index i
// dp[i] = max(dp[j] + 1) for all j < i where nums[j] < nums[i]
//
// Binary Search approach:
// Maintain a "tails" array where tails[i] is the smallest tail element
// for an increasing subsequence of length i+1.
// ============================================================================

// O(n²) DP approach
pub fn length_of_lis(nums: &[i32]) -> i32 {
    let n = nums.len();
    let mut dp = vec![1; n];

    for i in 1..n {
        for j in 0..i {
            if nums[j] < nums[i] {
                dp[i] = dp[i].max(dp[j] + 1);
            }
        }
    }

    *dp.iter().max().unwrap_or(&0)
}

// O(n log n) Binary Search approach
pub fn length_of_lis_binary_search(nums: &[i32]) -> i32 {
    let mut tails: Vec<i32> = Vec::new();

    for &num in nums {
        // Find the position to replace or extend
        match tails.binary_search(&num) {
            Ok(pos) => tails[pos] = num,
            Err(pos) => {
                if pos == tails.len() {
                    tails.push(num);
                } else {
                    tails[pos] = num;
                }
            }
        }
    }

    tails.len() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(length_of_lis(&[10, 9, 2, 5, 3, 7, 101, 18]), 4);
    }

    #[test]
    fn test_increasing() {
        assert_eq!(length_of_lis(&[1, 2, 3, 4, 5]), 5);
    }

    #[test]
    fn test_decreasing() {
        assert_eq!(length_of_lis(&[5, 4, 3, 2, 1]), 1);
    }

    #[test]
    fn test_single() {
        assert_eq!(length_of_lis(&[1]), 1);
    }

    #[test]
    fn test_binary_search() {
        assert_eq!(length_of_lis_binary_search(&[10, 9, 2, 5, 3, 7, 101, 18]), 4);
    }
}
