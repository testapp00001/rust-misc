//! # Exercise: Sharded Cache with Consistent Hashing
//!
//! ## Theory
//!
//! Consistent hashing maps both keys and nodes onto a hash ring. To find which
//! node owns a key, hash the key and walk clockwise until you hit a node. When
//! a node is added or removed, only the keys between the new/removed node and
//! its clockwise neighbor need to move.
//!
//! Virtual nodes (replicas) improve load distribution by placing each physical
//! node at multiple points on the ring. With enough virtual nodes, the standard
//! deviation of load across nodes approaches the ideal uniform distribution.
//!
//! ## Proof / Intuition
//!
//! Without consistent hashing, adding a node to N nodes causes ~1/(N+1) of
//! all keys to move. With consistent hashing, only keys that map to the
//! region between the new node and its predecessor move. In expectation,
//! this is ~1/(N+1) keys, but the movement is localized and doesn't require
//! rehashing all keys.
//!
//! The key property: with V virtual nodes per physical node and N physical
//! nodes, adding/removing a node causes approximately K/(N*V) keys to move,
//! where K is the total number of keys.
//!
//! ## Implementation Task
//!
//! 1. Implement a HashRing using BTreeMap for O(log n) lookups
//! 2. Support virtual nodes for better distribution
//! 3. Implement a ShardedCache that routes keys via the hash ring
//! 4. Measure key movement when adding/removing shards
//! 5. Verify correct routing and minimal disruption
//!
//! ## Verification
//!
//! - Test that keys route consistently to the same shard
//! - Test that adding a shard only moves a minimal number of keys
//! - Test get/put operations work correctly through the sharded cache

use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Hash a key to a u64 position on the ring.
fn hash_to_u64<T: Hash>(key: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

/// A consistent hash ring that maps keys to nodes.
pub struct HashRing {
    /// Map from hash position -> node index.
    ring: BTreeMap<u64, usize>,
    /// Number of virtual nodes per physical node.
    replicas_per_node: usize,
    /// Number of physical nodes.
    virtual_nodes: usize,
}

impl HashRing {
    /// Create a new empty hash ring.
    pub fn new(replicas_per_node: usize) -> Self {
        Self {
            ring: BTreeMap::new(),
            replicas_per_node,
            virtual_nodes: 0,
        }
    }

    /// Add a node to the hash ring with virtual nodes.
    pub fn add_node(&mut self, node_id: usize) {
        for i in 0..self.replicas_per_node {
            let key = format!("node-{}-vnode-{}", node_id, i);
            let hash = hash_to_u64(&key);
            self.ring.insert(hash, node_id);
        }
        self.virtual_nodes += 1;
    }

    /// Remove a node from the hash ring.
    pub fn remove_node(&mut self, node_id: usize) {
        for i in 0..self.replicas_per_node {
            let key = format!("node-{}-vnode-{}", node_id, i);
            let hash = hash_to_u64(&key);
            self.ring.remove(&hash);
        }
        self.virtual_nodes -= 1;
    }

    /// Get the node that owns a given key.
    /// Returns None if the ring is empty.
    pub fn get_node(&self, key: &str) -> Option<usize> {
        if self.ring.is_empty() {
            return None;
        }

        let hash = hash_to_u64(&key);

        // Find the first node clockwise from the hash position
        // BTreeMap's range gives us the next entry >= hash
        if let Some((_, &node_id)) = self.ring.range(hash..).next() {
            Some(node_id)
        } else {
            // Wrap around to the first node on the ring
            self.ring.values().next().copied()
        }
    }

    /// Get the total number of entries on the ring.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    /// Check if the ring is empty.
    pub fn is_empty(&self) -> bool {
        self.ring.is_empty()
    }
}

/// A single cache shard storing key-value pairs.
#[derive(Debug, Clone)]
pub struct CacheShard {
    /// The shard's data.
    data: HashMap<String, String>,
    /// Unique identifier for this shard.
    id: usize,
}

impl CacheShard {
    /// Create a new cache shard.
    pub fn new(id: usize) -> Self {
        Self {
            data: HashMap::new(),
            id,
        }
    }

    /// Get a value from this shard.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    /// Put a value into this shard.
    pub fn put(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    /// Remove a value from this shard.
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    /// Get the shard ID.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the number of entries in this shard.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the shard is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// A sharded cache using consistent hashing for key distribution.
pub struct ShardedCache {
    /// The consistent hash ring.
    hash_ring: HashRing,
    /// The cache shards, indexed by shard ID.
    shards: Vec<CacheShard>,
}

impl ShardedCache {
    /// Create a new sharded cache with the given number of shards.
    pub fn new(num_shards: usize, replicas_per_node: usize) -> Self {
        let mut hash_ring = HashRing::new(replicas_per_node);
        let mut shards = Vec::new();

        for i in 0..num_shards {
            shards.push(CacheShard::new(i));
            hash_ring.add_node(i);
        }

        Self { hash_ring, shards }
    }

    /// Add a new shard to the cache.
    pub fn add_shard(&mut self, id: usize) {
        self.hash_ring.add_node(id);
        self.shards.push(CacheShard::new(id));
    }

    /// Remove a shard from the cache by ID.
    pub fn remove_shard(&mut self, id: usize) -> bool {
        if let Some(pos) = self.shards.iter().position(|s| s.id() == id) {
            self.hash_ring.remove_node(id);
            self.shards.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get the shard index (in self.shards) that owns a given key.
    fn get_shard_index(&self, key: &str) -> Option<usize> {
        self.hash_ring
            .get_node(key)
            .and_then(|shard_id| self.shards.iter().position(|s| s.id() == shard_id))
    }

    /// Get a value from the cache, routing to the correct shard.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.get_shard_index(key)
            .and_then(|idx| self.shards[idx].get(key))
    }

    /// Put a value into the cache, routing to the correct shard.
    pub fn put(&mut self, key: &str, value: &str) {
        if let Some(idx) = self.get_shard_index(key) {
            self.shards[idx].put(key, value);
        }
    }

    /// Get the shard ID that would own a given key.
    pub fn get_shard_for_key(&self, key: &str) -> Option<usize> {
        self.hash_ring.get_node(key)
    }

    /// Count how many keys from the given set would move if a shard were added.
    pub fn count_keys_that_move(&self, keys: &[&str], new_shard_id: usize) -> usize {
        // Count keys currently assigned to their shard
        let current: Vec<Option<usize>> = keys
            .iter()
            .map(|k| self.hash_ring.get_node(k))
            .collect();

        // Temporarily add the new shard and count
        let mut ring_clone = HashRing::new(self.hash_ring.replicas_per_node);
        // Rebuild the ring with all existing nodes
        let existing_nodes: Vec<usize> = self
            .shards
            .iter()
            .map(|s| s.id())
            .collect();
        for node in &existing_nodes {
            ring_clone.add_node(*node);
        }
        ring_clone.add_node(new_shard_id);

        let mut moved = 0;
        for (i, key) in keys.iter().enumerate() {
            let new_owner = ring_clone.get_node(key);
            if current[i] != new_owner {
                moved += 1;
            }
        }
        moved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_routing() {
        let cache = ShardedCache::new(5, 10);

        // The same key should always route to the same shard
        let key = "user:12345";
        let shard1 = cache.get_shard_for_key(key);
        let shard2 = cache.get_shard_for_key(key);
        let shard3 = cache.get_shard_for_key(key);

        assert_eq!(shard1, shard2);
        assert_eq!(shard2, shard3);
    }

    #[test]
    fn test_minimal_movement_on_shard_change() {
        let cache = ShardedCache::new(5, 15);

        let keys: Vec<&str> = (0..100)
            .map(|i| -> &str {
                // Leak the string to get 'static lifetime
                Box::leak(format!("key-{}", i).into_boxed_str())
            })
            .collect();

        // Count how many keys move when adding a new shard
        let moved = cache.count_keys_that_move(&keys, 100);

        // With consistent hashing, only ~1/(N+1) keys should move
        // N=5 shards, adding 1 => expect ~16 out of 100 keys to move
        // Allow up to 40% to account for hash distribution variance
        assert!(
            moved < 50,
            "Too many keys moved: {} out of 100",
            moved
        );
    }

    #[test]
    fn test_get_put_through_sharded_cache() {
        let mut cache = ShardedCache::new(4, 10);

        cache.put("name", "Alice");
        cache.put("age", "30");
        cache.put("city", "Portland");

        assert_eq!(cache.get("name"), Some("Alice"));
        assert_eq!(cache.get("age"), Some("30"));
        assert_eq!(cache.get("city"), Some("Portland"));
        assert_eq!(cache.get("missing"), None);
    }

    #[test]
    fn test_overwrite_value() {
        let mut cache = ShardedCache::new(3, 10);

        cache.put("key", "value1");
        assert_eq!(cache.get("key"), Some("value1"));

        cache.put("key", "value2");
        assert_eq!(cache.get("key"), Some("value2"));
    }

    #[test]
    fn test_shard_count() {
        let cache = ShardedCache::new(8, 10);
        // Should have 8 shards
        assert_eq!(cache.shards.len(), 8);
    }
}
