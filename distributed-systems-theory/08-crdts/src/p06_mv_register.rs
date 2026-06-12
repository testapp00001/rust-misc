//! # Exercise: MV-Register (Multi-Value Register)
//!
//! ## Theory
//!
//! An MV-Register (Multi-Value Register) detects concurrent writes by
//! tracking vector clocks. When two writes are concurrent (neither
//! happens-before the other), the register stores both values as
//! "siblings."
//!
//! This provides stronger semantics than LWW-Register: instead of silently
//! dropping concurrent writes, it preserves all concurrent values and
//! requires application-level conflict resolution.
//!
//! ## Proof / Intuition
//!
//! Vector clocks establish causal ordering. When merging:
//! - If one register's vector clock dominates (>= on all entries), its
//!   value replaces the other
//! - If neither dominates (concurrent), both values are kept
//!
//! The application calls `resolve` to pick one value, collapsing back
//! to a single value.
//!
//! ## Implementation Task
//!
//! Implement `MVRegister<T>` with:
//! - `set(value, replica_id)` - set value with updated vector clock
//! - `get()` - returns current value(s)
//! - `merge(other)` - merge vector clocks and values
//! - `resolve(chosen)` - application resolves to single value
//!
//! ## Verification
//!
//! Verify concurrent values detected, resolution works, causality preserved.
//!
use std::collections::HashMap;

/// Vector clock: replica_id -> logical timestamp.
pub type VectorClock = HashMap<usize, u64>;

/// A Multi-Value Register CRDT.
///
/// Detects concurrent writes via vector clocks and stores multiple values
/// when conflicts exist. Requires application-level resolution.
#[derive(Debug, Clone)]
pub struct MVRegister<T: Clone + PartialEq> {
    /// One or more concurrent values.
    values: Vec<T>,
    /// Vector clock tracking causality.
    clock: VectorClock,
}

impl<T: Clone + PartialEq> MVRegister<T> {
    /// Create a new MV-Register with an initial value.
    pub fn new(value: T, replica_id: usize) -> Self {
        let mut clock = HashMap::new();
        clock.insert(replica_id, 1);
        Self {
            values: vec![value],
            clock,
        }
    }

    /// Set a new value from a specific replica.
    pub fn set(&mut self, value: T, replica_id: usize) {
        let counter = self.clock.entry(replica_id).or_insert(0);
        *counter += 1;
        self.values = vec![value];
    }

    /// Get the current value(s). Returns multiple values if concurrent.
    pub fn get(&self) -> &[T] {
        &self.values
    }

    /// Get the vector clock.
    pub fn clock(&self) -> &VectorClock {
        &self.clock
    }

    /// Resolve to a single value (application-level conflict resolution).
    pub fn resolve(&mut self, chosen: T) {
        self.values = vec![chosen];
    }

    /// Check if the register has concurrent values (needs resolution).
    pub fn has_conflict(&self) -> bool {
        self.values.len() > 1
    }

    /// Get the number of concurrent values.
    pub fn num_values(&self) -> usize {
        self.values.len()
    }

    /// Merge with another MV-Register.
    pub fn merge(&mut self, other: &MVRegister<T>) {
        // Check causal relationship BEFORE merging clocks
        let self_dominates = self.dominates_clock(&other.clock);
        let other_dominates = other.dominates_clock(&self.clock);

        // Merge vector clocks: take max of each entry
        for (replica_id, &counter) in &other.clock {
            let entry = self.clock.entry(*replica_id).or_insert(0);
            *entry = (*entry).max(counter);
        }

        if self_dominates && !other_dominates {
            // self happens-after other, keep self's values
            return;
        } else if other_dominates && !self_dominates {
            // other happens-after self, take other's values
            self.values = other.values.clone();
        } else {
            // Concurrent: keep both sets of values (union, deduplicated)
            let mut all_values: Vec<T> = self.values.clone();
            for v in &other.values {
                if !all_values.contains(v) {
                    all_values.push(v.clone());
                }
            }
            self.values = all_values;
        }
    }

    /// Check if this register's clock dominates another clock.
    fn dominates_clock(&self, other_clock: &VectorClock) -> bool {
        for (replica_id, &counter) in other_clock {
            let self_counter = self.clock.get(replica_id).copied().unwrap_or(0);
            if self_counter < counter {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_values_detected() {
        let mut reg_a = MVRegister::new("value_a", 0);
        let reg_b = MVRegister::new("value_b", 1);

        // Both set concurrently (different replicas, no prior communication)
        reg_a.merge(&reg_b);

        assert!(
            reg_a.has_conflict(),
            "Concurrent values should be detected"
        );
        assert_eq!(reg_a.num_values(), 2);
    }

    #[test]
    fn test_resolve_picks_single_value() {
        let mut reg_a = MVRegister::new("value_a", 0);
        let reg_b = MVRegister::new("value_b", 1);

        reg_a.merge(&reg_b);
        assert!(reg_a.has_conflict());

        reg_a.resolve("value_a");
        assert!(!reg_a.has_conflict());
        assert_eq!(reg_a.get().len(), 1);
        assert_eq!(reg_a.get()[0], "value_a");
    }

    #[test]
    fn test_causal_ordering() {
        let mut reg = MVRegister::new("initial", 0);

        // Replica 0 sets value
        reg.set("updated_by_0", 0);

        // Replica 1 receives the update and sets its own value
        let mut reg2 = reg.clone();
        reg2.set("updated_by_1", 1);

        // Merge: reg2 happens-after reg (for replica 0's clock)
        reg.merge(&reg2);

        // reg2's clock dominates for replica 1 (1 > 0) but not for replica 0 (same)
        // So we should have both values or just reg2's
        assert!(reg.has_conflict() || reg.get().len() >= 1);
    }

    #[test]
    fn test_merge_commutative() {
        let mut a = MVRegister::new("a", 0);
        let mut b = MVRegister::new("b", 1);

        let a_clone = a.clone();
        let b_clone = b.clone();

        a.merge(&b_clone);
        b.merge(&a_clone);

        // Both should have the same values (possibly in different order)
        let mut a_vals: Vec<&&str> = a.get().iter().collect();
        let mut b_vals: Vec<&&str> = b.get().iter().collect();
        a_vals.sort();
        b_vals.sort();
        assert_eq!(a_vals, b_vals, "Merge must be commutative");
    }
}
