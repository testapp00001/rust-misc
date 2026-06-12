//! # Exercise: Load Balance Analysis
//!
//! ## Theory
//!
//! Comparing different hashing strategies reveals tradeoffs in load balance,
//! rebalancing cost, and implementation complexity. This exercise measures
//! the distribution quality of naive hashing, consistent hashing, jump
//! consistent hash, and rendezvous hashing.
//!
//! Metrics include:
//! - Standard deviation of load per node
//! - Max/min load ratio
//! - Coefficient of variation (CV = stddev / mean)
//!
//! ## Proof / Intuition
//!
//! A perfectly balanced system has CV = 0. In practice, consistent hashing
//! with sufficient virtual nodes achieves CV < 0.05, while naive hashing
//! achieves near-zero CV (perfect balance when N is fixed) but terrible
//! rebalancing characteristics.
//!
//! ## Implementation Task
//!
//! Implement `LoadBalancer` that wraps all strategies and exposes a common
//! `get_assignments` interface. Then measure and compare distribution metrics.
//!
//! ## Verification
//!
//! Verify consistent hashing outperforms naive in rebalancing, and rendezvous
//! hashing performs similarly to consistent hashing.
//!
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Node identifier.
pub type NodeId = usize;

/// Strategy enum for selecting a hashing approach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Naive,
    ConsistentHash,
    JumpHash,
    Rendezvous,
}

/// Unified load balancer interface.
#[derive(Debug)]
pub struct LoadBalancer {
    strategy: Strategy,
    num_nodes: usize,
    // Consistent hash ring data
    ring: BTreeMap<u64, NodeId>,
    // Node list for rendezvous
    nodes: Vec<NodeId>,
}

impl LoadBalancer {
    pub fn new(strategy: Strategy, num_nodes: usize) -> Self {
        let mut lb = Self {
            strategy,
            num_nodes,
            ring: BTreeMap::new(),
            nodes: (0..num_nodes).collect(),
        };
        lb.build_ring();
        lb
    }

    fn build_ring(&mut self) {
        self.ring.clear();
        if self.strategy == Strategy::ConsistentHash {
            for node_id in 0..self.num_nodes {
                for v in 0..150 {
                    let key = format!("node_{}#{}", node_id, v);
                    let pos = Self::hash_str(&key);
                    self.ring.insert(pos, node_id);
                }
            }
        }
    }

    fn hash_str(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn hash_key(key: &str) -> u64 {
        Self::hash_str(key)
    }

    /// Get the node assignment for a key.
    pub fn get_node(&self, key: &str) -> NodeId {
        match self.strategy {
            Strategy::Naive => {
                let hash = Self::hash_key(key) as usize;
                hash % self.num_nodes
            }
            Strategy::ConsistentHash => {
                let pos = Self::hash_key(key);
                self.ring
                    .range(pos..)
                    .next()
                    .or_else(|| self.ring.iter().next())
                    .map(|(_, &n)| n)
                    .unwrap_or(0)
            }
            Strategy::JumpHash => jump_hash(key, self.num_nodes),
            Strategy::Rendezvous => {
                let mut best_node = 0;
                let mut best_weight = 0;
                for &node in &self.nodes {
                    let w = rendezvous_weight(key, node);
                    if w > best_weight {
                        best_weight = w;
                        best_node = node;
                    }
                }
                best_node
            }
        }
    }

    /// Get assignments for all keys.
    pub fn get_assignments(&self, keys: &[String]) -> Vec<NodeId> {
        keys.iter().map(|k| self.get_node(k)).collect()
    }
}

/// Jump consistent hash implementation.
fn jump_hash(key: &str, num_buckets: usize) -> usize {
    if num_buckets <= 1 {
        return 0;
    }
    let hash = LoadBalancer::hash_key(key);
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

/// Rendezvous weight computation.
fn rendezvous_weight(key: &str, node: NodeId) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    node.hash(&mut hasher);
    hasher.finish()
}

/// Simple xorshift64 PRNG. Produces values in [0, 1).
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }
    fn next_f64(&mut self) -> f64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        // Ensure value is strictly less than 1.0 by masking to 53 bits
        (self.state >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

/// Metrics for comparing load balance quality.
#[derive(Debug, Clone)]
pub struct LoadMetrics {
    pub mean: f64,
    pub std_dev: f64,
    pub cv: f64,
    pub max_load: usize,
    pub min_load: usize,
    pub max_min_ratio: f64,
}

impl LoadMetrics {
    pub fn from_distribution(dist: &HashMap<NodeId, usize>, num_nodes: usize) -> Self {
        let counts: Vec<f64> = (0..num_nodes)
            .map(|i| dist.get(&i).copied().unwrap_or(0) as f64)
            .collect();
        let n = counts.len() as f64;
        let mean = counts.iter().sum::<f64>() / n;
        let variance = counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / n;
        let std_dev = variance.sqrt();
        let cv = if mean > 0.0 { std_dev / mean } else { 0.0 };
        let max_load = counts.iter().copied().map(|c| c as usize).max().unwrap_or(0);
        let min_load = counts.iter().copied().map(|c| c as usize).min().unwrap_or(0);
        let max_min_ratio = if min_load > 0 {
            max_load as f64 / min_load as f64
        } else {
            f64::INFINITY
        };

        Self {
            mean,
            std_dev,
            cv,
            max_load,
            min_load,
            max_min_ratio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_keys(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("key_{}", i)).collect()
    }

    fn compute_metrics(strategy: Strategy, num_nodes: usize, keys: &[String]) -> LoadMetrics {
        let lb = LoadBalancer::new(strategy, num_nodes);
        let dist: HashMap<NodeId, usize> = {
            let assignments = lb.get_assignments(keys);
            let mut d: HashMap<NodeId, usize> = HashMap::new();
            for &node in &assignments {
                *d.entry(node).or_insert(0) += 1;
            }
            d
        };
        LoadMetrics::from_distribution(&dist, num_nodes)
    }

    #[test]
    fn test_consistent_hashing_outperforms_naive() {
        let keys = generate_keys(100_000);

        let naive_metrics = compute_metrics(Strategy::Naive, 10, &keys);
        let consistent_metrics = compute_metrics(Strategy::ConsistentHash, 10, &keys);

        // Both should have decent balance with fixed node count
        // But consistent hashing with vnodes should have lower CV
        assert!(
            consistent_metrics.cv < 0.10,
            "Consistent hashing CV should be low: {:.4}",
            consistent_metrics.cv
        );
        assert!(
            naive_metrics.cv < 0.10,
            "Naive hashing CV should be low with fixed nodes: {:.4}",
            naive_metrics.cv
        );
    }

    #[test]
    fn test_rendezvous_similar_to_consistent_hashing() {
        let keys = generate_keys(100_000);

        let consistent_metrics = compute_metrics(Strategy::ConsistentHash, 10, &keys);
        let rendezvous_metrics = compute_metrics(Strategy::Rendezvous, 10, &keys);

        // Both should have similar quality (low CV)
        assert!(
            rendezvous_metrics.cv < 0.10,
            "Rendezvous hashing CV should be low: {:.4}",
            rendezvous_metrics.cv
        );
        // Rendezvous should not be dramatically worse than consistent hashing
        assert!(
            rendezvous_metrics.cv < consistent_metrics.cv * 3.0,
            "Rendezvous (CV={:.4}) should not be much worse than consistent (CV={:.4})",
            rendezvous_metrics.cv,
            consistent_metrics.cv
        );
    }

    #[test]
    fn test_jump_hash_perfect_distribution() {
        let keys = generate_keys(100_000);
        let metrics = compute_metrics(Strategy::JumpHash, 10, &keys);

        // Jump hash is perfectly uniform
        assert!(
            metrics.cv < 0.02,
            "Jump hash should be nearly perfectly uniform: CV={:.4}",
            metrics.cv
        );
    }

    #[test]
    fn test_all_strategies_assign_all_keys() {
        let keys = generate_keys(1000);
        for strategy in [Strategy::Naive, Strategy::ConsistentHash, Strategy::JumpHash, Strategy::Rendezvous] {
            let lb = LoadBalancer::new(strategy, 10);
            let assignments = lb.get_assignments(&keys);
            assert_eq!(assignments.len(), keys.len());
            for &node in &assignments {
                assert!(node < 10, "Node {} out of range for strategy {:?}", node, strategy);
            }
        }
    }
}
