// ============================================================================
// Problem: Trapping Rain Water (LeetCode #42)
// ============================================================================
// Given `n` non-negative integers representing an elevation map where the
// width of each bar is 1, compute how much water it can trap after raining.
//
// Example:
//   Input:  height = [0,1,0,2,1,0,1,3,2,1,2,1]
//   Output: 6
//
// ============================================================================
// APPROACH: Two Pointers (O(n) time, O(1) space)
// ============================================================================
//
// Water at position i = min(max_left[i], max_right[i]) - height[i]
//
// Two-pointer approach:
// 1. left=0, right=n-1, left_max=0, right_max=0.
// 2. If height[left] < height[right]:
//    - If height[left] >= left_max: update left_max.
//    - Else: water += left_max - height[left].
//    - Move left right.
// 3. Else: symmetric for right.
//
// Why this works: We process from both ends. The side with the shorter
// height determines the water level (limited by the other side's max).
// ============================================================================



pub fn trap(height: Vec<i32>) -> i32 {
    todo!("Implement trap")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(trap(vec![0, 1, 0, 2, 1, 0, 1, 3, 2, 1, 2, 1]), 6);
    }

    #[test]
    fn test_no_water() {
        assert_eq!(trap(vec![1, 2, 3, 4, 5]), 0);
    }

    #[test]
    fn test_flat() {
        assert_eq!(trap(vec![5, 5, 5, 5]), 0);
    }

    #[test]
    fn test_valley() {
        assert_eq!(trap(vec![5, 1, 5]), 4);
    }

    #[test]
    fn test_empty() {
        assert_eq!(trap(vec![]), 0);
    }

    #[test]
    fn test_single() {
        assert_eq!(trap(vec![5]), 0);
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
        //     let _ = trap(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}