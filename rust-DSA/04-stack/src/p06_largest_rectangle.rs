// ============================================================================
// Problem: Largest Rectangle in Histogram (LeetCode #84)
// ============================================================================
// Given an array of integers `heights` representing the histogram's bar
// heights where the width of each bar is 1, return the area of the largest
// rectangle in the histogram.
//
// Example:
//   Input:  heights = [2,1,5,6,2,3]
//   Output: 10
//
// ============================================================================
// APPROACH: Monotonic Stack (O(n) time, O(n) space)
// ============================================================================
//
// Use a stack that stores indices of bars in increasing order of height:
// 1. For each bar, while the stack top has greater height:
//    - Pop it and calculate the area with that height.
//    - Width = current_index - stack_top - 1 (or current_index if stack empty).
// 2. Push the current index.
// 3. After processing all bars, pop remaining bars and calculate areas.
//
// The stack maintains increasing heights. When we find a shorter bar,
// it limits the width of all taller bars in the stack.
// ============================================================================

pub fn largest_rectangle_area(heights: Vec<i32>) -> i32 {
    let mut stack: Vec<usize> = Vec::new();
    let mut max_area = 0;
    let n = heights.len();

    for i in 0..=n {
        let h = if i < n { heights[i] } else { 0 };

        while !stack.is_empty() && heights[*stack.last().unwrap()] > h {
            let height = heights[stack.pop().unwrap()];
            let width = if stack.is_empty() {
                i
            } else {
                i - stack.last().unwrap() - 1
            };
            max_area = max_area.max(height * width as i32);
        }

        stack.push(i);
    }

    max_area
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(largest_rectangle_area(vec![2, 1, 5, 6, 2, 3]), 10);
    }

    #[test]
    fn test_all_same() {
        assert_eq!(largest_rectangle_area(vec![3, 3, 3]), 9);
    }

    #[test]
    fn test_increasing() {
        assert_eq!(largest_rectangle_area(vec![1, 2, 3, 4]), 6);
    }

    #[test]
    fn test_single() {
        assert_eq!(largest_rectangle_area(vec![5]), 5);
    }

    #[test]
    fn test_empty() {
        assert_eq!(largest_rectangle_area(vec![]), 0);
    }
}
