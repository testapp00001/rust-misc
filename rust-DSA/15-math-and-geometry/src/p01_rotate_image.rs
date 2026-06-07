// ============================================================================
// Problem: Rotate Image (LeetCode #48)
// ============================================================================
// Given an n x n matrix, rotate it 90 degrees clockwise in-place.
//
// Example:
//   Input:  [[1,2,3],[4,5,6],[7,8,9]]
//   Output: [[7,4,1],[8,5,2],[9,6,3]]
//
// ============================================================================
// APPROACH: Transpose + Reverse Rows (O(n²) time, O(1) space)
// ============================================================================
//
// 90° clockwise rotation = transpose + reverse each row
//
// Transpose: swap matrix[i][j] with matrix[j][i]
// Reverse: reverse each row
// ============================================================================

pub fn rotate(matrix: &mut Vec<Vec<i32>>) {
    let n = matrix.len();

    // Transpose
    for i in 0..n {
        for j in i + 1..n {
            let temp = matrix[i][j];
            matrix[i][j] = matrix[j][i];
            matrix[j][i] = temp;
        }
    }

    // Reverse each row
    for row in matrix.iter_mut() {
        row.reverse();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        rotate(&mut matrix);
        assert_eq!(matrix, vec![vec![7, 4, 1], vec![8, 5, 2], vec![9, 6, 3]]);
    }

    #[test]
    fn test_4x4() {
        let mut matrix = vec![
            vec![5, 1, 9, 11],
            vec![2, 4, 8, 10],
            vec![13, 3, 6, 7],
            vec![15, 14, 12, 16],
        ];
        rotate(&mut matrix);
        assert_eq!(
            matrix,
            vec![
                vec![15, 13, 2, 5],
                vec![14, 3, 4, 1],
                vec![12, 6, 8, 9],
                vec![16, 7, 10, 11],
            ]
        );
    }

    #[test]
    fn test_single() {
        let mut matrix = vec![vec![1]];
        rotate(&mut matrix);
        assert_eq!(matrix, vec![vec![1]]);
    }
}
