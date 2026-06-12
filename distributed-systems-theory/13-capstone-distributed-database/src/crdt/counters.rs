//! Conflict-free replicated counters.
//!
//! This module implements two varieties of CRDT counters:
//!
//! - **G-Counter** (Grow-only Counter): supports only increment. The value is the
//!   sum across all nodes. Merge takes the per-node maximum.
//! - **PN-Counter** (Positive-Negative Counter): supports both increment and
//!   decrement using two internal G-Counters. The value is increments minus
//!   decrements.

use std::collections::HashMap;

/// A grow-only counter (G-Counter) CRDT.
///
/// Each node maintains its own counter. The global value is the sum of all
/// per-node counters. Merge converges by taking the element-wise maximum.
#[derive(Debug, Clone)]
pub struct GCounter {
    /// Per-node counter values.
    counts: HashMap<u64, u64>,
}

impl GCounter {
    /// Create a new, empty G-Counter.
    pub fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    /// Increment this node's counter by the given amount.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node performing the increment.
    /// * `amount` - The amount to increment by.
    pub fn increment(&mut self, node_id: u64, amount: u64) {
        *self.counts.entry(node_id).or_insert(0) += amount;
    }

    /// Return the current total value of the counter (sum of all nodes).
    pub fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Merge another G-Counter into this one.
    ///
    /// For each node, the maximum of the two counters is kept. This guarantees
    /// convergence regardless of the order in which merges occur.
    ///
    /// # Arguments
    ///
    /// * `other` - The remote G-Counter to merge.
    pub fn merge(&mut self, other: &GCounter) {
        for (&node_id, &count) in &other.counts {
            let entry = self.counts.entry(node_id).or_insert(0);
            if count > *entry {
                *entry = count;
            }
        }
    }
}

/// A positive-negative counter (PN-Counter) CRDT.
///
/// Uses two internal G-Counters: one for increments and one for decrements.
/// The value is the difference between total increments and total decrements.
#[derive(Debug, Clone)]
pub struct PNCounter {
    /// Per-node increment counts.
    increments: HashMap<u64, u64>,
    /// Per-node decrement counts.
    decrements: HashMap<u64, u64>,
}

impl PNCounter {
    /// Create a new, empty PN-Counter.
    pub fn new() -> Self {
        Self {
            increments: HashMap::new(),
            decrements: HashMap::new(),
        }
    }

    /// Increment this node's counter by the given amount.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node performing the increment.
    /// * `amount` - The amount to increment by.
    pub fn increment(&mut self, node_id: u64, amount: u64) {
        *self.increments.entry(node_id).or_insert(0) += amount;
    }

    /// Decrement this node's counter by the given amount.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The id of the node performing the decrement.
    /// * `amount` - The amount to decrement by.
    pub fn decrement(&mut self, node_id: u64, amount: u64) {
        *self.decrements.entry(node_id).or_insert(0) += amount;
    }

    /// Return the current value of the counter (total increments minus total decrements).
    pub fn value(&self) -> i64 {
        let inc: u64 = self.increments.values().sum();
        let dec: u64 = self.decrements.values().sum();
        inc as i64 - dec as i64
    }

    /// Merge another PN-Counter into this one.
    ///
    /// For each node, the maximum is taken independently in both the increments
    /// and decrements maps.
    ///
    /// # Arguments
    ///
    /// * `other` - The remote PN-Counter to merge.
    pub fn merge(&mut self, other: &PNCounter) {
        for (&node_id, &count) in &other.increments {
            let entry = self.increments.entry(node_id).or_insert(0);
            if count > *entry {
                *entry = count;
            }
        }
        for (&node_id, &count) in &other.decrements {
            let entry = self.decrements.entry(node_id).or_insert(0);
            if count > *entry {
                *entry = count;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- G-Counter tests ---

    #[test]
    fn test_gcounter_increment_and_value() {
        let mut c = GCounter::new();
        c.increment(1, 5);
        c.increment(2, 3);
        assert_eq!(c.value(), 8);
    }

    #[test]
    fn test_gcounter_merge() {
        let mut c1 = GCounter::new();
        c1.increment(1, 5);
        c1.increment(2, 3);

        let mut c2 = GCounter::new();
        c2.increment(1, 2);
        c2.increment(3, 7);

        c1.merge(&c2);
        assert_eq!(c1.value(), 5 + 3 + 7); // max(5,2)=5, 3, 7
    }

    #[test]
    fn test_gcounter_merge_convergence() {
        // Merging in different orders should produce the same result.
        let mut a = GCounter::new();
        a.increment(1, 10);

        let mut b = GCounter::new();
        b.increment(1, 5);
        b.increment(2, 8);

        // Merge a into b
        let mut b_clone = b.clone();
        b_clone.merge(&a);
        assert_eq!(b_clone.value(), 10 + 8);

        // Merge b into a
        let mut a_clone = a.clone();
        a_clone.merge(&b);
        assert_eq!(a_clone.value(), 10 + 8);

        assert_eq!(b_clone.value(), a_clone.value());
    }

    // --- PN-Counter tests ---

    #[test]
    fn test_pncounter_increment_and_decrement() {
        let mut c = PNCounter::new();
        c.increment(1, 10);
        c.decrement(2, 3);
        assert_eq!(c.value(), 7);
    }

    #[test]
    fn test_pncounter_merge() {
        let mut c1 = PNCounter::new();
        c1.increment(1, 10);
        c1.decrement(2, 3);

        let mut c2 = PNCounter::new();
        c2.increment(1, 5);
        c2.decrement(3, 2);

        c1.merge(&c2);
        // Increments: max(10,5)=10 from node 1
        // Decrements: max(3,0)=3 from node 2, max(0,2)=2 from node 3
        assert_eq!(c1.value(), 10 - 3 - 2);
    }

    #[test]
    fn test_pncounter_negative_value() {
        let mut c = PNCounter::new();
        c.increment(1, 3);
        c.decrement(1, 10);
        assert_eq!(c.value(), -7);
    }
}
