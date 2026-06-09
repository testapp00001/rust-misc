// ============================================================================
// Problem: Two Sum II - Input Array Is Sorted (LeetCode #167)
// ============================================================================
// Given a 1-indexed array of integers `numbers` that is already sorted in
// non-decreasing order, find two numbers such that they add up to a specific
// target number. Return the indices (1-indexed).
//
// Example:
//   Input:  numbers = [2, 7, 11, 15], target = 9
//   Output: [1, 2]
//
// ============================================================================
// APPROACH: Two Pointers (O(n) time, O(1) space)
// ============================================================================
//
// Since the array is sorted, we can use two pointers:
// 1. Left at start, right at end.
// 2. If sum < target → move left right (need bigger sum).
// 3. If sum > target → move right left (need smaller sum).
// 4. If sum == target → found!
//
// This is more space-efficient than the HashMap approach (O(1) vs O(n)).
// ============================================================================

pub fn two_sum_sorted(numbers: &[i32], target: i32) -> Vec<i32> {
    let mut left = 0;
    let mut right = numbers.len() - 1;

    while left < right {
        let sum = numbers[left] + numbers[right];
        match sum.cmp(&target) {
            std::cmp::Ordering::Less => left += 1,
            std::cmp::Ordering::Greater => right -= 1,
            std::cmp::Ordering::Equal => return vec![(left + 1) as i32, (right + 1) as i32],
        }
    }

    vec![] // No solution found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(two_sum_sorted(&[2, 7, 11, 15], 9), vec![1, 2]);
    }

    #[test]
    fn test_middle_elements() {
        assert_eq!(two_sum_sorted(&[2, 3, 4], 6), vec![1, 3]);
    }

    #[test]
    fn test_adjacent() {
        assert_eq!(two_sum_sorted(&[-1, 0], -1), vec![1, 2]);
    }

    #[test]
    fn test_negative_numbers() {
        assert_eq!(two_sum_sorted(&[-3, -2, -1, 0, 1, 2], -1), vec![1, 6]);
    }


    #[test]
    #[ignore]
    fn bench_performance() {
        // ⏱️  Benchmark test
        // Run: cargo test -p <package> bench_performance -- --ignored --nocapture
        //
        // To use: uncomment and customize the code below with your function
        // and realistic test data.
        //
        // let iterations = 10_000;
        // let input = /* generate your test input here */;
        // let start = std::time::Instant::now();
        // for _ in 0..iterations {
        //     let _ = two_sum_sorted(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}