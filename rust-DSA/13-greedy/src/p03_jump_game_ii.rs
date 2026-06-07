// ============================================================================
// Problem: Jump Game II (LeetCode #45)
// ============================================================================
// Given an array `nums`, return the minimum number of jumps to reach the
// last index (you can always reach the last index).
//
// Example:
//   Input:  [2,3,1,1,4]
//   Output: 2  (0→1→3)
//
// ============================================================================
// APPROACH: Greedy BFS (O(n) time, O(1) space)
// ============================================================================
//
// Think of it as BFS levels:
// 1. Track the farthest position reachable with current jumps.
// 2. When we reach the end of the current level, increment jumps.
// 3. Update the end to the new farthest position.
// ============================================================================

pub fn jump(nums: &[i32]) -> i32 {
    let mut jumps = 0;
    let mut current_end = 0;
    let mut farthest = 0;

    for i in 0..nums.len() - 1 {
        farthest = farthest.max(i + nums[i] as usize);
        if i == current_end {
            jumps += 1;
            current_end = farthest;
        }
    }

    jumps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(jump(&[2, 3, 1, 1, 4]), 2);
    }

    #[test]
    fn test_longer() {
        assert_eq!(jump(&[2, 3, 0, 1, 4]), 2);
    }

    #[test]
    fn test_single() {
        assert_eq!(jump(&[0]), 0);
    }

    #[test]
    fn test_two() {
        assert_eq!(jump(&[1, 2]), 1);
    }
}
