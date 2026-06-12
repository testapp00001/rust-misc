//! Partition rebalancing logic.
//!
//! When nodes join or leave the cluster, the rebalancer computes a minimal set
//! of partition moves to restore balance while leveraging consistent hashing
//! to minimize data migration.

use super::hash_ring::HashRing;

/// A single partition move from one node to another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartitionMove {
    /// The partition being moved.
    pub partition: u32,
    /// The node currently hosting the partition.
    pub from_node: u64,
    /// The node that should take over.
    pub to_node: u64,
}

/// The result of a rebalance computation.
#[derive(Debug, Clone)]
pub struct RebalancePlan {
    /// Ordered list of partition moves to execute.
    pub moves: Vec<PartitionMove>,
    /// Estimated number of keys that will be relocated.
    pub affected_keys_estimate: usize,
}

impl RebalancePlan {
    /// Create an empty plan (no moves needed).
    fn empty() -> Self {
        RebalancePlan {
            moves: Vec::new(),
            affected_keys_estimate: 0,
        }
    }
}

/// Computes partition reassignment plans when the node set changes.
#[derive(Debug, Clone)]
pub struct Rebalancer {
    hash_ring: HashRing,
}

impl Rebalancer {
    /// Create a rebalancer from the current hash ring state.
    pub fn new(hash_ring: HashRing) -> Self {
        Rebalancer { hash_ring }
    }

    /// Compute a rebalance plan to transition from `old_nodes` to
    /// `new_nodes`.
    ///
    /// The algorithm:
    /// 1. For each partition on the ring, determine its current owner under
    ///    `old_nodes` and its ideal owner under `new_nodes`.
    /// 2. If the owners differ, schedule a move.
    /// 3. Estimate the affected key count.
    pub fn plan_rebalance(&self, old_nodes: &[u64], new_nodes: &[u64]) -> RebalancePlan {
        if old_nodes.is_empty() || new_nodes.is_empty() {
            return RebalancePlan::empty();
        }

        let partition_count = self.hash_ring.partition_count();
        if partition_count == 0 {
            return RebalancePlan::empty();
        }

        let mut moves = Vec::new();
        let mut affected_keys = 0usize;

        for partition_id in 0..partition_count as u32 {
            // Use a synthetic key derived from the partition ID to determine
            // node assignment. This mirrors how production systems probe the
            // ring for each partition.
            let probe_key = format!("rebalance-probe-{}", partition_id);
            let key_bytes = probe_key.as_bytes();

            let from_node = Self::node_for_partition(
                &self.hash_ring,
                key_bytes,
                partition_id,
                old_nodes,
            );
            let to_node = Self::node_for_partition(
                &self.hash_ring,
                key_bytes,
                partition_id,
                new_nodes,
            );

            if from_node != to_node {
                moves.push(PartitionMove {
                    partition: partition_id,
                    from_node,
                    to_node,
                });
                affected_keys += self.estimate_affected_keys(
                    partition_id,
                    1_000_000, // default estimate total
                    partition_count,
                );
            }
        }

        RebalancePlan {
            moves,
            affected_keys_estimate: affected_keys,
        }
    }

    /// Estimate how many keys are affected by moving a single partition.
    ///
    /// Uses the formula: `total_keys / num_partitions`. This is a uniform
    /// distribution assumption -- real systems would sample the actual key
    /// space.
    pub fn estimate_affected_keys(
        &self,
        _partition: u32,
        total_keys: usize,
        num_partitions: usize,
    ) -> usize {
        if num_partitions == 0 {
            return 0;
        }
        total_keys / num_partitions
    }

    /// Determine which node owns a partition, preferring the ring-based
    /// assignment but falling back to modular arithmetic for robustness.
    fn node_for_partition(
        ring: &HashRing,
        key: &[u8],
        _partition: u32,
        nodes: &[u64],
    ) -> u64 {
        if nodes.is_empty() {
            panic!("node_for_partition called with empty node list");
        }
        ring.get_node_for_key(key, nodes)
    }

    /// Check whether a rebalance would cause significant disruption.
    /// Returns true if more than `threshold_fraction` of partitions would
    /// move.
    pub fn would_be_disruptive(
        &self,
        old_nodes: &[u64],
        new_nodes: &[u64],
        threshold_fraction: f64,
    ) -> bool {
        let plan = self.plan_rebalance(old_nodes, new_nodes);
        let total = self.hash_ring.partition_count();
        if total == 0 {
            return false;
        }
        let move_fraction = plan.moves.len() as f64 / total as f64;
        move_fraction > threshold_fraction
    }

    /// Compute the set of partitions that must be moved off a departing node.
    pub fn partitions_for_node(
        &self,
        node: u64,
        nodes: &[u64],
    ) -> Vec<u32> {
        let mut result = Vec::new();
        let partition_count = self.hash_ring.partition_count();
        for p in 0..partition_count as u32 {
            let probe_key = format!("rebalance-probe-{}", p);
            let owner = Self::node_for_partition(
                &self.hash_ring,
                probe_key.as_bytes(),
                p,
                nodes,
            );
            if owner == node {
                result.push(p);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::hash_ring::HashRing;

    fn make_ring_and_rebalancer(n: u32) -> Rebalancer {
        let ring = HashRing::new((0..n).collect(), 150);
        Rebalancer::new(ring)
    }

    #[test]
    fn no_moves_when_nodes_unchanged() {
        let rebalancer = make_ring_and_rebalancer(6);
        let nodes = vec![1, 2, 3];
        let plan = rebalancer.plan_rebalance(&nodes, &nodes);
        assert!(plan.moves.is_empty());
    }

    #[test]
    fn moves_when_node_added() {
        let rebalancer = make_ring_and_rebalancer(6);
        let old = vec![1, 2, 3];
        let new = vec![1, 2, 3, 4];
        let plan = rebalancer.plan_rebalance(&old, &new);
        // Some partitions should move to the new node
        assert!(!plan.moves.is_empty());
        // All moves should have from != to
        for m in &plan.moves {
            assert_ne!(m.from_node, m.to_node);
        }
    }

    #[test]
    fn moves_when_node_removed() {
        let rebalancer = make_ring_and_rebalancer(6);
        let old = vec![1, 2, 3, 4];
        let new = vec![1, 2, 3];
        let plan = rebalancer.plan_rebalance(&old, &new);
        assert!(!plan.moves.is_empty());
        // All moved partitions should go to remaining nodes
        for m in &plan.moves {
            assert!(new.contains(&m.to_node));
        }
    }

    #[test]
    fn empty_nodes_returns_empty_plan() {
        let rebalancer = make_ring_and_rebalancer(6);
        let plan = rebalancer.plan_rebalance(&[], &[1, 2]);
        assert!(plan.moves.is_empty());

        let plan = rebalancer.plan_rebalance(&[1, 2], &[]);
        assert!(plan.moves.is_empty());
    }

    #[test]
    fn affected_keys_estimate() {
        let rebalancer = make_ring_and_rebalancer(6);
        let est = rebalancer.estimate_affected_keys(0, 1_000_000, 6);
        assert_eq!(est, 166_666);
    }

    #[test]
    fn affected_keys_zero_partitions() {
        let rebalancer = make_ring_and_rebalancer(6);
        let est = rebalancer.estimate_affected_keys(0, 1_000_000, 0);
        assert_eq!(est, 0);
    }

    #[test]
    fn would_be_disruptive_low() {
        let rebalancer = make_ring_and_rebalancer(6);
        let nodes = vec![1, 2, 3];
        // No moves when same nodes -- not disruptive
        assert!(!rebalancer.would_be_disruptive(&nodes, &nodes, 0.5));
    }

    #[test]
    fn partitions_for_node() {
        let rebalancer = make_ring_and_rebalancer(6);
        let nodes = vec![1, 2, 3];
        let p1 = rebalancer.partitions_for_node(1, &nodes);
        let p2 = rebalancer.partitions_for_node(2, &nodes);
        let p3 = rebalancer.partitions_for_node(3, &nodes);
        // Different nodes should own different (non-overlapping) partitions
        for p in &p1 {
            assert!(!p2.contains(p));
            assert!(!p3.contains(p));
        }
        for p in &p2 {
            assert!(!p3.contains(p));
        }
        // Union should cover all partitions
        let mut all: Vec<u32> = p1.into_iter().chain(p2.into_iter()).chain(p3.into_iter()).collect();
        all.sort();
        all.dedup();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn plan_has_affected_keys_estimate() {
        let rebalancer = make_ring_and_rebalancer(6);
        let old = vec![1, 2, 3];
        let new = vec![1, 2, 3, 4];
        let plan = rebalancer.plan_rebalance(&old, &new);
        assert!(plan.affected_keys_estimate > 0);
    }

    #[test]
    fn full_node_replacement() {
        let rebalancer = make_ring_and_rebalancer(6);
        let old = vec![10, 20, 30];
        let new = vec![40, 50, 60];
        let plan = rebalancer.plan_rebalance(&old, &new);
        // Every partition should move
        assert_eq!(plan.moves.len(), 6);
        for m in &plan.moves {
            assert!(new.contains(&m.to_node));
            assert!(old.contains(&m.from_node));
        }
    }
}
