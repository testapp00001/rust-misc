// ============================================================================
// Problem: Daily Temperatures (LeetCode #739)
// ============================================================================
// Given an array of temperatures, return an array where answer[i] is the
// number of days you have to wait after the ith day for a warmer temperature.
//
// Example:
//   Input:  [73,74,75,71,69,72,76,73]
//   Output: [1,1,4,2,1,1,0,0]
//
// ============================================================================
// APPROACH: Monotonic Stack (O(n) time, O(n) space)
// ============================================================================
//
// Use a stack that stores indices in decreasing order of temperature:
// 1. For each day, while stack top has cooler temperature:
//    - Pop it and calculate the waiting days.
// 2. Push current day onto the stack.
//
// This is the classic "next greater element" pattern.
// ============================================================================

pub fn daily_temperatures(temperatures: &[i32]) -> Vec<i32> {
    let n = temperatures.len();
    let mut answer = vec![0; n];
    let mut stack: Vec<usize> = Vec::new();

    for i in 0..n {
        while !stack.is_empty() && temperatures[i] > temperatures[*stack.last().unwrap()] {
            let j = stack.pop().unwrap();
            answer[j] = (i - j) as i32;
        }
        stack.push(i);
    }

    answer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            daily_temperatures(&[73, 74, 75, 71, 69, 72, 76, 73]),
            vec![1, 1, 4, 2, 1, 1, 0, 0]
        );
    }

    #[test]
    fn test_decreasing() {
        assert_eq!(daily_temperatures(&[3, 2, 1]), vec![0, 0, 0]);
    }

    #[test]
    fn test_increasing() {
        assert_eq!(daily_temperatures(&[1, 2, 3]), vec![1, 1, 0]);
    }

    #[test]
    fn test_single() {
        assert_eq!(daily_temperatures(&[50]), vec![0]);
    }
}
