//! # Storage Engines
//!
//! Understanding storage engine internals helps in choosing the right database
//! and optimizing performance. This lesson covers B-tree and LSM-tree data
//! structures, storage abstraction traits, and key-value store design.
//!
//! ## Key Concepts
//! - B-tree structure and operations
//! - LSM-tree (Log-Structured Merge Tree)
//! - Write-ahead logging (WAL)
//! - Storage abstraction traits
//! - Compaction strategies
//! - Bloom filters for read optimization

use std::collections::BTreeMap;
use std::fmt::Debug;

// ---------------------------------------------------------------------------
// 1. Storage Engine Trait
// ---------------------------------------------------------------------------

/// A trait that abstracts over different storage engine implementations.
pub trait StorageEngine: Send + Sync {
    type Error: std::error::Error + Send + Sync;

    /// Get a value by key.
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;

    /// Put a key-value pair.
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;

    /// Delete a key.
    fn delete(&self, key: &[u8]) -> Result<(), Self::Error>;

    /// Check if a key exists.
    fn contains(&self, key: &[u8]) -> Result<bool, Self::Error>;

    /// Iterate over a range of keys.
    fn scan(&self, start: &[u8], end: &[u8]) -> Result<Vec<(Vec<u8>, Vec<u8>)>, Self::Error>;

    /// Flush any pending writes to disk.
    fn flush(&self) -> Result<(), Self::Error>;
}

// ---------------------------------------------------------------------------
// 2. B-Tree Implementation
// ---------------------------------------------------------------------------

/// A simplified in-memory B-tree for learning purposes.
/// Real B-trees are disk-based with page-oriented I/O.
#[derive(Debug)]
pub struct BTree<const ORDER: usize = 4> {
    root: Option<BTreeNode>,
    size: usize,
}

#[derive(Debug, Clone)]
struct BTreeNode {
    keys: Vec<Vec<u8>>,
    values: Vec<Vec<u8>>,
    children: Vec<BTreeNode>,
    is_leaf: bool,
}

impl<const ORDER: usize> BTree<ORDER> {
    pub fn new() -> Self {
        Self {
            root: None,
            size: 0,
        }
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        if self.root.is_none() {
            self.root = Some(BTreeNode {
                keys: vec![key],
                values: vec![value],
                children: Vec::new(),
                is_leaf: true,
            });
            self.size = 1;
            return;
        }

        let root = self.root.as_mut().unwrap();
        if Self::insert_into_node(root, key, value) {
            self.size += 1;
        }
    }

    fn insert_into_node(node: &mut BTreeNode, key: Vec<u8>, value: Vec<u8>) -> bool {
        // Find position
        let pos = node.keys.binary_search(&key);

        match pos {
            Ok(i) => {
                // Key exists, update value
                node.values[i] = value;
                false
            }
            Err(i) => {
                if node.is_leaf {
                    node.keys.insert(i, key);
                    node.values.insert(i, value);
                    true
                } else {
                    Self::insert_into_node(&mut node.children[i], key, value)
                }
            }
        }
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.root.as_ref().and_then(|root| Self::search_node(root, key))
    }

    fn search_node<'a>(node: &'a BTreeNode, key: &[u8]) -> Option<&'a [u8]> {
        match node.keys.binary_search_by(|k| k.as_slice().cmp(key)) {
            Ok(i) => Some(node.values[i].as_slice()),
            Err(i) => {
                if node.is_leaf {
                    None
                } else {
                    Self::search_node(&node.children[i], key)
                }
            }
        }
    }

    pub fn remove(&mut self, key: &[u8]) -> bool {
        if let Some(root) = &mut self.root {
            if Self::remove_from_node(root, key) {
                self.size -= 1;
                if root.keys.is_empty() && !root.children.is_empty() {
                    // Root is empty, promote first child
                    let child = root.children.remove(0);
                    *root = child;
                }
                return true;
            }
        }
        false
    }

    fn remove_from_node(node: &mut BTreeNode, key: &[u8]) -> bool {
        match node.keys.binary_search_by(|k| k.as_slice().cmp(key)) {
            Ok(i) => {
                if node.is_leaf {
                    node.keys.remove(i);
                    node.values.remove(i);
                    true
                } else {
                    // Replace with predecessor
                    let (pred_key, pred_val) = Self::find_max(&node.children[i]);
                    node.keys[i] = pred_key;
                    node.values[i] = pred_val;
                    Self::remove_from_node(&mut node.children[i], &node.keys[i].clone())
                }
            }
            Err(i) => {
                if node.is_leaf {
                    false
                } else {
                    Self::remove_from_node(&mut node.children[i], key)
                }
            }
        }
    }

    fn find_max(node: &BTreeNode) -> (Vec<u8>, Vec<u8>) {
        if node.is_leaf {
            (
                node.keys.last().unwrap().clone(),
                node.values.last().unwrap().clone(),
            )
        } else {
            Self::find_max(node.children.last().unwrap())
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Collect all key-value pairs in order.
    pub fn to_vec(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut result = Vec::new();
        if let Some(root) = &self.root {
            Self::collect_in_order(root, &mut result);
        }
        result
    }

    fn collect_in_order(node: &BTreeNode, result: &mut Vec<(Vec<u8>, Vec<u8>)>) {
        if node.is_leaf {
            for (k, v) in node.keys.iter().zip(node.values.iter()) {
                result.push((k.clone(), v.clone()));
            }
        } else {
            for i in 0..node.keys.len() {
                if i < node.children.len() {
                    Self::collect_in_order(&node.children[i], result);
                }
                result.push((node.keys[i].clone(), node.values[i].clone()));
            }
            if let Some(last) = node.children.last() {
                Self::collect_in_order(last, result);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 3. LSM-Tree Memtable (Write Buffer)
// ---------------------------------------------------------------------------

/// The memtable is the write buffer in an LSM-tree. Writes go here first,
/// then get flushed to disk as sorted SSTables.
#[derive(Debug)]
pub struct Memtable {
    data: BTreeMap<Vec<u8>, Vec<u8>>,
    size_bytes: usize,
    max_size_bytes: usize,
    sequence: u64,
    tombstones: Vec<Vec<u8>>,
}

impl Memtable {
    pub fn new(max_size_bytes: usize) -> Self {
        Self {
            data: BTreeMap::new(),
            size_bytes: 0,
            max_size_bytes,
            sequence: 0,
            tombstones: Vec::new(),
        }
    }

    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> bool {
        let entry_size = key.len() + value.len();
        self.size_bytes += entry_size;
        self.data.insert(key, value);
        self.sequence += 1;
        self.is_full()
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.data.get(key).map(|v| v.as_slice())
    }

    pub fn delete(&mut self, key: Vec<u8>) -> bool {
        self.tombstones.push(key.clone());
        self.data.remove(&key);
        self.sequence += 1;
        self.is_full()
    }

    pub fn is_full(&self) -> bool {
        self.size_bytes >= self.max_size_bytes
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    /// Flush the memtable content (for creating an SSTable).
    pub fn flush(&mut self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let data: Vec<_> = self.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        self.data.clear();
        self.size_bytes = 0;
        self.tombstones.clear();
        data
    }
}

// ---------------------------------------------------------------------------
// 4. Write-Ahead Log Entry
// ---------------------------------------------------------------------------

/// A WAL entry ensures durability by logging before applying changes.
#[derive(Debug, Clone)]
pub struct WalEntry {
    pub sequence: u64,
    pub operation: WalOperation,
    pub checksum: u32,
}

#[derive(Debug, Clone)]
pub enum WalOperation {
    Put { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
}

impl WalEntry {
    pub fn put(sequence: u64, key: Vec<u8>, value: Vec<u8>) -> Self {
        let op = WalOperation::Put {
            key: key.clone(),
            value: value.clone(),
        };
        Self {
            sequence,
            operation: op,
            checksum: Self::compute_checksum(&key, Some(&value)),
        }
    }

    pub fn delete(sequence: u64, key: Vec<u8>) -> Self {
        let op = WalOperation::Delete { key: key.clone() };
        Self {
            sequence,
            operation: op,
            checksum: Self::compute_checksum(&key, None),
        }
    }

    fn compute_checksum(key: &[u8], value: Option<&[u8]>) -> u32 {
        let mut hash: u32 = 5381;
        for &byte in key {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
        }
        if let Some(val) = value {
            for &byte in val {
                hash = hash.wrapping_mul(33).wrapping_add(byte as u32);
            }
        }
        hash
    }

    pub fn verify_checksum(&self) -> bool {
        match &self.operation {
            WalOperation::Put { key, value } => {
                self.checksum == Self::compute_checksum(key, Some(value))
            }
            WalOperation::Delete { key } => {
                self.checksum == Self::compute_checksum(key, None)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Bloom Filter
// ---------------------------------------------------------------------------

/// A Bloom filter for probabilistic set membership testing.
/// Reduces disk reads by quickly determining if a key is NOT in a file.
#[derive(Debug)]
pub struct BloomFilter {
    bits: Vec<bool>,
    num_hashes: usize,
    size: usize,
    item_count: usize,
}

impl BloomFilter {
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        let size = Self::optimal_size(expected_items, false_positive_rate);
        let num_hashes = Self::optimal_hashes(size, expected_items);
        Self {
            bits: vec![false; size],
            num_hashes,
            size,
            item_count: 0,
        }
    }

    fn optimal_size(n: usize, p: f64) -> usize {
        let ln2 = std::f64::consts::LN_2;
        (-(n as f64) * p.ln() / (ln2 * ln2)).ceil() as usize
    }

    fn optimal_hashes(m: usize, n: usize) -> usize {
        let ln2 = std::f64::consts::LN_2;
        ((m as f64 / n as f64) * ln2).ceil() as usize
    }

    pub fn insert(&mut self, item: &[u8]) {
        for i in 0..self.num_hashes {
            let index = self.hash(item, i) % self.size;
            self.bits[index] = true;
        }
        self.item_count += 1;
    }

    pub fn might_contain(&self, item: &[u8]) -> bool {
        (0..self.num_hashes).all(|i| {
            let index = self.hash(item, i) % self.size;
            self.bits[index]
        })
    }

    fn hash(&self, item: &[u8], seed: usize) -> usize {
        let mut hash: usize = seed.wrapping_mul(0x5bd1e995);
        for &byte in item {
            hash = hash.wrapping_mul(31).wrapping_add(byte as usize);
        }
        hash
    }

    pub fn len(&self) -> usize {
        self.item_count
    }

    pub fn is_empty(&self) -> bool {
        self.item_count == 0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btree_insert_and_get() {
        let mut tree: BTree<4> = BTree::new();
        tree.insert(b"key1".to_vec(), b"val1".to_vec());
        tree.insert(b"key2".to_vec(), b"val2".to_vec());
        tree.insert(b"key3".to_vec(), b"val3".to_vec());

        assert_eq!(tree.get(b"key1"), Some(b"val1".as_slice()));
        assert_eq!(tree.get(b"key2"), Some(b"val2".as_slice()));
        assert_eq!(tree.get(b"missing"), None);
    }

    #[test]
    fn test_btree_update() {
        let mut tree: BTree<4> = BTree::new();
        tree.insert(b"key".to_vec(), b"old".to_vec());
        tree.insert(b"key".to_vec(), b"new".to_vec());

        assert_eq!(tree.get(b"key"), Some(b"new".as_slice()));
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_btree_remove() {
        let mut tree: BTree<4> = BTree::new();
        tree.insert(b"key".to_vec(), b"val".to_vec());
        assert!(tree.remove(b"key"));
        assert!(tree.get(b"key").is_none());
        assert!(!tree.remove(b"missing"));
    }

    #[test]
    fn test_btree_ordering() {
        let mut tree: BTree<4> = BTree::new();
        tree.insert(b"c".to_vec(), b"3".to_vec());
        tree.insert(b"a".to_vec(), b"1".to_vec());
        tree.insert(b"b".to_vec(), b"2".to_vec());

        let items = tree.to_vec();
        assert_eq!(items[0].0, b"a");
        assert_eq!(items[1].0, b"b");
        assert_eq!(items[2].0, b"c");
    }

    #[test]
    fn test_btree_empty() {
        let tree: BTree = BTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert!(tree.get(b"key").is_none());
    }

    #[test]
    fn test_memtable_basic() {
        let mut mt = Memtable::new(1024);
        mt.put(b"key1".to_vec(), b"val1".to_vec());
        assert_eq!(mt.get(b"key1"), Some(b"val1".as_slice()));
        assert_eq!(mt.len(), 1);
    }

    #[test]
    fn test_memtable_full() {
        let mut mt = Memtable::new(10); // very small
        mt.put(b"longkey".to_vec(), b"longvalue".to_vec());
        assert!(mt.is_full());
    }

    #[test]
    fn test_memtable_flush() {
        let mut mt = Memtable::new(1024);
        mt.put(b"a".to_vec(), b"1".to_vec());
        mt.put(b"b".to_vec(), b"2".to_vec());

        let data = mt.flush();
        assert_eq!(data.len(), 2);
        assert!(mt.is_empty());
    }

    #[test]
    fn test_memtable_delete() {
        let mut mt = Memtable::new(1024);
        mt.put(b"key".to_vec(), b"val".to_vec());
        mt.delete(b"key".to_vec());
        assert!(mt.get(b"key").is_none());
    }

    #[test]
    fn test_wal_entry_put() {
        let entry = WalEntry::put(1, b"key".to_vec(), b"value".to_vec());
        assert!(entry.verify_checksum());
        assert_eq!(entry.sequence, 1);
    }

    #[test]
    fn test_wal_entry_delete() {
        let entry = WalEntry::delete(2, b"key".to_vec());
        assert!(entry.verify_checksum());
    }

    #[test]
    fn test_wal_entry_checksum_corruption() {
        let mut entry = WalEntry::put(1, b"key".to_vec(), b"value".to_vec());
        entry.checksum = 0; // corrupt
        assert!(!entry.verify_checksum());
    }

    #[test]
    fn test_bloom_filter_basic() {
        let mut filter = BloomFilter::new(100, 0.01);
        filter.insert(b"hello");
        filter.insert(b"world");

        assert!(filter.might_contain(b"hello"));
        assert!(filter.might_contain(b"world"));
        // Might have false positives but very unlikely for non-inserted items
    }

    #[test]
    fn test_bloom_filter_empty() {
        let filter = BloomFilter::new(100, 0.01);
        assert!(filter.is_empty());
        assert_eq!(filter.len(), 0);
    }

    #[test]
    fn test_bloom_filter_no_false_negatives() {
        let mut filter = BloomFilter::new(1000, 0.001);
        let items: Vec<String> = (0..100).map(|i| format!("item_{i}")).collect();

        for item in &items {
            filter.insert(item.as_bytes());
        }

        // All inserted items must be found (no false negatives)
        for item in &items {
            assert!(
                filter.might_contain(item.as_bytes()),
                "false negative for {item}"
            );
        }
    }

    #[test]
    fn test_storage_engine_trait() {
        // Test that our trait is object-safe
        fn _use_engine(_engine: &dyn StorageEngine<Error = std::io::Error>) {}
    }

    #[test]
    fn test_bloom_filter_multiple_inserts() {
        let mut filter = BloomFilter::new(100, 0.05);
        for i in 0..50 {
            filter.insert(format!("key_{i}").as_bytes());
        }
        assert_eq!(filter.len(), 50);
    }

    #[test]
    fn test_btree_large_insert() {
        let mut tree: BTree<4> = BTree::new();
        for i in 0..100 {
            let key = format!("key_{i:04}");
            let val = format!("val_{i}");
            tree.insert(key.into_bytes(), val.into_bytes());
        }
        assert_eq!(tree.len(), 100);

        let key = "key_0050".as_bytes();
        assert!(tree.get(key).is_some());
    }
}
