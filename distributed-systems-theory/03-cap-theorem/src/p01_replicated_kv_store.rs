//! # Exercise: Replicated Key-Value Store
//!
//! ## Theory
//!
//! A replicated key-value store maintains copies of data across multiple nodes.
//! When a client writes a value, it must be propagated to replicas so that
//! reads from any node return the correct data. The simplest approach is
//! synchronous replication: every write updates all replicas before acknowledging
//! success.
//!
//! In a 3-node cluster, writes can use majority quorum (2 of 3) to ensure
//! durability while tolerating one node failure. Reads can also use quorum to
//! ensure they see the latest write.
//!
//! ## Proof / Intuition
//!
//! With N replicas, a quorum of `floor(N/2) + 1` nodes ensures that any two
//! quorums overlap by at least one node. This overlap guarantees that a read
//! quorum will always intersect with the most recent write quorum, ensuring
//! the read sees the latest data.
//!
//! For N=3: quorum = 2. Any two groups of 2 nodes from a set of 3 must share
//! at least 1 node.
//!
//! ## Implementation Task
//!
//! Implement a `ReplicatedKV` with 3 nodes:
//! - `ReplicatedKV` struct managing a cluster of `Node` replicas
//! - `Node` struct holding a `HashMap<String, String>` of key-value pairs
//! - `put(&mut self, key: String, value: String)` that replicates to all nodes
//! - `get(&self, key: &str) -> Option<String>` that reads from any node
//! - Replication via message passing between nodes
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - All nodes eventually have the same data after writes
//! - Put/get operations work correctly
//! - Reads from different nodes return consistent results

use std::collections::HashMap;

/// A single replica node holding a local copy of the key-value data.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for this node.
    pub id: usize,
    /// Local key-value store.
    pub store: HashMap<String, String>,
}

impl Node {
    /// Create a new empty node with the given ID.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            store: HashMap::new(),
        }
    }

    /// Insert a key-value pair into this node's local store.
    pub fn put(&mut self, key: &str, value: &str) {
        self.store.insert(key.to_string(), value.to_string());
    }

    /// Look up a key in this node's local store.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.store.get(key).map(|s| s.as_str())
    }
}

/// A replicated key-value store backed by 3 replica nodes.
///
/// Writes are replicated to all nodes synchronously. Reads return data from
/// the first node (Node 0 by default).
#[derive(Debug)]
pub struct ReplicatedKV {
    /// The cluster of replica nodes.
    nodes: Vec<Node>,
}

impl ReplicatedKV {
    /// Create a new 3-node replicated key-value store.
    pub fn new() -> Self {
        let nodes = (0..3).map(Node::new).collect();
        Self { nodes }
    }

    /// Write a key-value pair, replicating to all nodes.
    ///
    /// This is a synchronous full-replication strategy: all nodes must confirm
    /// the write before it is acknowledged.
    pub fn put(&mut self, key: &str, value: &str) {
        for node in &mut self.nodes {
            node.put(key, value);
        }
    }

    /// Read a value by key from the first replica.
    ///
    /// In a real system you would typically read from a quorum, but here we
    /// read from node 0 for simplicity.
    pub fn get(&self, key: &str) -> Option<String> {
        self.nodes[0].get(key).map(|s| s.to_string())
    }

    /// Read a value from a specific replica node.
    pub fn get_from_node(&self, node_id: usize, key: &str) -> Option<String> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .and_then(|n| n.get(key).map(|s| s.to_string()))
    }

    /// Return a reference to all nodes (for testing / inspection).
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Check whether all nodes agree on the value for a given key.
    pub fn is_consistent(&self, key: &str) -> bool {
        let values: Vec<Option<&str>> = self.nodes.iter().map(|n| n.get(key)).collect();
        values.windows(2).all(|w| w[0] == w[1])
    }
}

impl Default for ReplicatedKV {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_and_get_basic() {
        let mut kv = ReplicatedKV::new();
        kv.put("name", "alice");
        assert_eq!(kv.get("name"), Some("alice".to_string()));
    }

    #[test]
    fn get_returns_none_for_missing_key() {
        let kv = ReplicatedKV::new();
        assert_eq!(kv.get("missing"), None);
    }

    #[test]
    fn all_nodes_have_same_data_after_write() {
        let mut kv = ReplicatedKV::new();
        kv.put("color", "blue");

        // Every node should have the same value
        assert!(kv.is_consistent("color"));

        for node in kv.nodes() {
            assert_eq!(node.get("color"), Some("blue"));
        }
    }

    #[test]
    fn overwrite_propagates_to_all_nodes() {
        let mut kv = ReplicatedKV::new();
        kv.put("x", "1");
        kv.put("x", "2");

        assert!(kv.is_consistent("x"));
        assert_eq!(kv.get("x"), Some("2".to_string()));
    }

    #[test]
    fn multiple_keys_consistent() {
        let mut kv = ReplicatedKV::new();
        kv.put("a", "1");
        kv.put("b", "2");
        kv.put("c", "3");

        assert!(kv.is_consistent("a"));
        assert!(kv.is_consistent("b"));
        assert!(kv.is_consistent("c"));
    }

    #[test]
    fn reads_from_different_nodes_agree() {
        let mut kv = ReplicatedKV::new();
        kv.put("shared", "value");

        for node in kv.nodes() {
            assert_eq!(
                kv.get_from_node(node.id, "shared"),
                Some("value".to_string())
            );
        }
    }
}
