//! # Exercise: Commit Wait (Spanner-style)
//!
//! ## Theory
//!
//! In Google's Spanner database, "commit wait" is the mechanism that
//! guarantees external consistency (also called "real-time consistency").
//! The key insight is:
//!
//!   If transaction T1 commits at time t1 and transaction T2 starts after
//!   T1 commits, then T2 must have a timestamp greater than t1.
//!
//! Without commit wait, this guarantee cannot hold because clocks are
//! imperfect. A node's clock might be slightly behind another node's clock,
//! causing T2 to get a timestamp earlier than T1 even though T2 started
//! later in real time.
//!
//! The commit wait protocol works as follows:
//!
//!   1. A transaction acquires a commit timestamp from TrueTime.
//!   2. After writing, the system waits until it is certain that the
//!      commit timestamp is in the past (i.e., the current time has
//!      advanced past the commit timestamp + uncertainty).
//!   3. Only then does it report success to the client.
//!
//! The wait duration equals the clock uncertainty interval (epsilon).
//! During this wait, the system guarantees that:
//!   - The commit timestamp is definitely in the real-time past.
//!   - Any subsequent transaction will observe a wall clock >= commit_time.
//!
//! ## Proof / Intuition
//!
//! Let epsilon be the maximum clock uncertainty (bound on clock skew).
//!
//!   1. Transaction T1 commits with timestamp ts1 = wall_clock + epsilon/2.
//!   2. System waits until wall_clock >= ts1 + epsilon/2.
//!   3. At this point, the true time is at least ts1 (since wall_clock
//!      is at most true_time + epsilon/2, so true_time >= wall_clock
//!      - epsilon/2 >= ts1).
//!   4. Any transaction T2 that starts after T1 commits will have a
//!      wall_clock >= true_time >= ts1, so ts2 > ts1.
//!
//! The cost is latency: every commit pays an additional epsilon of
//! waiting time. Spanner typically has epsilon ~ 1-7ms with GPS/atomic
//! clocks.
//!
//! ## Implementation Task
//!
//! Implement the following:
//!
//! 1. `HLCTimestamp` with comparison (physical, logical, node_id).
//! 2. `CommitWait` with methods:
//!    - `new(uncertainty_ms: u64)` - set the uncertainty interval.
//!    - `commit(timestamp) -> u64` - record a commit and return the
//!      real-time when the client is notified (ts + uncertainty).
//!    - `commit_wait(commit_timestamp) -> u64` - return the wait duration.
//!    - `can_proceed(current_time, commit_time) -> bool` - check if
//!      enough time has elapsed.
//!    - `verify_external_consistency(t1, t2) -> bool` - check that if
//!      t1 was committed before t2 started, t1 < t2.
//! 3. Show that without waiting, external consistency can be violated.
//!
//! ## Verification
//!
//! The tests verify:
//! - Wait duration equals the uncertainty interval.
//! - External consistency holds when commit wait is enforced.
//! - Without waiting, external consistency can be violated.

use std::cmp::Ordering;

/// A hybrid logical clock timestamp for Spanner-style commit wait.
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

/// Tracks committed transactions and enforces the commit wait protocol.
///
/// After a transaction commits, the system waits for the uncertainty
/// interval to elapse before reporting success, guaranteeing that the
/// commit timestamp is in the real-time past.
#[derive(Debug)]
pub struct CommitWait {
    /// Maximum clock uncertainty in milliseconds.
    pub uncertainty_ms: u64,
    /// Committed transactions: (timestamp, commit_report_time).
    pub committed_transactions: Vec<(HLCTimestamp, u64)>,
    /// Current simulated time.
    pub current_time: u64,
}

impl CommitWait {
    /// Create a new CommitWait with the given uncertainty interval.
    pub fn new(uncertainty_ms: u64) -> Self {
        CommitWait {
            uncertainty_ms,
            committed_transactions: Vec::new(),
            current_time: 0,
        }
    }

    /// Commit a transaction with the given timestamp.
    ///
    /// Returns the time when the commit is reported to the client.
    /// This is `timestamp.physical_time + uncertainty_ms`, representing
    /// the point at which the commit is guaranteed to be in the past.
    pub fn commit(&mut self, timestamp: HLCTimestamp) -> u64 {
        let commit_report_time = timestamp.physical_time + self.uncertainty_ms;
        self.current_time = commit_report_time;
        self.committed_transactions
            .push((timestamp, commit_report_time));
        commit_report_time
    }

    /// Return the wait duration for a commit.
    ///
    /// The wait duration equals the uncertainty interval. During this
    /// time the system waits for the commit timestamp to be definitely
    /// in the past.
    pub fn commit_wait(&self, _commit_timestamp: &HLCTimestamp) -> u64 {
        self.uncertainty_ms
    }

    /// Check whether the system can proceed past a given commit time.
    ///
    /// Returns true if `current_time >= commit_time`, meaning the
    /// uncertainty interval has elapsed and the commit is safe to report.
    pub fn can_proceed(&self, current_time: u64, commit_time: u64) -> bool {
        current_time >= commit_time
    }

    /// Verify external consistency between two timestamps.
    ///
    /// If t1 was committed before t2 started (in real time), then
    /// t1 should be ordered before t2. This checks:
    ///   t1 < t2 (as defined by the Ord implementation).
    pub fn verify_external_consistency(&self, t1: &HLCTimestamp, t2: &HLCTimestamp) -> bool {
        t1 < t2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wait_duration_equals_uncertainty() {
        let cw = CommitWait::new(10);
        let ts = HLCTimestamp::new(1000, 0, 0);
        assert_eq!(cw.commit_wait(&ts), 10);
    }

    #[test]
    fn test_commit_returns_timestamp_plus_uncertainty() {
        let mut cw = CommitWait::new(10);
        let ts = HLCTimestamp::new(1000, 0, 0);
        let report_time = cw.commit(ts);
        // Commit report time = 1000 + 10 = 1010
        assert_eq!(report_time, 1010);
    }

    #[test]
    fn test_can_proceed_after_wait() {
        let mut cw = CommitWait::new(10);
        let ts = HLCTimestamp::new(1000, 0, 0);
        let report_time = cw.commit(ts);

        // At time 1005 -- cannot proceed yet.
        assert!(!cw.can_proceed(1005, report_time));

        // At time 1010 -- can proceed.
        assert!(cw.can_proceed(1010, report_time));

        // At time 1020 -- can proceed.
        assert!(cw.can_proceed(1020, report_time));
    }

    #[test]
    fn test_external_consistency_with_commit_wait() {
        let mut cw = CommitWait::new(10);

        // T1 commits at physical time 1000.
        let t1 = HLCTimestamp::new(1000, 0, 0);
        let report_time = cw.commit(t1);
        // report_time = 1010

        // T2 starts after T1 commits, at wall clock 1015.
        // T2 gets timestamp 1015 (which is > 1000).
        let t2 = HLCTimestamp::new(1015, 0, 1);

        // External consistency: t1 < t2.
        assert!(cw.verify_external_consistency(&t1, &t2));

        // Can proceed check: T1's report time is 1010, current time is 1015.
        assert!(cw.can_proceed(1015, report_time));
    }

    #[test]
    fn test_without_wait_consistency_could_violate() {
        // Without commit wait, a fast transaction on a slow clock
        // could get a timestamp earlier than a previously committed
        // transaction.
        let mut cw_no_wait = CommitWait::new(0); // zero uncertainty = no wait

        let t1 = HLCTimestamp::new(1000, 0, 0);
        let report_time = cw_no_wait.commit(t1);
        // With zero uncertainty, report_time = 1000 (no waiting).

        // T2 starts "after" T1 but on a node whose clock is at 990.
        let t2 = HLCTimestamp::new(990, 0, 1);

        // Without waiting, t2 < t1 even though T2 started after T1 committed.
        // This violates external consistency.
        assert!(!cw_no_wait.verify_external_consistency(&t1, &t2));

        // With zero uncertainty, can_proceed passes immediately (no wait needed).
        // current_time >= commit_time means we can proceed.
        assert!(cw_no_wait.can_proceed(report_time, report_time));
    }

    #[test]
    fn test_multiple_commits_in_order() {
        let mut cw = CommitWait::new(5);

        let t1 = HLCTimestamp::new(100, 0, 0);
        let t2 = HLCTimestamp::new(200, 0, 1);
        let t3 = HLCTimestamp::new(300, 0, 2);

        let r1 = cw.commit(t1);
        let r2 = cw.commit(t2);
        let r3 = cw.commit(t3);

        // Each commit report time is physical + uncertainty.
        assert_eq!(r1, 105);
        assert_eq!(r2, 205);
        assert_eq!(r3, 305);

        // External consistency among all pairs.
        assert!(cw.verify_external_consistency(&t1, &t2));
        assert!(cw.verify_external_consistency(&t2, &t3));
        assert!(cw.verify_external_consistency(&t1, &t3));
    }
}
