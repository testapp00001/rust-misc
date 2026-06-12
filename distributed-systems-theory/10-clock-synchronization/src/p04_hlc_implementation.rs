//! # Exercise: Hybrid Logical Clocks (HLC)
//!
//! ## Theory
//!
//! Hybrid Logical Clocks (HLC), introduced by Kulkarni et al. (2014), combine
//! the benefits of physical and logical clocks:
//!
//! - **Physical time**: Timestamps are close to real wall-clock time, making
//!   them useful for diagnostics, caching TTLs, and debugging.
//! - **Logical time**: Causality is tracked via a logical counter, ensuring
//!   that if event A happened-before event B, then HLC(A) < HLC(B).
//!
//! An HLC timestamp consists of two components:
//!   - `physical_time (pt)`: tracks the wall-clock time, updated on each event
//!   - `logical_counter (lc)`: a monotonically increasing counter for events
//!     within the same physical time unit
//!
//! The HLC algorithm:
//!
//! **On local event:**
//!   pt' = max(wall_clock, pt)
//!   if pt' == pt: lc' = lc + 1
//!   else:         lc' = 0  (wall clock advanced, reset counter)
//!
//! **On receiving a message with timestamp (pt_remote, lc_remote):**
//!   pt' = max(wall_clock, pt, pt_remote)
//!   if pt' == pt AND pt' == pt_remote: lc' = max(lc, lc_remote) + 1
//!   elif pt' == pt:   lc' = lc + 1
//!   elif pt' == pt_remote: lc' = lc_remote + 1
//!   else:             lc' = 0
//!
//! Key properties:
//!   1. Timestamps are monotonically non-decreasing
//!   2. Causality is preserved: if A happened-before B, then ts(A) < ts(B)
//!   3. Timestamps are close to physical time (within the clock skew bound)
//!   4. The protocol is symmetric - no central authority needed
//!
//! ## Proof / Intuition
//!
//! The max operation ensures that when two nodes communicate, the receiver's
//! timestamp is at least as large as the sender's. The counter handles the
//! case where multiple events occur within the same physical time tick.
//!
//! Causality preservation follows from:
//!   - If A happened-before B, there's a causal chain A -> ... -> B
//!   - Each link in the chain causes the timestamp to increase
//!   - By transitivity, ts(A) < ts(B)
//!
//! ## Implementation Task
//!
//! Implement `HLC` and `HLCTimestamp` with:
//!
//! 1. `HLCTimestamp`: a comparable timestamp with physical_time, logical_counter,
//!    and node_id.
//! 2. `HLC::new(node_id)`: create a new HLC for the given node.
//! 3. `HLC::local_event(&mut self, wall_clock)`: generate timestamp for a
//!    local event.
//! 4. `HLC::receive(&mut self, wall_clock, remote_pt, remote_lc)`: generate
//!    timestamp when receiving a remote message.
//! 5. `HLC::current_timestamp(&self)`: return the current internal timestamp.
//!
//! ## Verification
//!
//! The tests verify:
//! - Timestamps are monotonically non-decreasing.
//! - Causality is preserved (happened-before implies timestamp ordering).
//! - Timestamps track physical time closely.
use std::cmp::Ordering;

/// A Hybrid Logical Clock timestamp.
///
/// Consists of a physical component (tracking wall clock) and a logical
/// counter (for events within the same physical time unit). The node_id
/// provides deterministic tiebreaking.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HLCTimestamp {
    /// Physical time component (wall clock).
    pub physical_time: u64,
    /// Logical counter for events within the same physical time.
    pub logical_counter: u64,
    /// Node identifier for deterministic tiebreaking.
    pub node_id: usize,
}

impl HLCTimestamp {
    /// Create a new HLC timestamp.
    pub fn new(physical_time: u64, logical_counter: u64, node_id: usize) -> Self {
        HLCTimestamp {
            physical_time,
            logical_counter,
            node_id,
        }
    }
}

/// Ordering for HLC timestamps.
///
/// Compares by physical time first, then logical counter, then node_id.
/// This ensures a total ordering: every pair of timestamps is comparable.
impl Ord for HLCTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        self.physical_time
            .cmp(&other.physical_time)
            .then_with(|| self.logical_counter.cmp(&other.logical_counter))
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

impl PartialOrd for HLCTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A Hybrid Logical Clock.
///
/// Maintains local state (physical_time, logical_counter) and generates
/// timestamps for local events and received messages.
#[derive(Debug, Clone)]
pub struct HLC {
    /// This node's identifier.
    pub node_id: usize,
    /// Last known physical time.
    pub physical_time: u64,
    /// Last logical counter value.
    pub logical_counter: u64,
}

impl HLC {
    /// Create a new HLC for the given node.
    ///
    /// Starts with physical_time = 0 and logical_counter = 0.
    pub fn new(node_id: usize) -> Self {
        HLC {
            node_id,
            physical_time: 0,
            logical_counter: 0,
        }
    }

    /// Generate a timestamp for a local event.
    ///
    /// Algorithm:
    ///   pt' = max(wall_clock, physical_time)
    ///   if pt' == physical_time: lc' = logical_counter + 1
    ///   else:                   lc' = 0
    ///
    /// The wall_clock is the current local wall clock reading.
    pub fn local_event(&mut self, wall_clock: u64) -> HLCTimestamp {
        let pt = std::cmp::max(wall_clock, self.physical_time);

        let lc = if pt == self.physical_time {
            self.logical_counter + 1
        } else {
            0
        };

        self.physical_time = pt;
        self.logical_counter = lc;

        HLCTimestamp::new(pt, lc, self.node_id)
    }

    /// Generate a timestamp when receiving a remote message.
    ///
    /// Algorithm:
    ///   pt' = max(wall_clock, physical_time, remote_pt)
    ///   if pt' == physical_time AND pt' == remote_pt:
    ///       lc' = max(logical_counter, remote_lc) + 1
    ///   elif pt' == physical_time:
    ///       lc' = logical_counter + 1
    ///   elif pt' == remote_pt:
    ///       lc' = remote_lc + 1
    ///   else:
    ///       lc' = 0
    pub fn receive(
        &mut self,
        wall_clock: u64,
        remote_pt: u64,
        remote_lc: u64,
    ) -> HLCTimestamp {
        let pt = std::cmp::max(
            wall_clock,
            std::cmp::max(self.physical_time, remote_pt),
        );

        let lc = if pt == self.physical_time && pt == remote_pt {
            std::cmp::max(self.logical_counter, remote_lc) + 1
        } else if pt == self.physical_time {
            self.logical_counter + 1
        } else if pt == remote_pt {
            remote_lc + 1
        } else {
            0
        };

        self.physical_time = pt;
        self.logical_counter = lc;

        HLCTimestamp::new(pt, lc, self.node_id)
    }

    /// Return the current internal timestamp without modifying state.
    pub fn current_timestamp(&self) -> HLCTimestamp {
        HLCTimestamp::new(self.physical_time, self.logical_counter, self.node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_ordering_by_physical_time() {
        let a = HLCTimestamp::new(100, 0, 0);
        let b = HLCTimestamp::new(200, 0, 0);
        assert!(a < b);
        assert!(b > a);
    }

    #[test]
    fn test_timestamp_ordering_by_logical_counter() {
        let a = HLCTimestamp::new(100, 1, 0);
        let b = HLCTimestamp::new(100, 2, 0);
        assert!(a < b);
    }

    #[test]
    fn test_timestamp_ordering_by_node_id() {
        let a = HLCTimestamp::new(100, 0, 1);
        let b = HLCTimestamp::new(100, 0, 2);
        assert!(a < b);
    }

    #[test]
    fn test_timestamp_total_ordering() {
        let a = HLCTimestamp::new(100, 5, 1);
        let b = HLCTimestamp::new(100, 5, 1);
        assert_eq!(a, b);
        assert!(!(a < b));
        assert!(!(a > b));
    }

    #[test]
    fn test_local_event_monotonicity() {
        let mut hlc = HLC::new(0);

        let t1 = hlc.local_event(100);
        let t2 = hlc.local_event(100);
        let t3 = hlc.local_event(100);

        assert!(t1 < t2);
        assert!(t2 < t3);
    }

    #[test]
    fn test_local_event_advances_with_wall_clock() {
        let mut hlc = HLC::new(0);

        let t1 = hlc.local_event(100);
        assert_eq!(t1.physical_time, 100);
        assert_eq!(t1.logical_counter, 0);

        let t2 = hlc.local_event(200);
        assert_eq!(t2.physical_time, 200);
        assert_eq!(t2.logical_counter, 0); // reset because wall clock advanced
    }

    #[test]
    fn test_local_event_counter_increment() {
        let mut hlc = HLC::new(0);

        let _t1 = hlc.local_event(100);
        let t2 = hlc.local_event(100);
        let t3 = hlc.local_event(100);

        // Same wall clock time, counter increments
        assert_eq!(t2.logical_counter, 1);
        assert_eq!(t3.logical_counter, 2);
    }

    #[test]
    fn test_receive_advances_timestamp() {
        let mut hlc = HLC::new(0);
        let _local = hlc.local_event(100);

        // Receive a message from the future
        let received = hlc.receive(150, 200, 5);
        assert_eq!(received.physical_time, 200);
        assert_eq!(received.logical_counter, 6); // 5 + 1
    }

    #[test]
    fn test_receive_behind_wall_clock() {
        let mut hlc = HLC::new(0);
        let _local = hlc.local_event(100);

        // Wall clock is ahead of both local and remote
        let received = hlc.receive(300, 50, 3);
        assert_eq!(received.physical_time, 300);
        assert_eq!(received.logical_counter, 0); // reset
    }

    #[test]
    fn test_causality_preservation() {
        let mut hlc_a = HLC::new(1);
        let mut hlc_b = HLC::new(2);

        // Node A generates event
        let event_a = hlc_a.local_event(100);

        // Node B receives from A and generates response
        let event_b = hlc_b.receive(101, event_a.physical_time, event_a.logical_counter);

        // Causality: A happened before B's response
        assert!(event_a < event_b);
    }

    #[test]
    fn test_causality_chain() {
        let mut hlc_a = HLC::new(1);
        let mut hlc_b = HLC::new(2);
        let mut hlc_c = HLC::new(3);

        // A -> B -> C chain
        let t1 = hlc_a.local_event(100);
        let t2 = hlc_b.receive(101, t1.physical_time, t1.logical_counter);
        let t3 = hlc_c.receive(102, t2.physical_time, t2.logical_counter);

        // Timestamps should be strictly ordered along the causal chain
        assert!(t1 < t2);
        assert!(t2 < t3);
    }

    #[test]
    fn test_close_to_physical_time() {
        let mut hlc = HLC::new(0);

        // After syncing to wall clock 1000
        let t = hlc.local_event(1000);
        assert_eq!(t.physical_time, 1000);

        // After several events at same time, physical time stays close
        let _t2 = hlc.local_event(1000);
        let _t3 = hlc.local_event(1000);
        let t4 = hlc.local_event(1005);

        // Physical time jumped to wall clock, not far from reality
        assert_eq!(t4.physical_time, 1005);
    }

    #[test]
    fn test_current_timestamp() {
        let mut hlc = HLC::new(0);
        let _ = hlc.local_event(100);
        let _ = hlc.local_event(100);

        let current = hlc.current_timestamp();
        assert_eq!(current.physical_time, 100);
        // First event resets counter to 0 (wall clock advanced from initial 0),
        // second event increments to 1 (same physical time)
        assert_eq!(current.logical_counter, 1);
        assert_eq!(current.node_id, 0);
    }
}
