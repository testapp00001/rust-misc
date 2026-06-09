// ============================================================================
// Problem: K Closest Points to Origin (LeetCode #973)
// ============================================================================
// Given an array of points and an integer k, return the k closest points
// to the origin (0, 0). Distance is Euclidean: sqrt(x² + y²).
//
// Example:
//   Input:  points = [[1,3],[-2,2]], k = 1
//   Output: [[-2,2]]
//
// ============================================================================
// APPROACH: Max Heap of size k (O(n log k) time, O(k) space)
// ============================================================================
//
// Use a max heap to track the k closest points:
// 1. For each point, calculate distance² (no need for sqrt).
// 2. Push to heap with distance as key.
// 3. If heap size > k, pop the farthest.
//
// Rust: BinaryHeap with custom comparator (Reverse for min-heap behavior).
// ============================================================================


use std::collections::BinaryHeap;
use std::cmp::Reverse;

pub fn k_closest(points: Vec<Vec<i32>>, k: i32) -> Vec<Vec<i32>> {
    todo!("Implement k_closest")
}

pub fn k_closest_sort(mut points: Vec<Vec<i32>>, k: i32) -> Vec<Vec<i32>> {
    todo!("Implement k_closest_sort")
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
        let result = sorted(k_closest(vec![vec![1, 3], vec![-2, 2]], 1));
        assert_eq!(result, vec![vec![-2, 2]]);
    }

    #[test]
    fn test_multiple() {
        let result = sorted(k_closest(
            vec![vec![3, 3], vec![5, -1], vec![-2, 4]], 2,
        ));
        assert_eq!(result, sorted(vec![vec![3, 3], vec![-2, 4]]));
    }

    #[test]
    fn test_sort_approach() {
        let result = sorted(k_closest_sort(vec![vec![1, 3], vec![-2, 2]], 1));
        assert_eq!(result, vec![vec![-2, 2]]);
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
        //     let _ = k_closest(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}