// ============================================================================
// Problem: Number of Islands (LeetCode #200)
// ============================================================================
// Given an m x n grid of '1's (land) and '0's (water), count the number
// of islands. An island is surrounded by water and formed by connecting
// adjacent lands horizontally or vertically.
//
// Example:
//   Input:  [["1","1","0","0","0"],
//            ["1","1","0","0","0"],
//            ["0","0","1","0","0"],
//            ["0","0","0","1","1"]]
//   Output: 3
//
// ============================================================================
// APPROACH: DFS Flood Fill (O(m*n) time, O(m*n) space)
// ============================================================================
//
// 1. Iterate through each cell.
// 2. When we find a '1', increment island count and DFS to mark all
//    connected land as visited (set to '0').
// 3. Continue until all cells are processed.
// ============================================================================



pub fn num_islands(grid: &mut [Vec<char>]) -> i32 {
    todo!("Implement num_islands")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_char_grid(grid: &[&[&str]]) -> Vec<Vec<char>> {
        grid.iter().map(|row| row.iter().map(|s| s.chars().next().unwrap()).collect()).collect()
    }

    #[test]
    fn test_basic() {
        let mut grid = to_char_grid(&[
            &["1","1","0","0","0"],
            &["1","1","0","0","0"],
            &["0","0","1","0","0"],
            &["0","0","0","1","1"],
        ]);
        assert_eq!(num_islands(&mut grid), 3);
    }

    #[test]
    fn test_no_islands() {
        let mut grid = to_char_grid(&[&["0","0"], &["0","0"]]);
        assert_eq!(num_islands(&mut grid), 0);
    }

    #[test]
    fn test_all_land() {
        let mut grid = to_char_grid(&[&["1","1"], &["1","1"]]);
        assert_eq!(num_islands(&mut grid), 1);
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
        //     let _ = num_islands(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}