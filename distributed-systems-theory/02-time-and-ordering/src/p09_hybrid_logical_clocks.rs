//! # Exercise: Hybrid Logical Clocks
//!
//! ## Theory
//!
//! **Hybrid Logical Clocks (HLC)** (Kulkarni et al., 2014) combine the benefits of
//! physical clocks and logical clocks. An HLC timestamp is a tuple (physical_time,
//! logical_counter) that:
//!
//! 1. Stays close to physical time (within clock synchronization bounds)
//! 2. Preserves causality (if a -> b, then HLC(a) < HLC(b))
//! 3. Supports tie-breaking when physical times are equal
//!
//! The HLC update rules:
//! - **Local event**: `(pt, lc) = (max(pt, physical_time), lc + 1)`
//! - **Receive**: `(pt, lc) = (max(pt, physical_time, received_pt), lc + 1)` if
//!   physical_time or received_pt contribute to the max; else `(pt, lc + 1)`
//!
//! More precisely:
//! - If `physical_time > pt` or `received_pt > pt`: `(max(physical_time, received_pt), 0)`
//! - Else: `(pt, lc + 1)`
//!
//! ## Proof / Intuition
//!
//! The HLC preserves causality because:
//! - On send: the sender's HLC is incremented and attached to the message
//! - On receive: the receiver's HLC is set to max of its own, physical time, and
//!   received HLC. This ensures the new HLC is greater than the sender's HLC.
//!
//! The physical time component keeps the HLC close to real time. The logical
//! counter handles the case where physical time hasn't advanced, ensuring
//! monotonicity and causality.
//!
//! ## Implementation Task
//!
//! Implement `HybridLogicalClock` with:
//! - `local_event(physical_time)` - update as per HLC paper
//! - `receive(physical_time, received_pt, received_lc)` - merge with received
//! - `timestamp()` - return (pt, lc)
//! - `precedes(other)` - compare two timestamps
//!
//! ## Verification
//!
//! - Verify causality preservation
//! - Verify close to physical time
//! - Verify monotonicity

/// A Hybrid Logical Clock timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HLCTimestamp {
    /// Physical time component (milliseconds since epoch).
    pub pt: u64,
    /// Logical counter component.
    pub lc: u64,
}

impl HLCTimestamp {
    /// Create a new timestamp.
    pub fn new(pt: u64, lc: u64) -> Self {
        Self { pt, lc }
    }

    /// Check if this timestamp strictly precedes another.
    ///
    /// Precedes if:
    /// - pt < other.pt, OR
    /// - pt == other.pt AND lc < other.lc
    pub fn precedes(&self, other: &HLCTimestamp) -> bool {
        self.pt < other.pt || (self.pt == other.pt && self.lc < other.lc)
    }

    /// Check if two timestamps are concurrent (equal in both components).
    pub fn is_concurrent_with(&self, other: &HLCTimestamp) -> bool {
        self.pt == other.pt && self.lc == other.lc
    }
}

/// A Hybrid Logical Clock for a node.
#[derive(Debug, Clone)]
pub struct HybridLogicalClock {
    /// The current physical time component.
    pt: u64,
    /// The current logical counter.
    lc: u64,
    /// The node's unique identifier.
    node_id: usize,
}

impl HybridLogicalClock {
    /// Create a new HLC for a node.
    pub fn new(node_id: usize, initial_physical_time: u64) -> Self {
        Self {
            pt: initial_physical_time,
            lc: 0,
            node_id,
        }
    }

    /// Process a local event at the given physical time.
    ///
    /// Update rules:
    /// - If physical_time > pt: pt = physical_time, lc = 0
    /// - Else: lc = lc + 1
    pub fn local_event(&mut self, physical_time: u64) -> HLCTimestamp {
        if physical_time > self.pt {
            self.pt = physical_time;
            self.lc = 0;
        } else {
            self.lc += 1;
        }
        HLCTimestamp::new(self.pt, self.lc)
    }

    /// Process a received message.
    ///
    /// Update rules:
    /// - If max(physical_time, received_pt) > pt:
    ///   pt = max(physical_time, received_pt), lc = 0
    /// - Else if max(physical_time, received_pt) == pt AND received_lc > lc:
    ///   lc = received_lc + 1
    /// - Else:
    ///   lc = lc + 1
    pub fn receive(
        &mut self,
        physical_time: u64,
        received_pt: u64,
        received_lc: u64,
    ) -> HLCTimestamp {
        let max_physical = physical_time.max(received_pt);

        if max_physical > self.pt {
            self.pt = max_physical;
            self.lc = 0;
        } else if max_physical == self.pt && received_lc > self.lc {
            self.lc = received_lc + 1;
        } else {
            self.lc += 1;
        }

        HLCTimestamp::new(self.pt, self.lc)
    }

    /// Get the current timestamp.
    pub fn timestamp(&self) -> HLCTimestamp {
        HLCTimestamp::new(self.pt, self.lc)
    }

    /// Get the node ID.
    pub fn node_id(&self) -> usize {
        self.node_id
    }

    /// Get the physical time component.
    pub fn physical_time(&self) -> u64 {
        self.pt
    }

    /// Get the logical counter component.
    pub fn logical_counter(&self) -> u64 {
        self.lc
    }

    /// Calculate how far the HLC is from physical time.
    pub fn offset_from_physical(&self, physical_time: u64) -> i64 {
        self.pt as i64 - physical_time as i64
    }
}

/// Simulate message passing between two HLC nodes and verify causality.
pub fn verify_hlc_causality(
    events: &[(usize, u64, Option<(u64, u64)>)], // (node_id, physical_time, received_hlc)
) -> Vec<HLCTimestamp> {
    let mut nodes: Vec<HybridLogicalClock> = Vec::new();

    // Initialize nodes
    let max_node_id = events.iter().map(|(id, _, _)| *id).max().unwrap_or(0);
    for i in 0..=max_node_id {
        nodes.push(HybridLogicalClock::new(i, 0));
    }

    let mut timestamps = Vec::new();

    for &(node_id, physical_time, received) in events {
        let ts = if let Some((recv_pt, recv_lc)) = received {
            nodes[node_id].receive(physical_time, recv_pt, recv_lc)
        } else {
            nodes[node_id].local_event(physical_time)
        };
        timestamps.push(ts);
    }

    timestamps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn causality_preservation() {
        let mut node0 = HybridLogicalClock::new(0, 1000);
        let mut node1 = HybridLogicalClock::new(1, 1000);

        // Node 0 does a local event
        let ts_send = node0.local_event(1000);

        // Node 1 receives the message
        let ts_recv = node1.receive(1001, ts_send.pt, ts_send.lc);

        // Causality: send precedes receive
        assert!(
            ts_send.precedes(&ts_recv),
            "send {ts_send:?} should precede receive {ts_recv:?}"
        );
    }

    #[test]
    fn close_to_physical_time() {
        let mut hlc = HybridLogicalClock::new(0, 10000);

        // Do many local events
        for _ in 0..100 {
            hlc.local_event(10000);
        }

        // HLC should be close to physical time (same pt, just higher lc)
        assert_eq!(hlc.physical_time(), 10000);
        assert_eq!(hlc.logical_counter(), 100);
    }

    #[test]
    fn monotonicity() {
        let mut hlc = HybridLogicalClock::new(0, 1000);

        let ts1 = hlc.local_event(1000);
        let ts2 = hlc.local_event(1000);
        let ts3 = hlc.local_event(1001);

        assert!(ts1.precedes(&ts2));
        assert!(ts2.precedes(&ts3));
    }

    #[test]
    fn physical_time_advances_resets_counter() {
        let mut hlc = HybridLogicalClock::new(0, 1000);

        // Do some events at time 1000
        hlc.local_event(1000);
        hlc.local_event(1000);
        assert_eq!(hlc.logical_counter(), 2);

        // Physical time advances
        hlc.local_event(2000);
        assert_eq!(hlc.physical_time(), 2000);
        assert_eq!(hlc.logical_counter(), 0);
    }

    #[test]
    fn receive_takes_max_physical_time() {
        let mut node = HybridLogicalClock::new(0, 1000);

        // Node is at pt=1000, lc=5
        for _ in 0..5 {
            node.local_event(1000);
        }

        // Receive message from far future
        let ts = node.receive(5000, 5000, 0);

        assert_eq!(ts.pt, 5000);
        assert_eq!(ts.lc, 0);
    }

    #[test]
    fn receive_with_same_physical_time() {
        let mut node = HybridLogicalClock::new(0, 1000);

        // Node is at pt=1000, lc=3
        for _ in 0..3 {
            node.local_event(1000);
        }

        // Receive message with same pt but higher lc
        let ts = node.receive(1000, 1000, 5);

        assert_eq!(ts.pt, 1000);
        assert_eq!(ts.lc, 6); // received_lc + 1
    }

    #[test]
    fn offset_from_physical() {
        let mut hlc = HybridLogicalClock::new(0, 1000);

        hlc.local_event(1000);
        hlc.local_event(1000);

        assert_eq!(hlc.offset_from_physical(1000), 0);
        assert_eq!(hlc.offset_from_physical(999), 1);
    }
}
