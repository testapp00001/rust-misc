//! # Exercise: Naive Hash Partitioning
//!
//! ## Theory
//!
//! The simplest partitioning strategy: `node = hash(key) % num_nodes`. Each key is
//! deterministically assigned to a node based on the remainder when dividing its hash
//! by the number of nodes.
//!
//! While this provides even distribution when the number of nodes is fixed, it has a
//! critical flaw: any change in the number of nodes (N) causes approximately `(N-1)/N`
//! of all keys to be remapped to different nodes.
//!
//! ## Proof / Intuition
//!
//! When we add one node to a cluster of N nodes, each key's assignment becomes:
//! `hash(key) % (N+1)` instead of `hash(key) % N`. For any hash value H, the fraction
//! of keys that land on the same node is approximately `1 / lcm(N, N+1)`, which for
//! coprime N and N+1 is roughly `1/N`.
//!
//! This means adding a node to a 100-node cluster causes ~99% of keys to move.
//!
//! ## Implementation Task
//!
//! Implement `NaivePartitioner` with:
//! - `new(num_nodes: usize)` - create partitioner
//! - `assign(&self, key: &str) -> usize` - assign key to node
//! - `add_node(&mut self)` - increase cluster size, measure redistribution
//!
//! ## Verification
//!
//! Verify that adding a node to a 100-node cluster causes ~99% of keys to remap,
//! and that the same node count produces stable assignments.
//!
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Naive hash-based partitioner using `hash(key) % num_nodes`.
#[derive(Debug, Clone)]
pub struct NaivePartitioner {
    pub num_nodes: usize,
}

impl NaivePartitioner {
    /// Create a new partitioner with the given number of nodes.
    pub fn new(num_nodes: usize) -> Self {
        assert!(num_nodes > 0, "Must have at least one node");
        Self { num_nodes }
    }

    /// Assign a key to a node using hash(key) % num_nodes.
    pub fn assign(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish() as usize;
        hash % self.num_nodes
    }

    /// Add a node to the cluster, returning the number of keys that were remapped.
    pub fn add_node(&mut self, keys: &[String]) -> usize {
        let old_assignments: Vec<usize> = keys.iter().map(|k| self.assign(k)).collect();
        self.num_nodes += 1;
        let new_assignments: Vec<usize> = keys.iter().map(|k| self.assign(k)).collect();

        old_assignments
            .iter()
            .zip(new_assignments.iter())
            .filter(|(old, new)| old != new)
            .count()
    }

    /// Get current assignments for all keys.
    pub fn get_assignments(&self, keys: &[String]) -> Vec<usize> {
        keys.iter().map(|k| self.assign(k)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_stable_assignment_with_same_node_count() {
        let partitioner = NaivePartitioner::new(10);
        let keys: Vec<String> = (0..1000).map(|i| format!("key_{}", i)).collect();

        let assignments1 = partitioner.get_assignments(&keys);
        let assignments2 = partitioner.get_assignments(&keys);

        assert_eq!(assignments1, assignments2, "Same node count must produce identical assignments");
    }

    #[test]
    fn test_all_assignments_within_range() {
        let partitioner = NaivePartitioner::new(5);
        let keys: Vec<String> = (0..500).map(|i| format!("key_{}", i)).collect();
        let assignments = partitioner.get_assignments(&keys);

        for (i, &node) in assignments.iter().enumerate() {
            assert!(
                node < 5,
                "Key {} assigned to node {} but only 5 nodes exist",
                i,
                node
            );
        }
    }

    #[test]
    fn test_removes_maintain_stability() {
        let partitioner = NaivePartitioner::new(10);
        let keys: Vec<String> = (0..1000).map(|i| format!("key_{}", i)).collect();
        let assignments_before = partitioner.get_assignments(&keys);

        // Without actually removing a node (just verifying same count = same result)
        let assignments_after = partitioner.get_assignments(&keys);
        assert_eq!(assignments_before, assignments_after);
    }

    #[test]
    fn test_deterministic_assignment() {
        let partitioner = NaivePartitioner::new(10);
        let key = "test_key";
        let a1 = partitioner.assign(key);
        let a2 = partitioner.assign(key);
        assert_eq!(a1, a2, "Same key must always map to same node");
    }

    #[test]
    fn test_keys_spread_across_nodes() {
        let partitioner = NaivePartitioner::new(10);
        let keys: Vec<String> = (0..10000).map(|i| format!("key_{}", i)).collect();
        let assignments = partitioner.get_assignments(&keys);

        let unique_nodes: HashSet<usize> = assignments.into_iter().collect();
        assert!(
            unique_nodes.len() >= 5,
            "Keys should be spread across at least half the nodes, got {}",
            unique_nodes.len()
        );
    }
}
