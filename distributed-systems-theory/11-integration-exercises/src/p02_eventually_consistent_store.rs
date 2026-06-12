//! # Exercise: Eventually Consistent Store
//!
//! ## Theory
//!
//! Eventually consistent stores guarantee that if no new updates are made,
//! all replicas will eventually converge to the same value. This is achieved
//! through:
//!
//! - **Vector clocks**: Track causality between events. Each node maintains
//!   a counter for itself and every other node it knows about.
//! - **Anti-entropy**: Periodic synchronization between replicas using
//!   last-writer-wins (LWW) or other merge strategies.
//!
//! Vector clocks allow us to determine if one event happened-before another,
//! or if they are concurrent. When events are concurrent, we need a
//! deterministic conflict resolution strategy.
//!
//! ## Proof / Intuition
//!
//! Vector clock properties:
//! 1. If VC(A) < VC(B) component-wise, then A happened-before B
//! 2. If neither VC(A) <= VC(B) nor VC(B) <= VC(A), then A and B are concurrent
//!
//! When merging concurrent writes, LWW (last-writer-wins) uses wall clock
//! time as a tiebreaker. This is simple but can lose writes - it trades
//! availability for simplicity.
//!
//! Anti-entropy works by exchanging entire replica states and computing
//! the merge. With LWW, the merge is commutative, associative, and
//! idempotent - ensuring convergence regardless of sync order.
//!
//! ## Implementation Task
//!
//! 1. Implement `VectorClock` with increment, merge, happens_before, concurrent_with
//! 2. Implement `VersionedValue` to pair values with their vector clocks
//! 3. Implement `Replica` with local writes and anti-entropy sync
//! 4. Ensure writes on any replica are eventually visible everywhere
//! 5. Handle concurrent writes using vector clock comparison
//!
//! ## Verification
//!
//! - Test that writes succeed on any replica
//! - Test eventual convergence after syncs
//! - Test concurrent write handling (LWW resolution)

use std::collections::HashMap;

/// Node identifier type.
pub type NodeId = u64;

/// A vector clock tracking causality across nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VectorClock {
    /// Map from node ID to logical counter.
    clock: HashMap<u64, u64>,
}

impl VectorClock {
    /// Create a new empty vector clock.
    pub fn new() -> Self {
        Self {
            clock: HashMap::new(),
        }
    }

    /// Increment the counter for a given node.
    pub fn increment(&mut self, node_id: u64) {
        let counter = self.clock.entry(node_id).or_insert(0);
        *counter += 1;
    }

    /// Merge another vector clock into this one (element-wise maximum).
    pub fn merge(&mut self, other: &VectorClock) {
        for (&node_id, &counter) in &other.clock {
            let entry = self.clock.entry(node_id).or_insert(0);
            if counter > *entry {
                *entry = counter;
            }
        }
    }

    /// Check if this vector clock happened-before another.
    /// Returns true if all components of self <= other, and at least one is strictly less.
    pub fn happened_before(&self, other: &VectorClock) -> bool {
        let mut strictly_less = false;

        // Check all keys in self
        for (&node_id, &counter) in &self.clock {
            let other_counter = other.clock.get(&node_id).copied().unwrap_or(0);
            if counter > other_counter {
                return false;
            }
            if counter < other_counter {
                strictly_less = true;
            }
        }

        // Check keys only in other (self has 0 for those)
        for (&node_id, &counter) in &other.clock {
            if !self.clock.contains_key(&node_id) && counter > 0 {
                strictly_less = true;
            }
        }

        strictly_less
    }

    /// Check if this vector clock is concurrent with another.
    /// Two clocks are concurrent if neither happened-before the other.
    pub fn concurrent_with(&self, other: &VectorClock) -> bool {
        !self.happened_before(other) && !other.happened_before(self)
    }

    /// Get the counter for a specific node.
    pub fn get(&self, node_id: u64) -> u64 {
        self.clock.get(&node_id).copied().unwrap_or(0)
    }

    /// Get all node IDs in this vector clock.
    pub fn nodes(&self) -> Vec<u64> {
        self.clock.keys().copied().collect()
    }

    /// Check if this vector clock is empty.
    pub fn is_empty(&self) -> bool {
        self.clock.is_empty()
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

/// A value with an associated vector clock for conflict detection.
#[derive(Debug, Clone)]
pub struct VersionedValue {
    /// The actual value.
    pub value: String,
    /// The vector clock when this value was written.
    pub clock: VectorClock,
}

impl VersionedValue {
    /// Create a new versioned value.
    pub fn new(value: String, clock: VectorClock) -> Self {
        Self { value, clock }
    }

    /// Check if this value happened-before another.
    pub fn happened_before(&self, other: &VersionedValue) -> bool {
        self.clock.happened_before(&other.clock)
    }

    /// Check if this value is concurrent with another.
    pub fn is_concurrent_with(&self, other: &VersionedValue) -> bool {
        self.clock.concurrent_with(&other.clock)
    }
}

/// A replica in the eventually consistent store.
pub struct Replica {
    /// Unique ID of this replica.
    pub id: NodeId,
    /// Local data store: key -> versioned value.
    pub data: HashMap<String, VersionedValue>,
    /// Peer replicas for synchronization.
    pub replicas: Vec<NodeId>,
}

impl Replica {
    /// Create a new replica.
    pub fn new(id: NodeId) -> Self {
        Self {
            id,
            data: HashMap::new(),
            replicas: Vec::new(),
        }
    }

    /// Write a value locally, updating the vector clock.
    pub fn write(&mut self, key: &str, value: &str) {
        let existing = self.data.get(key);
        let mut clock = existing
            .map(|v| v.clock.clone())
            .unwrap_or_else(VectorClock::new);
        clock.increment(self.id);

        self.data.insert(
            key.to_string(),
            VersionedValue::new(value.to_string(), clock),
        );
    }

    /// Read a value from local storage.
    pub fn read(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|v| v.value.as_str())
    }

    /// Sync with another replica using anti-entropy.
    /// For each key, the value with the higher vector clock wins (LWW).
    pub fn sync_with(&mut self, other: &Replica) {
        for (key, other_value) in &other.data {
            match self.data.get(key) {
                Some(local_value) => {
                    if other_value.happened_before(local_value) {
                        // Local is newer, do nothing
                    } else if local_value.happened_before(other_value)
                        || local_value.is_concurrent_with(other_value)
                    {
                        // Other is newer or concurrent: use LWW (other wins)
                        self.data.insert(key.clone(), other_value.clone());
                    }
                }
                None => {
                    // Key doesn't exist locally, take it
                    self.data.insert(key.clone(), other_value.clone());
                }
            }
        }
    }

    /// Merge another replica's data, updating vector clocks.
    pub fn merge_replica(&mut self, other: &Replica) {
        for (key, other_value) in &other.data {
            match self.data.get_mut(key) {
                Some(local_value) => {
                    // Merge vector clocks
                    local_value.clock.merge(&other_value.clock);

                    // LWW: if other has higher max timestamp, use its value
                    let local_max = local_value
                        .clock
                        .nodes()
                        .iter()
                        .map(|&n| local_value.clock.get(n))
                        .max()
                        .unwrap_or(0);
                    let other_max = other_value
                        .clock
                        .nodes()
                        .iter()
                        .map(|&n| other_value.clock.get(n))
                        .max()
                        .unwrap_or(0);

                    if other_max >= local_max {
                        local_value.value = other_value.value.clone();
                    }
                }
                None => {
                    self.data.insert(key.clone(), other_value.clone());
                }
            }
        }
    }

    /// Get the number of entries in this replica.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if this replica is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_writes_on_any_replica() {
        let mut rep1 = Replica::new(1);
        let mut rep2 = Replica::new(2);
        let mut rep3 = Replica::new(3);

        // Write different keys to different replicas
        rep1.write("key1", "value1");
        rep2.write("key2", "value2");
        rep3.write("key3", "value3");

        // Each replica can read its own writes
        assert_eq!(rep1.read("key1"), Some("value1"));
        assert_eq!(rep2.read("key2"), Some("value2"));
        assert_eq!(rep3.read("key3"), Some("value3"));
    }

    #[test]
    fn test_eventual_convergence() {
        let mut rep1 = Replica::new(1);
        let mut rep2 = Replica::new(2);

        // Write to rep1
        rep1.write("shared", "from_rep1");

        // Sync rep2 with rep1
        rep2.sync_with(&rep1);
        assert_eq!(rep2.read("shared"), Some("from_rep1"));

        // Write to rep2
        rep2.write("shared", "from_rep2");

        // Sync rep1 with rep2
        rep1.sync_with(&rep2);

        // Both should converge to the latest value
        // (LWW with higher vector clock wins)
        assert_eq!(rep1.read("shared"), Some("from_rep2"));
        assert_eq!(rep2.read("shared"), Some("from_rep2"));
    }

    #[test]
    fn test_concurrent_write_handling() {
        let mut rep1 = Replica::new(1);
        let mut rep2 = Replica::new(2);

        // Concurrent writes to the same key
        rep1.write("race", "winner_is_1");
        rep2.write("race", "winner_is_2");

        // They are concurrent
        let v1 = rep1.data.get("race").unwrap();
        let v2 = rep2.data.get("race").unwrap();
        assert!(v1.is_concurrent_with(v2));

        // After sync, both should have the same value
        rep1.sync_with(&rep2);
        rep2.sync_with(&rep1);

        // Both converge to the same value (LWW)
        assert_eq!(rep1.read("race"), rep2.read("race"));
    }

    #[test]
    fn test_vector_clock_happens_before() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.increment(1);
        vc1.increment(1);

        vc2.increment(1);
        vc2.increment(1);
        vc2.increment(2);

        assert!(vc1.happened_before(&vc2));
        assert!(!vc2.happened_before(&vc1));
    }

    #[test]
    fn test_vector_clock_concurrent() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        vc1.increment(1);
        vc2.increment(2);

        assert!(vc1.concurrent_with(&vc2));
    }
}
