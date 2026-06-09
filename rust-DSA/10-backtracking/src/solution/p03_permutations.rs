// ============================================================================
// Problem: Permutations (LeetCode #46)
// ============================================================================
// Given an array `nums` of distinct integers, return all possible permutations.
//
// Example:
//   Input:  [1, 2, 3]
//   Output: [[1,2,3],[1,3,2],[2,1,3],[2,3,1],[3,1,2],[3,2,1]]
//
// ============================================================================
// APPROACH: Backtracking (O(n! * n) time, O(n) space)
// ============================================================================
//
// At each position, try every unused number:
// 1. Use a `visited` array to track which numbers are used.
// 2. For each unused number, add it and recurse.
// 3. When current.len() == nums.len(), we have a complete permutation.
// ============================================================================

pub fn permute(nums: &[i32]) -> Vec<Vec<i32>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut visited = vec![false; nums.len()];
    backtrack(nums, &mut visited, &mut current, &mut result);
    result
}

fn backtrack(
    nums: &[i32],
    visited: &mut [bool],
    current: &mut Vec<i32>,
    result: &mut Vec<Vec<i32>>,
) {
    if current.len() == nums.len() {
        result.push(current.clone());
        return;
    }

    for i in 0..nums.len() {
        if !visited[i] {
            visited[i] = true;
            current.push(nums[i]);
            backtrack(nums, visited, current, result);
            current.pop();
            visited[i] = false;
        }
    }
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
        let result = sorted(permute(&[1, 2, 3]));
        let expected = sorted(vec![
            vec![1, 2, 3], vec![1, 3, 2], vec![2, 1, 3],
            vec![2, 3, 1], vec![3, 1, 2], vec![3, 2, 1],
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_two() {
        let result = sorted(permute(&[1, 2]));
        assert_eq!(result, sorted(vec![vec![1, 2], vec![2, 1]]));
    }

    #[test]
    fn test_single() {
        assert_eq!(permute(&[1]), vec![vec![1]]);
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
        //     let _ = permute(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}