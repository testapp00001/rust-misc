//! # Exercise: HLC Preserves Causality
//!
//! ## Theory
//!
//! A Hybrid Logical Clock (HLC) combines the benefits of physical clocks
//! and logical clocks. Each HLC timestamp has three components:
//!
//!   - physical_time (pt): The maximum physical time observed so far.
//!   - logical_counter (lc): A counter that increments when the physical
//!     time does not advance, preserving causality within the same physical
//!     tick.
//!   - node_id: A unique identifier to break ties deterministically.
//!
//! The HLC algorithm works as follows:
//!
//!   On local event: pt = max(pt, wall_clock), lc = lc + 1 if pt did not
//!   change, else lc = 0.
//!
//!   On receive(wall_clock, remote_pt, remote_lc):
//!     pt = max(pt, wall_clock, remote_pt)
//!     lc = lc + 1 if pt == local_pt and pt == remote_pt, else lc = 0
//!     (with tie-breaking via node_id).
//!
//! Key properties:
//!   1. Causality: If event A happened-before event B, then HLC(A) < HLC(B).
//!   2. Physical proximity: HLC(t) is always close to the physical clock at t.
//!   3. Compactness: Unlike vector clocks, HLC requires only O(1) space.
//!
//! ## Proof / Intuition
//!
//! The happened-before relation (Lamport, 1978) is the transitive closure of:
//!   - Same-process ordering: A -> B if A and B are on the same process
//!     and A precedes B.
//!   - Message ordering: A -> B if A is the send of a message and B is the
//!     corresponding receive.
//!
//! HLC preserves causality because:
//!   1. On local events, the logical counter ensures the timestamp strictly
//!      increases even if the physical clock does not advance.
//!   2. On message receive, the HLC takes the max of local and remote
//!      physical times, then increments the counter, ensuring the receive
//!      timestamp is strictly greater than the send timestamp.
//!   3. The node_id tie-breaker ensures total ordering without ambiguity.
//!
//! The physical proximity property holds because:
//!   - pt is always >= wall_clock (never falls behind the physical clock).
//!   - pt is always <= max of all wall clocks seen (never exceeds the
//!     maximum physical time).
//!
//! ## Implementation Task
//!
//! Implement the following:
//!
//! 1. `HLCTimestamp` with `PartialEq`, `Eq`, `PartialOrd`, `Ord` that compare
//!    physical_time first, then logical_counter, then node_id.
//! 2. `HLClock` with methods:
//!    - `new(node_id: usize)` - create a new HLC for the given node.
//!    - `local_event(&mut self, wall_clock: u64) -> HLCTimestamp` - record a
//!      local event.
//!    - `receive(&mut self, wall_clock: u64, remote_pt: u64, remote_lc: u64)
//!      -> HLCTimestamp` - process a message receipt.
//! 3. `CausalProcess` wrapping an HLC with an event log and message queue.
//! 4. Simulate 3 processes exchanging messages and verify causality.
//!
//! ## Verification
//!
//! The tests verify:
//! - Send timestamp < receive timestamp (causality preserved).
//! - HLC physical component stays close to wall clock (within uncertainty).
//! - Multiple exchanges maintain causal order across processes.

use std::cmp::Ordering;

/// A hybrid logical clock timestamp with physical time, logical counter,
/// and node ID for tie-breaking.
#[derive(Debug, Clone, Copy)]
pub struct HLCTimestamp {
    /// Maximum physical time observed.
    pub physical_time: u64,
    /// Logical counter for causality within the same physical tick.
    pub logical_counter: u64,
    /// Unique node identifier for deterministic tie-breaking.
    pub node_id: usize,
}

impl HLCTimestamp {
    /// Create a new timestamp with the given components.
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

/// A Hybrid Logical Clock for a single node.
#[derive(Debug, Clone)]
pub struct HLClock {
    /// This node's unique identifier.
    pub node_id: usize,
    /// Current physical time component.
    pub physical_time: u64,
    /// Current logical counter.
    pub logical_counter: u64,
}

impl HLClock {
    /// Create a new HLC for the given node, starting at time 0.
    pub fn new(node_id: usize) -> Self {
        HLClock {
            node_id,
            physical_time: 0,
            logical_counter: 0,
        }
    }

    /// Record a local event at the given wall clock time.
    ///
    /// Updates the physical time to max(current_pt, wall_clock).
    /// If physical time did not advance, increments the logical counter;
    /// otherwise resets it to 0.
    pub fn local_event(&mut self, wall_clock: u64) -> HLCTimestamp {
        if wall_clock > self.physical_time {
            self.physical_time = wall_clock;
            self.logical_counter = 0;
        } else {
            self.logical_counter += 1;
        }
        HLCTimestamp::new(self.physical_time, self.logical_counter, self.node_id)
    }

    /// Process a message received from a remote node.
    ///
    /// Takes the maximum of local pt, wall_clock, and remote_pt, then
    /// increments the counter if all three are equal.
    pub fn receive(
        &mut self,
        wall_clock: u64,
        remote_pt: u64,
        remote_lc: u64,
    ) -> HLCTimestamp {
        let max_physical = self.physical_time.max(wall_clock).max(remote_pt);

        if max_physical > self.physical_time {
            self.physical_time = max_physical;
            self.logical_counter = 0;
        } else if max_physical == self.physical_time
            && max_physical == remote_pt
            && remote_lc >= self.logical_counter
        {
            // Remote is within the same physical tick and has equal or
            // higher logical counter -- increment to stay ahead.
            self.logical_counter = remote_lc + 1;
        } else {
            self.logical_counter += 1;
        }

        HLCTimestamp::new(self.physical_time, self.logical_counter, self.node_id)
    }
}

/// A process that uses an HLC to assign timestamps to local events
/// and message exchanges.
#[derive(Debug)]
pub struct CausalProcess {
    /// Unique process identifier.
    pub process_id: usize,
    /// The process's hybrid logical clock.
    pub hlc: HLClock,
    /// Log of all timestamps assigned to local events.
    pub event_log: Vec<HLCTimestamp>,
    /// Queue of sent messages: (destination_id, timestamp).
    pub sent_messages: Vec<(usize, HLCTimestamp)>,
}

impl CausalProcess {
    /// Create a new causal process with the given ID.
    pub fn new(process_id: usize) -> Self {
        CausalProcess {
            process_id,
            hlc: HLClock::new(process_id),
            event_log: Vec::new(),
            sent_messages: Vec::new(),
        }
    }

    /// Record a local event and return its timestamp.
    pub fn local_event(&mut self, wall_clock: u64) -> HLCTimestamp {
        let ts = self.hlc.local_event(wall_clock);
        self.event_log.push(ts);
        ts
    }

    /// Send a message. Returns (destination_placeholder, timestamp, wall_clock).
    /// In a real system the destination would be specified; here we return
    /// the timestamp for the caller to pass to the receiver.
    pub fn send_message(&mut self, wall_clock: u64) -> (usize, HLCTimestamp, u64) {
        let ts = self.hlc.local_event(wall_clock);
        self.event_log.push(ts);
        // Destination is a placeholder; the caller wires it up.
        let dest = self.process_id; // will be overridden by caller
        self.sent_messages.push((dest, ts));
        (dest, ts, wall_clock)
    }

    /// Receive a message from a remote process and return the receive timestamp.
    pub fn receive_message(
        &mut self,
        wall_clock: u64,
        remote_pt: u64,
        remote_lc: u64,
    ) -> HLCTimestamp {
        let ts = self.hlc.receive(wall_clock, remote_pt, remote_lc);
        self.event_log.push(ts);
        ts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: wire up a send from `from` to `to` and return both timestamps.
    fn exchange(
        from: &mut CausalProcess,
        to: &mut CausalProcess,
        send_wall: u64,
        recv_wall: u64,
    ) -> (HLCTimestamp, HLCTimestamp) {
        let (_, send_ts, _) = from.send_message(send_wall);
        let recv_ts =
            to.receive_message(recv_wall, send_ts.physical_time, send_ts.logical_counter);
        (send_ts, recv_ts)
    }

    #[test]
    fn test_causality_send_before_receive() {
        let mut p0 = CausalProcess::new(0);
        let mut p1 = CausalProcess::new(1);

        let (send_ts, recv_ts) = exchange(&mut p0, &mut p1, 100, 105);

        // Causality: send timestamp must be strictly less than receive timestamp.
        assert!(
            send_ts < recv_ts,
            "send_ts {:?} must be < recv_ts {:?}",
            send_ts,
            recv_ts
        );
    }

    #[test]
    fn test_hlc_physical_stays_close_to_wall_clock() {
        let mut hlc = HLClock::new(0);

        // Advance physical clock.
        let ts1 = hlc.local_event(1000);
        assert_eq!(ts1.physical_time, 1000);
        assert_eq!(ts1.logical_counter, 0);

        // Local event at the same physical time -- counter increments.
        let ts2 = hlc.local_event(1000);
        assert_eq!(ts2.physical_time, 1000);
        assert_eq!(ts2.logical_counter, 1);

        // Another event at the same time.
        let ts3 = hlc.local_event(1000);
        assert_eq!(ts3.logical_counter, 2);

        // Jump forward -- physical time tracks wall clock.
        let ts4 = hlc.local_event(2000);
        assert_eq!(ts4.physical_time, 2000);
        assert_eq!(ts4.logical_counter, 0);

        // All timestamps stay within [1000, 2000].
        for ts in &[ts1, ts2, ts3, ts4] {
            assert!(ts.physical_time >= 1000 && ts.physical_time <= 2000);
        }
    }

    #[test]
    fn test_multiple_exchanges_maintain_causal_order() {
        let mut p0 = CausalProcess::new(0);
        let mut p1 = CausalProcess::new(1);
        let mut p2 = CausalProcess::new(2);

        // p0 sends to p1 at time 100.
        let (s01, r01) = exchange(&mut p0, &mut p1, 100, 110);
        assert!(s01 < r01);

        // p1 sends to p2 at time 200, carrying the received timestamp.
        let (s12, r12) = exchange(&mut p1, &mut p2, 200, 210);
        assert!(s12 < r12);

        // Causal chain: s01 < r01 <= s12 < r12
        assert!(s01 < r01);
        assert!(r01 <= s12); // p1's local event at 200 >= p1's receive at 110
        assert!(s12 < r12);

        // p2 sends back to p0 at time 300.
        let (s20, r20) = exchange(&mut p2, &mut p0, 300, 310);
        assert!(s20 < r20);

        // Full causal chain across the ring.
        assert!(s01 < r12);
        assert!(r12 < s20);
        assert!(s20 < r20);
    }

    #[test]
    fn test_hlc_receive_advances_beyond_remote() {
        let mut hlc_a = HLClock::new(0);
        let mut hlc_b = HLClock::new(1);

        // A generates an event.
        let ts_a = hlc_a.local_event(500);
        assert_eq!(ts_a.physical_time, 500);

        // B receives the message -- timestamp must be > ts_a.
        let ts_b = hlc_b.receive(600, ts_a.physical_time, ts_a.logical_counter);
        assert!(ts_b > ts_a);
        assert!(ts_b.physical_time >= 500);
    }

    #[test]
    fn test_ordering_tie_breaks_by_node_id() {
        let ts_a = HLCTimestamp::new(100, 0, 0);
        let ts_b = HLCTimestamp::new(100, 0, 1);

        // Same physical and logical, different node_id.
        assert!(ts_a < ts_b);
        assert!(ts_b > ts_a);
    }
}
