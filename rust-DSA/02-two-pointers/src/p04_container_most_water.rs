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
    todo!("Implement max_area")
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
        //     let _ = max_area(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}