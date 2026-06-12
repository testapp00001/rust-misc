//! # Exercise: Virtual Node Impact Analysis
//!
//! ## Theory
//!
//! Virtual nodes (vnodes) improve load balance in consistent hashing by mapping
//! each physical node to multiple positions on the hash ring. More virtual nodes
//! lead to a more uniform distribution of keys across physical nodes.
//!
//! With 1 virtual node per physical node, the standard deviation of load can be
//! very high. As vnodes increase, the distribution approaches uniform with
//! diminishing returns beyond approximately 100-200 vnodes per physical node.
//!
//! ## Proof / Intuition
//!
//! The load imbalance follows the birthday problem. With N physical nodes and
//! V vnodes each, we have N*V points on the ring. The expected load per physical
//! node is K/N keys (where K is total keys). The standard deviation decreases
//! as O(1/sqrt(V)), meaning doubling vnodes reduces imbalance by ~30%.
//!
//! ## Implementation Task
//!
//! Implement `LoadAnalyzer` that measures key distribution across nodes for
//! different numbers of virtual nodes. Test with 1, 10, 50, 100, 200 vnodes.
//!
//! ## Verification
//!
//! Verify that more vnodes produce lower standard deviation and that returns
//! diminish past 100 vnodes.
//!
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Analyzes load distribution across nodes for varying virtual node counts.
#[derive(Debug)]
pub struct LoadAnalyzer {
    num_physical_nodes: usize,
    total_keys: usize,
}

impl LoadAnalyzer {
    pub fn new(num_physical_nodes: usize, total_keys: usize) -> Self {
        Self {
            num_physical_nodes,
            total_keys,
        }
    }

    /// Compute the key distribution for a given number of virtual nodes.
    /// Returns a map of node_id -> key_count.
    pub fn measure_distribution(&self, num_vnodes: usize) -> HashMap<usize, usize> {
        use std::collections::BTreeMap;

        let mut ring: BTreeMap<u64, usize> = BTreeMap::new();

        // Build ring with vnodes
        for node_id in 0..self.num_physical_nodes {
            for v in 0..num_vnodes {
                let key = format!("node_{}#{}", node_id, v);
                let pos = Self::hash_str(&key);
                ring.insert(pos, node_id);
            }
        }

        // Assign keys to nodes
        let mut distribution: HashMap<usize, usize> = HashMap::new();
        for i in 0..self.total_keys {
            let key = format!("key_{}", i);
            let pos = Self::hash_str(&key);
            let node = ring
                .range(pos..)
                .next()
                .or_else(|| ring.iter().next())
                .map(|(_, &n)| n)
                .unwrap_or(0);
            *distribution.entry(node).or_insert(0) += 1;
        }

        distribution
    }

    /// Compute standard deviation of load across nodes.
    pub fn std_dev(&self, distribution: &HashMap<usize, usize>) -> f64 {
        let counts: Vec<f64> = distribution.values().map(|&c| c as f64).collect();
        let n = counts.len() as f64;
        if n == 0.0 {
            return 0.0;
        }
        let mean = counts.iter().sum::<f64>() / n;
        let variance = counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / n;
        variance.sqrt()
    }

    /// Compute coefficient of variation (stddev / mean).
    pub fn coefficient_of_variation(&self, distribution: &HashMap<usize, usize>) -> f64 {
        let counts: Vec<f64> = distribution.values().map(|&c| c as f64).collect();
        let n = counts.len() as f64;
        if n == 0.0 {
            return 0.0;
        }
        let mean = counts.iter().sum::<f64>() / n;
        if mean == 0.0 {
            return 0.0;
        }
        let variance = counts.iter().map(|c| (c - mean).powi(2)).sum::<f64>() / n;
        variance.sqrt() / mean
    }

    /// Compute max/min load ratio.
    pub fn max_min_ratio(&self, distribution: &HashMap<usize, usize>) -> f64 {
        let counts: Vec<&usize> = distribution.values().collect();
        let max_val = counts.iter().copied().max().unwrap_or(&1);
        let min_val = counts.iter().copied().min().unwrap_or(&1);
        *max_val as f64 / *min_val as f64
    }

    fn hash_str(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_more_vnodes_improve_load_balance() {
        let analyzer = LoadAnalyzer::new(10, 100_000);

        let dist_1 = analyzer.measure_distribution(1);
        let dist_10 = analyzer.measure_distribution(10);
        let dist_100 = analyzer.measure_distribution(100);

        let std_1 = analyzer.std_dev(&dist_1);
        let std_10 = analyzer.std_dev(&dist_10);
        let std_100 = analyzer.std_dev(&dist_100);

        // More vnodes should reduce standard deviation
        assert!(
            std_10 < std_1,
            "10 vnodes (std={:.1}) should be better than 1 (std={:.1})",
            std_10,
            std_1
        );
        assert!(
            std_100 < std_10,
            "100 vnodes (std={:.1}) should be better than 10 (std={:.1})",
            std_100,
            std_10
        );
    }

    #[test]
    fn test_diminishing_returns_past_100_vnodes() {
        let analyzer = LoadAnalyzer::new(10, 100_000);

        let dist_50 = analyzer.measure_distribution(50);
        let dist_100 = analyzer.measure_distribution(100);
        let dist_200 = analyzer.measure_distribution(200);

        let cv_50 = analyzer.coefficient_of_variation(&dist_50);
        let cv_100 = analyzer.coefficient_of_variation(&dist_100);
        let cv_200 = analyzer.coefficient_of_variation(&dist_200);

        // Improvement from 100 to 200 should be less than from 50 to 100
        let improvement_50_100 = cv_50 - cv_100;
        let improvement_100_200 = cv_100 - cv_200;

        assert!(
            improvement_100_200 < improvement_50_100,
            "Diminishing returns: 50->100 improvement ({:.4}) should exceed 100->200 ({:.4})",
            improvement_50_100,
            improvement_100_200
        );
    }

    #[test]
    fn test_200_vnodes_near_uniform() {
        let analyzer = LoadAnalyzer::new(10, 100_000);
        let dist = analyzer.measure_distribution(200);

        // With 200 vnodes per node, coefficient of variation should be very low
        let cv = analyzer.coefficient_of_variation(&dist);
        assert!(
            cv < 0.15,
            "200 vnodes should produce near-uniform distribution (CV={:.4})",
            cv
        );
    }

    #[test]
    fn test_single_vnode_has_high_imbalance() {
        let analyzer = LoadAnalyzer::new(10, 100_000);
        let dist = analyzer.measure_distribution(1);

        let ratio = analyzer.max_min_ratio(&dist);
        assert!(
            ratio > 1.5,
            "Single vnode should produce noticeable imbalance (ratio={:.2})",
            ratio
        );
    }
}
