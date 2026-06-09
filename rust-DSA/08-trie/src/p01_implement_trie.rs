// ============================================================================
// Problem: Implement Trie (LeetCode #208)
// ============================================================================
// Implement a trie with insert, search, and startsWith operations.
//
// Example:
//   trie.insert("apple");
//   trie.search("apple");    // true
//   trie.search("app");      // false
//   trie.starts_with("app"); // true
//   trie.insert("app");
//   trie.search("app");      // true
//
// ============================================================================
// APPROACH: Trie (Prefix Tree)
// ============================================================================
//
// A trie is a tree-like data structure where each node represents a character.
// - Each node has up to 26 children (for lowercase English letters).
// - A boolean flag marks if a node is the end of a word.
//
// Operations:
// - insert: Traverse/create nodes for each character, mark last as end.
// - search: Traverse nodes for each character, check if last is end.
// - starts_with: Traverse nodes for each character, return true if path exists.
//
// Rust-specific tips:
// - Use `Option<Box<TrieNode>>` for children (or a Vec/HashMap).
// - Consider using an array of 26 Option<Box<TrieNode>> for O(1) access.
// ============================================================================

pub struct Trie {
    // TODO: Add fields to store the trie structure
    // Hint: You might want a root node with children for each letter
}

impl Trie {
    pub fn new() -> Self {
        todo!("Implement Trie::new()")
    }

    pub fn insert(&mut self, word: &str) {
        todo!("Implement Trie::insert()")
    }

    pub fn search(&self, word: &str) -> bool {
        todo!("Implement Trie::search()")
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        todo!("Implement Trie::starts_with()")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_operations() {
        let mut trie = Trie::new();
        trie.insert("apple");
        assert!(trie.search("apple"));
        assert!(!trie.search("app"));
        assert!(trie.starts_with("app"));
        trie.insert("app");
        assert!(trie.search("app"));
    }

    #[test]
    fn test_edge_cases() {
        let mut trie = Trie::new();
        trie.insert("a");
        assert!(trie.search("a"));
        assert!(!trie.search("b"));
        assert!(trie.starts_with("a"));
        assert!(!trie.starts_with("b"));
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
        //     let _ = solution(/* input */);
        // }
        // let elapsed = start.elapsed();
        // println!("\n  ⏱️  {} iterations: {:?}", iterations, elapsed);
        // println!("     Average: {:?}/call", elapsed / iterations);
    }
}