//! # Exercise: Linearizability Test
//!
//! ## Theory
//!
//! Linearizability (Herlihy & Wing, 1990) is the strongest single-object
//! consistency model. A history of operations is linearizable if there exists
//! a sequential ordering of those operations that:
//!
//! 1. **Respects real-time order:** If operation A completes before operation B
//!    begins, A must appear before B in the sequential ordering.
//! 2. **Is consistent:** Each operation appears to take effect atomically at
//!    some point between its invocation and completion.
//!
//! Testing linearizability is PSPACE-complete in general, but for small
//! histories we can enumerate all possible sequential orderings and check
//! each one.
//!
//! ## Proof / Intuition
//!
//! Given a history of operations, we extract all complete operations (those
//! with both invocation and response). We then enumerate all permutations
//! that respect the real-time partial order and check if any permutation is
//! consistent with the sequential specification of the object.
//!
//! For a key-value store, the sequential specification is simple: `get(key)`
//! returns the value of the most recent `put(key, value)`, or `None` if no
//! prior put.
//!
//! ## Implementation Task
//!
//! Implement a `LinearizabilityChecker`:
//! - `record_put(key, value, timestamp)`
//! - `record_get(key, expected_value, timestamp)`
//! - `is_linearizable() -> bool` -- check if the history is linearizable
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - A sequential history is linearizable
//! - A concurrent history that violates ordering is not linearizable
//! - An empty history is trivially linearizable

use std::collections::HashMap;

/// A recorded operation in the history.
#[derive(Debug, Clone)]
pub enum Operation {
    /// A put operation: key, value, timestamp.
    Put(String, String, u64),
    /// A get operation: key, expected_value, timestamp.
    Get(String, Option<String>, u64),
}

/// A linearizability checker for a key-value store.
///
/// Records a history of operations and checks whether there exists a
/// sequential ordering consistent with real-time constraints.
pub struct LinearizabilityChecker {
    history: Vec<Operation>,
}

impl LinearizabilityChecker {
    /// Create a new checker with an empty history.
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Record a put operation.
    pub fn record_put(&mut self, key: &str, value: &str, timestamp: u64) {
        self.history
            .push(Operation::Put(key.to_string(), value.to_string(), timestamp));
    }

    /// Record a get operation with its observed result.
    pub fn record_get(
        &mut self,
        key: &str,
        expected_value: Option<String>,
        timestamp: u64,
    ) {
        self.history.push(Operation::Get(
            key.to_string(),
            expected_value,
            timestamp,
        ));
    }

    /// Check if the recorded history is linearizable.
    ///
    /// Uses an enumeration approach: generates all permutations of the
    /// complete operations that respect real-time ordering and checks
    /// each against the sequential specification.
    pub fn is_linearizable(&self) -> bool {
        let ops = self.complete_ops();
        if ops.is_empty() {
            return true;
        }

        // Generate all permutations that respect real-time ordering
        let mut used = vec![false; ops.len()];
        let mut ordering = Vec::new();
        self.check_permutations(&ops, &mut used, &mut ordering)
    }

    /// Get the complete operations (those with timestamps).
    fn complete_ops(&self) -> Vec<Operation> {
        self.history.iter().cloned().collect()
    }

    /// Recursively generate all permutations and check each.
    fn check_permutations(
        &self,
        ops: &[Operation],
        used: &mut Vec<bool>,
        ordering: &mut Vec<usize>,
    ) -> bool {
        if ordering.len() == ops.len() {
            return self.is_valid_ordering(ops, ordering);
        }

        for i in 0..ops.len() {
            if used[i] {
                continue;
            }
            // Check real-time constraint: if an earlier op completed before
            // this one started, it must come first. Since we model timestamps
            // as the point of effect, enforce that ops are ordered by timestamp.
            if let Some(&last) = ordering.last() {
                let last_ts = self.get_timestamp(&ops[last]);
                let this_ts = self.get_timestamp(&ops[i]);
                if this_ts < last_ts {
                    continue; // Would violate real-time order
                }
            }

            used[i] = true;
            ordering.push(i);
            if self.check_permutations(ops, used, ordering) {
                return true;
            }
            ordering.pop();
            used[i] = false;
        }

        false
    }

    /// Get the timestamp from an operation.
    fn get_timestamp(&self, op: &Operation) -> u64 {
        match op {
            Operation::Put(_, _, ts) | Operation::Get(_, _, ts) => *ts,
        }
    }

    /// Check if a given ordering satisfies the sequential specification.
    fn is_valid_ordering(&self, ops: &[Operation], ordering: &[usize]) -> bool {
        let mut state: HashMap<String, String> = HashMap::new();

        for &idx in ordering {
            match &ops[idx] {
                Operation::Put(key, value, _) => {
                    state.insert(key.clone(), value.clone());
                }
                Operation::Get(key, expected, _) => {
                    let actual = state.get(key).cloned();
                    if actual != *expected {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Return the number of operations in the history.
    pub fn operation_count(&self) -> usize {
        self.history.len()
    }
}

impl Default for LinearizabilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_history_is_linearizable() {
        let checker = LinearizabilityChecker::new();
        assert!(checker.is_linearizable());
    }

    #[test]
    fn sequential_put_get_is_linearizable() {
        let mut checker = LinearizabilityChecker::new();
        checker.record_put("x", "1", 1);
        checker.record_get("x", Some("1".to_string()), 2);
        assert!(checker.is_linearizable());
    }

    #[test]
    fn inconsistent_get_is_not_linearizable() {
        let mut checker = LinearizabilityChecker::new();
        checker.record_put("x", "1", 1);
        // get returns "wrong" but the only possible value is "1"
        checker.record_get("x", Some("wrong".to_string()), 2);
        assert!(!checker.is_linearizable());
    }

    #[test]
    fn concurrent_puts_linearizable() {
        let mut checker = LinearizabilityChecker::new();
        // Two concurrent puts at the same timestamp -- either order is valid
        checker.record_put("x", "a", 1);
        checker.record_put("x", "b", 1);
        // A get after both puts sees the second one (either order works)
        checker.record_get("x", Some("b".to_string()), 2);
        assert!(checker.is_linearizable());
    }

    #[test]
    fn get_before_put_seeing_value_is_not_linearizable() {
        let mut checker = LinearizabilityChecker::new();
        // Get at time 1 sees value, but put is at time 2
        checker.record_get("x", Some("1".to_string()), 1);
        checker.record_put("x", "1", 2);
        // This should not be linearizable because get was before put
        assert!(!checker.is_linearizable());
    }

    #[test]
    fn multiple_keys_independent() {
        let mut checker = LinearizabilityChecker::new();
        checker.record_put("a", "1", 1);
        checker.record_put("b", "2", 2);
        checker.record_get("a", Some("1".to_string()), 3);
        checker.record_get("b", Some("2".to_string()), 4);
        assert!(checker.is_linearizable());
    }
}
