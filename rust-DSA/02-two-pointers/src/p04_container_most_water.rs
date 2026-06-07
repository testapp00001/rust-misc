// ============================================================================
// Problem: Container With Most Water (LeetCode #11)
// ============================================================================
// You are given an integer array `height` of length `n`. There are `n`
// vertical lines drawn at positions 0 to n-1. Find two lines that together
// with the x-axis form a container that holds the most water.
//
// The area = min(height[left], height[right]) * (right - left)
//
// Example:
//   Input:  height = [1,8,6,2,5,4,8,3,7]
//   Output: 49
//
// ============================================================================
// APPROACH: Two Pointers (O(n) time, O(1) space)
// ============================================================================
//
// 1. Start with the widest container (left=0, right=n-1).
// 2. Calculate area = min(height[left], height[right]) * width.
// 3. Move the pointer pointing to the shorter line inward.
//    (Moving the taller one can only decrease area.)
// 4. Track the maximum area.
//
// Why move the shorter one: The area is limited by the shorter line.
// Moving the taller one inward can only decrease the width without
// potentially increasing the height.
// ============================================================================

pub fn max_area(height: Vec<i32>) -> i32 {
    let mut left = 0;
    let mut right = height.len() - 1;
    let mut max_area = 0;

    while left < right {
        let h = height[left].min(height[right]);
        let w = (right - left) as i32;
        max_area = max_area.max(h * w);

        if height[left] < height[right] {
            left += 1;
        } else {
            right -= 1;
        }
    }

    max_area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(max_area(vec![1, 8, 6, 2, 5, 4, 8, 3, 7]), 49);
    }

    #[test]
    fn test_two_elements() {
        assert_eq!(max_area(vec![1, 1]), 1);
    }

    #[test]
    fn test_decreasing() {
        assert_eq!(max_area(vec![4, 3, 2, 1, 4]), 16);
    }

    #[test]
    fn test_all_same() {
        assert_eq!(max_area(vec![5, 5, 5, 5]), 15);
    }
}
