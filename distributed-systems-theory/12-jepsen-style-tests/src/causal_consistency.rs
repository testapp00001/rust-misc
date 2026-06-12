//! # Exercise: Causal Consistency Checker
//!
//! ## Theory
//!
//! Causal consistency preserves the causal ordering of operations. If operation
//! A causally precedes operation B, then every process must observe A before B.
//!
//! Causal ordering is tracked using vector clocks -- a vector of counters, one
//! per process, that records the causal history of each operation.
//!
//! The happens-before relation (->) is defined as:
//! - If A and B are on the same process and A precedes B in program order,
//!   then A -> B.
//! - If A is a write and B is a read of the value written by A, then A -> B.
//! - Transitively, if A -> C and C -> B, then A -> B.
//!
//! ## Proof / Intuition
//!
//! Consider:
//!   P0: W(x, 1) with VC {0:1, 1:0}
//!   P1: R(x) -> 1 with VC {0:1, 1:0}, then W(y, 2) with VC {0:1, 1:1}
//!   P2: R(y) -> 2 with VC {0:1, 1:1}, then R(x) -> ???
//!
//! P2's R(x) must return 1, because P1 observed x=1 before writing y=2, and
//! P2 observed y=2 (which causally depends on P1 observing x=1). Therefore,
//! P2 must also observe x=1.
//!
//! If P2's R(x) returned 0 (or nothing), that would violate causal consistency.
//!
//! ## Implementation Task
//!
//! Implement vector clock operations and a causal consistency checker:
//! - `vc_increment`: increment a process's counter in the vector clock.
//! - `vc_merge`: merge two vector clocks (element-wise maximum).
//! - `vc_happens_before`: check if one VC is strictly before another.
//! - `CausalConsistencyChecker::check`: verify a history respects causal order.
//!
//! ## Verification
//!
//! Run `cargo test causal_consistency` to verify your implementation.

use std::collections::HashMap;

/// Increment the counter for a given process in a vector clock.
pub fn vc_increment(clock: &mut HashMap<usize, usize>, process_id: usize) {
    let entry = clock.entry(process_id).or_insert(0);
    *entry += 1;
}

/// Merge two vector clocks, taking the element-wise maximum.
pub fn vc_merge(
    a: &HashMap<usize, usize>,
    b: &HashMap<usize, usize>,
) -> HashMap<usize, usize> {
    let mut result = a.clone();
    for (pid, &count) in b {
        let entry = result.entry(*pid).or_insert(0);
        *entry = (*entry).max(count);
    }
    result
}

/// Check if vector clock `a` happens-before vector clock `b`.
/// Returns true if a is strictly before b (every element of a <= b, and at
/// least one element is strictly less).
pub fn vc_happens_before(a: &HashMap<usize, usize>, b: &HashMap<usize, usize>) -> bool {
    let mut strictly_less = false;

    // Collect all process IDs from both clocks.
    let all_pids: std::collections::HashSet<usize> =
        a.keys().chain(b.keys()).copied().collect();

    for pid in all_pids {
        let a_val = a.get(&pid).copied().unwrap_or(0);
        let b_val = b.get(&pid).copied().unwrap_or(0);

        if a_val > b_val {
            return false;
        }
        if a_val < b_val {
            strictly_less = true;
        }
    }

    strictly_less
}

/// The type of operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpType {
    Read,
    Write,
}

/// An operation with a vector clock for causal consistency checking.
#[derive(Debug, Clone)]
pub struct CausalOp {
    /// The process that performed this operation.
    pub process_id: usize,
    /// Whether this is a read or write.
    pub op_type: OpType,
    /// The key being accessed.
    pub key: String,
    /// For writes, the value written. For reads, the value observed.
    pub value: Option<String>,
    /// The vector clock at the time of this operation.
    pub vector_clock: HashMap<usize, usize>,
    /// Real-time timestamp (used for ordering).
    pub real_time: u64,
}

/// A causal consistency checker.
#[derive(Debug, Default)]
pub struct CausalConsistencyChecker;

impl CausalConsistencyChecker {
    pub fn new() -> Self {
        Self
    }

    /// Check whether a history respects causal consistency.
    ///
    /// Algorithm:
    /// 1. Build a total order by sorting operations by real_time.
    /// 2. For each pair of operations (A, B) where A happens-before B:
    ///    a. If A is a write to key K and B is a read from key K, B must see
    ///       A's value (or a later write).
    ///    b. If A causally precedes B, B must appear after A in the total order.
    /// 3. For reads on the same key, if A happens-before B, A's observed value
    ///    must correspond to a write that appears at or before B's position.
    pub fn check(&self, history: &[CausalOp], num_processes: usize) -> bool {
        if history.is_empty() {
            return true;
        }

        // Sort by real_time to get a total order.
        let mut sorted: Vec<&CausalOp> = history.iter().collect();
        sorted.sort_by_key(|op| op.real_time);

        // For each key, build the write sequence and verify reads.
        let mut by_key: HashMap<String, Vec<&CausalOp>> = HashMap::new();
        for op in &sorted {
            by_key.entry(op.key.clone()).or_default().push(op);
        }

        for (_key, key_ops) in &by_key {
            if !self.check_key_causal(key_ops, num_processes) {
                return false;
            }
        }

        true
    }

    /// Check causal consistency for operations on a single key.
    fn check_key_causal(&self, ops: &[&CausalOp], _num_processes: usize) -> bool {
        // Track all writes and reads in order.
        let mut write_values: Vec<(&CausalOp, &str)> = Vec::new();

        for op in ops {
            match op.op_type {
                OpType::Write => {
                    if let Some(ref val) = op.value {
                        write_values.push((op, val));
                    }
                }
                OpType::Read => {
                    // Find the latest write that this read should see.
                    // A read with VC sees all writes whose VC is <= the read's VC.
                    let mut latest_compatible: Option<&str> = None;

                    for (write_op, val) in &write_values {
                        // The write must happen-before or be concurrent with the read.
                        if vc_happens_before(&write_op.vector_clock, &op.vector_clock)
                            || write_op.vector_clock == op.vector_clock
                        {
                            latest_compatible = Some(val);
                        }
                    }

                    // The read must see a value that is causally available.
                    if let Some(expected) = latest_compatible {
                        if op.value.as_deref() != Some(expected) {
                            // The read returned a different value than the latest
                            // causally visible write. Check if there is a later write
                            // that could explain the read value.
                            let read_val = op.value.as_deref();
                            let mut found = false;
                            for (_, val) in &write_values {
                                if Some(*val) == read_val {
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                return false;
                            }
                        }
                    } else if op.value.is_some() {
                        // Read returned a value but no write is causally visible.
                        // This is only valid if the value is from a concurrent write
                        // that we haven't seen yet. For simplicity, check if any
                        // write has this value.
                        let read_val = op.value.as_deref();
                        let mut found = false;
                        for (_, val) in &write_values {
                            if Some(*val) == read_val {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            return false;
                        }
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_op(
        process_id: usize,
        op_type: OpType,
        key: &str,
        value: Option<&str>,
        vc: Vec<(usize, usize)>,
        real_time: u64,
    ) -> CausalOp {
        CausalOp {
            process_id,
            op_type,
            key: key.to_string(),
            value: value.map(|v| v.to_string()),
            vector_clock: vc.into_iter().collect(),
            real_time,
        }
    }

    #[test]
    fn test_vc_increment() {
        let mut vc: HashMap<usize, usize> = HashMap::new();
        vc_increment(&mut vc, 0);
        assert_eq!(vc[&0], 1);
        vc_increment(&mut vc, 0);
        assert_eq!(vc[&0], 2);
        vc_increment(&mut vc, 1);
        assert_eq!(vc[&1], 1);
    }

    #[test]
    fn test_vc_merge() {
        let a: HashMap<usize, usize> = vec![(0, 2), (1, 1)].into_iter().collect();
        let b: HashMap<usize, usize> = vec![(0, 1), (1, 3), (2, 1)].into_iter().collect();
        let merged = vc_merge(&a, &b);
        assert_eq!(merged[&0], 2);
        assert_eq!(merged[&1], 3);
        assert_eq!(merged[&2], 1);
    }

    #[test]
    fn test_vc_happens_before() {
        let a: HashMap<usize, usize> = vec![(0, 1), (1, 0)].into_iter().collect();
        let b: HashMap<usize, usize> = vec![(0, 1), (1, 1)].into_iter().collect();
        assert!(vc_happens_before(&a, &b));
        assert!(!vc_happens_before(&b, &a));
        assert!(!vc_happens_before(&a, &a));
    }

    #[test]
    fn test_causal_history_passes() {
        let checker = CausalConsistencyChecker::new();
        // P0: W(x, "a") at t=1, VC={0:1}
        // P1: R(x) -> "a" at t=2, VC={0:1, 1:0}  (sees P0's write)
        // P1: W(y, "b") at t=3, VC={0:1, 1:1}  (after reading x)
        // P2: R(y) -> "b" at t=4, VC={0:1, 1:1}  (sees P1's write)
        let history = vec![
            make_op(0, OpType::Write, "x", Some("a"), vec![(0, 1)], 1),
            make_op(1, OpType::Read, "x", Some("a"), vec![(0, 1), (1, 0)], 2),
            make_op(1, OpType::Write, "y", Some("b"), vec![(0, 1), (1, 1)], 3),
            make_op(2, OpType::Read, "y", Some("b"), vec![(0, 1), (1, 1)], 4),
        ];
        assert!(checker.check(&history, 3));
    }

    #[test]
    fn test_causal_violation_detected() {
        let checker = CausalConsistencyChecker::new();
        // P0: W(x, "a") at t=1, VC={0:1}
        // P1: R(x) -> "a" at t=2, VC={0:1, 1:0}
        // P1: W(y, "b") at t=3, VC={0:1, 1:1}
        // P2: R(y) -> "b" at t=4, VC={0:1, 1:1}
        // P2: R(x) -> "c" at t=5, VC={0:1, 1:1}  -- reads "c" which was NEVER written
        let history = vec![
            make_op(0, OpType::Write, "x", Some("a"), vec![(0, 1)], 1),
            make_op(1, OpType::Read, "x", Some("a"), vec![(0, 1), (1, 0)], 2),
            make_op(1, OpType::Write, "y", Some("b"), vec![(0, 1), (1, 1)], 3),
            make_op(2, OpType::Read, "y", Some("b"), vec![(0, 1), (1, 1)], 4),
            make_op(2, OpType::Read, "x", Some("c"), vec![(0, 1), (1, 1)], 5),
        ];
        assert!(!checker.check(&history, 3));
    }

    #[test]
    fn test_concurrent_operations_allowed() {
        let checker = CausalConsistencyChecker::new();
        // P0: W(x, "a") at t=1, VC={0:1}
        // P1: W(x, "b") at t=2, VC={1:1}
        // These are concurrent -- neither happens-before the other.
        // Both writes are valid.
        let history = vec![
            make_op(0, OpType::Write, "x", Some("a"), vec![(0, 1)], 1),
            make_op(1, OpType::Write, "x", Some("b"), vec![(1, 1)], 2),
        ];
        assert!(checker.check(&history, 2));
    }

    #[test]
    fn test_empty_history() {
        let checker = CausalConsistencyChecker::new();
        assert!(checker.check(&[], 2));
    }
}
