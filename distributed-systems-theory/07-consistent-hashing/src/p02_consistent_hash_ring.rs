//! # Exercise: Consistent Hash Ring
//!
//! ## Theory
//!
//! Consistent hashing maps both keys and nodes to points on a ring (0 to 2^64).
//! A key is assigned to the first node found by walking clockwise from the key's
//! position. Each physical node maps to multiple virtual nodes on the ring for
//! better load distribution.
//!
//! The key insight is that adding or removing a node only affects keys between
//! that node and its neighbor on the ring -- roughly 1/N of all keys.
//!
//! ## Proof / Intuition
//!
//! Consider a ring with N physical nodes, each with V virtual nodes. The ring
//! is divided into approximately N*V segments. When adding a new node with V
//! virtual nodes, approximately V segments are split, affecting roughly
//! V/(N*V) = 1/N of all keys. This holds regardless of the number of nodes.
//!
//! ## Implementation Task
//!
//! Implement `ConsistentHashRing` with:
//! - `add_node(node_id, num_virtual)` - add node with virtual nodes
//! - `remove_node(node_id)` - remove node and all its virtual nodes
//! - `get_node(key) -> NodeId` - find the node responsible for a key
//!
//! ## Verification
//!
//! Verify that adding a node only remaps ~1/N keys, and keys are
//! evenly distributed across nodes.
//!
use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A node identifier.
pub type NodeId = usize;

/// Consistent hash ring with virtual nodes.
#[derive(Debug)]
pub struct ConsistentHashRing {
    /// Ring: hash position -> node ID. Sorted by hash position.
    ring: BTreeMap<u64, NodeId>,
    /// Track which virtual node positions belong to which physical node.
    node_positions: HashMap<NodeId, Vec<u64>>,
}

impl ConsistentHashRing {
    /// Create a new empty hash ring.
    pub fn new() -> Self {
        Self {
            ring: BTreeMap::new(),
            node_positions: HashMap::new(),
        }
    }

    /// Hash a key to a position on the ring.
    fn hash(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Create a virtual node key from a node ID and virtual node index.
    fn virtual_node_key(node_id: NodeId, index: usize) -> String {
        format!("{}#{}", node_id, index)
    }

    /// Add a node with the specified number of virtual nodes.
    pub fn add_node(&mut self, node_id: NodeId, num_virtual: usize) {
        let mut positions = Vec::new();
        for i in 0..num_virtual {
            let vkey = Self::virtual_node_key(node_id, i);
            let pos = Self::hash(&vkey);
            self.ring.insert(pos, node_id);
            positions.push(pos);
        }
        self.node_positions.insert(node_id, positions);
    }

    /// Remove a node and all its virtual nodes from the ring.
    pub fn remove_node(&mut self, node_id: NodeId) {
        if let Some(positions) = self.node_positions.remove(&node_id) {
            for pos in positions {
                self.ring.remove(&pos);
            }
        }
    }

    /// Find the node responsible for the given key.
    /// Walks clockwise from the key's hash position.
    pub fn get_node(&self, key: &str) -> Option<NodeId> {
        if self.ring.is_empty() {
            return None;
        }
        let pos = Self::hash(key);
        // Find the first node at or after this position (clockwise)
        self.ring
            .range(pos..)
            .next()
            .or_else(|| self.ring.iter().next()) // wrap around
            .map(|(_, &node_id)| node_id)
    }

    /// Get the number of keys assigned to each node.
    pub fn distribution(&self, keys: &[String]) -> HashMap<NodeId, usize> {
        let mut dist = HashMap::new();
        for key in keys {
            if let Some(node) = self.get_node(key) {
                *dist.entry(node).or_insert(0) += 1;
            }
        }
        dist
    }

    /// Get all node IDs currently on the ring.
    pub fn nodes(&self) -> Vec<NodeId> {
        self.node_positions.keys().copied().collect()
    }

    /// Number of physical nodes on the ring.
    pub fn num_nodes(&self) -> usize {
        self.node_positions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_keys(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("key_{}", i)).collect()
    }

    #[test]
    fn test_keys_assign_to_nodes() {
        let mut ring = ConsistentHashRing::new();
        ring.add_node(0, 100);
        ring.add_node(1, 100);
        ring.add_node(2, 100);

        let keys = generate_keys(1000);
        let dist = ring.distribution(&keys);

        assert_eq!(dist.len(), 3, "All 3 nodes should have keys");
        for (node, count) in &dist {
            assert!(
                *count > 0,
                "Node {} should have at least one key",
                node
            );
        }
    }

    #[test]
    fn test_add_node_remaps_few_keys() {
        let mut ring = ConsistentHashRing::new();
        for i in 0..10 {
            ring.add_node(i, 150);
        }

        let keys = generate_keys(10000);
        let assignments_before: Vec<NodeId> = keys.iter().map(|k| ring.get_node(k).unwrap()).collect();

        ring.add_node(10, 150);

        let assignments_after: Vec<NodeId> = keys.iter().map(|k| ring.get_node(k).unwrap()).collect();

        let remapped = assignments_before
            .iter()
            .zip(assignments_after.iter())
            .filter(|(a, b)| a != b)
            .count();

        let remap_ratio = remapped as f64 / keys.len() as f64;
        // Should remap roughly 1/N keys (with 10 nodes before, ~10%)
        assert!(
            remap_ratio < 0.20,
            "Adding a node should only remap ~1/N keys, but remapped {:.1}% ({} keys)",
            remap_ratio * 100.0,
            remapped
        );
    }

    #[test]
    fn test_remove_node_remaps_few_keys() {
        let mut ring = ConsistentHashRing::new();
        for i in 0..10 {
            ring.add_node(i, 150);
        }

        let keys = generate_keys(10000);
        let assignments_before: Vec<NodeId> = keys.iter().map(|k| ring.get_node(k).unwrap()).collect();

        ring.remove_node(5);

        let assignments_after: Vec<NodeId> = keys.iter().map(|k| ring.get_node(k).unwrap()).collect();

        let remapped = assignments_before
            .iter()
            .zip(assignments_after.iter())
            .filter(|(a, b)| a != b)
            .count();

        let remap_ratio = remapped as f64 / keys.len() as f64;
        assert!(
            remap_ratio < 0.20,
            "Removing a node should only remap ~1/N keys, but remapped {:.1}%",
            remap_ratio * 100.0
        );
    }

    #[test]
    fn test_consistent_assignment_without_modifications() {
        let mut ring = ConsistentHashRing::new();
        ring.add_node(0, 100);
        ring.add_node(1, 100);

        let keys = generate_keys(500);
        let a1: Vec<NodeId> = keys.iter().map(|k| ring.get_node(k).unwrap()).collect();
        let a2: Vec<NodeId> = keys.iter().map(|k| ring.get_node(k).unwrap()).collect();

        assert_eq!(a1, a2, "Same ring must always assign keys identically");
    }

    #[test]
    fn test_empty_ring_returns_none() {
        let ring = ConsistentHashRing::new();
        assert!(ring.get_node("any_key").is_none());
    }
}
