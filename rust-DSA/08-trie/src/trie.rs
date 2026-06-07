// ============================================================================
// Trie Data Structure Implementation
// ============================================================================
// A Trie (prefix tree) is a tree-like data structure used for efficient
// retrieval of keys in a dataset of strings.
//
// Key properties:
// - Each node represents a character.
// - Root is empty.
// - Each path from root to a marked node represents a word.
//
// Operations:
// - insert(word): O(m) where m is word length.
// - search(word): O(m)
// - starts_with(prefix): O(m)
//
// Use cases:
// - Autocomplete
// - Spell checkers
// - IP routing
// - Word games
//
// Rust-specific tips:
// - Use `Option<Box<TrieNode>>` for children (like a linked list).
// - Array of 26 for lowercase English letters.
// ============================================================================

#[derive(Default)]
pub struct TrieNode {
    children: [Option<Box<TrieNode>>; 26],
    is_end: bool,
}

impl TrieNode {
    pub fn new() -> Self {
        TrieNode {
            children: Default::default(),
            is_end: false,
        }
    }
}

pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
        }
    }

    pub fn insert(&mut self, word: &str) {
        let mut node = &mut self.root;
        for c in word.bytes() {
            let idx = (c - b'a') as usize;
            node = node.children[idx].get_or_insert_with(|| Box::new(TrieNode::new()));
        }
        node.is_end = true;
    }

    pub fn search(&self, word: &str) -> bool {
        self.find_node(word).map_or(false, |n| n.is_end)
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let mut trie = Trie::new();
        trie.insert("apple");
        assert!(trie.search("apple"));
        assert!(!trie.search("app"));
        assert!(trie.starts_with("app"));
        trie.insert("app");
        assert!(trie.search("app"));
    }

    #[test]
    fn test_empty() {
        let trie = Trie::new();
        assert!(!trie.search(""));
        assert!(trie.starts_with(""));
    }

    #[test]
    fn test_multiple_words() {
        let mut trie = Trie::new();
        trie.insert("hello");
        trie.insert("world");
        trie.insert("help");
        assert!(trie.search("hello"));
        assert!(trie.search("world"));
        assert!(trie.search("help"));
        assert!(!trie.search("hell"));
        assert!(trie.starts_with("hel"));
    }
}
