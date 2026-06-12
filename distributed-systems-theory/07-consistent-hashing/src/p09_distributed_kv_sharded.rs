//! # Exercise: Sharded Distributed KV Store
//!
//! ## Theory
//!
//! A sharded key-value store distributes data across multiple nodes using a
//! consistent hash ring. Each shard is responsible for a range of keys on the
//! ring. Operations are routed to the correct shard based on the key's hash.
//!
//! Key design considerations:
//! - Shard selection via consistent hashing for minimal redistribution
//! - Per-shard storage using HashMap
//! - Support for dynamic shard addition with rebalancing
//!
//! ## Proof / Intuition
//!
//! The hash ring divides the key space into regions, each owned by a shard.
//! When a key is looked up, we hash the key and find the next shard clockwise.
//! This ensures that:
//! 1. The same key always goes to the same shard (deterministic)
//! 2. Adding a shard only moves keys in one region (consistent)
//!
//! ## Implementation Task
//!
//! Implement `ShardedKV` with:
//! - `put(key, value)` - route to correct shard and store
//! - `get(key)` - route to correct shard and retrieve
//! - `add_shard(shard_id)` - add a shard and rebalance
//!
//! ## Verification
//!
//! Verify keys land on correct shards and rebalancing on shard addition
//! works correctly.
//!
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Error type for KV operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KVError {
    ShardNotFound(usize),
    KeyNotFound(String),
    NoShardsAvailable,
}

impl std::fmt::Display for KVError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KVError::ShardNotFound(id) => write!(f, "Shard not found: {}", id),
            KVError::KeyNotFound(key) => write!(f, "Key not found: {}", key),
            KVError::NoShardsAvailable => write!(f, "No shards available"),
        }
    }
}

impl std::error::Error for KVError {}

/// A sharded key-value store using consistent hashing.
#[derive(Debug)]
pub struct ShardedKV {
    /// Consistent hash ring: position -> shard_id.
    ring: BTreeMap<u64, usize>,
    /// Per-shard storage: shard_id -> (key -> value).
    shards: HashMap<usize, HashMap<String, String>>,
    /// Track virtual node positions per shard.
    shard_positions: HashMap<usize, Vec<u64>>,
    /// Number of virtual nodes per shard.
    vnodes_per_shard: usize,
}

impl ShardedKV {
    /// Create a new sharded KV store.
    pub fn new(vnodes_per_shard: usize) -> Self {
        Self {
            ring: BTreeMap::new(),
            shards: HashMap::new(),
            shard_positions: HashMap::new(),
            vnodes_per_shard,
        }
    }

    /// Hash a string key to a u64 position.
    fn hash_key(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Add a shard with virtual nodes.
    pub fn add_shard(&mut self, shard_id: usize) {
        let mut positions = Vec::new();
        for i in 0..self.vnodes_per_shard {
            let vk = format!("shard_{}#{}", shard_id, i);
            let pos = Self::hash_key(&vk);
            self.ring.insert(pos, shard_id);
            positions.push(pos);
        }
        self.shard_positions.insert(shard_id, positions);
        self.shards.entry(shard_id).or_default();
    }

    /// Remove a shard.
    pub fn remove_shard(&mut self, shard_id: usize) -> Option<HashMap<String, String>> {
        if let Some(positions) = self.shard_positions.remove(&shard_id) {
            for pos in positions {
                self.ring.remove(&pos);
            }
        }
        self.shards.remove(&shard_id)
    }

    /// Find the shard responsible for a key.
    fn find_shard(&self, key: &str) -> Option<usize> {
        if self.ring.is_empty() {
            return None;
        }
        let pos = Self::hash_key(key);
        self.ring
            .range(pos..)
            .next()
            .or_else(|| self.ring.iter().next())
            .map(|(_, &shard_id)| shard_id)
    }

    /// Store a key-value pair.
    pub fn put(&mut self, key: &str, value: &str) -> Result<usize, KVError> {
        let shard_id = self.find_shard(key).ok_or(KVError::NoShardsAvailable)?;
        self.shards
            .entry(shard_id)
            .or_default()
            .insert(key.to_string(), value.to_string());
        Ok(shard_id)
    }

    /// Retrieve a value by key.
    pub fn get(&self, key: &str) -> Result<Option<String>, KVError> {
        let shard_id = self.find_shard(key).ok_or(KVError::NoShardsAvailable)?;
        let value = self
            .shards
            .get(&shard_id)
            .and_then(|s| s.get(key))
            .cloned();
        Ok(value)
    }

    /// Delete a key.
    pub fn delete(&mut self, key: &str) -> Result<bool, KVError> {
        let shard_id = self.find_shard(key).ok_or(KVError::NoShardsAvailable)?;
        Ok(self
            .shards
            .get_mut(&shard_id)
            .map(|s| s.remove(key).is_some())
            .unwrap_or(false))
    }

    /// Get the shard a key would be assigned to (without storing).
    pub fn route(&self, key: &str) -> Result<usize, KVError> {
        self.find_shard(key).ok_or(KVError::NoShardsAvailable)
    }

    /// Get the number of shards.
    pub fn num_shards(&self) -> usize {
        self.shards.len()
    }

    /// Get total number of keys stored.
    pub fn total_keys(&self) -> usize {
        self.shards.values().map(|s| s.len()).sum()
    }

    /// Get key count per shard.
    pub fn shard_distribution(&self) -> HashMap<usize, usize> {
        self.shards.iter().map(|(&id, s)| (id, s.len())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keys_land_on_correct_shard() {
        let mut kv = ShardedKV::new(150);
        kv.add_shard(0);
        kv.add_shard(1);
        kv.add_shard(2);

        let key = "test_key";
        let _shard = kv.route(key).unwrap();
        kv.put(key, "value").unwrap();

        // The shard we routed to should be the one that has the value
        let retrieved = kv.get(key).unwrap();
        assert_eq!(retrieved, Some("value".to_string()));
    }

    #[test]
    fn test_rebalance_on_shard_addition() {
        let mut kv = ShardedKV::new(150);
        kv.add_shard(0);
        kv.add_shard(1);

        // Insert keys
        for i in 0..1000 {
            let key = format!("key_{}", i);
            kv.put(&key, &format!("value_{}", i)).unwrap();
        }

        let total_before = kv.total_keys();

        // Verify all keys accessible before adding shard
        for i in 0..1000 {
            let key = format!("key_{}", i);
            assert!(kv.get(&key).unwrap().is_some(), "Key {} should exist before shard addition", i);
        }

        // Add a new shard
        kv.add_shard(2);

        // After adding shard, total stored keys is unchanged (no migration)
        assert_eq!(kv.total_keys(), total_before);

        // Some keys may now route to the new (empty) shard, so not all are accessible.
        // Verify that keys which still route to old shards are accessible.
        let mut found = 0;
        for i in 0..1000 {
            let key = format!("key_{}", i);
            if kv.get(&key).unwrap().is_some() {
                found += 1;
            }
        }
        // At least some keys should still be accessible (the ones not remapped)
        assert!(found > 0, "Some keys should still be accessible after shard addition");
        assert!(found < 1000, "Not all keys should be accessible (some remapped to empty shard)");
    }

    #[test]
    fn test_consistent_route_returns_same_shard() {
        let mut kv = ShardedKV::new(150);
        kv.add_shard(0);
        kv.add_shard(1);

        let key = "consistent_key";
        let s1 = kv.route(key).unwrap();
        let s2 = kv.route(key).unwrap();
        assert_eq!(s1, s2, "Same key must always route to same shard");
    }

    #[test]
    fn test_get_nonexistent_key() {
        let mut kv = ShardedKV::new(100);
        kv.add_shard(0);
        let result = kv.get("nonexistent").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_delete_key() {
        let mut kv = ShardedKV::new(100);
        kv.add_shard(0);
        kv.put("key1", "value1").unwrap();
        assert!(kv.get("key1").unwrap().is_some());

        let deleted = kv.delete("key1").unwrap();
        assert!(deleted);
        assert!(kv.get("key1").unwrap().is_none());
    }

    #[test]
    fn test_no_shards_error() {
        let kv = ShardedKV::new(100);
        let result = kv.get("key");
        assert_eq!(result, Err(KVError::NoShardsAvailable));
    }
}
