//! # Exercise: CP Mode (Consistent, Partition-Tolerant)
//!
//! ## Theory
//!
//! A CP system prioritizes consistency over availability. During a network
//! partition, a CP system rejects writes that cannot be replicated to a
//! majority (quorum) of nodes. This ensures that the data remains consistent
//! across all available replicas, even though some clients may receive errors.
//!
//! For a 3-node cluster, the quorum size is 2. A write succeeds only if at
//! least 2 nodes confirm it. If a partition prevents reaching a quorum, the
//! write is rejected with an error.
//!
//! ## Proof / Intuition
//!
//! Quorum requires `ceil(N/2)` nodes for both reads and writes. For N=3,
//! quorum = 2. Because any two quorums must overlap by at least one node
//! (2 + 2 - 3 = 1), a read quorum will always see at least one node that
//! acknowledged the most recent write quorum. This overlap is what guarantees
//! linearizability.
//!
//! When a partition splits nodes into {A} and {B, C}, neither side has a
//! majority of the full cluster. Both sides must reject writes, preserving
//! consistency at the cost of availability -- the CP trade-off.
//!
//! ## Implementation Task
//!
//! Extend the replicated KV store with CP semantics:
//! - `put_with_quorum(&mut self, key, value) -> Result<(), QuorumError>` that
//!   requires confirmation from a majority of nodes
//! - `QuorumError` enum distinguishing partition-induced failures from other errors
//! - Track which nodes are reachable (simulating partitions)
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Writes fail when a partition prevents quorum formation
//! - Reads return the latest value after partition heals
//! - Quorum requirement is correctly enforced

use std::collections::{HashMap, HashSet};

/// Error type for operations that cannot achieve quorum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuorumError {
    /// Not enough reachable nodes to form a quorum.
    InsufficientNodes { available: usize, required: usize },
    /// The node is partitioned away from the cluster.
    Partitioned,
}

/// A single replica node with reachability tracking.
#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub store: HashMap<String, String>,
}

impl Node {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            store: HashMap::new(),
        }
    }

    pub fn put(&mut self, key: &str, value: &str) {
        self.store.insert(key.to_string(), value.to_string());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.store.get(key).map(|s| s.as_str())
    }
}

/// A CP (Consistent, Partition-Tolerant) replicated key-value store.
///
/// Writes require a quorum of reachable nodes. If a partition prevents
/// reaching a quorum, writes are rejected.
#[derive(Debug)]
pub struct CpKVStore {
    nodes: Vec<Node>,
    /// Set of node IDs that are currently partitioned away.
    partitioned: HashSet<usize>,
    /// Quorum size: floor(N/2) + 1.
    quorum: usize,
}

impl CpKVStore {
    /// Create a new CP KV store with the given number of nodes.
    pub fn new(num_nodes: usize) -> Self {
        let nodes = (0..num_nodes).map(Node::new).collect();
        let quorum = num_nodes / 2 + 1;
        Self {
            nodes,
            partitioned: HashSet::new(),
            quorum,
        }
    }

    /// Create a 3-node CP store (the canonical example).
    pub fn three_node() -> Self {
        Self::new(3)
    }

    /// Partition a node, making it unreachable from the coordinator.
    pub fn partition_node(&mut self, node_id: usize) {
        self.partitioned.insert(node_id);
    }

    /// Heal the partition on a specific node.
    pub fn heal_node(&mut self, node_id: usize) {
        self.partitioned.remove(&node_id);
    }

    /// Heal all partitions.
    pub fn heal_all(&mut self) {
        self.partitioned.clear();
    }

    /// Check whether a node is reachable.
    pub fn is_reachable(&self, node_id: usize) -> bool {
        !self.partitioned.contains(&node_id)
    }

    /// Count the number of reachable nodes.
    fn reachable_count(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| !self.partitioned.contains(&n.id))
            .count()
    }

    /// Write a key-value pair using quorum replication.
    ///
    /// Returns `Err(QuorumError)` if a quorum of nodes cannot be reached.
    pub fn put_with_quorum(
        &mut self,
        key: &str,
        value: &str,
    ) -> Result<(), QuorumError> {
        let available = self.reachable_count();
        if available < self.quorum {
            return Err(QuorumError::InsufficientNodes {
                available,
                required: self.quorum,
            });
        }

        // Write to all reachable nodes
        for node in &mut self.nodes {
            if !self.partitioned.contains(&node.id) {
                node.put(key, value);
            }
        }
        Ok(())
    }

    /// Read a value from the quorum (read from first reachable node).
    pub fn get(&self, key: &str) -> Option<String> {
        self.nodes
            .iter()
            .find(|n| !self.partitioned.contains(&n.id))
            .and_then(|n| n.get(key).map(|s| s.to_string()))
    }

    /// Read from a specific node (even if partitioned).
    pub fn get_from_node(&self, node_id: usize, key: &str) -> Option<String> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .and_then(|n| n.get(key).map(|s| s.to_string()))
    }

    /// Return the quorum size.
    pub fn quorum_size(&self) -> usize {
        self.quorum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_succeeds_with_full_cluster() {
        let mut store = CpKVStore::three_node();
        assert!(store.put_with_quorum("key", "value").is_ok());
        assert_eq!(store.get("key"), Some("value".to_string()));
    }

    #[test]
    fn write_fails_when_quorum_unreachable() {
        let mut store = CpKVStore::three_node();
        // Partition 2 of 3 nodes -- only 1 reachable, quorum is 2
        store.partition_node(1);
        store.partition_node(2);

        let result = store.put_with_quorum("key", "value");
        assert!(result.is_err());
        match result.unwrap_err() {
            QuorumError::InsufficientNodes { available, required } => {
                assert_eq!(available, 1);
                assert_eq!(required, 2);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn write_succeeds_with_majority_available() {
        let mut store = CpKVStore::three_node();
        // Partition only 1 node -- 2 reachable, quorum is 2
        store.partition_node(2);

        assert!(store.put_with_quorum("key", "value").is_ok());
    }

    #[test]
    fn read_returns_latest_after_partition_heals() {
        let mut store = CpKVStore::three_node();

        // Write before partition
        store.put_with_quorum("x", "before").unwrap();

        // Partition one node
        store.partition_node(2);
        // Write with 2/3 nodes
        store.put_with_quorum("x", "during").unwrap();

        // Heal partition
        store.heal_all();
        assert_eq!(store.get("x"), Some("during".to_string()));
    }

    #[test]
    fn partitioned_node_has_stale_data() {
        let mut store = CpKVStore::three_node();
        store.put_with_quorum("y", "v1").unwrap();

        // Partition node 2, then write to remaining nodes
        store.partition_node(2);
        store.put_with_quorum("y", "v2").unwrap();

        // Partitioned node still has old value
        assert_eq!(
            store.get_from_node(2, "y"),
            Some("v1".to_string())
        );

        // Reachable nodes have new value
        assert_eq!(
            store.get_from_node(0, "y"),
            Some("v2".to_string())
        );
    }

    #[test]
    fn quorum_size_is_correct_for_three_nodes() {
        let store = CpKVStore::three_node();
        assert_eq!(store.quorum_size(), 2);
    }
}
