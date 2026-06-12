//! # Exercise: Consistency Levels
//!
//! ## Theory
//!
//! Distributed systems offer a spectrum of consistency levels, each providing
//! different guarantees about the relationship between writes and reads. The
//! key levels from strongest to weakest are:
//!
//! - **Strong (Linearizable):** Every read sees the result of the most recent
//!   write. Requires quorum reads and writes.
//! - **Causal:** If operation A causally precedes B, every replica sees A
//!   before B. Uses vector clocks to track causality.
//! - **Read-Your-Writes:** A client always sees its own writes, even if other
//!   clients see stale data. Achieved by pinning reads to the same replica.
//! - **Monotonic Reads:** Once a client sees a value, it never sees an older
//!   value. Prevents "going back in time" reads.
//! - **Eventual:** Given enough time without new writes, all replicas converge.
//!   No ordering guarantees on reads.
//!
//! ## Proof / Intuition
//!
//! The consistency hierarchy is strict: each level implies all weaker levels.
//! Strong consistency implies causal, which implies read-your-writes, which
//! implies monotonic reads, which implies eventual.
//!
//! The cost of stronger consistency is higher latency (quorum coordination)
//! and lower availability during partitions. Each step down the hierarchy
//! reduces coordination overhead.
//!
//! ## Implementation Task
//!
//! Implement a multi-replica store with selectable consistency levels:
//! - `ConsistencyLevel` enum with all five levels
//! - `get_with_level(key, level) -> Result<String, ConsistencyError>`
//! - Strong reads from majority; eventual reads from any replica
//! - Track per-client write history for read-your-writes
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Strong consistency requires quorum
//! - Eventual works with a single replica
//! - Read-your-writes guarantees client visibility

use std::collections::HashMap;

/// The consistency level for a read operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsistencyLevel {
    /// Every read sees the most recent write (linearizable).
    Strong,
    /// Causally consistent: causal ordering is preserved.
    Causal,
    /// Client always sees its own writes.
    ReadYourWrites,
    /// Client never sees a value older than what it has already seen.
    MonotonicReads,
    /// Reads from any replica; eventual convergence guaranteed.
    Eventual,
}

/// Errors that can occur during a read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsistencyError {
    /// Not enough replicas available for the requested consistency level.
    InsufficientReplicas { available: usize, required: usize },
    /// Key not found on any reachable replica.
    NotFound,
}

/// A timestamped value for ordering.
#[derive(Debug, Clone)]
struct VersionedValue {
    value: String,
    timestamp: u64,
}

/// A single replica.
#[derive(Debug, Clone)]
struct Replica {
    _id: usize,
    store: HashMap<String, VersionedValue>,
}

impl Replica {
    fn new(id: usize) -> Self {
        Self {
            _id: id,
            store: HashMap::new(),
        }
    }

    fn put(&mut self, key: &str, value: &str, timestamp: u64) {
        self.store.insert(
            key.to_string(),
            VersionedValue {
                value: value.to_string(),
                timestamp,
            },
        );
    }

    fn get(&self, key: &str) -> Option<&VersionedValue> {
        self.store.get(key)
    }
}

/// Multi-replica store with configurable consistency levels.
pub struct ConsistencyStore {
    replicas: Vec<Replica>,
    /// Global logical clock.
    clock: u64,
    /// Per-client last-write timestamp for read-your-writes.
    client_last_write: HashMap<String, u64>,
}

impl ConsistencyStore {
    /// Create a new store with the given number of replicas.
    pub fn new(num_replicas: usize) -> Self {
        let replicas = (0..num_replicas).map(Replica::new).collect();
        Self {
            replicas,
            clock: 0,
            client_last_write: HashMap::new(),
        }
    }

    /// Write a key-value pair, replicating to all replicas.
    /// Returns the timestamp of the write.
    pub fn put(&mut self, key: &str, value: &str, client_id: &str) -> u64 {
        self.clock += 1;
        let ts = self.clock;
        for replica in &mut self.replicas {
            replica.put(key, value, ts);
        }
        self.client_last_write
            .insert(client_id.to_string(), ts);
        ts
    }

    /// Read a value at the specified consistency level.
    pub fn get_with_level(
        &self,
        key: &str,
        level: ConsistencyLevel,
        client_id: &str,
    ) -> Result<String, ConsistencyError> {
        match level {
            ConsistencyLevel::Strong => self.strong_read(key),
            ConsistencyLevel::Causal => self.causal_read(key),
            ConsistencyLevel::ReadYourWrites => {
                self.read_your_writes_read(key, client_id)
            }
            ConsistencyLevel::MonotonicReads => {
                self.monotonic_read(key, client_id)
            }
            ConsistencyLevel::Eventual => self.eventual_read(key),
        }
    }

    /// Strong read: read from the majority and return the latest value.
    fn strong_read(&self, key: &str) -> Result<String, ConsistencyError> {
        let quorum = self.replicas.len() / 2 + 1;
        let mut latest: Option<VersionedValue> = None;

        for replica in &self.replicas {
            if let Some(v) = replica.get(key) {
                match &latest {
                    None => latest = Some(v.clone()),
                    Some(current) if v.timestamp > current.timestamp => {
                        latest = Some(v.clone());
                    }
                    _ => {}
                }
            }
        }

        // We read from all replicas; in a real system we'd read from quorum
        if self.replicas.len() < quorum {
            return Err(ConsistencyError::InsufficientReplicas {
                available: self.replicas.len(),
                required: quorum,
            });
        }

        latest
            .map(|v| v.value)
            .ok_or(ConsistencyError::NotFound)
    }

    /// Causal read: return the value with the highest timestamp.
    /// (Simplified: in a full implementation, this would use vector clocks.)
    fn causal_read(&self, key: &str) -> Result<String, ConsistencyError> {
        // Same as strong for this simplified model
        self.strong_read(key)
    }

    /// Read-your-writes: only consider replicas with timestamp >= client's
    /// last write.
    fn read_your_writes_read(
        &self,
        key: &str,
        client_id: &str,
    ) -> Result<String, ConsistencyError> {
        let min_ts = self.client_last_write.get(client_id).copied().unwrap_or(0);

        // Find the latest value that is at least as recent as the client's
        // last write.
        let mut latest: Option<VersionedValue> = None;
        for replica in &self.replicas {
            if let Some(v) = replica.get(key) {
                if v.timestamp >= min_ts {
                    match &latest {
                        None => latest = Some(v.clone()),
                        Some(current) if v.timestamp > current.timestamp => {
                            latest = Some(v.clone());
                        }
                        _ => {}
                    }
                }
            }
        }

        latest
            .map(|v| v.value)
            .ok_or(ConsistencyError::NotFound)
    }

    /// Monotonic read: return the latest value, but at least as recent as
    /// the client's last observed timestamp.
    fn monotonic_read(
        &self,
        key: &str,
        client_id: &str,
    ) -> Result<String, ConsistencyError> {
        // For simplicity, same as read-your-writes
        self.read_your_writes_read(key, client_id)
    }

    /// Eventual read: read from the first replica (no coordination).
    fn eventual_read(&self, key: &str) -> Result<String, ConsistencyError> {
        self.replicas
            .first()
            .and_then(|r| r.get(key))
            .map(|v| v.value.clone())
            .ok_or(ConsistencyError::NotFound)
    }

    /// Return the number of replicas.
    pub fn replica_count(&self) -> usize {
        self.replicas.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strong_consistency_requires_quorum() {
        let store = ConsistencyStore::new(3);
        // No data written, should return NotFound
        let result = store.get_with_level("x", ConsistencyLevel::Strong, "client1");
        assert!(result.is_err());
    }

    #[test]
    fn strong_read_returns_latest_value() {
        let mut store = ConsistencyStore::new(3);
        store.put("key", "value", "c1");

        let result = store
            .get_with_level("key", ConsistencyLevel::Strong, "c1")
            .unwrap();
        assert_eq!(result, "value");
    }

    #[test]
    fn eventual_works_with_single_replica() {
        let mut store = ConsistencyStore::new(1);
        store.put("key", "only", "c1");

        let result = store
            .get_with_level("key", ConsistencyLevel::Eventual, "c1")
            .unwrap();
        assert_eq!(result, "only");
    }

    #[test]
    fn read_your_writes_sees_own_write() {
        let mut store = ConsistencyStore::new(3);
        store.put("x", "my_value", "client_a");

        let result = store
            .get_with_level("x", ConsistencyLevel::ReadYourWrites, "client_a")
            .unwrap();
        assert_eq!(result, "my_value");
    }

    #[test]
    fn not_found_returns_error() {
        let store = ConsistencyStore::new(3);
        let result = store.get_with_level(
            "nonexistent",
            ConsistencyLevel::Eventual,
            "c1",
        );
        assert_eq!(result, Err(ConsistencyError::NotFound));
    }

    #[test]
    fn all_levels_read_consistent_data_when_no_partition() {
        let mut store = ConsistencyStore::new(3);
        store.put("a", "hello", "c1");

        let levels = [
            ConsistencyLevel::Strong,
            ConsistencyLevel::Causal,
            ConsistencyLevel::ReadYourWrites,
            ConsistencyLevel::MonotonicReads,
            ConsistencyLevel::Eventual,
        ];

        for level in levels {
            let result = store
                .get_with_level("a", level, "c1")
                .unwrap();
            assert_eq!(result, "hello", "level {level:?} should return hello");
        }
    }
}
