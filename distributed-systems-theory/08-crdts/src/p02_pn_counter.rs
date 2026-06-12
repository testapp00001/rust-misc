//! # Exercise: PN-Counter (Positive-Negative Counter)
//!
//! ## Theory
//!
//! A PN-Counter extends the G-Counter to support both increments and decrements.
//! It uses two G-Counters: one for positive increments (P) and one for negative
//! increments (N). The value is P.value() - N.value().
//!
//! This works because:
//! - Increments only touch P, decrements only touch N
//! - Both P and N are G-Counters with proven merge properties
//! - The difference P - N gives the net count
//!
//! ## Proof / Intuition
//!
//! Since P and N are independently merged G-Counters, the merge of a PN-Counter
//! is simply merging both components. The value function (P - N) commutes because
//! subtraction of merged values preserves the algebraic properties.
//!
//! ## Implementation Task
//!
//! Implement `PNCounter` with:
//! - `increment(replica_id)` / `decrement(replica_id)`
//! - `value()` - P.value() - N.value()
//! - `merge(other)` - merge both P and N counters
//!
//! ## Verification
//!
//! Verify increment/decrement work, merge is commutative, value = P - N.
//!
use std::collections::HashMap;

/// Internal G-Counter for tracking positive or negative counts.
#[derive(Debug, Clone, PartialEq, Eq)]
struct InnerCounter {
    counts: HashMap<usize, u64>,
}

impl InnerCounter {
    fn new() -> Self {
        Self {
            counts: HashMap::new(),
        }
    }

    fn increment(&mut self, replica_id: usize) {
        *self.counts.entry(replica_id).or_insert(0) += 1;
    }

    fn value(&self) -> u64 {
        self.counts.values().sum()
    }

    fn merge(&mut self, other: &InnerCounter) {
        for (replica_id, &count) in &other.counts {
            let entry = self.counts.entry(*replica_id).or_insert(0);
            *entry = (*entry).max(count);
        }
    }
}

/// A Positive-Negative Counter CRDT.
///
/// Supports both increment and decrement operations. The value is
/// calculated as the sum of positive counts minus the sum of negative counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PNCounter {
    positive: InnerCounter,
    negative: InnerCounter,
}

impl PNCounter {
    /// Create a new empty PN-Counter.
    pub fn new() -> Self {
        Self {
            positive: InnerCounter::new(),
            negative: InnerCounter::new(),
        }
    }

    /// Increment the counter for the given replica.
    pub fn increment(&mut self, replica_id: usize) {
        self.positive.increment(replica_id);
    }

    /// Increment by a specific amount.
    pub fn increment_by(&mut self, replica_id: usize, amount: u64) {
        *self.positive.counts.entry(replica_id).or_insert(0) += amount;
    }

    /// Decrement the counter for the given replica.
    pub fn decrement(&mut self, replica_id: usize) {
        self.negative.increment(replica_id);
    }

    /// Decrement by a specific amount.
    pub fn decrement_by(&mut self, replica_id: usize, amount: u64) {
        *self.negative.counts.entry(replica_id).or_insert(0) += amount;
    }

    /// Get the current value: P.value() - N.value().
    pub fn value(&self) -> i64 {
        self.positive.value() as i64 - self.negative.value() as i64
    }

    /// Get the positive count.
    pub fn positive_value(&self) -> u64 {
        self.positive.value()
    }

    /// Get the negative count.
    pub fn negative_value(&self) -> u64 {
        self.negative.value()
    }

    /// Merge with another PN-Counter.
    pub fn merge(&mut self, other: &PNCounter) {
        self.positive.merge(&other.positive);
        self.negative.merge(&other.negative);
    }
}

impl Default for PNCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_increment_and_decrement() {
        let mut counter = PNCounter::new();
        counter.increment(0);
        counter.increment(0);
        counter.increment(1);
        counter.decrement(2);

        assert_eq!(counter.value(), 2); // 3 - 1
        assert_eq!(counter.positive_value(), 3);
        assert_eq!(counter.negative_value(), 1);
    }

    #[test]
    fn test_value_is_positive_minus_negative() {
        let mut counter = PNCounter::new();
        counter.increment_by(0, 10);
        counter.decrement_by(0, 7);

        assert_eq!(counter.value(), 3);
        assert_eq!(counter.positive_value(), 10);
        assert_eq!(counter.negative_value(), 7);
    }

    #[test]
    fn test_merge_commutative() {
        let mut a = PNCounter::new();
        a.increment(0);
        a.increment(0);
        a.decrement(1);

        let mut b = PNCounter::new();
        b.increment(1);
        b.decrement(0);
        b.decrement(0);
        b.decrement(2);

        let a_clone = a.clone();
        let b_clone = b.clone();

        a.merge(&b_clone);
        let ab_value = a.value();

        b.merge(&a_clone);
        let ba_value = b.value();

        assert_eq!(ab_value, ba_value, "Merge must be commutative");
        assert_eq!(a, b, "Merge must produce same state");
    }

    #[test]
    fn test_merge_idempotent() {
        let mut counter = PNCounter::new();
        counter.increment(0);
        counter.decrement(1);

        let original = counter.clone();
        counter.merge(&original.clone());

        assert_eq!(counter, original, "Merge with self must be idempotent");
    }

    #[test]
    fn test_merge_associative() {
        let mut a = PNCounter::new();
        a.increment(0);

        let mut b = PNCounter::new();
        b.increment(1);
        b.decrement(0);

        let mut c = PNCounter::new();
        c.decrement(1);
        c.decrement(2);

        let mut ab_then_c = a.clone();
        ab_then_c.merge(&b);
        ab_then_c.merge(&c);

        let mut a_then_bc = a;
        let mut bc = b.clone();
        bc.merge(&c);
        a_then_bc.merge(&bc);

        assert_eq!(ab_then_c.value(), a_then_bc.value(), "Merge must be associative");
    }
}
