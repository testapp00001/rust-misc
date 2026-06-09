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
    if heights.is_empty() {
        return vec![];
    }

    let rows = heights.len();
    let cols = heights[0].len();

    let mut pacific = vec![vec![false; cols]; rows];
    let mut atlantic = vec![vec![false; cols]; rows];

    // BFS from Pacific border (top row, left column)
    let mut queue = VecDeque::new();
    for r in 0..rows {
        queue.push_back((r, 0));
        pacific[r][0] = true;
    }
    for c in 1..cols {
        queue.push_back((0, c));
        pacific[0][c] = true;
    }
    bfs(&heights, &mut queue, &mut pacific);

    // BFS from Atlantic border (bottom row, right column)
    queue.clear();
    for r in 0..rows {
        queue.push_back((r, cols - 1));
        atlantic[r][cols - 1] = true;
    }
    for c in 0..cols - 1 {
        queue.push_back((rows - 1, c));
        atlantic[rows - 1][c] = true;
    }
    bfs(&heights, &mut queue, &mut atlantic);

    // Find intersection
    let mut result = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            if pacific[r][c] && atlantic[r][c] {
                result.push(vec![r as i32, c as i32]);
            }
        }
    }

    result
}

fn bfs(
    heights: &[Vec<i32>],
    queue: &mut VecDeque<(usize, usize)>,
    visited: &mut [Vec<bool>],
) {
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    while let Some((r, c)) = queue.pop_front() {
        for (dr, dc) in directions {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr >= 0
                && nc >= 0
                && (nr as usize) < heights.len()
                && (nc as usize) < heights[0].len()
                && !visited[nr as usize][nc as usize]
                && heights[nr as usize][nc as usize] >= heights[r][c]
            {
                visited[nr as usize][nc as usize] = true;
                queue.push_back((nr as usize, nc as usize));
            }
        }
    }
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