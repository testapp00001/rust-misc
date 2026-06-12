//! Consistent hash ring for partition assignment.
//!
//! Maps arbitrary byte-string keys to partition IDs with minimal redistribution
//! when nodes are added or removed. Each physical partition is mapped to
//! `virtual_nodes` positions on the ring to improve balance.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A consistent hash ring that maps keys to partition IDs.
///
/// The ring is stored as a `BTreeMap<u64, u32>` where keys are hash values
/// and values are partition IDs. Lookup is O(log N) via range queries.
#[derive(Debug, Clone)]
pub struct HashRing {
    /// Sorted map of hash positions to partition IDs.
    ring: BTreeMap<u64, u32>,
    /// Number of virtual nodes per physical partition.
    virtual_nodes: usize,
    /// Active partition IDs.
    partitions: Vec<u32>,
}

impl HashRing {
    /// Create a new hash ring with the given partitions and virtual node
    /// count. More virtual nodes means better key distribution at the cost
    /// of memory and lookup time.
    pub fn new(partitions: Vec<u32>, virtual_nodes: usize) -> Self {
        let mut ring = BTreeMap::new();
        for &partition in &partitions {
            for i in 0..virtual_nodes {
                let key = format!("partition-{}-vn-{}", partition, i);
                let hash = Self::hash_str(&key);
                ring.insert(hash, partition);
            }
        }
        HashRing {
            ring,
            virtual_nodes,
            partitions,
        }
    }

    /// Map a byte-string key to a partition ID using consistent hashing.
    /// The key is hashed and the first partition whose hash position is >=
    /// the key hash is returned, wrapping around if necessary.
    pub fn get_partition(&self, key: &[u8]) -> u32 {
        if self.ring.is_empty() {
            return 0;
        }
        let hash = Self::hash_bytes(key);
        // Find the first entry whose hash >= key hash
        if let Some((_, &partition)) = self.ring.range(hash..).next() {
            partition
        } else {
            // Wrap around to the first entry on the ring
            *self.ring.values().next().unwrap()
        }
    }

    /// Determine which node (by index into `nodes`) is responsible for the
    /// given key. The partition derived from the key selects a node via
    /// modular arithmetic.
    pub fn get_node_for_key(&self, key: &[u8], nodes: &[u64]) -> u64 {
        if nodes.is_empty() {
            panic!("get_node_for_key called with empty node list");
        }
        let partition = self.get_partition(key);
        nodes[partition as usize % nodes.len()]
    }

    /// Add a new partition to the ring, creating its virtual nodes.
    pub fn add_partition(&mut self, partition: u32) {
        for i in 0..self.virtual_nodes {
            let key = format!("partition-{}-vn-{}", partition, i);
            let hash = Self::hash_str(&key);
            self.ring.insert(hash, partition);
        }
        self.partitions.push(partition);
    }

    /// Remove a partition and all its virtual nodes from the ring.
    pub fn remove_partition(&mut self, partition: u32) {
        self.ring.retain(|_, &mut p| p != partition);
        self.partitions.retain(|&p| p != partition);
    }

    /// Number of physical partitions on the ring.
    pub fn partition_count(&self) -> usize {
        self.partitions.len()
    }

    /// Number of virtual nodes per physical partition.
    pub fn virtual_node_count(&self) -> usize {
        self.virtual_nodes
    }

    /// Total number of positions on the ring.
    pub fn ring_size(&self) -> usize {
        self.ring.len()
    }

    /// Compute the balance ratio: the ratio of the largest partition share
    /// to the ideal equal share. A value close to 1.0 means perfect balance.
    pub fn balance_ratio(&self, sample_keys: &[&[u8]]) -> f64 {
        if self.partitions.is_empty() || sample_keys.is_empty() {
            return 0.0;
        }

        let mut counts = std::collections::HashMap::new();
        for key in sample_keys {
            let p = self.get_partition(key);
            *counts.entry(p).or_insert(0usize) += 1;
        }

        let total = sample_keys.len() as f64;
        let ideal = total / self.partitions.len() as f64;
        let max_count = counts.values().copied().max().unwrap_or(0) as f64;

        if ideal == 0.0 {
            return 0.0;
        }
        max_count / ideal
    }

    /// Hash a string key to a u64 position on the ring.
    fn hash_str(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Hash a byte slice to a u64 position on the ring.
    fn hash_bytes(data: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ring(n: u32) -> HashRing {
        HashRing::new((0..n).collect(), 150)
    }

    #[test]
    fn basic_lookup() {
        let ring = make_ring(6);
        let p = ring.get_partition(b"hello");
        assert!(p < 6);
    }

    #[test]
    fn same_key_always_same_partition() {
        let ring = make_ring(6);
        let p1 = ring.get_partition(b"key-abc");
        let p2 = ring.get_partition(b"key-abc");
        assert_eq!(p1, p2);
    }

    #[test]
    fn different_keys_distribute() {
        let ring = make_ring(10);
        let mut seen = std::collections::HashSet::new();
        for i in 0..1000 {
            let key = format!("msg-{}", i);
            seen.insert(ring.get_partition(key.as_bytes()));
        }
        // With 1000 keys and 10 partitions, we should hit at least 5
        // different partitions (generous lower bound).
        assert!(seen.len() >= 5, "Distribution too skewed: {:?}", seen);
    }

    #[test]
    fn add_partition_minimal_redistribution() {
        let mut ring = make_ring(4);
        let keys: Vec<String> = (0..200).map(|i| format!("key-{}", i)).collect();

        let before: Vec<u32> = keys.iter().map(|k| ring.get_partition(k.as_bytes())).collect();

        ring.add_partition(4);

        let after: Vec<u32> = keys.iter().map(|k| ring.get_partition(k.as_bytes())).collect();

        let mut moved = 0;
        for (b, a) in before.iter().zip(after.iter()) {
            if b != a {
                moved += 1;
            }
        }
        // Consistent hashing should move roughly 1/5 of keys (20%)
        // Allow up to 40% to be safe with small samples
        let move_ratio = moved as f64 / keys.len() as f64;
        assert!(
            move_ratio < 0.5,
            "Too many keys moved: {}/{} ({:.1}%)",
            moved,
            keys.len(),
            move_ratio * 100.0
        );
    }

    #[test]
    fn remove_partition() {
        let mut ring = make_ring(6);
        ring.remove_partition(3);
        assert_eq!(ring.partition_count(), 5);
        assert_eq!(ring.ring_size(), 5 * 150);

        // All lookups should return a valid partition
        for i in 0..100 {
            let key = format!("k-{}", i);
            let p = ring.get_partition(key.as_bytes());
            assert!(p < 6);
            assert_ne!(p, 3);
        }
    }

    #[test]
    fn get_node_for_key() {
        let ring = make_ring(4);
        let nodes = vec![100, 200, 300, 400, 500];
        let node = ring.get_node_for_key(b"test", &nodes);
        assert!(nodes.contains(&node));
    }

    #[test]
    fn single_partition() {
        let ring = HashRing::new(vec![0], 10);
        assert_eq!(ring.get_partition(b"anything"), 0);
        assert_eq!(ring.get_partition(b"something-else"), 0);
    }

    #[test]
    fn balance_ratio_reasonable() {
        let ring = make_ring(10);
        let keys: Vec<String> = (0..10000).map(|i| format!("item-{}", i)).collect();
        let key_refs: Vec<&[u8]> = keys.iter().map(|k| k.as_bytes()).collect();
        let ratio = ring.balance_ratio(&key_refs);
        // Perfect balance is 1.0; with 150 vnodes we should be close
        assert!(ratio < 2.0, "Balance ratio too high: {}", ratio);
        assert!(ratio >= 1.0, "Balance ratio should be >= 1.0: {}", ratio);
    }

    #[test]
    fn ring_size_after_operations() {
        let mut ring = make_ring(4);
        assert_eq!(ring.ring_size(), 4 * 150);
        ring.add_partition(5);
        assert_eq!(ring.ring_size(), 5 * 150);
        ring.remove_partition(2);
        assert_eq!(ring.ring_size(), 4 * 150);
    }
}
