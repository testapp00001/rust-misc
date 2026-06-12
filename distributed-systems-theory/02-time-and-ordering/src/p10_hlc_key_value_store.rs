//! # Exercise: HLC Key-Value Store
//!
//! ## Theory
//!
//! A **causal-consistent** key-value store uses timestamps that preserve causality.
//! If operation A causally precedes operation B (A -> B), then all processes observe
//! A before B.
//!
//! Hybrid Logical Clocks (HLC) are ideal for this because they:
//! 1. Stay close to physical time (enabling debugging and TTL-based expiry)
//! 2. Preserve causality (ensuring causal consistency)
//! 3. Use bounded space (unlike vector clocks)
//!
//! ## Proof / Intuition
//!
//! Causal consistency guarantees:
//! - If a client performs write A then write B, all other clients see A before B
//! - Concurrent writes may be seen in different orders (conflict)
//!
//! With HLC:
//! - Each write gets an HLC timestamp
//! - When a client sends its state to another, it includes the HLC timestamps
//! - The receiving client advances its HLC to stay >= all received timestamps
//! - This ensures causal ordering is maintained across the system
//!
//! ## Implementation Task
//!
//! Implement `HLCKeyValueStore` with:
//! - Operations that use HLC timestamps
//! - A method to simulate causal dependency (send state)
//! - Conflict detection for concurrent updates
//!
//! ## Verification
//!
//! - Verify causal consistency (A -> B implies A observed before B)
//! - Verify HLC stays close to physical time
//! - Verify concurrent writes are detected

use std::collections::HashMap;

use super::p09_hybrid_logical_clocks::{HybridLogicalClock, HLCTimestamp};

/// The result of a write operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteResult {
    /// Write succeeded with the given timestamp.
    Success(HLCTimestamp),
    /// Write created a conflict (concurrent updates).
    Conflict(HLCTimestamp),
}

/// A value with its HLC timestamp.
#[derive(Debug, Clone)]
pub struct VersionedEntry {
    pub value: String,
    pub timestamp: HLCTimestamp,
}

/// A key-value store using HLC for causal consistency.
pub struct HLCKeyValueStore {
    /// Node identifier.
    node_id: usize,
    /// The hybrid logical clock for this node.
    clock: HybridLogicalClock,
    /// The key-value data with version information.
    data: HashMap<String, Vec<VersionedEntry>>,
}

impl HLCKeyValueStore {
    /// Create a new HLC key-value store.
    pub fn new(node_id: usize, initial_physical_time: u64) -> Self {
        Self {
            node_id,
            clock: HybridLogicalClock::new(node_id, initial_physical_time),
            data: HashMap::new(),
        }
    }

    /// Put a value for a key.
    ///
    /// The HLC timestamp is advanced for this operation.
    pub fn put(&mut self, key: &str, value: String, physical_time: u64) -> WriteResult {
        let ts = self.clock.local_event(physical_time);

        let entries = self.data.entry(key.to_string()).or_insert_with(Vec::new);

        // Check for conflicts with existing entries
        let has_concurrent = entries.iter().any(|e| {
            e.timestamp.pt == ts.pt && e.timestamp.lc == ts.lc
        });

        entries.push(VersionedEntry {
            value,
            timestamp: ts.clone(),
        });

        if has_concurrent {
            WriteResult::Conflict(ts)
        } else {
            WriteResult::Success(ts)
        }
    }

    /// Get the latest value for a key (based on HLC ordering).
    pub fn get(&self, key: &str) -> Option<&VersionedEntry> {
        self.data.get(key).and_then(|entries| {
            entries
                .iter()
                .max_by(|a, b| {
                    if a.timestamp.precedes(&b.timestamp) {
                        std::cmp::Ordering::Less
                    } else if b.timestamp.precedes(&a.timestamp) {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Equal
                    }
                })
        })
    }

    /// Get all versions for a key (useful for conflict detection).
    pub fn get_all_versions(&self, key: &str) -> Option<&[VersionedEntry]> {
        self.data.get(key).map(|v| v.as_slice())
    }

    /// Simulate receiving state from another node.
    ///
    /// This advances the local HLC to be >= all received timestamps,
    /// maintaining causal consistency.
    pub fn receive_state(
        &mut self,
        key: &str,
        value: &str,
        remote_ts: HLCTimestamp,
        physical_time: u64,
    ) {
        // Advance HLC as if receiving a message
        self.clock
            .receive(physical_time, remote_ts.pt, remote_ts.lc);

        let entries = self.data.entry(key.to_string()).or_insert_with(Vec::new);

        // Check if we already have this exact version
        let already_has = entries
            .iter()
            .any(|e| e.timestamp.pt == remote_ts.pt && e.timestamp.lc == remote_ts.lc);

        if !already_has {
            entries.push(VersionedEntry {
                value: value.to_string(),
                timestamp: remote_ts,
            });
        }
    }

    /// Check if a key has conflicting versions.
    ///
    /// Since HLC produces a total order, "conflict" here means two entries
    /// have the same timestamp -- indicating a collision where two nodes wrote
    /// at the same logical instant.
    pub fn has_conflict(&self, key: &str) -> bool {
        self.data
            .get(key)
            .map(|entries| {
                // Conflict exists if multiple entries share the same timestamp
                for i in 0..entries.len() {
                    for j in (i + 1)..entries.len() {
                        let a = &entries[i].timestamp;
                        let b = &entries[j].timestamp;
                        if a.pt == b.pt && a.lc == b.lc {
                            return true;
                        }
                    }
                }
                false
            })
            .unwrap_or(false)
    }

    /// Get the current HLC timestamp.
    pub fn current_timestamp(&self) -> HLCTimestamp {
        self.clock.timestamp()
    }

    /// Get the node ID.
    pub fn node_id(&self) -> usize {
        self.node_id
    }

    /// Calculate how far the HLC is from physical time.
    pub fn offset_from_physical(&self, physical_time: u64) -> i64 {
        self.clock.offset_from_physical(physical_time)
    }

    /// Get all keys.
    pub fn keys(&self) -> Vec<&str> {
        self.data.keys().map(|s| s.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn causal_consistency_basic() {
        let mut store = HLCKeyValueStore::new(0, 1000);

        // Write A at time 1000
        let result_a = store.put("key", "value_A".to_string(), 1000);
        let ts_a = match result_a {
            WriteResult::Success(ts) => ts,
            _ => panic!("expected success"),
        };

        // Write B at time 1001 (causal: A happened before B)
        let result_b = store.put("key", "value_B".to_string(), 1001);
        let ts_b = match result_b {
            WriteResult::Success(ts) => ts,
            _ => panic!("expected success"),
        };

        // A should precede B
        assert!(ts_a.precedes(&ts_b));

        // Latest value should be B
        let latest = store.get("key").unwrap();
        assert_eq!(latest.value, "value_B");
    }

    #[test]
    fn hlc_stays_close_to_physical_time() {
        let mut store = HLCKeyValueStore::new(0, 1000);

        // Do many operations at the same physical time
        for i in 0..50 {
            store.put(&format!("key_{i}"), format!("val_{i}"), 1000);
        }

        // HLC should be close to physical time
        let offset = store.offset_from_physical(1000);
        assert!(
            offset.abs() < 100,
            "HLC should stay close to physical time, offset is {offset}"
        );
    }

    #[test]
    fn concurrent_writes_detected() {
        let mut store = HLCKeyValueStore::new(0, 1000);

        // HLC produces a total order, so "concurrent" writes are those
        // with identical timestamps (both pt and lc equal). This happens
        // when two nodes have synchronized physical clocks and both do a
        // local event at the same instant.
        let entries = store.data.entry("key".to_string()).or_insert_with(Vec::new);
        entries.push(VersionedEntry {
            value: "A".to_string(),
            timestamp: HLCTimestamp::new(1000, 5),
        });
        entries.push(VersionedEntry {
            value: "B".to_string(),
            timestamp: HLCTimestamp::new(1000, 5),
        });

        assert!(store.has_conflict("key"));
    }

    #[test]
    fn receive_state_advances_hlc() {
        let mut store = HLCKeyValueStore::new(0, 1000);

        // Simulate receiving state from another node with a higher timestamp
        let remote_ts = HLCTimestamp::new(5000, 3);
        store.receive_state("key", "remote_value", remote_ts, 5000);

        // Local HLC should have advanced
        let current = store.current_timestamp();
        assert!(current.pt >= 5000 || current.lc > 0);
    }

    #[test]
    fn multiple_versions_tracked() {
        let mut store = HLCKeyValueStore::new(0, 1000);

        store.put("key", "v1".to_string(), 1000);
        store.put("key", "v2".to_string(), 1001);

        let versions = store.get_all_versions("key").unwrap();
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn no_conflict_for_causal_writes() {
        let mut store = HLCKeyValueStore::new(0, 1000);

        // Causal writes (different physical times)
        store.put("key", "A".to_string(), 1000);
        store.put("key", "B".to_string(), 1001);
        store.put("key", "C".to_string(), 1002);

        assert!(!store.has_conflict("key"));
    }
}
