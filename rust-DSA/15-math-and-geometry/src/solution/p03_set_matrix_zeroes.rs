// ============================================================================
// Problem: Set Matrix Zeroes (LeetCode #73)
// ============================================================================
// Given an m x n matrix, if an element is 0, set its entire row and column
// to 0. Do it in-place.
//
// ============================================================================
// APPROACH: Use First Row/Column as Markers (O(m*n) time, O(1) space)
// ============================================================================
//
// 1. Use the first row and first column as markers.
// 2. For each cell (i,j) that is 0, mark matrix[i][0] and matrix[0][j].
// 3. Use a separate variable for the first row (since it's also used as marker).
// 4. Iterate again and set cells to 0 based on markers.
// ============================================================================

pub fn set_zeroes(matrix: &mut Vec<Vec<i32>>) {
    let rows = matrix.len();
    let cols = matrix[0].len();
    let mut first_row_zero = false;

    // Check if first row has a zero
    for c in 0..cols {
        if matrix[0][c] == 0 {
            first_row_zero = true;
            break;
        }
    }

    // Use first row/column as markers
    for r in 1..rows {
        for c in 0..cols {
            if matrix[r][c] == 0 {
                matrix[r][0] = 0;
                matrix[0][c] = 0;
            }
        }
    }

    // Set cells to 0 based on markers (skip first row/column)
    for r in 1..rows {
        for c in 1..cols {
            if matrix[r][0] == 0 || matrix[0][c] == 0 {
                matrix[r][c] = 0;
            }
        }
    }

    // Handle first column
    if matrix[0][0] == 0 {
        for r in 0..rows {
            matrix[r][0] = 0;
        }
    }

    // Handle first row
    if first_row_zero {
        for c in 0..cols {
            matrix[0][c] = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut matrix = vec![vec![1, 1, 1], vec![1, 0, 1], vec![1, 1, 1]];
        set_zeroes(&mut matrix);
        assert_eq!(matrix, vec![vec![1, 0, 1], vec![0, 0, 0], vec![1, 0, 1]]);
    }

    #[test]
    fn test_multiple_zeros() {
        let mut matrix = vec![vec![0, 1, 2, 0], vec![3, 4, 5, 2], vec![1, 3, 1, 5]];
        set_zeroes(&mut matrix);
        assert_eq!(
            matrix,
            vec![vec![0, 0, 0, 0], vec![0, 4, 5, 0], vec![0, 3, 1, 0]]
        );
    }

    #[test]
    fn test_single() {
        let mut matrix = vec![vec![1]];
        set_zeroes(&mut matrix);
        assert_eq!(matrix, vec![vec![1]]);
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
        //     let _ = set_zeroes(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}