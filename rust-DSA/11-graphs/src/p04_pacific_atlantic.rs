// ============================================================================
// Problem: Pacific Atlantic Water Flow (LeetCode #417)
// ============================================================================
// Given an m x n matrix of heights, return all coordinates where water can
// flow to both the Pacific and Atlantic oceans.
//
// Pacific: top and left edges. Atlantic: bottom and right edges.
// Water flows from higher or equal height to lower or equal height.
//
// ============================================================================
// APPROACH: Multi-source BFS/DFS (O(m*n) time, O(m*n) space)
// ============================================================================
//
// 1. Start BFS from all Pacific border cells → mark reachable cells.
// 2. Start BFS from all Atlantic border cells → mark reachable cells.
// 3. Intersection = cells reachable from both oceans.
// ============================================================================


use std::collections::VecDeque;

pub fn pacific_atlantic(heights: Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    todo!("Implement pacific_atlantic")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let heights = vec![
            vec![1, 2, 2, 3, 5],
            vec![3, 2, 3, 4, 4],
            vec![2, 4, 5, 3, 1],
            vec![6, 7, 1, 4, 5],
            vec![5, 1, 1, 2, 4],
        ];
        let result = pacific_atlantic(heights);
        assert!(result.contains(&vec![0, 4]));
        assert!(result.contains(&vec![1, 3]));
        assert!(result.contains(&vec![1, 4]));
        assert!(result.contains(&vec![2, 2]));
        assert!(result.contains(&vec![3, 0]));
        assert!(result.contains(&vec![3, 1]));
        assert!(result.contains(&vec![4, 0]));
    }

    #[test]
    fn test_single() {
        let heights = vec![vec![1]];
        assert_eq!(pacific_atlantic(heights), vec![vec![0, 0]]);
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
        //     let _ = pacific_atlantic(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}