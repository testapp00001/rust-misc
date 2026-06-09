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

pub fn find_words(board: Vec<Vec<char>>, words: Vec<String>) -> Vec<String> {
    todo!("Implement find_words")
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
        //     let _ = find_words(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}