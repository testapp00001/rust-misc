# Trie

## Overview
A Trie (prefix tree) is a tree-like data structure for efficient string operations. Each node represents a character, and paths from root to marked nodes form words.

## Key Concepts

### TrieNode Structure
```rust
#[derive(Default)]
struct TrieNode {
    children: [Option<Box<TrieNode>>; 26], // For lowercase English
    is_end: bool,
}
```

### Operations
- **Insert**: O(m) where m is word length
- **Search**: O(m)
- **StartsWith**: O(m)

## Common Patterns

### 1. Basic Trie
```rust
struct Trie {
    root: TrieNode,
}

impl Trie {
    fn new() -> Self {
        Trie { root: TrieNode::default() }
    }

    fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for c in word.bytes() {
            let idx = (c - b'a') as usize;
            node = node.children[idx].get_or_insert_with(|| Box::new(TrieNode::default()));
        }
        node.is_end = true;
    }

    fn search(&self, word: &str) -> bool {
        self.find_node(word).map_or(false, |n| n.is_end)
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.find_node(prefix).is_some()
    }

    fn find_node(&self, prefix: &str) -> Option<&TrieNode> {
        let mut node = &self.root;
        for c in prefix.bytes() {
            let idx = (c - b'a') as usize;
            match &node.children[idx] {
                Some(child) => node = child,
                None => return None,
            }
        }
        Some(node)
    }
}
```

### 2. Word Search II (Trie + DFS)
```rust
fn find_words(board: Vec<Vec<char>>, words: Vec<String>) -> Vec<String> {
    // Build Trie from words
    let mut root = TrieNode::default();
    for word in &words {
        root.insert(word);
    }

    // DFS on board using Trie for pruning
    let mut result = HashSet::new();
    let mut visited = vec![vec![false; board[0].len()]; board.len()];

    for r in 0..board.len() {
        for c in 0..board[0].len() {
            dfs(&board, r, c, &root, &mut visited, &mut result);
        }
    }

    result.into_iter().collect()
}
```

## Problems in This Topic

| # | Problem | Difficulty | Key Insight |
|---|---------|------------|-------------|
| 1 | Implement Trie | Medium | Array of children |
| 2 | Word Search II | Hard | Trie + DFS backtracking |

## Tips for Rust

1. **`get_or_insert_with()`**: Insert if not present, return mutable reference.
2. **Array of `Option<Box<T>>`**: Efficient for fixed alphabet size.
3. **`Default` trait**: Derive for automatic initialization.
