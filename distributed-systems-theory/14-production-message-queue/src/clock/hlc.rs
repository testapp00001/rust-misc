//! Hybrid Logical Clock (HLC) implementation.
//!
//! Combines physical time with a logical counter to produce timestamps that
//! are both wall-clock-close and totally ordered across nodes.  The algorithm
//! follows the paper by Kulkarni et al. (2014).

use serde::{Deserialize, Serialize};
use std::cmp::max;

/// A Hybrid Logical Clock timestamp.
///
/// Ordering is lexicographic: first by `physical`, then by `logical`, then
/// by `node_id` (for tie-breaking between nodes that incremented at the
/// same logical tick).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HLCTimestamp {
    pub physical: u64,
    pub logical: u16,
    pub node_id: u16,
}

/// A Hybrid Logical Clock that produces monotonically non-decreasing
/// timestamps tied to (but not bound by) wall-clock time.
pub struct HybridLogicalClock {
    physical: u64,
    logical: u16,
    node_id: u16,
}

impl HybridLogicalClock {
    /// Create a new HLC anchored to the given node.
    pub fn new(node_id: u16) -> Self {
        HybridLogicalClock {
            physical: 0,
            logical: 0,
            node_id,
        }
    }

    /// Generate a new timestamp for a local event at the given wall-clock
    /// time (milliseconds since Unix epoch).
    pub fn now(&mut self, wall_clock_ms: u64) -> HLCTimestamp {
        if wall_clock_ms > self.physical {
            self.physical = wall_clock_ms;
            self.logical = 0;
        } else {
            self.logical += 1;
        }
        HLCTimestamp {
            physical: self.physical,
            logical: self.logical,
            node_id: self.node_id,
        }
    }

    /// Update the clock upon receiving a remote timestamp, then produce
    /// a new local timestamp for the receive event.
    pub fn receive(
        &mut self,
        wall_clock_ms: u64,
        remote: HLCTimestamp,
    ) -> HLCTimestamp {
        if wall_clock_ms > self.physical && wall_clock_ms > remote.physical {
            // Wall clock is ahead of both local and remote.
            self.physical = wall_clock_ms;
            self.logical = 0;
        } else if remote.physical > self.physical {
            // Remote physical time is ahead.
            self.physical = remote.physical;
            self.logical = remote.logical + 1;
        } else if self.physical == remote.physical {
            // Same physical time -- take the max logical and increment.
            self.logical = max(self.logical, remote.logical) + 1;
        } else {
            // Local physical time is ahead -- just bump logical.
            self.logical += 1;
        }
        HLCTimestamp {
            physical: self.physical,
            logical: self.logical,
            node_id: self.node_id,
        }
    }

    /// Return the current clock state without advancing it.
    pub fn current(&self) -> HLCTimestamp {
        HLCTimestamp {
            physical: self.physical,
            logical: self.logical,
            node_id: self.node_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_timestamp_is_zero() {
        let clock = HybridLogicalClock::new(1);
        let ts = clock.current();
        assert_eq!(ts.physical, 0);
        assert_eq!(ts.logical, 0);
        assert_eq!(ts.node_id, 1);
    }

    #[test]
    fn now_advances_physical_clock() {
        let mut clock = HybridLogicalClock::new(1);
        let t1 = clock.now(1000);
        assert_eq!(t1.physical, 1000);
        assert_eq!(t1.logical, 0);

        let t2 = clock.now(2000);
        assert_eq!(t2.physical, 2000);
        assert_eq!(t2.logical, 0);
    }

    #[test]
    fn now_increments_logical_on_same_physical() {
        let mut clock = HybridLogicalClock::new(1);
        let t1 = clock.now(1000);
        let t2 = clock.now(1000);
        let t3 = clock.now(1000);

        assert_eq!(t1.logical, 0);
        assert_eq!(t2.logical, 1);
        assert_eq!(t3.logical, 2);
    }

    #[test]
    fn now_uses_wall_clock_when_ahead() {
        let mut clock = HybridLogicalClock::new(1);
        clock.now(1000);
        clock.now(1000); // logical = 1

        let t = clock.now(2000); // wall clock ahead -> reset logical
        assert_eq!(t.physical, 2000);
        assert_eq!(t.logical, 0);
    }

    #[test]
    fn receive_with_wall_clock_ahead_of_both() {
        let mut clock1 = HybridLogicalClock::new(1);
        let mut clock2 = HybridLogicalClock::new(2);

        let t1 = clock1.now(1000);
        let t2 = clock2.receive(5000, t1);

        assert_eq!(t2.physical, 5000);
        assert_eq!(t2.logical, 0);
    }

    #[test]
    fn receive_with_remote_ahead() {
        let mut clock1 = HybridLogicalClock::new(1);
        let mut clock2 = HybridLogicalClock::new(2);

        let t1 = clock1.now(3000);
        let t2 = clock2.receive(1000, t1);

        assert_eq!(t2.physical, 3000);
        assert_eq!(t2.logical, 1);
    }

    #[test]
    fn receive_with_same_physical_time() {
        let mut clock1 = HybridLogicalClock::new(1);
        let mut clock2 = HybridLogicalClock::new(2);

        let t1 = clock1.now(1000);
        // clock2 also at physical 1000, logical 5
        clock2.now(1000); // logical 0 (physical jumps to 1000)
        clock2.now(1000); // logical 1
        clock2.now(1000); // logical 2
        clock2.now(1000); // logical 3
        clock2.now(1000); // logical 4
        clock2.now(1000); // logical 5
        let t2 = clock2.receive(1000, t1);

        assert_eq!(t2.physical, 1000);
        // max(5, 0) + 1 = 6
        assert_eq!(t2.logical, 6);
    }

    #[test]
    fn receive_with_local_ahead() {
        let mut clock1 = HybridLogicalClock::new(1);
        let mut clock2 = HybridLogicalClock::new(2);

        let t1 = clock1.now(500); // remote at physical 500
        // clock2 is at physical 2000
        clock2.now(2000);
        let t2 = clock2.receive(1000, t1); // wall clock 1000 < 2000, remote 500 < 2000

        assert_eq!(t2.physical, 2000);
        // local logical was 0, now incremented to 1
        assert_eq!(t2.logical, 1);
    }

    #[test]
    fn timestamps_are_totally_ordered() {
        let mut clock1 = HybridLogicalClock::new(1);
        let mut clock2 = HybridLogicalClock::new(2);

        let t1 = clock1.now(1000);
        let t2 = clock2.receive(1000, t1);
        let t3 = clock1.receive(1000, t2);

        assert!(t1 < t2);
        assert!(t2 < t3);
    }

    #[test]
    fn ordering_across_nodes_is_consistent() {
        let mut c1 = HybridLogicalClock::new(1);
        let mut c2 = HybridLogicalClock::new(2);

        // Simulate a message exchange
        let a1 = c1.now(100);
        let b1 = c2.receive(150, a1);
        let a2 = c1.receive(200, b1);

        assert!(a1 < b1);
        assert!(b1 < a2);
    }

    #[test]
    fn node_id_is_preserved() {
        let clock = HybridLogicalClock::new(42);
        let ts = clock.current();
        assert_eq!(ts.node_id, 42);
    }
}
