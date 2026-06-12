//! # Exercise: Sequential Consistency Checker
//!
//! ## Theory
//!
//! Sequential consistency requires that the result of any execution is the same
//! as if all operations were executed in some sequential order, and the
//! operations of each individual process appear in this sequence in program
//! order.
//!
//! Unlike linearizability, sequential consistency does NOT require real-time
//! ordering. If P1 completes before P2 starts, P2's operations can still
//! appear before P1's in the sequential order, as long as each process's own
//! operations remain in order.
//!
//! ## Proof / Intuition
//!
//! Given processes P1 and P2:
//!
//!   P1: W(x, 1) -> R(y) -> 0
//!   P2: W(y, 1) -> R(x) -> 0
//!
//! Under sequential consistency, the interleaving:
//!   W(x,1), W(y,1), R(y)->1, R(x)->1
//!   is valid, even though P2 started before P1 completed.
//!
//! But this would NOT be linearizable if P2's R(x) started before P1's W(x)
//! completed and returned 0 -- because real-time ordering would require P1's
//! write to appear first.
//!
//! ## Implementation Task
//!
//! Implement `SequentialConsistencyChecker::check()` which verifies that there
//! exists a total order of operations consistent with each process's program
//! order.
//!
//! The approach:
//! 1. For each process, record the ordered sequence of operations.
//! 2. Try to merge all process sequences into a single total order such that
//!    each process's operations appear in their original order.
//! 3. Verify the resulting total order is consistent with read/write semantics.
//!
//! ## Verification
//!
//! Run `cargo test sequential_consistency` to verify your implementation.

use std::collections::HashMap;

/// The type of operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpType {
    Read,
    Write,
}

/// A sequentially consistent operation with a per-process sequence number.
#[derive(Debug, Clone)]
pub struct SequentialOp {
    /// The process that performed this operation.
    pub process_id: usize,
    /// Whether this is a read or write.
    pub op_type: OpType,
    /// The key being accessed.
    pub key: String,
    /// For writes, the value written. For reads, the value observed.
    pub value: Option<String>,
    /// The position of this operation in the process's program order.
    pub seq_num: usize,
}

/// A sequential consistency checker.
#[derive(Debug, Default)]
pub struct SequentialConsistencyChecker;

impl SequentialConsistencyChecker {
    pub fn new() -> Self {
        Self
    }

    /// Check whether a history is sequentially consistent.
    ///
    /// Algorithm:
    /// 1. Group operations by process, sorted by seq_num.
    /// 2. For each key, group operations and try to find a total order that:
    ///    a. Respects each process's program order for that key.
    ///    b. Has every read return the value of the most recent write.
    /// 3. If any key fails, the history is not sequentially consistent.
    ///
    /// For simplicity, we check that for each key, when we interleave operations
    /// from different processes while respecting program order, every read can
    /// return the correct value.
    pub fn check(&self, history: &[SequentialOp], num_processes: usize) -> bool {
        if history.is_empty() {
            return true;
        }

        // Group operations by process, in program order.
        let mut by_process: Vec<Vec<&SequentialOp>> = vec![Vec::new(); num_processes];
        for op in history {
            if op.process_id < num_processes {
                by_process[op.process_id].push(op);
            }
        }
        for process_ops in &mut by_process {
            process_ops.sort_by_key(|op| op.seq_num);
        }

        // Group operations by key.
        let mut by_key: HashMap<String, Vec<&SequentialOp>> = HashMap::new();
        for op in history {
            by_key.entry(op.key.clone()).or_default().push(op);
        }

        // For each key, check if there is a valid total order.
        for (_key, key_ops) in &by_key {
            // Build per-process orderings for this key using owned clones.
            let mut process_seqs: HashMap<usize, Vec<SequentialOp>> = HashMap::new();
            for op in key_ops {
                process_seqs
                    .entry(op.process_id)
                    .or_default()
                    .push((*op).clone());
            }

            // Sort each process's ops by seq_num.
            for ops in process_seqs.values_mut() {
                ops.sort_by_key(|op| op.seq_num);
            }

            // Try all possible interleavings via recursive merge.
            if !self.can_merge_and_validate(&process_seqs, num_processes) {
                return false;
            }
        }

        true
    }

    /// Try to merge all process sequences for a key into a valid total order
    /// where every read returns the most recent write.
    fn can_merge_and_validate(
        &self,
        process_seqs: &HashMap<usize, Vec<SequentialOp>>,
        num_processes: usize,
    ) -> bool {
        // Build a list of (process_id, current_index) for each process that has ops.
        let mut heads: Vec<(usize, usize)> = Vec::new();
        for pid in 0..num_processes {
            if let Some(ops) = process_seqs.get(&pid) {
                if !ops.is_empty() {
                    heads.push((pid, 0));
                }
            }
        }

        self.try_merge(&heads, process_seqs, &mut Vec::new(), num_processes)
    }

    fn try_merge(
        &self,
        heads: &[(usize, usize)],
        process_seqs: &HashMap<usize, Vec<SequentialOp>>,
        merged: &mut Vec<SequentialOp>,
        num_processes: usize,
    ) -> bool {
        // Check if all processes are exhausted.
        if heads.iter().all(|(pid, idx)| {
            process_seqs
                .get(pid)
                .map_or(true, |ops| *idx >= ops.len())
        }) {
            return self.validate_merged(merged);
        }

        // Try advancing each process that still has ops.
        for (i, &(pid, idx)) in heads.iter().enumerate() {
            if let Some(ops) = process_seqs.get(&pid) {
                if idx < ops.len() {
                    let mut new_heads: Vec<(usize, usize)> = heads.to_vec();
                    new_heads[i] = (pid, idx + 1);
                    merged.push(ops[idx].clone());

                    if self.try_merge(&new_heads, process_seqs, merged, num_processes) {
                        return true;
                    }

                    merged.pop();
                }
            }
        }

        false
    }

    /// Validate that a merged sequence has every read returning the most recent
    /// write.
    fn validate_merged(&self, merged: &[SequentialOp]) -> bool {
        let mut last_write: HashMap<String, Option<String>> = HashMap::new();

        for op in merged {
            match op.op_type {
                OpType::Write => {
                    last_write.insert(op.key.clone(), op.value.clone());
                }
                OpType::Read => {
                    let expected = last_write.get(&op.key).cloned().flatten();
                    if op.value != expected {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// Build a valid sequentially consistent history.
///
///   P0: W(x, "a") [seq=0]  -> R(x) -> "a" [seq=1]
///   P1: W(x, "b") [seq=0]  -> R(x) -> "b" [seq=1]
///
/// Valid total order: W(x,"a"), W(x,"b"), R(x)->"b", R(x)->"b"
pub fn valid_history() -> Vec<SequentialOp> {
    vec![
        SequentialOp {
            process_id: 0,
            op_type: OpType::Write,
            key: "x".to_string(),
            value: Some("a".to_string()),
            seq_num: 0,
        },
        SequentialOp {
            process_id: 0,
            op_type: OpType::Read,
            key: "x".to_string(),
            value: Some("a".to_string()),
            seq_num: 1,
        },
        SequentialOp {
            process_id: 1,
            op_type: OpType::Write,
            key: "x".to_string(),
            value: Some("b".to_string()),
            seq_num: 0,
        },
        SequentialOp {
            process_id: 1,
            op_type: OpType::Read,
            key: "x".to_string(),
            value: Some("b".to_string()),
            seq_num: 1,
        },
    ]
}

/// Build an invalid history that violates sequential consistency.
///
///   P0: W(x, "a") [seq=0]  -> R(x) -> "a" [seq=1]
///   P1: R(x) -> "a" [seq=0]  -> W(x, "b") [seq=1]
///
/// P1 reads "a" then writes "b". In P1's program order, the read must come
/// before the write. But in the only valid interleaving where P1's read sees
/// "a", the total order must have W(x,"a") before R(x)->"a" before W(x,"b").
/// P0 reads "a" which is fine. This is actually valid.
///
/// Let's make a truly invalid one:
///   P0: W(x, "a") [seq=0] -> R(x) -> "b" [seq=1]
///   P1: W(x, "b") [seq=0]
///
/// P0 writes "a", then reads "b". For P0's program order, W(x,"a") must come
/// before R(x)->"b". So the total order must have W(x,"a") before R(x)->"b".
/// But there is no write of "b" between them (P1's W(x,"b") could be before
/// or after). If W(x,"b") is after R(x)->"b", then R(x)->"b" is wrong. If
/// W(x,"b") is before R(x)->"b", that could work... actually this depends on
/// P1's order.
///
/// Simplest invalid case: P0 reads a value it wrote as something else.
pub fn invalid_history() -> Vec<SequentialOp> {
    vec![
        SequentialOp {
            process_id: 0,
            op_type: OpType::Write,
            key: "x".to_string(),
            value: Some("a".to_string()),
            seq_num: 0,
        },
        SequentialOp {
            process_id: 0,
            op_type: OpType::Read,
            key: "x".to_string(),
            value: Some("b".to_string()),
            seq_num: 1,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sequential_history_passes() {
        let checker = SequentialConsistencyChecker::new();
        let history = valid_history();
        assert!(checker.check(&history, 2));
    }

    #[test]
    fn test_invalid_history_fails() {
        let checker = SequentialConsistencyChecker::new();
        let history = invalid_history();
        assert!(!checker.check(&history, 1));
    }

    #[test]
    fn test_empty_history_passes() {
        let checker = SequentialConsistencyChecker::new();
        assert!(checker.check(&[], 2));
    }

    #[test]
    fn test_single_process_sequential_order() {
        let checker = SequentialConsistencyChecker::new();
        // P0 writes then reads its own write.
        let history = vec![
            SequentialOp {
                process_id: 0,
                op_type: OpType::Write,
                key: "x".to_string(),
                value: Some("hello".to_string()),
                seq_num: 0,
            },
            SequentialOp {
                process_id: 0,
                op_type: OpType::Read,
                key: "x".to_string(),
                value: Some("hello".to_string()),
                seq_num: 1,
            },
        ];
        assert!(checker.check(&history, 1));
    }

    #[test]
    fn test_two_process_interleave() {
        let checker = SequentialConsistencyChecker::new();
        // P0: W(x, 1) -> R(y) -> 0
        // P1: W(y, 1) -> R(x) -> 1
        // This is sequentially consistent with order:
        // W(x,1), W(y,1), R(y)->1, R(x)->1
        let history = vec![
            SequentialOp {
                process_id: 0,
                op_type: OpType::Write,
                key: "x".to_string(),
                value: Some("1".to_string()),
                seq_num: 0,
            },
            SequentialOp {
                process_id: 0,
                op_type: OpType::Read,
                key: "y".to_string(),
                value: Some("1".to_string()),
                seq_num: 1,
            },
            SequentialOp {
                process_id: 1,
                op_type: OpType::Write,
                key: "y".to_string(),
                value: Some("1".to_string()),
                seq_num: 0,
            },
            SequentialOp {
                process_id: 1,
                op_type: OpType::Read,
                key: "x".to_string(),
                value: Some("1".to_string()),
                seq_num: 1,
            },
        ];
        assert!(checker.check(&history, 2));
    }

    #[test]
    fn test_read_returns_wrong_value_for_key() {
        let checker = SequentialConsistencyChecker::new();
        // P0: W(x, "a") then R(x) -> "z" (wrong!)
        let history = vec![
            SequentialOp {
                process_id: 0,
                op_type: OpType::Write,
                key: "x".to_string(),
                value: Some("a".to_string()),
                seq_num: 0,
            },
            SequentialOp {
                process_id: 0,
                op_type: OpType::Read,
                key: "x".to_string(),
                value: Some("z".to_string()),
                seq_num: 1,
            },
        ];
        assert!(!checker.check(&history, 1));
    }
}
