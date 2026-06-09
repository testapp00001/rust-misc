// ============================================================================
// Problem: Rotting Oranges (LeetCode #994)
// ============================================================================
// Given an m x n grid where:
// - 0 = empty, 1 = fresh orange, 2 = rotten orange
// Each minute, rotten oranges rot adjacent fresh oranges.
// Return the minimum time until no fresh orange remains (-1 if impossible).
//
// ============================================================================
// APPROACH: Multi-source BFS (O(m*n) time, O(m*n) space)
// ============================================================================
//
// 1. Start BFS from all rotten oranges simultaneously.
// 2. Track time as BFS levels.
// 3. After BFS, check if any fresh orange remains.
// ============================================================================

use std::collections::VecDeque;

pub fn oranges_rotting(grid: &[Vec<i32>]) -> i32 {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut grid = grid.to_vec();
    let mut queue = VecDeque::new();
    let mut fresh = 0;

    // Find all rotten oranges and count fresh ones
    for r in 0..rows {
        for c in 0..cols {
            if grid[r][c] == 2 {
                queue.push_back((r, c));
            } else if grid[r][c] == 1 {
                fresh += 1;
            }
        }
    }

    if fresh == 0 {
        return 0;
    }

    let mut minutes = 0;
    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

    while !queue.is_empty() {
        let level_size = queue.len();
        let mut rotted = false;

        for _ in 0..level_size {
            let (r, c) = queue.pop_front().unwrap();

            for (dr, dc) in directions {
                let nr = r as i32 + dr;
                let nc = c as i32 + dc;
                if nr >= 0
                    && nc >= 0
                    && (nr as usize) < rows
                    && (nc as usize) < cols
                    && grid[nr as usize][nc as usize] == 1
                {
                    grid[nr as usize][nc as usize] = 2;
                    fresh -= 1;
                    rotted = true;
                    queue.push_back((nr as usize, nc as usize));
                }
            }
        }

        if rotted {
            minutes += 1;
        }
    }

    if fresh == 0 { minutes } else { -1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let grid = vec![vec![2, 1, 1], vec![1, 1, 0], vec![0, 1, 1]];
        assert_eq!(oranges_rotting(&grid), 4);
    }

    #[test]
    fn test_impossible() {
        let grid = vec![vec![2, 1, 1], vec![0, 1, 1], vec![1, 0, 1]];
        assert_eq!(oranges_rotting(&grid), -1);
    }

    #[test]
    fn test_no_fresh() {
        let grid = vec![vec![2, 2], vec![0, 0]];
        assert_eq!(oranges_rotting(&grid), 0);
    }

    #[test]
    fn test_no_rotten() {
        let grid = vec![vec![1, 1], vec![1, 1]];
        assert_eq!(oranges_rotting(&grid), -1);
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
        //     let _ = oranges_rotting(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}