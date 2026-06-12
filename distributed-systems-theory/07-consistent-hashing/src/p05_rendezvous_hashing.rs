//! # Exercise: Rendezvous Hashing
//!
//! ## Theory
//!
//! Rendezvous Hashing (also called Highest Random Weight or HRW hashing) assigns
//! keys to nodes by computing `hash(key, node)` for every node and selecting the
//! node with the highest hash value.
//!
//! This naturally provides consistent hashing properties: adding or removing a
//! node only affects keys assigned to that node (or keys that now find a higher
//! hash with the new node).
//!
//! ## Proof / Intuition
//!
//! For a key k and nodes {n1, n2, ..., nk}, the node with the highest
//! hash(k, ni) "wins" that key. When node n_k+1 is added, a key moves only
//! if hash(k, n_k+1) > hash(k, winner). Since each hash is independent and
//! uniform, this happens with probability 1/(k+1) -- exactly 1/N of keys.
//!
//! ## Implementation Task
//!
//! Implement `RendezvousHasher` with:
//! - `get_node(key) -> NodeId` - find highest-weight node
//! - `add_node(node)` - add a node to the set
//! - `remove_node(node)` - remove a node from the set
//!
//! ## Verification
//!
//! Verify that removing a node only remaps keys that were assigned to it,
//! and that assignment is consistent.
//!
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A node identifier.
pub type NodeId = usize;

/// Rendezvous (HRW) hasher for consistent key-to-node mapping.
#[derive(Debug, Clone)]
pub struct RendezvousHasher {
    nodes: Vec<NodeId>,
}

impl RendezvousHasher {
    /// Create a new hasher with the given nodes.
    pub fn new(nodes: Vec<NodeId>) -> Self {
        Self { nodes }
    }

    /// Compute the hash weight for a (key, node) pair.
    fn weight(key: &str, node: NodeId) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        node.hash(&mut hasher);
        hasher.finish()
    }

    /// Find the node with the highest weight for the given key.
    pub fn get_node(&self, key: &str) -> Option<NodeId> {
        self.nodes
            .iter()
            .map(|&node| (node, Self::weight(key, node)))
            .max_by_key(|(_, w)| *w)
            .map(|(node, _)| node)
    }

    /// Add a node. Returns true if the node was newly added.
    pub fn add_node(&mut self, node: NodeId) -> bool {
        if self.nodes.contains(&node) {
            false
        } else {
            self.nodes.push(node);
            true
        }
    }

    /// Remove a node. Returns true if the node was present.
    pub fn remove_node(&mut self, node: NodeId) -> bool {
        let len_before = self.nodes.len();
        self.nodes.retain(|&n| n != node);
        self.nodes.len() < len_before
    }

    /// Get all current nodes.
    pub fn nodes(&self) -> &[NodeId] {
        &self.nodes
    }

    /// Get the number of nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Compute assignment for a set of keys.
    pub fn get_assignments(&self, keys: &[String]) -> Vec<Option<NodeId>> {
        keys.iter().map(|k| self.get_node(k)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_keys(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("key_{}", i)).collect()
    }

    #[test]
    fn test_only_removed_node_keys_remapped() {
        let nodes: Vec<NodeId> = (0..10).collect();
        let hasher = RendezvousHasher::new(nodes);
        let keys = generate_keys(10_000);

        let assignments_before: Vec<NodeId> = keys
            .iter()
            .map(|k| hasher.get_node(k).unwrap())
            .collect();

        let mut hasher2 = hasher.clone();
        let node_to_remove = 5;
        hasher2.remove_node(node_to_remove);

        let assignments_after: Vec<NodeId> = keys
            .iter()
            .map(|k| hasher2.get_node(k).unwrap())
            .collect();

        let remapped: Vec<(usize, NodeId, NodeId)> = assignments_before
            .iter()
            .zip(assignments_after.iter())
            .enumerate()
            .filter(|(_, (a, b))| a != b)
            .map(|(i, (&a, &b))| (i, a, b))
            .collect();

        // All remapped keys should have previously been assigned to the removed node
        for (idx, old_node, _new_node) in &remapped {
            assert_eq!(
                *old_node, node_to_remove,
                "Key {} was remapped from node {} but should only remap keys from removed node {}",
                idx, old_node, node_to_remove
            );
        }

        // Only keys on the removed node should remap
        let keys_on_removed = assignments_before.iter().filter(|&&n| n == node_to_remove).count();
        assert_eq!(
            remapped.len(),
            keys_on_removed,
            "All keys on removed node should remap"
        );
    }

    #[test]
    fn test_consistent_assignment() {
        let nodes: Vec<NodeId> = (0..5).collect();
        let hasher = RendezvousHasher::new(nodes);
        let keys = generate_keys(500);

        let a1: Vec<NodeId> = keys.iter().map(|k| hasher.get_node(k).unwrap()).collect();
        let a2: Vec<NodeId> = keys.iter().map(|k| hasher.get_node(k).unwrap()).collect();

        assert_eq!(a1, a2, "Same hasher must always produce same assignments");
    }

    #[test]
    fn test_add_node_only_affects_some_keys() {
        let nodes: Vec<NodeId> = (0..10).collect();
        let hasher = RendezvousHasher::new(nodes);
        let keys = generate_keys(10_000);

        let assignments_before: Vec<NodeId> = keys
            .iter()
            .map(|k| hasher.get_node(k).unwrap())
            .collect();

        let mut hasher2 = hasher.clone();
        hasher2.add_node(10);

        let assignments_after: Vec<NodeId> = keys
            .iter()
            .map(|k| hasher2.get_node(k).unwrap())
            .collect();

        let remapped = assignments_before
            .iter()
            .zip(assignments_after.iter())
            .filter(|(a, b)| a != b)
            .count();

        let ratio = remapped as f64 / keys.len() as f64;
        assert!(
            ratio < 0.20,
            "Adding a node should only remap ~1/N keys, but remapped {:.1}%",
            ratio * 100.0
        );
    }

    #[test]
    fn test_empty_hasher_returns_none() {
        let hasher = RendezvousHasher::new(vec![]);
        assert!(hasher.get_node("key").is_none());
    }

    #[test]
    fn test_all_keys_assigned() {
        let nodes: Vec<NodeId> = (0..5).collect();
        let hasher = RendezvousHasher::new(nodes);
        let keys = generate_keys(1000);

        for key in &keys {
            assert!(
                hasher.get_node(key).is_some(),
                "Every key must be assigned to a node"
            );
        }
    }
}
