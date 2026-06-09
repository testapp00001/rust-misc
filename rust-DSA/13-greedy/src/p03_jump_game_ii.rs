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
    todo!("Implement jump")
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
        //     let _ = jump(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}