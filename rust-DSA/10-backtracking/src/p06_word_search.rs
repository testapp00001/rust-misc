// ============================================================================
// Problem: Word Search (LeetCode #79)
// ============================================================================
// Given an m x n grid of characters and a string `word`, return true if
// `word` exists in the grid. Letters must be adjacent (horizontally or
// vertically) and each cell can be used only once.
//
// ============================================================================
// APPROACH: DFS Backtracking (O(m*n*4^L) time, O(L) space)
// ============================================================================
//
// For each cell, start a DFS search:
// 1. If current char matches, mark as visited and recurse on neighbors.
// 2. If we reach the end of the word, return true.
// 3. Unmark when backtracking.
// ============================================================================



pub fn exist(board: &mut [Vec<char>], word: &str) -> bool {
    todo!("Implement exist")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_found() {
        let mut board = vec![
            vec!['A', 'B', 'C', 'E'],
            vec!['S', 'F', 'C', 'S'],
            vec!['A', 'D', 'E', 'E'],
        ];
        assert!(exist(&mut board, "ABCCED"));
    }

    #[test]
    fn test_not_found() {
        let mut board = vec![
            vec!['A', 'B', 'C', 'E'],
            vec!['S', 'F', 'C', 'S'],
            vec!['A', 'D', 'E', 'E'],
        ];
        assert!(!exist(&mut board, "ABCB"));
    }

    #[test]
    fn test_single_cell() {
        let mut board = vec![vec!['A']];
        assert!(exist(&mut board, "A"));
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
        //     let _ = exist(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}