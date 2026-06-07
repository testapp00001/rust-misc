// ============================================================================
// Problem: Word Search II (LeetCode #212)
// ============================================================================
// Given an m x n board of characters and a list of words, return all words
// on the board using adjacent cells (horizontally or vertically).
//
// Each cell can be used at most once per word.
//
// ============================================================================
// APPROACH: Trie + DFS Backtracking (O(m*n*4^L) time, O(W*L) space)
// ============================================================================
//
// 1. Build a Trie from the word list.
// 2. For each cell on the board, start a DFS.
// 3. Use the Trie to prune invalid paths early.
// 4. Mark found words to avoid duplicates.
//
// This is much more efficient than searching for each word separately.
// ============================================================================

use std::collections::HashSet;

#[derive(Default)]
struct TrieNode {
    children: [Option<Box<TrieNode>>; 26],
    word: Option<String>,
}

impl TrieNode {
    fn insert(&mut self, word: &str) {
        let mut node = self;
        for c in word.bytes() {
            let idx = (c - b'a') as usize;
            node = node.children[idx].get_or_insert_with(|| Box::new(TrieNode::default()));
        }
        node.word = Some(word.to_string());
    }
}

pub fn find_words(board: Vec<Vec<char>>, words: Vec<String>) -> Vec<String> {
    // Build Trie
    let mut root = TrieNode::default();
    for word in &words {
        root.insert(word);
    }

    let rows = board.len();
    let cols = board[0].len();
    let mut result = HashSet::new();
    let mut visited = vec![vec![false; cols]; rows];

    for r in 0..rows {
        for c in 0..cols {
            dfs(&board, r, c, &root, &mut visited, &mut result);
        }
    }

    result.into_iter().collect()
}

fn dfs(
    board: &[Vec<char>],
    r: usize,
    c: usize,
    node: &TrieNode,
    visited: &mut Vec<Vec<bool>>,
    result: &mut HashSet<String>,
) {
    if r >= board.len() || c >= board[0].len() || visited[r][c] {
        return;
    }

    let ch = board[r][c];
    let idx = (ch as u8 - b'a') as usize;
    let child = match &node.children[idx] {
        Some(child) => child,
        None => return,
    };

    if let Some(ref word) = child.word {
        result.insert(word.clone());
    }

    visited[r][c] = true;

    let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
    for (dr, dc) in directions {
        let nr = r as i32 + dr;
        let nc = c as i32 + dc;
        if nr >= 0 && nc >= 0 {
            dfs(board, nr as usize, nc as usize, child, visited, result);
        }
    }

    visited[r][c] = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_char_vec(board: &[&str]) -> Vec<Vec<char>> {
        board.iter().map(|s| s.chars().collect()).collect()
    }

    fn sorted(mut v: Vec<String>) -> Vec<String> {
        v.sort();
        v
    }

    #[test]
    fn test_basic() {
        let board = to_char_vec(&["oaan", "etae", "ihkr", "iflv"]);
        let words: Vec<String> = ["oath", "pea", "eat", "rain"]
            .iter().map(|s| s.to_string()).collect();
        let mut result = sorted(find_words(board, words));
        result.sort();
        assert_eq!(result, vec!["eat", "oath"]);
    }

    #[test]
    fn test_no_match() {
        let board = to_char_vec(&["ab", "cd"]);
        let words: Vec<String> = ["xyz"].iter().map(|s| s.to_string()).collect();
        assert_eq!(find_words(board, words), Vec::<String>::new());
    }
}
