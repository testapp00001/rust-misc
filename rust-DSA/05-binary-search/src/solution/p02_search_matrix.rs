// ============================================================================
// Problem: Search a 2D Matrix (LeetCode #74)
// ============================================================================
// Write an efficient algorithm that searches for a value `target` in an
// m x n integer matrix. The matrix has the following properties:
// - Integers in each row are sorted from left to right.
// - The first integer of each row is greater than the last integer of
//   the previous row.
//
// Example:
//   matrix = [[1,3,5,7],[10,11,16,20],[23,30,34,60]], target = 3 → true
//
// ============================================================================
// APPROACH: Binary Search on Flattened Index (O(log(m*n)) time, O(1) space)
// ============================================================================
//
// Treat the 2D matrix as a 1D sorted array:
// - Index i maps to matrix[i/n][i%n].
// - Perform standard binary search on the virtual 1D array.
// ============================================================================

pub fn search_matrix(matrix: &[Vec<i32>], target: i32) -> bool {
    if matrix.is_empty() || matrix[0].is_empty() {
        return false;
    }

    let rows = matrix.len();
    let cols = matrix[0].len();
    let mut left = 0;
    let mut right = rows * cols;

    while left < right {
        let mid = left + (right - left) / 2;
        let val = matrix[mid / cols][mid % cols];
        match val.cmp(&target) {
            std::cmp::Ordering::Equal => return true,
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Greater => right = mid,
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_found() {
        let matrix = vec![vec![1, 3, 5, 7], vec![10, 11, 16, 20], vec![23, 30, 34, 60]];
        assert!(search_matrix(&matrix, 3));
    }

    #[test]
    fn test_not_found() {
        let matrix = vec![vec![1, 3, 5, 7], vec![10, 11, 16, 20], vec![23, 30, 34, 60]];
        assert!(!search_matrix(&matrix, 13));
    }

    #[test]
    fn test_single_element() {
        let matrix = vec![vec![1]];
        assert!(search_matrix(&matrix, 1));
        assert!(!search_matrix(&matrix, 2));
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
        //     let _ = search_matrix(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}