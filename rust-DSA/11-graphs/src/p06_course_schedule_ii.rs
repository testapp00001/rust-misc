// ============================================================================
// Problem: Course Schedule II (LeetCode #210)
// ============================================================================
// Return the ordering of courses to take to finish all courses. If it's
// impossible, return an empty array.
//
// Example:
//   numCourses = 4, prerequisites = [[1,0],[2,0],[3,1],[3,2]]
//   Output: [0,2,1,3] or [0,1,2,3]
//
// ============================================================================
// APPROACH: Kahn's Algorithm / Topological Sort (O(V+E) time, O(V+E) space)
// ============================================================================
//
// BFS-based topological sort:
// 1. Build adjacency list and compute in-degrees.
// 2. Start with nodes having in-degree 0.
// 3. Process nodes, reducing in-degrees of neighbors.
// 4. If we process all nodes → valid order. Otherwise → cycle exists.
// ============================================================================


use std::collections::VecDeque;

pub fn find_order(num_courses: i32, prerequisites: Vec<Vec<i32>>) -> Vec<i32> {
    todo!("Implement find_order")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let result = find_order(4, vec![vec![1, 0], vec![2, 0], vec![3, 1], vec![3, 2]]);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_single() {
        assert_eq!(find_order(1, vec![]), vec![0]);
    }

    #[test]
    fn test_cycle() {
        assert_eq!(find_order(2, vec![vec![1, 0], vec![0, 1]]), vec![]);
    }

    #[test]
    fn test_no_prereqs() {
        let result = find_order(3, vec![]);
        assert_eq!(result.len(), 3);
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
        //     let _ = find_order(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}