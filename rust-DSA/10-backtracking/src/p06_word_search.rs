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
    let rows = board.len();
    let cols = board[0].len();
    let word_chars: Vec<char> = word.chars().collect();

    for r in 0..rows {
        for c in 0..cols {
            if dfs(board, r, c, &word_chars, 0) {
                return true;
            }
        }
    }

    false
}

fn dfs(board: &mut [Vec<char>], r: usize, c: usize, word: &[char], idx: usize) -> bool {
    if idx == word.len() {
        return true;
    }
    if r >= board.len() || c >= board[0].len() || board[r][c] != word[idx] {
        return false;
    }

    let saved = board[r][c];
    board[r][c] = '#'; // Mark as visited

    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    for (dr, dc) in directions {
        let nr = r as i32 + dr;
        let nc = c as i32 + dc;
        if nr >= 0 && nc >= 0 && dfs(board, nr as usize, nc as usize, word, idx + 1) {
            board[r][c] = saved;
            return true;
        }
    }

    board[r][c] = saved; // Unmark
    false
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
}
