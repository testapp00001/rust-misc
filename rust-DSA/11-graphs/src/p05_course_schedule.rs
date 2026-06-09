// ============================================================================
// Problem: Course Schedule (LeetCode #207)
// ============================================================================
// There are `numCourses` courses. Some have prerequisites. Return true if
// you can finish all courses (no cycle in the dependency graph).
//
// Example:
//   numCourses = 2, prerequisites = [[1,0]]
//   Output: true  (take course 0, then course 1)
//
// ============================================================================
// APPROACH: Topological Sort / Cycle Detection (O(V+E) time, O(V+E) space)
// ============================================================================
//
// Use DFS to detect cycles in the directed graph:
// 1. Build adjacency list from prerequisites.
// 2. For each unvisited node, run DFS.
// 3. If we visit a node that's currently in the recursion stack → cycle.
// 4. If no cycle found → all courses can be finished.
//
// Alternative: Kahn's algorithm (BFS-based topological sort).
// ============================================================================


use std::collections::VecDeque;

pub fn can_finish(num_courses: i32, prerequisites: Vec<Vec<i32>>) -> bool {
    todo!("Implement can_finish")
}

pub fn can_finish_bfs(num_courses: i32, prerequisites: Vec<Vec<i32>>) -> bool {
    todo!("Implement can_finish_bfs")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_finish() {
        assert!(can_finish(2, vec![vec![1, 0]]));
    }

    #[test]
    fn test_cannot_finish() {
        assert!(!can_finish(2, vec![vec![1, 0], vec![0, 1]]));
    }

    #[test]
    fn test_no_prereqs() {
        assert!(can_finish(3, vec![]));
    }

    #[test]
    fn test_bfs_approach() {
        assert!(can_finish_bfs(2, vec![vec![1, 0]]));
        assert!(!can_finish_bfs(2, vec![vec![1, 0], vec![0, 1]]));
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
        //     let _ = can_finish(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}