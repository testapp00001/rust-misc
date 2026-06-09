// ============================================================================
// Problem: Jump Game (LeetCode #55)
// ============================================================================
// You are given an array `nums` where each element represents the maximum
// jump length from that position. Return true if you can reach the last index.
//
// Example:
//   Input:  [2,3,1,1,4]
//   Output: true  (jump 1 step from 0 to 1, then 3 steps to the last)
//
// ============================================================================
// APPROACH: Greedy (O(n) time, O(1) space)
// ============================================================================
//
// Track the farthest position we can reach:
// 1. For each position i, if i > farthest → can't reach i → return false.
// 2. Update farthest = max(farthest, i + nums[i]).
// 3. If farthest >= last index → return true.
// ============================================================================



pub fn can_jump(nums: &[i32]) -> bool {
    todo!("Implement can_jump")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_jump() {
        assert!(can_jump(&[2, 3, 1, 1, 4]));
    }

    #[test]
    fn test_cannot_jump() {
        assert!(!can_jump(&[3, 2, 1, 0, 4]));
    }

    #[test]
    fn test_single() {
        assert!(can_jump(&[0]));
    }

    #[test]
    fn test_zero() {
        assert!(!can_jump(&[1, 0, 1]));
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
        //     let _ = can_jump(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}