// ============================================================================
// Problem: Subsets (LeetCode #78)
// ============================================================================
// Given an integer array `nums` of unique elements, return all possible
// subsets (the power set).
//
// Example:
//   Input:  [1, 2, 3]
//   Output: [[], [1], [2], [3], [1,2], [1,3], [2,3], [1,2,3]]
//
// ============================================================================
// APPROACH: Backtracking (O(n * 2^n) time, O(n) space for recursion)
// ============================================================================
//
// At each element, we have two choices:
// 1. Include it in the current subset.
// 2. Don't include it.
//
// This creates a binary tree of decisions, resulting in 2^n subsets.
//
// Rust-specific tips:
// - Pass `current` by reference and clone when adding to result.
// - Use `start` index to avoid generating duplicates.
// ============================================================================



pub fn subsets(nums: &[i32]) -> Vec<Vec<i32>> {
    todo!("Implement subsets")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted(mut v: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
        v.sort();
        v
    }

    #[test]
    fn test_basic() {
        let result = sorted(subsets(&[1, 2, 3]));
        let expected = sorted(vec![
            vec![], vec![1], vec![2], vec![3],
            vec![1, 2], vec![1, 3], vec![2, 3], vec![1, 2, 3],
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_single() {
        let result = sorted(subsets(&[1]));
        assert_eq!(result, sorted(vec![vec![], vec![1]]));
    }

    #[test]
    fn test_empty() {
        assert_eq!(subsets(&[]), vec![vec![]]);
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
        //     let _ = subsets(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}