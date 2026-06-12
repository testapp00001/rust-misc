//! # Exercise: Partition Tolerance Tests
//!
//! ## Theory
//!
//! Network partitions split a distributed system into isolated groups of nodes
//! that cannot communicate with each other. The CAP theorem tells us that
//! during a partition, we must choose between:
//!
//! - **Consistency (CP)**: Reject operations that cannot be safely performed
//!   because a quorum is unavailable.
//! - **Availability (AP)**: Accept operations locally and reconcile later.
//!
//! CP systems (e.g., etcd, ZooKeeper) prioritize correctness and will reject
//! writes or return errors when a majority cannot be reached.
//!
//! AP systems (e.g., Cassandra, DynamoDB) prioritize availability and will
//! accept writes even during a partition, reconciling via anti-entropy or
//! conflict resolution when the partition heals.
//!
//! ## Proof / Intuition
//!
//! Consider a 3-node cluster {N0, N1, N2} with quorum size 2.
//!
//! Under a partition that isolates N0 from {N1, N2}:
//! - **CP behavior**: N0 rejects writes (cannot reach quorum). N1 and N2
//!   continue accepting writes (they form a majority).
//! - **AP behavior**: N0 accepts writes locally. N1 and N2 also accept writes.
//!   After the partition heals, we must reconcile.
//!
//! ## Implementation Task
//!
//! Implement a partition simulation with:
//! - `Node` with local data and alive/dead status.
//! - `Partition` representing the network topology and partition groups.
//! - `PartitionTest` with CP and AP write/read strategies.
//!
//! ## Verification
//!
//! Run `cargo test partition_tolerance` to verify your implementation.

use std::collections::{HashMap, HashSet};

/// A single node in the distributed system.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier.
    pub id: usize,
    /// Local key-value data.
    pub data: HashMap<String, String>,
    /// Whether this node is currently alive (not crashed).
    pub is_alive: bool,
}

impl Node {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            data: HashMap::new(),
            is_alive: true,
        }
    }
}

/// Represents a network partition state.
#[derive(Debug, Clone)]
pub struct Partition {
    /// Adjacency list: for each node, which other nodes it can reach.
    pub network: HashMap<usize, Vec<usize>>,
    /// Groups of nodes that are isolated from each other.
    /// Each HashSet is a connected component.
    pub partitioned_groups: Vec<HashSet<usize>>,
}

impl Partition {
    /// Create a fully connected network with no partitions.
    pub fn fully_connected(node_ids: &[usize]) -> Self {
        let mut network = HashMap::new();
        for &id in node_ids {
            let others: Vec<usize> = node_ids.iter().copied().filter(|&other| other != id).collect();
            network.insert(id, others);
        }
        let all: HashSet<usize> = node_ids.iter().copied().collect();
        Self {
            network,
            partitioned_groups: vec![all],
        }
    }

    /// Create a partition that splits nodes into two groups.
    pub fn split(node_ids: &[usize], group1: &[usize], group2: &[usize]) -> Self {
        let mut network = HashMap::new();

        // Nodes in group1 can only reach other nodes in group1.
        for &id in group1 {
            let reachable: Vec<usize> = group1
                .iter()
                .copied()
                .filter(|&other| other != id)
                .collect();
            network.insert(id, reachable);
        }

        // Nodes in group2 can only reach other nodes in group2.
        for &id in group2 {
            let reachable: Vec<usize> = group2
                .iter()
                .copied()
                .filter(|&other| other != id)
                .collect();
            network.insert(id, reachable);
        }

        // Nodes not in either group (shouldn't happen but handle it).
        for &id in node_ids {
            if !group1.contains(&id) && !group2.contains(&id) {
                network.insert(id, vec![]);
            }
        }

        let g1: HashSet<usize> = group1.iter().copied().collect();
        let g2: HashSet<usize> = group2.iter().copied().collect();

        Self {
            network,
            partitioned_groups: vec![g1, g2],
        }
    }

    /// Check if two nodes can communicate (are in the same partition group).
    pub fn can_communicate(&self, a: usize, b: usize) -> bool {
        self.network
            .get(&a)
            .map_or(false, |reachable| reachable.contains(&b))
    }

    /// Find which partition group a node belongs to.
    pub fn find_group(&self, node_id: usize) -> Option<&HashSet<usize>> {
        self.partitioned_groups
            .iter()
            .find(|group| group.contains(&node_id))
    }
}

/// A test harness for partition tolerance.
#[derive(Debug)]
pub struct PartitionTest {
    /// All nodes in the system.
    pub nodes: HashMap<usize, Node>,
    /// Current network partition state.
    pub partition: Partition,
}

impl PartitionTest {
    /// Create a new partition test with the given node IDs.
    pub fn new(node_ids: &[usize]) -> Self {
        let mut nodes = HashMap::new();
        for &id in node_ids {
            nodes.insert(id, Node::new(id));
        }
        let partition = Partition::fully_connected(node_ids);
        Self { nodes, partition }
    }

    /// Create a partition splitting the given groups.
    pub fn create_partition(&mut self, group1: &[usize], group2: &[usize]) {
        let all_ids: Vec<usize> = self.nodes.keys().copied().collect();
        self.partition = Partition::split(&all_ids, group1, group2);
    }

    /// Find which nodes are reachable from a given node.
    fn reachable_from(&self, node_id: usize) -> Vec<usize> {
        self.partition
            .network
            .get(&node_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Count how many alive nodes in the reachable set.
    fn alive_quorum_size(&self, node_id: usize) -> (usize, usize) {
        let reachable = self.reachable_from(node_id);
        let total_alive = reachable
            .iter()
            .filter(|&&id| {
                self.nodes
                    .get(&id)
                    .map_or(false, |n| n.is_alive)
            })
            .count();
        // Include self in the count.
        let self_alive = self
            .nodes
            .get(&node_id)
            .map_or(false, |n| n.is_alive);
        let alive_reachable = total_alive + if self_alive { 1 } else { 0 };
        let total_nodes = self.nodes.len();
        let majority = total_nodes / 2 + 1;
        (alive_reachable, majority)
    }

    /// CP handle write: reject if the node cannot reach a quorum.
    /// Returns Ok(()) on success, Err(message) on failure.
    pub fn cp_handle_write(
        &mut self,
        node_id: usize,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        let (alive_count, majority) = self.alive_quorum_size(node_id);

        if alive_count < majority {
            return Err(format!(
                "Node {} cannot reach quorum ({} alive, {} required)",
                node_id, alive_count, majority
            ));
        }

        // Write locally and to all reachable alive nodes.
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.data.insert(key.to_string(), value.to_string());
        }

        let reachable = self.reachable_from(node_id);
        for &target_id in &reachable {
            if target_id != node_id {
                if let Some(node) = self.nodes.get_mut(&target_id) {
                    if node.is_alive {
                        node.data
                            .insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        Ok(())
    }

    /// AP handle write: accept the write locally, no quorum check.
    pub fn ap_handle_write(&mut self, node_id: usize, key: &str, value: &str) {
        if let Some(node) = self.nodes.get_mut(&node_id) {
            node.data.insert(key.to_string(), value.to_string());
        }
    }

    /// CP read: return error if cannot reach majority, otherwise return the
    /// value from the local node.
    pub fn cp_read(&self, node_id: usize, key: &str) -> Result<Option<String>, String> {
        let (alive_count, majority) = self.alive_quorum_size(node_id);

        if alive_count < majority {
            return Err(format!(
                "Node {} cannot reach quorum for read ({} alive, {} required)",
                node_id, alive_count, majority
            ));
        }

        Ok(self
            .nodes
            .get(&node_id)
            .and_then(|n| n.data.get(key).cloned()))
    }

    /// AP read: return local data without quorum check.
    pub fn ap_read(&self, node_id: usize, key: &str) -> Option<String> {
        self.nodes
            .get(&node_id)
            .and_then(|n| n.data.get(key).cloned())
    }

    /// Reconcile data across all nodes by propagating all writes.
    /// This simulates what happens after a partition heals.
    pub fn reconcile(&mut self) {
        // Collect all data from all nodes.
        let mut merged: HashMap<String, String> = HashMap::new();
        for node in self.nodes.values() {
            for (k, v) in &node.data {
                merged.insert(k.clone(), v.clone());
            }
        }

        // Write merged data to all alive nodes.
        for node in self.nodes.values_mut() {
            if node.is_alive {
                for (k, v) in &merged {
                    node.data.insert(k.clone(), v.clone());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_rejects_writes_during_partition() {
        let mut test = PartitionTest::new(&[0, 1, 2]);
        // Partition: {0} | {1, 2}
        test.create_partition(&[0], &[1, 2]);

        // Node 0 is isolated -- it cannot reach a quorum.
        let result = test.cp_handle_write(0, "x", "hello");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("quorum"));

        // Nodes 1 and 2 form a majority -- writes should succeed.
        let result = test.cp_handle_write(1, "x", "world");
        assert!(result.is_ok());
    }

    #[test]
    fn test_ap_accepts_writes_during_partition() {
        let mut test = PartitionTest::new(&[0, 1, 2]);
        // Partition: {0} | {1, 2}
        test.create_partition(&[0], &[1, 2]);

        // Node 0 accepts writes locally even though it's isolated.
        test.ap_handle_write(0, "x", "hello");
        assert_eq!(test.ap_read(0, "x").unwrap(), "hello");

        // Node 1 also accepts writes.
        test.ap_handle_write(1, "x", "world");
        assert_eq!(test.ap_read(1, "x").unwrap(), "world");

        // Nodes 0 and 1 have different values for x.
        assert_ne!(
            test.ap_read(0, "x"),
            test.ap_read(1, "x")
        );
    }

    #[test]
    fn test_data_reconciles_after_partition_heals() {
        let mut test = PartitionTest::new(&[0, 1, 2]);
        // Partition: {0} | {1, 2}
        test.create_partition(&[0], &[1, 2]);

        // Write different values to different sides.
        test.ap_handle_write(0, "x", "from_node_0");
        test.ap_handle_write(1, "y", "from_node_1");

        // Heal partition (re-create fully connected).
        let all_ids: Vec<usize> = test.nodes.keys().copied().collect();
        test.partition = Partition::fully_connected(&all_ids);

        // Reconcile.
        test.reconcile();

        // All nodes should have both x and y.
        for node in test.nodes.values() {
            assert_eq!(node.data.get("x").unwrap(), "from_node_0");
            assert_eq!(node.data.get("y").unwrap(), "from_node_1");
        }
    }

    #[test]
    fn test_cp_read_rejects_when_isolated() {
        let mut test = PartitionTest::new(&[0, 1, 2]);
        test.create_partition(&[0], &[1, 2]);

        // Write via CP on the majority side.
        let result = test.cp_handle_write(1, "x", "val");
        assert!(result.is_ok());

        // Node 0 is isolated -- CP read should fail.
        let result = test.cp_read(0, "x");
        assert!(result.is_err());
    }

    #[test]
    fn test_partition_communication_check() {
        let test = PartitionTest::new(&[0, 1, 2]);
        // Fully connected: all can communicate.
        assert!(test.partition.can_communicate(0, 1));
        assert!(test.partition.can_communicate(1, 2));
        assert!(test.partition.can_communicate(0, 2));
    }
}
