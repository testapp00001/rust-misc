// ============================================================================
// Problem: Find Minimum in Rotated Sorted Array (LeetCode #153)
// ============================================================================
// Suppose an array of length `n` sorted in ascending order is rotated
// between 1 and n times. Given the sorted rotated array `nums`, return
// the minimum element.
//
// You must write an algorithm that runs in O(log n) time.
//
// Example:
//   Input:  [3,4,5,1,2]
//   Output: 1
//
// ============================================================================
// APPROACH: Binary Search (O(log n) time, O(1) space)
// ============================================================================
//
// The minimum is at the "rotation point". Use binary search:
// 1. If nums[mid] > nums[right] → minimum is in the right half.
// 2. If nums[mid] <= nums[right] → minimum is in the left half (including mid).
// 3. Continue until left == right.
//
// Why compare with right: The minimum is always in the unsorted half.
// If nums[mid] > nums[right], the right half is unsorted.
// ============================================================================

pub fn find_min(nums: &[i32]) -> i32 {
    let mut left = 0;
    let mut right = nums.len() - 1;

    while left < right {
        let mid = left + (right - left) / 2;
        if nums[mid] > nums[right] {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    nums[left]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotated() {
        assert_eq!(find_min(&[3, 4, 5, 1, 2]), 1);
    }

    #[test]
    fn test_rotated_more() {
        assert_eq!(find_min(&[4, 5, 6, 7, 0, 1, 2]), 0);
    }

    #[test]
    fn test_not_rotated() {
        assert_eq!(find_min(&[1, 2, 3, 4, 5]), 1);
    }

    #[test]
    fn test_single() {
        assert_eq!(find_min(&[1]), 1);
    }

    #[test]
    fn test_two() {
        assert_eq!(find_min(&[2, 1]), 1);
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
        //     let _ = find_min(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}