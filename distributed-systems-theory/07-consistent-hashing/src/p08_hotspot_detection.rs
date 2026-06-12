//! # Exercise: Hotspot Detection and Handling
//!
//! ## Theory
//!
//! Hotspots occur when certain keys or shards receive disproportionate traffic.
//! In a sharded system, this can cause single-node overload while other nodes
//! sit idle. Detection uses access frequency tracking, and mitigation strategies
//! include shard splitting, caching, rate limiting, and key salting.
//!
//! ## Proof / Intuition
//!
//! A shard is considered "hot" if its access frequency significantly exceeds
//! the mean. A common threshold is 2x or 3x the mean frequency. Once detected,
//! mitigation strategies redistribute the load:
//! - Shard splitting: Divide the hot shard into multiple sub-shards
//! - Caching: Store frequent results in a fast cache layer
//! - Rate limiting: Throttle excessive requests
//! - Key salting: Append random suffixes to create virtual sub-keys
//!
//! ## Implementation Task
//!
//! Implement `HotspotDetector` that tracks per-shard access frequency and
//! identifies hotspots. Implement mitigation strategies.
//!
//! ## Verification
//!
//! Verify hotspot detection identifies overloaded shards and mitigation
//! reduces the load on the hot shard.
//!
use std::collections::HashMap;

/// A shard identifier.
pub type ShardId = usize;

/// Detects hotspots based on access frequency.
#[derive(Debug)]
pub struct HotspotDetector {
    /// Access count per shard.
    access_counts: HashMap<ShardId, usize>,
    /// Threshold multiplier above mean to flag as hotspot.
    threshold_multiplier: f64,
    /// Total accesses tracked.
    total_accesses: usize,
}

impl HotspotDetector {
    /// Create a new detector with the given threshold (e.g., 2.0 = 2x mean).
    pub fn new(threshold_multiplier: f64) -> Self {
        Self {
            access_counts: HashMap::new(),
            threshold_multiplier,
            total_accesses: 0,
        }
    }

    /// Record an access to the given shard.
    pub fn record_access(&mut self, shard_id: ShardId) {
        *self.access_counts.entry(shard_id).or_insert(0) += 1;
        self.total_accesses += 1;
    }

    /// Record multiple accesses to the given shard.
    pub fn record_batch(&mut self, shard_id: ShardId, count: usize) {
        *self.access_counts.entry(shard_id).or_insert(0) += count;
        self.total_accesses += count;
    }

    /// Get the mean access frequency across all shards.
    pub fn mean_frequency(&self) -> f64 {
        if self.access_counts.is_empty() {
            return 0.0;
        }
        self.total_accesses as f64 / self.access_counts.len() as f64
    }

    /// Check if a specific shard is a hotspot.
    pub fn is_hotspot(&self, shard_id: ShardId) -> bool {
        let mean = self.mean_frequency();
        if mean == 0.0 {
            return false;
        }
        let count = self.access_counts.get(&shard_id).copied().unwrap_or(0) as f64;
        count > mean * self.threshold_multiplier
    }

    /// Get all hotspot shard IDs.
    pub fn get_hotspots(&self) -> Vec<ShardId> {
        self.access_counts
            .keys()
            .filter(|&&id| self.is_hotspot(id))
            .copied()
            .collect()
    }

    /// Get the access count for a shard.
    pub fn get_count(&self, shard_id: ShardId) -> usize {
        self.access_counts.get(&shard_id).copied().unwrap_or(0)
    }

    /// Get all shard access counts.
    pub fn get_all_counts(&self) -> &HashMap<ShardId, usize> {
        &self.access_counts
    }
}

/// Mitigation strategy for hotspots.
#[derive(Debug, Clone)]
pub enum MitigationStrategy {
    /// Split a hot shard into N sub-shards.
    SplitShard { shard_id: ShardId, sub_shards: usize },
    /// Cache hot data for the given shard.
    CacheData { shard_id: ShardId, ttl_seconds: u64 },
    /// Rate limit requests to the given shard.
    RateLimit { shard_id: ShardId, max_per_second: usize },
    /// Salt keys in the given shard to distribute load.
    SaltKeys { shard_id: ShardId, salt_range: usize },
}

/// Applies mitigation strategies and tracks their effect.
#[derive(Debug)]
pub struct MitigationManager {
    /// Current load per shard (simulated).
    shard_load: HashMap<ShardId, usize>,
}

impl MitigationManager {
    pub fn new() -> Self {
        Self {
            shard_load: HashMap::new(),
        }
    }

    pub fn set_load(&mut self, shard_id: ShardId, load: usize) {
        self.shard_load.insert(shard_id, load);
    }

    pub fn get_load(&self, shard_id: ShardId) -> usize {
        self.shard_load.get(&shard_id).copied().unwrap_or(0)
    }

    /// Apply a mitigation strategy and return the resulting load distribution.
    pub fn apply_strategy(&mut self, strategy: MitigationStrategy) {
        match strategy {
            MitigationStrategy::SplitShard {
                shard_id,
                sub_shards,
            } => {
                let load = self.shard_load.remove(&shard_id).unwrap_or(0);
                let per_sub = load / sub_shards;
                let remainder = load % sub_shards;
                for i in 0..sub_shards {
                    let sub_load = if i == 0 {
                        per_sub + remainder
                    } else {
                        per_sub
                    };
                    self.shard_load
                        .insert(shard_id * 1000 + i, sub_load);
                }
            }
            MitigationStrategy::CacheData { shard_id, .. } => {
                // Caching reduces load by ~60% (simulated)
                let load = self.shard_load.entry(shard_id).or_insert(0);
                *load = (*load as f64 * 0.4) as usize;
            }
            MitigationStrategy::RateLimit {
                shard_id,
                max_per_second,
            } => {
                let load = self.shard_load.entry(shard_id).or_insert(0);
                if *load > max_per_second {
                    *load = max_per_second;
                }
            }
            MitigationStrategy::SaltKeys {
                shard_id,
                salt_range,
            } => {
                let load = self.shard_load.remove(&shard_id).unwrap_or(0);
                for i in 0..salt_range {
                    let sub_load = load / salt_range;
                    self.shard_load.insert(shard_id * 1000 + i, sub_load);
                }
            }
        }
    }

    /// Get the maximum load across all shards.
    pub fn max_load(&self) -> usize {
        self.shard_load.values().copied().max().unwrap_or(0)
    }

    /// Get total load across all shards.
    pub fn total_load(&self) -> usize {
        self.shard_load.values().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotspot_detection() {
        let mut detector = HotspotDetector::new(2.0);

        // Create a normal distribution: 10 shards, ~100 accesses each
        for shard in 0..10 {
            detector.record_batch(shard, 100);
        }

        // Make shard 3 a hotspot: 1000 accesses
        detector.record_batch(3, 900);

        assert!(detector.is_hotspot(3), "Shard 3 should be detected as hotspot");
        assert!(!detector.is_hotspot(0), "Shard 0 should not be a hotspot");
    }

    #[test]
    fn test_mitigation_reduces_hot_shard_load() {
        let mut manager = MitigationManager::new();
        manager.set_load(0, 100);
        manager.set_load(1, 100);
        manager.set_load(2, 1000); // Hot shard

        let hot_before = manager.get_load(2);
        assert!(hot_before > 100);

        // Apply caching mitigation
        manager.apply_strategy(MitigationStrategy::CacheData {
            shard_id: 2,
            ttl_seconds: 60,
        });

        let hot_after = manager.get_load(2);
        assert!(
            hot_after < hot_before,
            "Caching should reduce load on hot shard (before: {}, after: {})",
            hot_before,
            hot_after
        );
    }

    #[test]
    fn test_shard_split_distribution() {
        let mut manager = MitigationManager::new();
        manager.set_load(5, 1000);

        manager.apply_strategy(MitigationStrategy::SplitShard {
            shard_id: 5,
            sub_shards: 4,
        });

        // Original shard should be gone
        assert_eq!(manager.get_load(5), 0);

        // Sub-shards should share the load
        let sub_load: usize = (0..4)
            .map(|i| manager.get_load(5 * 1000 + i))
            .sum();
        assert_eq!(sub_load, 1000, "Total load should be preserved after split");
    }

    #[test]
    fn test_rate_limiting() {
        let mut manager = MitigationManager::new();
        manager.set_load(3, 500);

        manager.apply_strategy(MitigationStrategy::RateLimit {
            shard_id: 3,
            max_per_second: 100,
        });

        assert_eq!(
            manager.get_load(3),
            100,
            "Rate limiting should cap load at max_per_second"
        );
    }
}
