//! # Exercise: Rebalancing Cost
//!
//! ## Theory
//!
//! When a node is added or removed from a cluster, keys must be redistributed.
//! The rebalancing cost is the fraction of keys that need to move to a different
//! node. Lower cost means less data migration and faster recovery.
//!
//! Naive hashing: ~`(N-1)/N` of keys move (catastrophic).
//! Consistent hashing: ~`1/N` of keys move.
//! Jump consistent hash: ~`1/(N+1)` of keys move.
//! Rendezvous hashing: ~`1/N` of keys move.
//!
//! ## Proof / Intuition
//!
//! In naive hashing, adding a node changes the modulus from N to N+1. For any
//! key K, the new position is `K % (N+1)` which is independent of `K % N`.
//! This means ~N/(N+1) * (N-1)/N keys move -- approximately (N-1)/N.
//!
//! In consistent hashing, a key moves only if it falls between the new node
//! and its predecessor on the ring. This region covers ~1/N of the ring.
//!
//! ## Implementation Task
//!
//! Implement `RebalancingCost` struct and `measure_rebalance` function that
//! compares rebalancing cost across different strategies.
//!
//! ## Verification
//!
//! Verify consistent hashing moves ~1/N keys and naive moves ~(N-1)/N keys.
//!
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Cost of rebalancing after a node change.
#[derive(Debug, Clone)]
pub struct RebalancingCost {
    pub keys_moved: usize,
    pub total_keys: usize,
}

impl RebalancingCost {
    /// Fraction of keys that moved.
    pub fn fraction_moved(&self) -> f64 {
        if self.total_keys == 0 {
            return 0.0;
        }
        self.keys_moved as f64 / self.total_keys as f64
    }
}

/// Hashing strategy for rebalancing measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RebalanceStrategy {
    Naive,
    ConsistentHash,
    JumpHash,
}

fn hash_str(key: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

/// Compute naive hash assignment.
fn naive_assign(key: &str, num_nodes: usize) -> usize {
    hash_str(key) as usize % num_nodes
}

/// Compute jump consistent hash assignment.
fn jump_assign(key: &str, num_buckets: usize) -> usize {
    if num_buckets <= 1 {
        return 0;
    }
    let hash = hash_str(key);
    let mut key = hash;
    let mut b: i64 = -1;
    let mut j: i64 = 0;

    while (j as usize) < num_buckets {
        b = j;
        key = key.wrapping_mul(2862933555777941757).wrapping_add(1);
        let mut k = key >> 33;
        k = k.wrapping_add(1);
        let k_f = k as f64;
        let b_plus_1_f = (b + 1) as f64;
        j = (b_plus_1_f * (2147483648.0 / k_f)) as i64;
    }

    b as usize
}

/// Compute consistent hash ring assignment.
fn consistent_assign(key: &str, ring: &BTreeMap<u64, usize>) -> usize {
    let pos = hash_str(key);
    ring.range(pos..)
        .next()
        .or_else(|| ring.iter().next())
        .map(|(_, &n)| n)
        .unwrap_or(0)
}

fn build_ring(num_nodes: usize) -> BTreeMap<u64, usize> {
    let mut ring = BTreeMap::new();
    for node_id in 0..num_nodes {
        for v in 0..150 {
            let vk = format!("node_{}#{}", node_id, v);
            ring.insert(hash_str(&vk), node_id);
        }
    }
    ring
}

/// Measure rebalancing cost for a given strategy.
pub fn measure_rebalance(
    strategy: RebalanceStrategy,
    old_num_nodes: usize,
    new_num_nodes: usize,
    keys: &[String],
) -> RebalancingCost {
    let total = keys.len();

    let moved = match strategy {
        RebalanceStrategy::Naive => {
            keys.iter()
                .filter(|k| naive_assign(k, old_num_nodes) != naive_assign(k, new_num_nodes))
                .count()
        }
        RebalanceStrategy::JumpHash => {
            keys.iter()
                .filter(|k| jump_assign(k, old_num_nodes) != jump_assign(k, new_num_nodes))
                .count()
        }
        RebalanceStrategy::ConsistentHash => {
            let old_ring = build_ring(old_num_nodes);
            let new_ring = build_ring(new_num_nodes);
            keys.iter()
                .filter(|k| consistent_assign(k, &old_ring) != consistent_assign(k, &new_ring))
                .count()
        }
    };

    RebalancingCost {
        keys_moved: moved,
        total_keys: total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_keys(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("key_{}", i)).collect()
    }

    #[test]
    fn test_consistent_hashing_moves_few_keys() {
        let keys = generate_keys(100_000);
        let cost = measure_rebalance(RebalanceStrategy::ConsistentHash, 10, 11, &keys);

        let fraction = cost.fraction_moved();
        // Should move roughly 1/N = 1/10 = 10% of keys
        assert!(
            fraction < 0.20,
            "Consistent hashing should move ~1/N keys, moved {:.1}%",
            fraction * 100.0
        );
        assert!(
            fraction > 0.01,
            "Consistent hashing should move at least some keys, moved {:.1}%",
            fraction * 100.0
        );
    }

    #[test]
    fn test_naive_hashing_moves_most_keys() {
        let keys = generate_keys(100_000);
        let cost = measure_rebalance(RebalanceStrategy::Naive, 10, 11, &keys);

        let fraction = cost.fraction_moved();
        // Should move roughly (N-1)/N = 9/10 = 90% of keys
        assert!(
            fraction > 0.70,
            "Naive hashing should move ~(N-1)/N keys, moved {:.1}%",
            fraction * 100.0
        );
    }

    #[test]
    fn test_jump_hash_moves_few_keys() {
        let keys = generate_keys(100_000);
        let cost = measure_rebalance(RebalanceStrategy::JumpHash, 10, 11, &keys);

        let fraction = cost.fraction_moved();
        // Should move roughly 1/(N+1) = 1/11 of keys
        assert!(
            fraction < 0.20,
            "Jump hash should move ~1/(N+1) keys, moved {:.1}%",
            fraction * 100.0
        );
    }

    #[test]
    fn test_consistent_hash_moves_fewer_than_naive() {
        let keys = generate_keys(50_000);
        let consistent_cost = measure_rebalance(RebalanceStrategy::ConsistentHash, 10, 11, &keys);
        let naive_cost = measure_rebalance(RebalanceStrategy::Naive, 10, 11, &keys);

        assert!(
            consistent_cost.keys_moved < naive_cost.keys_moved,
            "Consistent hashing ({} moved) should move fewer keys than naive ({} moved)",
            consistent_cost.keys_moved,
            naive_cost.keys_moved
        );
    }

    #[test]
    fn test_zero_keys_no_cost() {
        let keys: Vec<String> = vec![];
        let cost = measure_rebalance(RebalanceStrategy::ConsistentHash, 10, 11, &keys);
        assert_eq!(cost.fraction_moved(), 0.0);
    }
}
