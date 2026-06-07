// ============================================================================
// Problem: N-Queens (LeetCode #51)
// ============================================================================
// The n-queens puzzle is the problem of placing n queens on an n×n
// chessboard such that no two queens attack each other.
//
// Return all distinct solutions.
//
// Example:
//   n = 4
//   Output: [[".Q..","...Q","Q...","..Q."], ["..Q.","Q...","...Q",".Q.."]]
//
// ============================================================================
// APPROACH: Backtracking (O(n!) time, O(n²) space)
// ============================================================================
//
// Place queens row by row. For each row, try each column:
// 1. Check if placing a queen at (row, col) is safe.
// 2. If safe, place it and recurse to the next row.
// 3. When all rows are filled, we have a solution.
//
// Safety check: No queen in the same column, or same diagonal.
// ============================================================================

pub fn solve_n_queens(n: i32) -> Vec<Vec<String>> {
    let n = n as usize;
    let mut result = Vec::new();
    let mut board = vec![vec!['.'; n]; n];
    backtrack(&mut board, 0, &mut result);
    result
}

fn backtrack(board: &mut Vec<Vec<char>>, row: usize, result: &mut Vec<Vec<String>>) {
    if row == board.len() {
        result.push(board.iter().map(|r| r.iter().collect()).collect());
        return;
    }

    for col in 0..board.len() {
        if is_safe(board, row, col) {
            board[row][col] = 'Q';
            backtrack(board, row + 1, result);
            board[row][col] = '.';
        }
    }
}

fn is_safe(board: &[Vec<char>], row: usize, col: usize) -> bool {
    // Check column
    for i in 0..row {
        if board[i][col] == 'Q' {
            return false;
        }
    }

    // Check upper-left diagonal
    let mut r = row as i32 - 1;
    let mut c = col as i32 - 1;
    while r >= 0 && c >= 0 {
        if board[r as usize][c as usize] == 'Q' {
            return false;
        }
        r -= 1;
        c -= 1;
    }

    // Check upper-right diagonal
    let mut r = row as i32 - 1;
    let mut c = col as i32 + 1;
    while r >= 0 && (c as usize) < board.len() {
        if board[r as usize][c as usize] == 'Q' {
            return false;
        }
        r -= 1;
        c += 1;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_n4() {
        let result = solve_n_queens(4);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_n1() {
        let result = solve_n_queens(1);
        assert_eq!(result, vec![vec!["Q"]]);
    }

    #[test]
    fn test_n2() {
        assert_eq!(solve_n_queens(2).len(), 0);
    }

    #[test]
    fn test_n3() {
        assert_eq!(solve_n_queens(3).len(), 0);
    }

    #[test]
    fn test_n8() {
        assert_eq!(solve_n_queens(8).len(), 92);
    }
}
