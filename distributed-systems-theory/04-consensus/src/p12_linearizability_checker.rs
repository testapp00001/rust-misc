//! # Exercise: Linearizability Checker
//!
//! ## Theory
//!
//! Linearizability (Herlihy & Wing, 1990) is the strongest single-object
//! consistency model. A history of operations on a concurrent object is
//! linearizable if there exists a sequential ordering that:
//!
//! 1. **Is consistent with the sequential specification** of the object.
//! 2. **Respects real-time order:** if operation A completes before B starts,
//!    A must precede B in the ordering.
//!
//! Formally, each operation appears to take effect atomically at some point
//! between its invocation and its response.
//!
//! Testing linearizability is PSPACE-complete in general. However, for
//! small histories with few concurrent operations, we can enumerate
//! possible orderings.
//!
//! ## Proof / Intuition
//!
//! For a key-value store, the sequential specification is:
//! - `Put(k, v)` sets the value of key k to v.
//! - `Get(k)` returns the most recent value written to k, or None.
//!
//! Given a history, we collect all completed operations, generate
//! orderings that respect real-time constraints, and check each
//! against the specification.
//!
//! ## Implementation Task
//!
//! Implement a `LinearizabilityChecker`:
//! - Record operation history (invocation + completion with timestamps)
//! - `is_linearizable()` -- check all valid orderings
//! - Detect linearizability violations
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Linearizable histories pass
//! - Non-linearizable histories fail
//! - Empty histories are trivially linearizable

use std::collections::HashMap;

/// An operation in the history.
#[derive(Debug, Clone)]
pub struct Operation {
    pub op_type: OpType,
    pub key: String,
    pub value: Option<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpType {
    Put,
    Get,
}

/// Result of checking linearizability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinearizabilityResult {
    /// The history is linearizable.
    Linearizable,
    /// The history is not linearizable.
    NotLinearizable {
        reason: String,
    },
}

/// A linearizability checker for a key-value store.
pub struct LinearizabilityChecker {
    history: Vec<Operation>,
}

impl LinearizabilityChecker {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
        }
    }

    /// Record a put operation.
    pub fn record_put(&mut self, key: &str, value: &str, timestamp: u64) {
        self.history.push(Operation {
            op_type: OpType::Put,
            key: key.to_string(),
            value: Some(value.to_string()),
            timestamp,
        });
    }

    /// Record a get operation with its observed result.
    pub fn record_get(&mut self, key: &str, result: Option<String>, timestamp: u64) {
        self.history.push(Operation {
            op_type: OpType::Get,
            key: key.to_string(),
            value: result,
            timestamp,
        });
    }

    /// Check if the recorded history is linearizable.
    pub fn check(&self) -> LinearizabilityResult {
        if self.history.is_empty() {
            return LinearizabilityResult::Linearizable;
        }

        // Generate all permutations respecting timestamp order
        let ops = &self.history;
        let mut used = vec![false; ops.len()];
        let mut ordering = Vec::new();

        if self.check_recursion(ops, &mut used, &mut ordering) {
            LinearizabilityResult::Linearizable
        } else {
            LinearizabilityResult::NotLinearizable {
                reason: "no valid sequential ordering found".to_string(),
            }
        }
    }

    /// Shorthand: returns true if linearizable.
    pub fn is_linearizable(&self) -> bool {
        self.check() == LinearizabilityResult::Linearizable
    }

    fn check_recursion(
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
            // Maintain timestamp ordering
            if let Some(&last_idx) = ordering.last() {
                if ops[i].timestamp < ops[last_idx].timestamp {
                    continue;
                }
            }

            used[i] = true;
            ordering.push(i);
            if self.check_recursion(ops, used, ordering) {
                return true;
            }
            ordering.pop();
            used[i] = false;
        }

        false
    }

    fn is_valid_ordering(&self, ops: &[Operation], ordering: &[usize]) -> bool {
        let mut state: HashMap<String, String> = HashMap::new();

        for &idx in ordering {
            let op = &ops[idx];
            match op.op_type {
                OpType::Put => {
                    state.insert(op.key.clone(), op.value.clone().unwrap());
                }
                OpType::Get => {
                    let actual = state.get(&op.key).cloned();
                    if actual != op.value {
                        return false;
                    }
                }
            }
        }
        true
    }

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
    fn non_linearizable_history_detected() {
        let mut checker = LinearizabilityChecker::new();
        // Get sees "wrong" value before any put
        checker.record_get("x", Some("1".to_string()), 1);
        checker.record_put("x", "1", 2);
        assert!(!checker.is_linearizable());
    }

    #[test]
    fn concurrent_puts_linearizable() {
        let mut checker = LinearizabilityChecker::new();
        checker.record_put("x", "a", 1);
        checker.record_put("x", "b", 1); // Same timestamp = concurrent
        checker.record_get("x", Some("b".to_string()), 2);
        assert!(checker.is_linearizable());
    }

    #[test]
    fn inconsistent_get_not_linearizable() {
        let mut checker = LinearizabilityChecker::new();
        checker.record_put("x", "42", 1);
        checker.record_get("x", Some("99".to_string()), 2);
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

    #[test]
    fn checker_returns_correct_result_type() {
        let mut checker = LinearizabilityChecker::new();
        checker.record_put("x", "1", 1);
        checker.record_get("x", Some("1".to_string()), 2);
        assert_eq!(checker.check(), LinearizabilityResult::Linearizable);
    }
}
