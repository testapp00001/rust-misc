// ============================================================================
// Problem: Climbing Stairs (LeetCode #70)
// ============================================================================
// You are climbing a staircase with n steps. Each time you can climb 1 or
// 2 steps. How many distinct ways can you climb to the top?
//
// Example:
//   n = 3 → Output: 3 (1+1+1, 1+2, 2+1)
//
// ============================================================================
// APPROACH: Dynamic Programming (O(n) time, O(1) space)
// ============================================================================
//
// This is essentially the Fibonacci sequence:
// ways(n) = ways(n-1) + ways(n-2)
//
// Base cases: ways(1) = 1, ways(2) = 2
//
// We only need the last two values, so use two variables instead of an array.
// ============================================================================

pub fn climb_stairs(n: i32) -> i32 {
    if n <= 2 {
        return n;
    }

    let mut prev2 = 1; // ways(1)
    let mut prev1 = 2; // ways(2)

    for _ in 3..=n {
        let current = prev1 + prev2;
        prev2 = prev1;
        prev1 = current;
    }

    prev1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(climb_stairs(2), 2);
        assert_eq!(climb_stairs(3), 3);
    }

    #[test]
    fn test_larger() {
        assert_eq!(climb_stairs(5), 8);
        assert_eq!(climb_stairs(10), 89);
    }

    #[test]
    fn test_base_cases() {
        assert_eq!(climb_stairs(1), 1);
        assert_eq!(climb_stairs(2), 2);
    }
}
