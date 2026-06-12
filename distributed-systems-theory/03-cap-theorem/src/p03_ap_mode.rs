//! # Exercise: AP Mode (Available, Partition-Tolerant)
//!
//! ## Theory
//!
//! An AP system prioritizes availability over consistency. During a network
//! partition, every node continues to accept writes, even if it cannot
//! communicate with all other replicas. This means different partitions may
//! accept conflicting writes, leading to divergent data.
//!
//! When the partition heals, the system must resolve conflicts. The simplest
//! strategy is Last-Write-Wins (LWW), where the write with the most recent
//! timestamp is chosen. While LWW can lose data (concurrent writes to the
//! same key), it guarantees that all replicas converge to the same value.
//!
//! ## Proof / Intuition
//!
//! AP systems guarantee eventual consistency: if no new writes occur, all
//! replicas will eventually converge to the same state. This is proven by
//! showing that conflict resolution (like LWW) is a deterministic function
//! that all replicas can execute independently on the same set of writes.
//!
//! The trade-off is that during a partition, reads may return stale data.
//! Different replicas may have different values for the same key. This is
//! the fundamental tension between consistency and availability.
//!
//! ## Implementation Task
//!
//! Implement an AP replicated KV store:
//! - Accept writes on any node, even during partition, assigning a
//! - timestamp to each write
//! - Resolve conflicts on partition heal using last-write-wins
//! - Track vector clocks or timestamps for ordering
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Writes succeed even during a partition
//! - Stale reads are possible during a partition
//! - Replicas eventually converge after partition heals

use std::collections::HashMap;

/// A timestamped value for conflict resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimestampedValue {
    pub value: String,
    /// Logical timestamp for ordering (higher = more recent).
    pub timestamp: u64,
}

impl TimestampedValue {
    pub fn new(value: &str, timestamp: u64) -> Self {
        Self {
            value: value.to_string(),
            timestamp,
        }
    }
}

/// A single AP replica node.
#[derive(Debug, Clone)]
pub struct ApNode {
    pub id: usize,
    pub store: HashMap<String, TimestampedValue>,
    /// Local logical clock.
    pub clock: u64,
}

impl ApNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            store: HashMap::new(),
            clock: 0,
        }
    }

    /// Write a key-value pair with an incremented timestamp.
    pub fn put(&mut self, key: &str, value: &str) {
        self.clock += 1;
        self.store.insert(
            key.to_string(),
            TimestampedValue::new(value, self.clock),
        );
    }

    /// Read a value by key.
    pub fn get(&self, key: &str) -> Option<&TimestampedValue> {
        self.store.get(key)
    }

    /// Merge a remote value using last-write-wins.
    pub fn merge(&mut self, key: &str, remote: &TimestampedValue) {
        match self.store.get(key) {
            Some(local) if local.timestamp > remote.timestamp => {
                // Local is newer or equal; keep it.
                // Update clock to be at least as high.
                self.clock = self.clock.max(remote.timestamp + 1);
            }
            _ => {
                // Remote is newer or key doesn't exist locally.
                self.clock = self.clock.max(remote.timestamp + 1);
                self.store.insert(key.to_string(), remote.clone());
            }
        }
    }

    /// Return a snapshot of all key-value pairs.
    pub fn snapshot(&self) -> HashMap<String, TimestampedValue> {
        self.store.clone()
    }
}

/// An AP (Available, Partition-Tolerant) replicated key-value store.
///
/// Writes are always accepted. Conflict resolution uses last-write-wins.
#[derive(Debug)]
pub struct ApKVStore {
    nodes: Vec<ApNode>,
    /// Which nodes are currently partitioned (set of pairs that cannot talk).
    partitioned: Vec<(usize, usize)>,
}

impl ApKVStore {
    pub fn new(num_nodes: usize) -> Self {
        let nodes = (0..num_nodes).map(ApNode::new).collect();
        Self {
            nodes,
            partitioned: Vec::new(),
        }
    }

    pub fn three_node() -> Self {
        Self::new(3)
    }

    /// Create a partition between two nodes.
    pub fn create_partition(&mut self, a: usize, b: usize) {
        self.partitioned.push((a, b));
    }

    /// Heal all partitions.
    pub fn heal_all(&mut self) {
        self.partitioned.clear();
    }

    /// Check if two nodes can communicate.
    pub fn can_communicate(&self, a: usize, b: usize) -> bool {
        !self.partitioned.iter().any(|&(x, y)| {
            (x == a && y == b) || (x == b && y == a)
        })
    }

    /// Write to a specific node (always succeeds -- AP semantics).
    pub fn put_on_node(&mut self, node_id: usize, key: &str, value: &str) {
        let node = self
            .nodes
            .iter_mut()
            .find(|n| n.id == node_id)
            .expect("node not found");
        node.put(key, value);
    }

    /// Read from a specific node.
    pub fn get_from_node(&self, node_id: usize, key: &str) -> Option<String> {
        self.nodes
            .iter()
            .find(|n| n.id == node_id)
            .and_then(|n| n.get(key))
            .map(|tv| tv.value.clone())
    }

    /// Heal partitions and sync all replicas using last-write-wins merge.
    pub fn heal_and_sync(&mut self) {
        self.partitioned.clear();
        // Sync all nodes by merging every pair
        let snapshots: Vec<(usize, HashMap<String, TimestampedValue>)> = self
            .nodes
            .iter()
            .map(|n| (n.id, n.snapshot()))
            .collect();

        for node in &mut self.nodes {
            for (_other_id, snapshot) in &snapshots {
                for (key, value) in snapshot {
                    node.merge(key, value);
                }
            }
        }
    }

    /// Check if all nodes are consistent (same values for all keys).
    pub fn is_converged(&self) -> bool {
        if self.nodes.is_empty() {
            return true;
        }
        let first = &self.nodes[0].store;
        self.nodes[1..]
            .iter()
            .all(|n| n.store == *first)
    }

    pub fn nodes(&self) -> &[ApNode] {
        &self.nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_always_succeeds_even_without_partition() {
        let mut store = ApKVStore::three_node();
        store.put_on_node(0, "x", "1");
        assert_eq!(store.get_from_node(0, "x"), Some("1".to_string()));
    }

    #[test]
    fn writes_succeed_during_partition() {
        let mut store = ApKVStore::three_node();
        store.create_partition(0, 1);
        store.create_partition(0, 2);

        // Node 0 is isolated but can still accept writes
        store.put_on_node(0, "key", "from_isolated");
        assert_eq!(
            store.get_from_node(0, "key"),
            Some("from_isolated".to_string())
        );

        // Other nodes can also write
        store.put_on_node(1, "key", "from_majority");
        assert_eq!(
            store.get_from_node(1, "key"),
            Some("from_majority".to_string())
        );
    }

    #[test]
    fn stale_reads_possible_during_partition() {
        let mut store = ApKVStore::three_node();
        store.put_on_node(0, "x", "v1");
        store.heal_and_sync();

        // Now partition node 2 away and update nodes 0 and 1
        store.create_partition(2, 0);
        store.create_partition(2, 1);
        store.put_on_node(0, "x", "v2");
        store.put_on_node(1, "x", "v2");

        // Node 2 still sees the old value
        assert_eq!(store.get_from_node(2, "x"), Some("v1".to_string()));
        // Nodes 0 and 1 see the new value
        assert_eq!(store.get_from_node(0, "x"), Some("v2".to_string()));
    }

    #[test]
    fn eventual_convergence_after_heal() {
        let mut store = ApKVStore::three_node();

        // Initial sync
        store.put_on_node(0, "a", "1");
        store.heal_and_sync();

        // Partition and write conflicting values
        store.create_partition(0, 1);
        store.create_partition(0, 2);
        store.put_on_node(0, "a", "from_0");
        store.put_on_node(1, "a", "from_1");

        // Not converged yet (node 0 has different value)
        assert!(!store.is_converged());

        // Heal and sync
        store.heal_and_sync();
        assert!(store.is_converged());
    }

    #[test]
    fn last_write_wins_conflict_resolution() {
        let mut store = ApKVStore::three_node();

        // Both write concurrently (simulated by writing to different nodes)
        store.put_on_node(0, "conflict", "value_a");
        store.put_on_node(1, "conflict", "value_b");

        // The node with the higher timestamp wins (node 1 because it wrote second)
        store.heal_and_sync();
        let final_value = store.get_from_node(0, "conflict").unwrap();
        assert_eq!(final_value, "value_b");
    }

    #[test]
    fn multiple_partitions_and_heals() {
        let mut store = ApKVStore::three_node();

        // First partition round
        store.create_partition(0, 2);
        store.put_on_node(0, "k", "round1");
        store.heal_and_sync();

        // Second partition round
        store.create_partition(1, 2);
        store.put_on_node(1, "k", "round2");
        store.heal_and_sync();

        assert!(store.is_converged());
        assert_eq!(store.get_from_node(0, "k"), Some("round2".to_string()));
    }
}
