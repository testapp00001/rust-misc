//! # Exercise: Version Vectors for Key-Value Stores
//!
//! ## Theory
//!
//! **Version Vectors** (Parker et al. 1983, Peterson & Leitao 1993) are vector
//! clocks applied to individual keys in a distributed data store. They track the
//! causal history of updates to each key, enabling conflict detection.
//!
//! Unlike vector clocks (which track process-level causality), version vectors
//! track **key-level** causality. Each key has its own version vector that records
//! which client's updates are included in the current value.
//!
//! ## Proof / Intuition
//!
//! When two clients concurrently update the same key:
//! - Client A puts (key, value_A) with version vector [1, 0]
//! - Client B puts (key, value_B) with version vector [0, 1]
//!
//! Neither version vector precedes the other -- this indicates a **conflict**.
//! The store must either:
//! 1. Keep both values (multi-version)
//! 2. Apply a conflict resolution strategy (last-writer-wins, merge, etc.)
//!
//! ## Implementation Task
//!
//! Implement:
//! - `VersionVector` struct
//! - `KVStore` with put, get, and merge operations
//! - Conflict detection via version vector comparison
//!
//! ## Verification
//!
//! - Concurrent updates create conflicts
//! - Causal updates don't conflict
//! - Merge correctly identifies conflicts

use std::collections::HashMap;

/// A version vector tracking causal history per client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionVector {
    /// Maps client_id -> version counter.
    versions: HashMap<usize, u64>,
}

impl VersionVector {
    /// Create a new empty version vector.
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
        }
    }

    /// Increment the version for a specific client.
    pub fn increment(&mut self, client_id: usize) {
        *self.versions.entry(client_id).or_insert(0) += 1;
    }

    /// Get the version for a specific client.
    pub fn get(&self, client_id: usize) -> u64 {
        self.versions.get(&client_id).copied().unwrap_or(0)
    }

    /// Merge with another version vector (element-wise max).
    pub fn merge(&mut self, other: &VersionVector) {
        for (&client_id, &version) in &other.versions {
            let entry = self.versions.entry(client_id).or_insert(0);
            *entry = (*entry).max(version);
        }
    }

    /// Check if this version vector strictly precedes another (component-wise <=
    /// with at least one strict <).
    pub fn precedes(&self, other: &VersionVector) -> bool {
        let mut strictly_less = false;

        // Check all clients known to either vector
        let all_clients: std::collections::HashSet<usize> = self
            .versions
            .keys()
            .chain(other.versions.keys())
            .copied()
            .collect();

        for client_id in all_clients {
            let v1 = self.get(client_id);
            let v2 = other.get(client_id);
            if v1 > v2 {
                return false;
            }
            if v1 < v2 {
                strictly_less = true;
            }
        }

        strictly_less
    }

    /// Check if two version vectors are equal.
    pub fn equals(&self, other: &VersionVector) -> bool {
        self.versions == other.versions
    }

    /// Check if two version vectors are concurrent (neither precedes the other).
    pub fn is_concurrent_with(&self, other: &VersionVector) -> bool {
        !self.precedes(other) && !other.precedes(self) && !self.equals(other)
    }

    /// Return the number of clients tracked.
    pub fn num_clients(&self) -> usize {
        self.versions.len()
    }
}

impl Default for VersionVector {
    fn default() -> Self {
        Self::new()
    }
}

/// A value with its version vector.
#[derive(Debug, Clone)]
pub struct VersionedValue<V> {
    pub value: V,
    pub version: VersionVector,
}

/// A key-value store using version vectors for conflict detection.
pub struct KVStore<V: Clone> {
    data: HashMap<String, VersionedValue<V>>,
}

impl<V: Clone + std::fmt::Debug> KVStore<V> {
    /// Create a new empty KV store.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Put a value for a key from a specific client.
    ///
    /// The version vector is updated to include the client's new version.
    pub fn put(&mut self, key: &str, value: V, client_id: usize) {
        let entry = self
            .data
            .entry(key.to_string())
            .or_insert_with(|| VersionedValue {
                value: value.clone(),
                version: VersionVector::new(),
            });

        entry.version.increment(client_id);
        entry.value = value;
    }

    /// Get a value and its version vector for a key.
    pub fn get(&self, key: &str) -> Option<&VersionedValue<V>> {
        self.data.get(key)
    }

    /// Merge a remote update into the store.
    ///
    /// Returns `Ok(true)` if the update was applied without conflict.
    /// Returns `Ok(false)` if a conflict was detected and both versions are kept.
    pub fn merge(
        &mut self,
        key: &str,
        remote_value: V,
        remote_vv: VersionVector,
    ) -> Result<bool, String> {
        match self.data.get(key) {
            Some(local) => {
                if local.version.precedes(&remote_vv) {
                    // Local is older -- apply remote
                    self.data.insert(
                        key.to_string(),
                        VersionedValue {
                            value: remote_value,
                            version: remote_vv,
                        },
                    );
                    Ok(true)
                } else if remote_vv.precedes(&local.version) {
                    // Remote is older -- keep local
                    Ok(true)
                } else if local.version.equals(&remote_vv) {
                    // Same version -- no conflict
                    Ok(true)
                } else {
                    // Concurrent -- conflict!
                    Err(format!(
                        "Conflict on key '{key}': local={:?}, remote={:?}",
                        local.version, remote_vv
                    ))
                }
            }
            None => {
                // Key doesn't exist yet -- apply
                self.data.insert(
                    key.to_string(),
                    VersionedValue {
                        value: remote_value,
                        version: remote_vv,
                    },
                );
                Ok(true)
            }
        }
    }

    /// Check if a key has a conflict with a given version vector.
    pub fn has_conflict(&self, key: &str, remote_vv: &VersionVector) -> bool {
        match self.data.get(key) {
            Some(local) => local.version.is_concurrent_with(remote_vv),
            None => false,
        }
    }

    /// Get all keys in the store.
    pub fn keys(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }

    /// Return the number of entries.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Return true if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<V: Clone + std::fmt::Debug> Default for KVStore<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_vector_basic_operations() {
        let mut vv = VersionVector::new();
        assert_eq!(vv.get(0), 0);

        vv.increment(0);
        assert_eq!(vv.get(0), 1);

        vv.increment(0);
        assert_eq!(vv.get(0), 2);

        vv.increment(1);
        assert_eq!(vv.get(1), 1);
    }

    #[test]
    fn concurrent_updates_create_conflicts() {
        let mut store = KVStore::new();

        // Client 0 puts value A
        store.put("key", "value_A", 0);

        // Client 1 independently puts value B (concurrent)
        let mut vv1 = VersionVector::new();
        vv1.increment(1);

        let result = store.merge("key", "value_B", vv1);
        assert!(result.is_err(), "concurrent updates should conflict");
    }

    #[test]
    fn causal_updates_dont_conflict() {
        let mut store = KVStore::new();

        // Client 0 puts value A
        store.put("key", "value_A", 0);

        // Client 0 puts value B (causal -- same client)
        let _result = store.put("key", "value_B", 0);

        // The store now has value_B with version {0: 2}
        let entry = store.get("key").unwrap();
        assert_eq!(entry.value, "value_B");
        assert_eq!(entry.version.get(0), 2);
    }

    #[test]
    fn merge_precedes_updates_value() {
        let mut store = KVStore::new();

        // Set initial value
        store.put("key", "old", 0);

        // Remote has a newer version (includes client 0's update + more)
        let mut remote_vv = VersionVector::new();
        remote_vv.increment(0);
        remote_vv.increment(1);

        let result = store.merge("key", "new", remote_vv);
        assert!(result.unwrap(), "remote is newer, should be applied");

        let entry = store.get("key").unwrap();
        assert_eq!(entry.value, "new");
    }

    #[test]
    fn merge_with_older_version_keeps_local() {
        let mut store = KVStore::new();

        store.put("key", "current", 0);
        store.put("key", "latest", 0);

        // Remote has older version
        let mut remote_vv = VersionVector::new();
        remote_vv.increment(0);

        let result = store.merge("key", "old", remote_vv);
        assert!(result.unwrap(), "remote is older, local is kept");

        let entry = store.get("key").unwrap();
        assert_eq!(entry.value, "latest");
    }

    #[test]
    fn version_vector_precedes() {
        let mut vv1 = VersionVector::new();
        vv1.increment(0);
        vv1.increment(0);

        let mut vv2 = VersionVector::new();
        vv2.increment(0);
        vv2.increment(0);
        vv2.increment(1);

        assert!(vv1.precedes(&vv2));
        assert!(!vv2.precedes(&vv1));
    }
}
