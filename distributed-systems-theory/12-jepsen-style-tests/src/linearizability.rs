//! # Exercise: Linearizability Checker
//!
//! ## Theory
//!
//! Linearizability is the strongest single-object consistency model. A history
//! of operations is linearizable if there exists a total ordering of all
//! operations such that:
//!
//! 1. The ordering is consistent with real-time: if operation A completes
//!    before operation B begins, then A appears before B in the ordering.
//! 2. The ordering is consistent with each operation's semantics: every read
//!    returns the value of the most recent write in the ordering.
//!
//! Checking linearizability is PSPACE-complete in the general case. For small
//! histories we can use a brute-force approach that enumerates all valid orderings.
//!
//! ## Proof / Intuition
//!
//! Consider a register that supports read and write operations. If we have:
//!
//!   P1: W(x, 1) ----|
//!   P2:         R(x) -> 1
//!
//! This is linearizable: P1's write appears to take effect before P2's read.
//!
//! But if we have:
//!
//!   P1: W(x, 1) ----|
//!   P2:    W(x, 2) --|
//!   P3:         R(x) -> ???
//!
//! Both writes are concurrent. A linearization could order P1 then P2 (read
//! returns 2) or P2 then P1 (read returns 2). Either way the read must return
//! 1 or 2, never something else.
//!
//! ## Implementation Task
//!
//! Implement `LinearizabilityChecker::check()` which takes a history of
//! operations and returns true if the history is linearizable.
//!
//! The approach:
//! - Group operations by key.
//! - For each key, sort by end_time to establish real-time order.
//! - Build a sequential execution by processing operations in this order.
//! - Verify every read returns the value of the most recent write.
//!
//! ## Verification
//!
//! Run `cargo test linearizability` to verify your implementation.

use std::collections::HashMap;

/// The type of operation performed on a register.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperationType {
    Read,
    Write,
}

/// A single operation in a distributed system history.
#[derive(Debug, Clone)]
pub struct Operation {
    /// The process that performed this operation.
    pub process_id: usize,
    /// Whether this is a read or write.
    pub op_type: OperationType,
    /// The key being accessed.
    pub key: String,
    /// For writes, the value being written. For reads, the value observed.
    pub value: Option<String>,
    /// The logical or physical time when the operation was invoked.
    pub start_time: u64,
    /// The logical or physical time when the operation completed.
    pub end_time: u64,
}

/// A linearizability checker that verifies histories against the linearizability
/// consistency model.
#[derive(Debug, Default)]
pub struct LinearizabilityChecker;

impl LinearizabilityChecker {
    /// Create a new linearizability checker.
    pub fn new() -> Self {
        Self
    }

    /// Check whether a history of operations is linearizable.
    ///
    /// The algorithm works as follows:
    /// 1. Group operations by key.
    /// 2. For each key, sort operations by end_time (real-time completion order).
    ///    Break ties by putting writes before reads.
    /// 3. Process operations in this order, tracking the last written value.
    /// 4. If any read observes a value that does not match the most recent write,
    ///    the history is not linearizable.
    ///
    /// This is a conservative check: it tests one specific linearization
    /// (sorted by end_time). For a complete check, one would need to explore
    /// all valid orderings, but that is exponential in the number of concurrent
    /// operations.
    pub fn check(&self, history: &[Operation]) -> bool {
        // Group operations by key.
        let mut by_key: HashMap<String, Vec<&Operation>> = HashMap::new();
        for op in history {
            by_key.entry(op.key.clone()).or_default().push(op);
        }

        // For each key, try to find a valid linearization.
        for (_key, ops) in &by_key {
            if !self.check_key_history(ops) {
                return false;
            }
        }

        true
    }

    /// Check that operations for a single key admit a valid linearization.
    ///
    /// We sort by end_time, breaking ties: writes before reads. Then we walk
    /// through and verify every read returns the most recent write value.
    fn check_key_history(&self, ops: &[&Operation]) -> bool {
        let mut sorted: Vec<&Operation> = ops.to_vec();
        sorted.sort_by(|a, b| {
            a.end_time
                .cmp(&b.end_time)
                .then_with(|| match (&a.op_type, &b.op_type) {
                    (OperationType::Write, OperationType::Read) => std::cmp::Ordering::Less,
                    (OperationType::Read, OperationType::Write) => std::cmp::Ordering::Greater,
                    _ => std::cmp::Ordering::Equal,
                })
        });

        // Walk the linearization and track the latest write value.
        let mut last_value: Option<String> = None;
        for op in &sorted {
            match op.op_type {
                OperationType::Write => {
                    last_value = op.value.clone();
                }
                OperationType::Read => {
                    let observed = &op.value;
                    if observed != &last_value {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Try all permutations of concurrent operations for a single key to
    /// exhaustively check linearizability. Only feasible for small histories
    /// (<= ~10 concurrent operations).
    pub fn check_exhaustive(&self, history: &[Operation]) -> bool {
        let mut by_key: HashMap<String, Vec<Operation>> = HashMap::new();
        for op in history {
            by_key
                .entry(op.key.clone())
                .or_default()
                .push(op.clone());
        }

        for (_key, ops) in &by_key {
            if !self.check_key_exhaustive(&ops) {
                return false;
            }
        }

        true
    }

    /// Partition operations into groups separated by real-time gaps, then
    /// enumerate all permutations within each group of concurrent operations.
    fn check_key_exhaustive(&self, ops: &[Operation]) -> bool {
        if ops.is_empty() {
            return true;
        }

        // Sort by start_time, then by end_time.
        let mut sorted = ops.to_vec();
        sorted.sort_by(|a, b| {
            a.start_time
                .cmp(&b.start_time)
                .then(a.end_time.cmp(&b.end_time))
        });

        // Split into groups of concurrent operations.
        let groups = self.split_into_concurrent_groups(&sorted);

        // For each group, try all permutations and check if any is valid.
        self.check_groups_exhaustive(&groups, 0, &mut Vec::new())
    }

    fn split_into_concurrent_groups(&self, ops: &[Operation]) -> Vec<Vec<Operation>> {
        if ops.is_empty() {
            return vec![];
        }

        let mut groups: Vec<Vec<Operation>> = Vec::new();
        let mut current_group: Vec<Operation> = vec![ops[0].clone()];
        let mut group_end = ops[0].end_time;

        for op in ops.iter().skip(1) {
            if op.start_time < group_end {
                current_group.push(op.clone());
                if op.end_time > group_end {
                    group_end = op.end_time;
                }
            } else {
                groups.push(current_group);
                current_group = vec![op.clone()];
                group_end = op.end_time;
            }
        }
        groups.push(current_group);
        groups
    }

    fn check_groups_exhaustive(
        &self,
        groups: &[Vec<Operation>],
        group_idx: usize,
        linearization: &mut Vec<Operation>,
    ) -> bool {
        if group_idx >= groups.len() {
            return self.verify_linearization(linearization);
        }

        let group = &groups[group_idx];
        let permutations = self.generate_permutations(group);

        for perm in &permutations {
            linearization.extend(perm.iter().cloned());
            if self.check_groups_exhaustive(groups, group_idx + 1, linearization) {
                return true;
            }
            linearization.truncate(linearization.len() - perm.len());
        }

        false
    }

    fn generate_permutations(&self, items: &[Operation]) -> Vec<Vec<Operation>> {
        let mut result = Vec::new();
        let mut current = items.to_vec();
        self.heap_permutation(&mut current, items.len(), &mut result);
        result
    }

    fn heap_permutation(
        &self,
        items: &mut Vec<Operation>,
        size: usize,
        result: &mut Vec<Vec<Operation>>,
    ) {
        if size == 1 {
            result.push(items.clone());
            return;
        }

        for i in 0..size {
            self.heap_permutation(items, size - 1, result);
            if size % 2 == 1 {
                items.swap(0, size - 1);
            } else {
                items.swap(i, size - 1);
            }
        }
    }

    fn verify_linearization(&self, linearization: &[Operation]) -> bool {
        let mut last_value: Option<String> = None;
        for op in linearization {
            match op.op_type {
                OperationType::Write => {
                    last_value = op.value.clone();
                }
                OperationType::Read => {
                    if op.value != last_value {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// Build a valid (linearizable) history for testing.
///
/// History:
///   P0: W(x, "a") at t=1..2
///   P1:         R(x) -> "a" at t=3..4
///   P2:                 W(x, "b") at t=5..6
///   P3:                         R(x) -> "b" at t=7..8
pub fn build_valid_history() -> Vec<Operation> {
    vec![
        Operation {
            process_id: 0,
            op_type: OperationType::Write,
            key: "x".to_string(),
            value: Some("a".to_string()),
            start_time: 1,
            end_time: 2,
        },
        Operation {
            process_id: 1,
            op_type: OperationType::Read,
            key: "x".to_string(),
            value: Some("a".to_string()),
            start_time: 3,
            end_time: 4,
        },
        Operation {
            process_id: 2,
            op_type: OperationType::Write,
            key: "x".to_string(),
            value: Some("b".to_string()),
            start_time: 5,
            end_time: 6,
        },
        Operation {
            process_id: 3,
            op_type: OperationType::Read,
            key: "x".to_string(),
            value: Some("b".to_string()),
            start_time: 7,
            end_time: 8,
        },
    ]
}

/// Build an invalid (non-linearizable) history for testing.
///
/// History:
///   P0: W(x, "a") at t=1..2
///   P1:         R(x) -> "c" at t=3..4
///
/// The read returns "c" which was never written.
pub fn build_invalid_history() -> Vec<Operation> {
    vec![
        Operation {
            process_id: 0,
            op_type: OperationType::Write,
            key: "x".to_string(),
            value: Some("a".to_string()),
            start_time: 1,
            end_time: 2,
        },
        Operation {
            process_id: 1,
            op_type: OperationType::Read,
            key: "x".to_string(),
            value: Some("c".to_string()),
            start_time: 3,
            end_time: 4,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_history_passes() {
        let checker = LinearizabilityChecker::new();
        let history = build_valid_history();
        assert!(checker.check(&history));
    }

    #[test]
    fn test_non_linear_history_fails() {
        let checker = LinearizabilityChecker::new();
        let history = build_invalid_history();
        assert!(!checker.check(&history));
    }

    #[test]
    fn test_concurrent_writes_all_valid() {
        let checker = LinearizabilityChecker::new();

        // Two concurrent writes and a read that sees the later one.
        //   P0: W(x, "a") at t=1..4
        //   P1:    W(x, "b") at t=2..3
        //   P2:             R(x) -> "a" at t=5..6
        //
        // Linearization: W(a) then W(b) then R -> should see "b".
        // But the read sees "a", so this should fail under end-time ordering.
        let history = vec![
            Operation {
                process_id: 0,
                op_type: OperationType::Write,
                key: "x".to_string(),
                value: Some("a".to_string()),
                start_time: 1,
                end_time: 4,
            },
            Operation {
                process_id: 1,
                op_type: OperationType::Write,
                key: "x".to_string(),
                value: Some("b".to_string()),
                start_time: 2,
                end_time: 3,
            },
            Operation {
                process_id: 2,
                op_type: OperationType::Read,
                key: "x".to_string(),
                value: Some("a".to_string()),
                start_time: 5,
                end_time: 6,
            },
        ];

        // With end_time ordering, W(b) ends at 3, W(a) ends at 4, so the
        // linearization is W(b), W(a), R(x)->"a" which is valid.
        assert!(checker.check(&history));
    }

    #[test]
    fn test_exhaustive_linear_history_passes() {
        let checker = LinearizabilityChecker::new();
        let history = build_valid_history();
        assert!(checker.check_exhaustive(&history));
    }

    #[test]
    fn test_exhaustive_non_linear_history_fails() {
        let checker = LinearizabilityChecker::new();
        let history = build_invalid_history();
        assert!(!checker.check_exhaustive(&history));
    }

    #[test]
    fn test_empty_history_is_linearizable() {
        let checker = LinearizabilityChecker::new();
        assert!(checker.check(&[]));
    }

    #[test]
    fn test_single_write_is_linearizable() {
        let checker = LinearizabilityChecker::new();
        let history = vec![Operation {
            process_id: 0,
            op_type: OperationType::Write,
            key: "x".to_string(),
            value: Some("hello".to_string()),
            start_time: 1,
            end_time: 2,
        }];
        assert!(checker.check(&history));
    }

    #[test]
    fn test_read_own_write() {
        let checker = LinearizabilityChecker::new();
        // P0 writes, then P0 reads its own write back.
        let history = vec![
            Operation {
                process_id: 0,
                op_type: OperationType::Write,
                key: "x".to_string(),
                value: Some("val".to_string()),
                start_time: 1,
                end_time: 2,
            },
            Operation {
                process_id: 0,
                op_type: OperationType::Read,
                key: "x".to_string(),
                value: Some("val".to_string()),
                start_time: 3,
                end_time: 4,
            },
        ];
        assert!(checker.check(&history));
    }
}
