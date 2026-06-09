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
    if matrix.is_empty() {
        return vec![];
    }

    let mut result = Vec::new();
    let mut top = 0;
    let mut bottom = matrix.len() - 1;
    let mut left = 0;
    let mut right = matrix[0].len() - 1;

    while top <= bottom && left <= right {
        // Left to right
        for c in left..=right {
            result.push(matrix[top][c]);
        }
        top += 1;

        // Top to bottom
        for r in top..=bottom {
            result.push(matrix[r][right]);
        }
        if right == 0 { break; }
        right -= 1;

        // Right to left
        if top <= bottom {
            for c in (left..=right).rev() {
                result.push(matrix[bottom][c]);
            }
            if bottom == 0 { break; }
            bottom -= 1;
        }

        // Bottom to top
        if left <= right {
            for r in (top..=bottom).rev() {
                result.push(matrix[r][left]);
            }
            left += 1;
        }
    }

    result
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