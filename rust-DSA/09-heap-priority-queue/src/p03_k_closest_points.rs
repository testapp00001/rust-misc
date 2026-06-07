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
    // Use max-heap to keep k closest points (smallest distances)
    let mut heap: BinaryHeap<(i32, usize)> = BinaryHeap::new();

    for (i, point) in points.iter().enumerate() {
        let dist = point[0] * point[0] + point[1] * point[1];
        heap.push((dist, i));
        if heap.len() > k as usize {
            heap.pop(); // Remove the farthest point
        }
    }

    heap.into_iter().map(|(_, i)| points[i].clone()).collect()
}

// Alternative: Sort approach
pub fn k_closest_sort(mut points: Vec<Vec<i32>>, k: i32) -> Vec<Vec<i32>> {
    points.sort_by_key(|p| p[0] * p[0] + p[1] * p[1]);
    points.into_iter().take(k as usize).collect()
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
}
