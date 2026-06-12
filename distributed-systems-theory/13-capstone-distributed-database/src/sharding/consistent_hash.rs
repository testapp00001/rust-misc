//! Consistent hash ring for data partitioning across shards.
//!
//! A consistent hash ring maps keys to shards using virtual nodes spread around
//! a circular address space. When shards are added or removed, only a minimal
//! fraction of keys need to be remapped, making this ideal for dynamic scaling.

use std::collections::{BTreeMap, HashSet};

/// A consistent hash ring that maps keys to shard ids via virtual nodes.
#[derive(Debug, Clone)]
pub struct ConsistentHashRing {
    /// Mapping from hash position to the shard id that owns that position.
    ring: BTreeMap<u64, usize>,
    /// Number of virtual nodes per physical shard.
    num_virtual_nodes: usize,
    /// Set of shard ids currently on the ring.
    shards: HashSet<usize>,
}

impl ConsistentHashRing {
    /// Create a new, empty consistent hash ring.
    ///
    /// # Arguments
    ///
    /// * `num_virtual_nodes` - The number of virtual nodes to create for each shard.
    pub fn new(num_virtual_nodes: usize) -> Self {
        Self {
            ring: BTreeMap::new(),
            num_virtual_nodes,
            shards: HashSet::new(),
        }
    }

    /// Add a shard to the ring with its virtual nodes distributed around the ring.
    ///
    /// # Arguments
    ///
    /// * `shard_id` - The unique identifier for the shard.
    pub fn add_shard(&mut self, shard_id: usize) {
        self.shards.insert(shard_id);
        for i in 0..self.num_virtual_nodes {
            let position = self.hash_position(&format!("shard-{}-vnode-{}", shard_id, i));
            self.ring.insert(position, shard_id);
        }
    }

    /// Remove a shard and all of its virtual nodes from the ring.
    ///
    /// # Arguments
    ///
    /// * `shard_id` - The shard to remove.
    pub fn remove_shard(&mut self, shard_id: usize) {
        self.shards.remove(&shard_id);
        let keys_to_remove: Vec<u64> = self
            .ring
            .iter()
            .filter(|(_, &sid)| sid == shard_id)
            .map(|(&pos, _)| pos)
            .collect();
        for key in keys_to_remove {
            self.ring.remove(&key);
        }
    }

    /// Determine which shard owns the given key.
    ///
    /// The key is hashed and the ring is walked clockwise from that position to
    /// find the next virtual node.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up.
    ///
    /// # Returns
    ///
    /// The shard id that owns the key, or `None` if the ring is empty.
    pub fn get_shard(&self, key: &str) -> Option<usize> {
        if self.ring.is_empty() {
            return None;
        }
        let position = self.hash_position(key);
        // Find the first node at or clockwise from the key's position.
        // BTreeMap::range returns entries >= the given key.
        self.ring
            .range(position..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, &shard_id)| shard_id)
    }

    /// Return the number of distinct shards on the ring.
    pub fn num_shards(&self) -> usize {
        self.shards.len()
    }

    /// Compute a hash position for a key.
    ///
    /// Uses FNV-1a hashing for good distribution across the ring.
    fn hash_position(&self, key: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325; // FNV offset basis
        for byte in key.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3); // FNV prime
        }
        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_ring() {
        let ring = ConsistentHashRing::new(10);
        assert_eq!(ring.get_shard("key"), None);
        assert_eq!(ring.num_shards(), 0);
    }

    #[test]
    fn test_add_shard_and_lookup() {
        let mut ring = ConsistentHashRing::new(10);
        ring.add_shard(0);
        ring.add_shard(1);
        assert_eq!(ring.num_shards(), 2);

        // All keys should map to some shard.
        for i in 0..100 {
            let shard = ring.get_shard(&format!("key-{}", i));
            assert!(shard.is_some());
        }
    }

    #[test]
    fn test_distribution() {
        let mut ring = ConsistentHashRing::new(150);
        ring.add_shard(0);
        ring.add_shard(1);

        let mut counts = [0u64; 2];
        for i in 0..10000 {
            let shard = ring.get_shard(&format!("key-{}", i)).unwrap();
            counts[shard] += 1;
        }
        // With virtual nodes the distribution should be reasonably balanced.
        // Each shard should get at least 20% of keys.
        assert!(counts[0] > 2000, "shard 0 got only {} keys", counts[0]);
        assert!(counts[1] > 2000, "shard 1 got only {} keys", counts[1]);
    }

    #[test]
    fn test_remove_shard() {
        let mut ring = ConsistentHashRing::new(10);
        ring.add_shard(0);
        ring.add_shard(1);
        ring.remove_shard(0);
        assert_eq!(ring.num_shards(), 1);

        // All keys should now map to shard 1.
        for i in 0..100 {
            assert_eq!(ring.get_shard(&format!("key-{}", i)), Some(1));
        }
    }

    #[test]
    fn test_minimal_remapping() {
        let mut ring = ConsistentHashRing::new(150);
        ring.add_shard(0);
        ring.add_shard(1);

        // Record assignments before adding shard 2.
        let mut before = Vec::new();
        for i in 0..1000 {
            before.push(ring.get_shard(&format!("key-{}", i)));
        }

        ring.add_shard(2);

        // After adding shard 2, at most ~1/3 of keys should have moved.
        let mut moved = 0u64;
        for i in 0..1000 {
            let after = ring.get_shard(&format!("key-{}", i));
            if before[i] != after {
                moved += 1;
            }
        }
        // With consistent hashing, far fewer than 50% should move.
        assert!(moved < 500, "too many keys moved: {}", moved);
    }
}
