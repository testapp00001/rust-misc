// ============================================================================
// Problem: Max Area of Island (LeetCode #695)
// ============================================================================
// Given an m x n binary matrix grid, return the maximum area of an island.
// An island is a group of 1s connected horizontally or vertically.
//
// ============================================================================
// APPROACH: DFS (O(m*n) time, O(m*n) space)
// ============================================================================
//
// Similar to Number of Islands, but track the area of each island.
// ============================================================================



pub fn max_area_of_island(grid: &mut [Vec<i32>]) -> i32 {
    todo!("Implement max_area_of_island")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut grid = vec![
            vec![0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
            vec![0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0],
            vec![0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0],
            vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0],
        ];
        assert_eq!(max_area_of_island(&mut grid), 6);
    }

    #[test]
    fn test_no_island() {
        let mut grid = vec![vec![0, 0, 0, 0]];
        assert_eq!(max_area_of_island(&mut grid), 0);
    }

    #[test]
    fn test_all_land() {
        let mut grid = vec![vec![1, 1], vec![1, 1]];
        assert_eq!(max_area_of_island(&mut grid), 4);
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
        //     let _ = max_area_of_island(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}