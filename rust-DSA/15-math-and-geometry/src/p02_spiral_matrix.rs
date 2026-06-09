// ============================================================================
// Problem: Spiral Matrix (LeetCode #54)
// ============================================================================
// Given an m x n matrix, return all elements in spiral order.
//
// Example:
//   Input:  [[1,2,3],[4,5,6],[7,8,9]]
//   Output: [1,2,3,6,9,8,7,4,5]
//
// ============================================================================
// APPROACH: Layer-by-layer (O(m*n) time, O(1) space)
// ============================================================================
//
// Use four boundaries (top, bottom, left, right) and traverse:
// 1. Left to right along top row.
// 2. Top to bottom along right column.
// 3. Right to left along bottom row (if rows remain).
// 4. Bottom to top along left column (if cols remain).
// ============================================================================



pub fn spiral_order(matrix: Vec<Vec<i32>>) -> Vec<i32> {
    todo!("Implement spiral_order")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(
            spiral_order(vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]]),
            vec![1, 2, 3, 6, 9, 8, 7, 4, 5]
        );
    }

    #[test]
    fn test_rectangular() {
        assert_eq!(
            spiral_order(vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]]),
            vec![1, 2, 3, 4, 8, 12, 11, 10, 9, 5, 6, 7]
        );
    }

    #[test]
    fn test_single_row() {
        assert_eq!(spiral_order(vec![vec![1, 2, 3]]), vec![1, 2, 3]);
    }

    #[test]
    fn test_single_col() {
        assert_eq!(spiral_order(vec![vec![1], vec![2], vec![3]]), vec![1, 2, 3]);
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
        //     let _ = spiral_order(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}