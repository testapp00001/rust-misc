// ============================================================================
// Problem: Search in Rotated Sorted Array (LeetCode #33)
// ============================================================================
// Given a rotated sorted array `nums` and a `target`, return the index of
// `target` if it exists, otherwise return -1.
//
// Example:
//   Input:  nums = [4,5,6,7,0,1,2], target = 0
//   Output: 4
//
// ============================================================================
// APPROACH: Modified Binary Search (O(log n) time, O(1) space)
// ============================================================================
//
// At each step, one half of the array is sorted:
// 1. If left half [left..mid] is sorted:
//    - If target is in [nums[left], nums[mid]) → search left.
//    - Else → search right.
// 2. If right half [mid..right] is sorted:
//    - If target is in (nums[mid], nums[right]) → search right.
//    - Else → search left.
// ============================================================================

pub fn search_rotated(nums: &[i32], target: i32) -> i32 {
    let mut left = 0;
    let mut right = nums.len();

    while left < right {
        let mid = left + (right - left) / 2;

        if nums[mid] == target {
            return mid as i32;
        }

        // Check if left half is sorted
        if nums[left] <= nums[mid] {
            // Target is in the sorted left half
            if nums[left] <= target && target < nums[mid] {
                right = mid;
            } else {
                left = mid + 1;
            }
        } else {
            // Right half is sorted
            if nums[mid] < target && target <= nums[right - 1] {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
    }

    -1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_found() {
        assert_eq!(search_rotated(&[4, 5, 6, 7, 0, 1, 2], 0), 4);
    }

    #[test]
    fn test_not_found() {
        assert_eq!(search_rotated(&[4, 5, 6, 7, 0, 1, 2], 3), -1);
    }

    #[test]
    fn test_not_rotated() {
        assert_eq!(search_rotated(&[1, 2, 3, 4], 3), 2);
    }

    #[test]
    fn test_single() {
        assert_eq!(search_rotated(&[1], 1), 0);
        assert_eq!(search_rotated(&[1], 0), -1);
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
        //     let _ = search_rotated(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}