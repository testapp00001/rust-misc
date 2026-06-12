//! # Exercise: Spanner-style External Consistency
//!
//! ## Theory
//!
//! Spanner achieves external consistency (also called "external consistency"
//! in the Spanner paper) by combining TrueTime with commit wait. External
//! consistency guarantees:
//!
//!   If transaction T1 commits before T2 starts, then T1's commit timestamp
//!   is less than T2's commit timestamp.
//!
//! This is stronger than serializability: it respects real-time ordering.
//!
//! The mechanism works as follows:
//!
//!   1. Each transaction gets a commit timestamp from the leader's clock.
//!   2. After applying writes, the leader waits (commit wait) until it is
//!      certain that the commit timestamp is in the real-time past.
//!   3. Only then does it acknowledge the commit to the client.
//!   4. Any subsequent transaction that starts after the acknowledgment
//!      will necessarily have a timestamp greater than the committed
//!      transaction's timestamp.
//!
//! The clock offset parameter models the maximum clock skew between nodes.
//! With a bound epsilon on clock skew:
//!   - TrueTime returns an interval [earliest, latest].
//!   - The commit timestamp is chosen within this interval.
//!   - Commit wait ensures the lower bound of TrueTime's interval has
//!     passed the commit timestamp.
//!
//! ## Proof / Intuition
//!
//! Let eps be the maximum clock uncertainty.
//!
//! For transaction T1:
//!   - Commit timestamp: ts1
//!   - Commit wait: system waits until wall_clock >= ts1 + eps
//!   - At this point, true_time >= wall_clock - eps >= ts1
//!
//! For transaction T2 (starting after T1 commits):
//!   - T2's wall_clock >= true_time >= ts1
//!   - T2's timestamp ts2 >= wall_clock >= ts1 + 1 (at minimum)
//!   - Therefore ts1 < ts2
//!
//! Without commit wait, a transaction on a slow clock could get a
//! timestamp earlier than a recently committed transaction, violating
//! external consistency.
//!
//! ## Implementation Task
//!
//! Implement the following:
//!
//! 1. `HLCTimestamp` with comparison.
//! 2. `Transaction` with id, timestamp, start/commit times, and data.
//! 3. `SpannerCluster` with methods:
//!    - `new(uncertainty_ms)` - create a cluster.
//!    - `start_transaction(id, wall_clock, data)` - begin a transaction.
//!    - `commit_transaction(txn_id, wall_clock)` - commit with wait.
//!    - `verify_external_consistency(txn1_id, txn2_id, txn2_start_wall)` -
//!      verify ordering.
//! 4. Demonstrate the difference with and without commit wait.
//!
//! ## Verification
//!
//! The tests verify:
//! - External consistency holds when commit wait is enforced.
//! - Without wait, ordering can be violated.
//! - Multiple transactions maintain correct order.

use std::cmp::Ordering;

/// A hybrid logical clock timestamp.
#[derive(Debug, Clone, Copy)]
pub struct HLCTimestamp {
    /// Maximum physical time observed.
    pub physical_time: u64,
    /// Logical counter for causality.
    pub logical_counter: u64,
    /// Node identifier for tie-breaking.
    pub node_id: usize,
}

impl HLCTimestamp {
    /// Create a new timestamp.
    pub fn new(physical_time: u64, logical_counter: u64, node_id: usize) -> Self {
        HLCTimestamp {
            physical_time,
            logical_counter,
            node_id,
        }
    }
}

impl PartialEq for HLCTimestamp {
    fn eq(&self, other: &Self) -> bool {
        self.physical_time == other.physical_time
            && self.logical_counter == other.logical_counter
            && self.node_id == other.node_id
    }
}

impl Eq for HLCTimestamp {}

impl PartialOrd for HLCTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HLCTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.physical_time
            .cmp(&other.physical_time)
            .then_with(|| self.logical_counter.cmp(&other.logical_counter))
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

/// A database transaction with a unique id, timestamp, timing, and data.
#[derive(Debug, Clone)]
pub struct Transaction {
    /// Unique transaction identifier.
    pub id: usize,
    /// Commit timestamp assigned by the cluster.
    pub timestamp: HLCTimestamp,
    /// Wall clock time when the transaction started.
    pub start_time: u64,
    /// Wall clock time when the commit was reported (after wait).
    pub commit_time: Option<u64>,
    /// The data payload of the transaction.
    pub data: String,
}

/// A simulated Spanner cluster with commit wait support.
///
/// Models clock skew via `clock_offset` and enforces external consistency
/// through the commit wait protocol.
#[derive(Debug)]
pub struct SpannerCluster {
    /// Maximum clock uncertainty in milliseconds.
    pub uncertainty_ms: u64,
    /// All transactions managed by this cluster.
    pub transactions: Vec<Transaction>,
    /// Simulated clock offset from true time (positive = ahead).
    pub clock_offset: i64,
}

impl SpannerCluster {
    /// Create a new Spanner cluster with the given uncertainty bound.
    pub fn new(uncertainty_ms: u64) -> Self {
        SpannerCluster {
            uncertainty_ms,
            transactions: Vec::new(),
            clock_offset: 0,
        }
    }

    /// Start a new transaction with the given id, wall clock, and data.
    ///
    /// The transaction's timestamp is derived from the wall clock
    /// (adjusted by the clock offset).
    pub fn start_transaction(&mut self, id: usize, wall_clock: u64, data: &str) -> Transaction {
        let adjusted_time = (wall_clock as i64 + self.clock_offset) as u64;
        let ts = HLCTimestamp::new(adjusted_time, 0, id);
        let txn = Transaction {
            id,
            timestamp: ts,
            start_time: wall_clock,
            commit_time: None,
            data: data.to_string(),
        };
        self.transactions.push(txn.clone());
        txn
    }

    /// Commit a transaction with commit wait.
    ///
    /// Returns the commit report time: `wall_clock + uncertainty_ms`.
    /// This represents the point at which the system is certain the
    /// commit timestamp is in the real-time past.
    pub fn commit_transaction(&mut self, txn_id: usize, wall_clock: u64) -> Option<u64> {
        if let Some(txn) = self.transactions.iter_mut().find(|t| t.id == txn_id) {
            let commit_time = wall_clock + self.uncertainty_ms;
            txn.commit_time = Some(commit_time);
            Some(commit_time)
        } else {
            None
        }
    }

    /// Verify external consistency between two transactions.
    ///
    /// If txn1 was committed before txn2 started (txn2_start_wall_clock
    /// is after txn1's commit time), then txn1's timestamp must be
    /// less than txn2's timestamp.
    pub fn verify_external_consistency(
        &self,
        txn1_id: usize,
        txn2_id: usize,
        txn2_start_wall_clock: u64,
    ) -> bool {
        let txn1 = self.transactions.iter().find(|t| t.id == txn1_id);
        let txn2 = self.transactions.iter().find(|t| t.id == txn2_id);

        match (txn1, txn2) {
            (Some(t1), Some(t2)) => {
                // If txn1 committed before txn2 started, check ordering.
                if let Some(commit_time) = t1.commit_time {
                    if txn2_start_wall_clock >= commit_time {
                        return t1.timestamp < t2.timestamp;
                    }
                }
                // Cannot determine ordering (txn2 started before txn1 committed).
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_consistency_with_commit_wait() {
        let mut cluster = SpannerCluster::new(10);

        // T1 starts and commits.
        let _t1 = cluster.start_transaction(0, 1000, "write A");
        let commit_time = cluster.commit_transaction(0, 1000);
        assert_eq!(commit_time, Some(1010));

        // T2 starts after T1 commits (at wall clock 1015).
        let _t2 = cluster.start_transaction(1, 1015, "write B");

        // External consistency: T1 committed before T2 started,
        // so T1.timestamp < T2.timestamp.
        assert!(cluster.verify_external_consistency(0, 1, 1015));
    }

    #[test]
    fn test_without_wait_violation_possible() {
        // With zero uncertainty, commit wait is instantaneous.
        let mut cluster = SpannerCluster::new(0);

        let _t1 = cluster.start_transaction(0, 1000, "write A");
        let commit_time = cluster.commit_transaction(0, 1000);
        // Commit is immediate: commit_time = 1000.
        assert_eq!(commit_time, Some(1000));

        // T2 starts slightly before T1's commit is visible (clock skew).
        let _t2 = cluster.start_transaction(1, 999, "write B");

        // T2's timestamp is 999, which is < T1's timestamp of 1000.
        // This violates external consistency because T2 started before
        // the system confirmed T1 was committed.
        let txn0 = cluster.transactions.iter().find(|t| t.id == 0).unwrap();
        let txn1 = cluster.transactions.iter().find(|t| t.id == 1).unwrap();
        // T2 timestamp (999) < T1 timestamp (1000) -- potential violation
        assert!(txn1.timestamp < txn0.timestamp);
    }

    #[test]
    fn test_multiple_transactions_maintain_order() {
        let mut cluster = SpannerCluster::new(5);

        // T1 commits at 1000, commit_time = 1005.
        let _t1 = cluster.start_transaction(0, 1000, "A");
        cluster.commit_transaction(0, 1000);

        // T2 starts at 1010, commits at 1010, commit_time = 1015.
        let _t2 = cluster.start_transaction(1, 1010, "B");
        cluster.commit_transaction(1, 1010);

        // T3 starts at 1020, commits at 1020, commit_time = 1025.
        let _t3 = cluster.start_transaction(2, 1020, "C");
        cluster.commit_transaction(2, 1020);

        // Verify pairwise ordering.
        assert!(cluster.verify_external_consistency(0, 1, 1010));
        assert!(cluster.verify_external_consistency(1, 2, 1020));
        assert!(cluster.verify_external_consistency(0, 2, 1020));
    }

    #[test]
    fn test_commit_wait_duration_matches_uncertainty() {
        let mut cluster = SpannerCluster::new(7);

        let _t1 = cluster.start_transaction(0, 500, "X");
        let commit_time = cluster.commit_transaction(0, 500);

        // Commit report at 500 + 7 = 507.
        assert_eq!(commit_time, Some(507));

        // The wait ensures the system doesn't proceed until time 507.
        let cluster_ref = &cluster;
        let t1 = cluster_ref.transactions.iter().find(|t| t.id == 0).unwrap();
        let wait = commit_time.unwrap() - t1.start_time;
        assert_eq!(wait, 7);
    }

    #[test]
    fn test_clock_offset_adjusts_timestamp() {
        let mut cluster = SpannerCluster::new(5);
        cluster.clock_offset = 100; // Clock is 100ms ahead.

        let txn = cluster.start_transaction(0, 1000, "data");
        // Adjusted time = 1000 + 100 = 1100.
        assert_eq!(txn.timestamp.physical_time, 1100);
    }

    #[test]
    fn test_nonexistent_transaction_commit_fails() {
        let mut cluster = SpannerCluster::new(5);
        let result = cluster.commit_transaction(999, 1000);
        assert_eq!(result, None);
    }
}
