//! # Exercise: Network Partition Simulator
//!
//! ## Theory
//!
//! A network partition occurs when nodes in a distributed system are divided
//! into two or more groups that cannot communicate with each other. Partitions
//! are a fundamental challenge in distributed systems because they force the
//! system to choose between consistency and availability (CAP theorem).
//!
//! A partition simulator models this by tracking which nodes can communicate.
//! When a partition is created between two groups, messages between nodes in
//! different groups are dropped. When the partition is healed, communication
//! is restored.
//!
//! ## Proof / Intuition
//!
//! Real network partitions can be asymmetric (A can reach B but B cannot
//! reach A), can involve more than two groups, and can be partial (some
//! messages get through while others don't). Our simulator models the
//! simplest case: symmetric, binary partitions between groups.
//!
//! Even this simple model captures the essential difficulty: any message
//! sent across a partition boundary is lost, and the sender may not know
//! whether the message was delivered.
//!
//! ## Implementation Task
//!
//! Implement a `PartitionSimulator` that:
//! - Tracks a set of node IDs
//! - `create_partition(group_a, group_b)` -- blocks communication between groups
//! - `heal_partition()` -- restores all communication
//! - `can_communicate(a, b) -> bool` -- check if two nodes can communicate
//! - `send(from, to, msg) -> Result<T, PartitionError>` -- send with partition check
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Partitions block communication between groups
//! - Healing restores communication
//! - Self-communication always works
//! - Partition is symmetric

use std::collections::{HashSet, VecDeque};

/// Error when a message is blocked by a partition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionError;

/// A partition represented as two disjoint sets of node IDs.
#[derive(Debug, Clone)]
struct Partition {
    group_a: HashSet<usize>,
    group_b: HashSet<usize>,
}

impl Partition {
    /// Check if two nodes are on opposite sides of this partition.
    fn blocks(&self, a: usize, b: usize) -> bool {
        (self.group_a.contains(&a) && self.group_b.contains(&b))
            || (self.group_b.contains(&a) && self.group_a.contains(&b))
    }
}

/// Network partition simulator.
///
/// Tracks which partitions exist and determines whether two nodes can
/// communicate. Supports multiple simultaneous partitions.
#[derive(Debug)]
pub struct PartitionSimulator {
    nodes: HashSet<usize>,
    partitions: Vec<Partition>,
    /// Buffered messages waiting to be delivered after heal.
    pending_messages: VecDeque<(usize, usize, Vec<u8>)>,
}

impl PartitionSimulator {
    /// Create a new simulator with the given set of node IDs.
    pub fn new(node_ids: &[usize]) -> Self {
        Self {
            nodes: node_ids.iter().copied().collect(),
            partitions: Vec::new(),
            pending_messages: VecDeque::new(),
        }
    }

    /// Create a 3-node simulator (the standard cluster size).
    pub fn three_node() -> Self {
        Self::new(&[0, 1, 2])
    }

    /// Create a network partition between two groups of nodes.
    ///
    /// Messages between nodes in different groups will be blocked.
    pub fn create_partition(&mut self, group_a: Vec<usize>, group_b: Vec<usize>) {
        let set_a: HashSet<usize> = group_a.into_iter().collect();
        let set_b: HashSet<usize> = group_b.into_iter().collect();

        // Validate that groups are disjoint and all nodes exist
        assert!(
            set_a.is_disjoint(&set_b),
            "groups must be disjoint"
        );
        for &id in set_a.iter().chain(set_b.iter()) {
            assert!(self.nodes.contains(&id), "node {id} not in simulator");
        }

        self.partitions.push(Partition {
            group_a: set_a,
            group_b: set_b,
        });
    }

    /// Heal all partitions, restoring full communication.
    pub fn heal_partition(&mut self) {
        self.partitions.clear();
        self.pending_messages.clear();
    }

    /// Check whether two nodes can communicate.
    ///
    /// Returns `false` if any active partition blocks communication between them.
    /// A node can always communicate with itself.
    pub fn can_communicate(&self, a: usize, b: usize) -> bool {
        if a == b {
            return true;
        }
        !self.partitions.iter().any(|p| p.blocks(a, b))
    }

    /// Try to send a message between two nodes.
    ///
    /// Returns `Ok(msg)` if the message would be delivered, or
    /// `Err(PartitionError)` if blocked by a partition.
    pub fn send<T>(
        &self,
        from: usize,
        to: usize,
        msg: T,
    ) -> Result<T, PartitionError> {
        if self.can_communicate(from, to) {
            Ok(msg)
        } else {
            Err(PartitionError)
        }
    }

    /// Check if a specific node is partitioned from any other node.
    pub fn is_isolated(&self, node_id: usize) -> bool {
        self.nodes
            .iter()
            .filter(|&&other| other != node_id)
            .all(|&other| !self.can_communicate(node_id, other))
    }

    /// Return the number of active partitions.
    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    /// Return all node IDs.
    pub fn node_ids(&self) -> Vec<usize> {
        self.nodes.iter().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_blocks_communication() {
        let mut sim = PartitionSimulator::three_node();
        sim.create_partition(vec![0], vec![1, 2]);

        assert!(!sim.can_communicate(0, 1));
        assert!(!sim.can_communicate(0, 2));
        // But nodes within the same group can still talk
        assert!(sim.can_communicate(1, 2));
    }

    #[test]
    fn heal_restores_communication() {
        let mut sim = PartitionSimulator::three_node();
        sim.create_partition(vec![0], vec![1, 2]);
        assert!(!sim.can_communicate(0, 1));

        sim.heal_partition();
        assert!(sim.can_communicate(0, 1));
        assert!(sim.can_communicate(0, 2));
    }

    #[test]
    fn self_communication_always_works() {
        let mut sim = PartitionSimulator::three_node();
        sim.create_partition(vec![0], vec![1, 2]);

        assert!(sim.can_communicate(0, 0));
        assert!(sim.can_communicate(1, 1));
        assert!(sim.can_communicate(2, 2));
    }

    #[test]
    fn partition_is_symmetric() {
        let mut sim = PartitionSimulator::three_node();
        sim.create_partition(vec![0], vec![1]);

        assert_eq!(
            sim.can_communicate(0, 1),
            sim.can_communicate(1, 0),
            "partition should be symmetric"
        );
    }

    #[test]
    fn send_returns_ok_when_no_partition() {
        let sim = PartitionSimulator::three_node();
        let msg = "hello".to_string();
        let result = sim.send(0, 1, msg.clone());
        assert_eq!(result, Ok(msg));
    }

    #[test]
    fn send_returns_err_when_partitioned() {
        let mut sim = PartitionSimulator::three_node();
        sim.create_partition(vec![0], vec![1]);
        let result = sim.send(0, 1, "hello".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn multiple_partitions_coexist() {
        let mut sim = PartitionSimulator::new(&[0, 1, 2, 3]);
        sim.create_partition(vec![0], vec![1]);
        sim.create_partition(vec![2], vec![3]);

        assert!(!sim.can_communicate(0, 1));
        assert!(!sim.can_communicate(2, 3));
        assert!(sim.can_communicate(0, 2));
        assert!(sim.can_communicate(1, 3));
        assert_eq!(sim.partition_count(), 2);
    }

    #[test]
    fn isolated_node_detection() {
        let mut sim = PartitionSimulator::three_node();
        assert!(!sim.is_isolated(0));

        sim.create_partition(vec![0], vec![1, 2]);
        assert!(sim.is_isolated(0));
        assert!(!sim.is_isolated(1)); // 1 can still talk to 2
    }
}
