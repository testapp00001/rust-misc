//! # Exercise: G-Counter (Grow-only Counter)
//!
//! ## Theory
//!
//! A G-Counter (Grow-only Counter) is the simplest CRDT. It maintains a vector
//! of counters, one per replica. Each replica only increments its own slot.
//! The total value is the sum of all slots.
//!
//! Properties:
//! - Commutative: merge(a, b) = merge(b, a)
//! - Associative: merge(merge(a, b), c) = merge(a, merge(b, c))
//! - Idempotent: merge(a, a) = a
//!
//! ## Proof / Intuition
//!
//! Since each replica owns a unique slot and only increments it, two replicas
//! can never conflict. Merging takes the element-wise maximum, which is
//! commutative (max is commutative), associative (max is associative), and
//! idempotent (max(x, x) = x).
//!
//! ## Implementation Task
//!
//! Implement `GCounter` with:
//! - `increment(replica_id)` - increment this replica's counter
//! - `value()` - return total count (sum of all slots)
//! - `merge(other)` - element-wise max
//!
//! ## Verification
//!
//! Verify increment works, merge is commutative, merge is idempotent.
//!
use std::collections::HashMap;

/// A Grow-only Counter CRDT.
///
/// Each replica owns a slot (identified by replica_id) and can only
/// increment it. The total value is the sum of all slots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GCounter {
    /// Count per replica: replica_id -> count.
    counts: HashMap<usize, u64>,
}

impl GCounter {
    /// Create a new empty G-Counter.
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Create a G-Counter with a specific number of replicas.
    pub fn with_replicas(num_replicas: usize) -> Self {
        let mut counts = HashMap::new();
        for i in 0..num_replicas {
            counts.insert(i, 0);
        }
        Self { counts }
    }

    /// Increment the counter for the given replica.
    pub fn increment(&mut self, replica_id: usize) {
        *self.counts.entry(replica_id).or_insert(0) += 1;
    }

    /// Increment the counter by a specific amount.
    pub fn increment_by(&mut self, replica_id: usize, amount: u64) {
        *self.counts.entry(replica_id).or_insert(0) += amount;
    }

    /// Get the total count (sum of all replica counts).
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Get the count for a specific replica.
    pub fn get(&self, replica_id: usize) -> u64 {
        self.counts.get(&replica_id).copied().unwrap_or(0)
    }

    /// Merge with another G-Counter using element-wise max.
    ///
    /// This is the core CRDT merge operation. It is:
    /// - Commutative: merge(a, b) = merge(b, a)
    /// - Associative: merge(merge(a, b), c) = merge(a, merge(b, c))
    /// - Idempotent: merge(a, a) = a
    pub fn merge(&mut self, other: &GCounter) {
        for (replica_id, &count) in &other.counts {
            let entry = self.counts.entry(*replica_id).or_insert(0);
            *entry = (*entry).max(count);
        }
    }

    /// Get the number of replicas tracked.
    pub fn num_replicas(&self) -> usize {
        self.counts.len()
    }
}

impl Default for GCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment() {
        let mut counter = GCounter::new();
        counter.increment(0);
        counter.increment(0);
        counter.increment(1);

        assert_eq!(counter.value(), 3);
        assert_eq!(counter.get(0), 2);
        assert_eq!(counter.get(1), 1);
        assert_eq!(counter.get(2), 0);
    }

    #[test]
    fn test_merge_is_commutative() {
        let mut a = GCounter::new();
        a.increment(0);
        a.increment(0);
        a.increment(1);

        let mut b = GCounter::new();
        b.increment(0);
        b.increment(2);
        b.increment(2);
        b.increment(2);

        let a_clone = a.clone();
        a.merge(&b);
        let ab_value = a.value();

        b.merge(&a_clone);
        let ba_value = b.value();

        assert_eq!(ab_value, ba_value, "Merge must be commutative");
        assert_eq!(a, b, "Merge must be commutative (same final state)");
    }

    #[test]
    fn test_merge_is_idempotent() {
        let mut counter = GCounter::new();
        counter.increment(0);
        counter.increment(1);
        counter.increment(1);

        let original = counter.clone();
        counter.merge(&original.clone());

        assert_eq!(counter, original, "Merge with self must not change state (idempotent)");
    }

    #[test]
    fn test_merge_takes_max() {
        let mut a = GCounter::new();
        a.increment(0);
        a.increment(0);
        a.increment(0); // replica 0: 3

        let mut b = GCounter::new();
        b.increment(0);
        b.increment(0); // replica 0: 2

        a.merge(&b);
        assert_eq!(a.get(0), 3, "Merge should keep the higher count");
    }

    #[test]
    fn test_merge_associative() {
        let mut a = GCounter::new();
        a.increment(0);
        a.increment(0);

        let mut b = GCounter::new();
        b.increment(0);
        b.increment(1);

        let mut c = GCounter::new();
        c.increment(1);
        c.increment(1);
        c.increment(2);

        let mut ab_then_c = a.clone();
        ab_then_c.merge(&b);
        ab_then_c.merge(&c);

        let mut a_then_bc = a;
        let mut bc = b.clone();
        bc.merge(&c);
        a_then_bc.merge(&bc);

        assert_eq!(ab_then_c, a_then_bc, "Merge must be associative");
    }
}
