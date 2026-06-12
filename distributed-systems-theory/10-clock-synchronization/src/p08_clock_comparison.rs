//! # Exercise: Compare All Clock Types
//!
//! ## Theory
//!
//! Distributed systems use several types of logical and physical clocks,
//! each with different trade-offs:
//!
//! **Physical Clocks**
//!   - Wall clock time (e.g., NTP-synchronized).
//!   - Provides real-time ordering but is subject to clock skew.
//!   - Space: O(1) per node.
//!
//! **Lamport Clocks**
//!   - A single integer counter, incremented on each event.
//!   - On send: include current value in message.
//!   - On receive: set local = max(local, received) + 1.
//!   - Captures causality (A -> B implies L(A) < L(B)).
//!   - Does NOT distinguish concurrent events (L(A) < L(B) does not
//!     imply A -> B).
//!   - Space: O(1) per node.
//!
//! **Vector Clocks**
//!   - An array of size N (one counter per process).
//!   - On event at process i: increment v[i].
//!   - On receive from process j: v[k] = max(v[k], remote_v[k]) for
//!     all k, then increment v[i].
//!   - Captures causality AND detects concurrency.
//!   - A -> B iff v(A) < v(B) (element-wise).
//!   - Space: O(N) per node.
//!
//! **Hybrid Logical Clocks (HLC)**
//!   - Combines physical time with a logical counter.
//!   - Physical component stays close to wall clock.
//!   - Logical component preserves causality within same physical tick.
//!   - Space: O(1) per node.
//!
//! **TrueTime (Spanner)**
//!   - Returns a confidence interval [earliest, latest].
//!   - True time is within this interval with high probability.
//!   - Used for external consistency via commit wait.
//!   - Space: O(1) per node.
//!
//! ## Proof / Intuition
//!
//! Comparison of properties:
//!
//! | Clock Type     | Causality | Concurrency | Physical Proximity | Space  |
//! |----------------|-----------|-------------|--------------------|--------|
//! | Physical       | No        | N/A         | Exact              | O(1)   |
//! | Lamport        | Yes       | No          | No                 | O(1)   |
//! | Vector         | Yes       | Yes         | No                 | O(N)   |
//! | HLC            | Yes       | No          | Yes (bounded)      | O(1)   |
//! | TrueTime       | N/A       | N/A         | Interval           | O(1)   |
//!
//! - Lamport clocks are minimal but lose information about concurrency.
//! - Vector clocks are maximally informative but expensive.
//! - HLCs are a practical compromise: O(1) space, causality, and
//!   physical time proximity.
//! - TrueTime provides uncertainty bounds rather than exact times.
//!
//! ## Implementation Task
//!
//! Implement a `ClockSimulator` that runs the same set of events through
//! all clock types and records their values. Then verify properties:
//!
//! 1. `ClockType` enum with all five variants.
//! 2. `ClockEntry` storing all components for a process.
//! 3. `ClockSimulator` with methods:
//!    - `new(num_processes, clock_type)` - initialize clocks.
//!    - `local_event(process_id, wall_clock)` - process a local event.
//!    - `send_message(from, to, wall_clock)` - send a message.
//!    - `receive_message(from, to, wall_clock)` - receive a message.
//!    - `get_clock_value(process_id) -> ClockEntry` - read current state.
//!
//! ## Verification
//!
//! The tests verify:
//! - Lamport clocks capture causality (send < receive) but cannot
//!   distinguish concurrent events.
//! - Vector clocks distinguish concurrent events (v1 < v2 is false
//!   when neither happens-before the other).
//! - HLC stays close to the physical time (physical component tracks
//!   wall clock within bounds).
//! - TrueTime provides an interval containing the true time.

/// The type of clock being simulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockType {
    /// A simple wall clock (no logical component).
    Physical,
    /// Lamport's logical clock (single counter).
    Lamport,
    /// Vector clock (one counter per process).
    Vector(usize),
    /// Hybrid logical clock (physical + logical).
    HLC,
    /// TrueTime-style interval clock.
    TrueTime,
}

/// A snapshot of all clock components for a single process.
#[derive(Debug, Clone)]
pub struct ClockEntry {
    /// The process this entry belongs to.
    pub process_id: usize,
    /// Which clock type this entry represents.
    pub clock_type: ClockType,
    /// Physical (wall clock) time.
    pub physical: u64,
    /// Logical counter (for Lamport and HLC).
    pub logical: u64,
    /// Vector clock state (for Vector clocks).
    pub vector: Vec<u64>,
    /// HLC physical component.
    pub hlc_physical: u64,
    /// HLC logical component.
    pub hlc_logical: u64,
    /// TrueTime lower bound.
    pub tt_earliest: u64,
    /// TrueTime upper bound.
    pub tt_latest: u64,
}

/// Simulates a set of processes using a given clock type and tracks
/// all clock values through a sequence of events.
pub struct ClockSimulator {
    /// Current clock entries for each process.
    pub clocks: Vec<ClockEntry>,
    /// The clock type being used.
    pub clock_type: ClockType,
    /// Lamport counters (indexed by process id).
    lamport_counters: Vec<u64>,
    /// Vector clocks (indexed by process id).
    vector_clocks: Vec<Vec<u64>>,
    /// HLC physical times.
    hlc_physicals: Vec<u64>,
    /// HLC logical counters.
    hlc_logicals: Vec<u64>,
    /// TrueTime intervals: (earliest, latest) per process.
    tt_intervals: Vec<(u64, u64)>,
    /// TrueTime uncertainty per process.
    tt_uncertainty: u64,
}

impl ClockSimulator {
    /// Create a new simulator for `num_processes` using the given clock type.
    pub fn new(num_processes: usize, clock_type: ClockType) -> Self {
        let mut clocks = Vec::with_capacity(num_processes);
        for i in 0..num_processes {
            clocks.push(ClockEntry {
                process_id: i,
                clock_type,
                physical: 0,
                logical: 0,
                vector: vec![0; num_processes],
                hlc_physical: 0,
                hlc_logical: 0,
                tt_earliest: 0,
                tt_latest: 0,
            });
        }
        ClockSimulator {
            clocks,
            clock_type,
            lamport_counters: vec![0; num_processes],
            vector_clocks: (0..num_processes).map(|_| vec![0u64; num_processes]).collect(),
            hlc_physicals: vec![0; num_processes],
            hlc_logicals: vec![0; num_processes],
            tt_intervals: vec![(0, 0); num_processes],
            tt_uncertainty: 10, // default uncertainty
        }
    }

    /// Process a local event at the given process with the given wall clock.
    pub fn local_event(&mut self, process_id: usize, wall_clock: u64) {
        let _n = self.clocks.len();
        match self.clock_type {
            ClockType::Physical => {
                self.clocks[process_id].physical = wall_clock;
            }
            ClockType::Lamport => {
                self.lamport_counters[process_id] += 1;
                self.clocks[process_id].logical = self.lamport_counters[process_id];
                self.clocks[process_id].physical = wall_clock;
            }
            ClockType::Vector(_) => {
                self.vector_clocks[process_id][process_id] += 1;
                self.clocks[process_id].vector = self.vector_clocks[process_id].clone();
                self.clocks[process_id].physical = wall_clock;
            }
            ClockType::HLC => {
                if wall_clock > self.hlc_physicals[process_id] {
                    self.hlc_physicals[process_id] = wall_clock;
                    self.hlc_logicals[process_id] = 0;
                } else {
                    self.hlc_logicals[process_id] += 1;
                }
                self.clocks[process_id].hlc_physical = self.hlc_physicals[process_id];
                self.clocks[process_id].hlc_logical = self.hlc_logicals[process_id];
                self.clocks[process_id].physical = wall_clock;
            }
            ClockType::TrueTime => {
                self.tt_intervals[process_id] = (wall_clock, wall_clock + self.tt_uncertainty);
                self.clocks[process_id].tt_earliest = wall_clock;
                self.clocks[process_id].tt_latest = wall_clock + self.tt_uncertainty;
                self.clocks[process_id].physical = wall_clock;
            }
        }
    }

    /// Simulate sending a message from `from` to `to`.
    ///
    /// For Lamport/HLC: increments the sender's counter.
    /// For Vector: increments the sender's vector at their index.
    pub fn send_message(&mut self, from: usize, _to: usize, wall_clock: u64) {
        match self.clock_type {
            ClockType::Physical => {
                self.clocks[from].physical = wall_clock;
            }
            ClockType::Lamport => {
                self.lamport_counters[from] += 1;
                self.clocks[from].logical = self.lamport_counters[from];
                self.clocks[from].physical = wall_clock;
            }
            ClockType::Vector(_) => {
                self.vector_clocks[from][from] += 1;
                self.clocks[from].vector = self.vector_clocks[from].clone();
                self.clocks[from].physical = wall_clock;
            }
            ClockType::HLC => {
                if wall_clock > self.hlc_physicals[from] {
                    self.hlc_physicals[from] = wall_clock;
                    self.hlc_logicals[from] = 0;
                } else {
                    self.hlc_logicals[from] += 1;
                }
                self.clocks[from].hlc_physical = self.hlc_physicals[from];
                self.clocks[from].hlc_logical = self.hlc_logicals[from];
                self.clocks[from].physical = wall_clock;
            }
            ClockType::TrueTime => {
                self.tt_intervals[from] = (wall_clock, wall_clock + self.tt_uncertainty);
                self.clocks[from].tt_earliest = wall_clock;
                self.clocks[from].tt_latest = wall_clock + self.tt_uncertainty;
                self.clocks[from].physical = wall_clock;
            }
        }
    }

    /// Simulate receiving a message from `from` at `to`.
    ///
    /// For Lamport: local = max(local, remote) + 1.
    /// For Vector: merge vectors, then increment local index.
    /// For HLC: merge physical times, increment logical counter.
    pub fn receive_message(&mut self, from: usize, to: usize, wall_clock: u64) {
        let n = self.clocks.len();
        match self.clock_type {
            ClockType::Physical => {
                self.clocks[to].physical = wall_clock;
            }
            ClockType::Lamport => {
                let remote_val = self.lamport_counters[from];
                let local_val = self.lamport_counters[to];
                self.lamport_counters[to] = local_val.max(remote_val) + 1;
                self.clocks[to].logical = self.lamport_counters[to];
                self.clocks[to].physical = wall_clock;
            }
            ClockType::Vector(_) => {
                // Merge: take element-wise max.
                for k in 0..n {
                    self.vector_clocks[to][k] =
                        self.vector_clocks[to][k].max(self.vector_clocks[from][k]);
                }
                // Increment local index.
                self.vector_clocks[to][to] += 1;
                self.clocks[to].vector = self.vector_clocks[to].clone();
                self.clocks[to].physical = wall_clock;
            }
            ClockType::HLC => {
                let max_physical = self.hlc_physicals[to]
                    .max(wall_clock)
                    .max(self.hlc_physicals[from]);

                if max_physical > self.hlc_physicals[to] {
                    self.hlc_physicals[to] = max_physical;
                    self.hlc_logicals[to] = 0;
                } else if max_physical == self.hlc_physicals[from]
                    && self.hlc_logicals[from] >= self.hlc_logicals[to]
                {
                    self.hlc_logicals[to] = self.hlc_logicals[from] + 1;
                } else {
                    self.hlc_logicals[to] += 1;
                }

                self.clocks[to].hlc_physical = self.hlc_physicals[to];
                self.clocks[to].hlc_logical = self.hlc_logicals[to];
                self.clocks[to].physical = wall_clock;
            }
            ClockType::TrueTime => {
                let (earliest, _latest) = self.tt_intervals[from];
                let local_latest = self.tt_intervals[to].1;
                // Merge intervals: earliest = max(local_earliest, remote_earliest),
                // latest = max(local_latest, remote_latest).
                let new_earliest = self.tt_intervals[to].0.max(earliest);
                let new_latest = local_latest.max(wall_clock + self.tt_uncertainty);
                self.tt_intervals[to] = (new_earliest, new_latest);
                self.clocks[to].tt_earliest = new_earliest;
                self.clocks[to].tt_latest = new_latest;
                self.clocks[to].physical = wall_clock;
            }
        }
    }

    /// Read the current clock state for a given process.
    pub fn get_clock_value(&self, process_id: usize) -> ClockEntry {
        self.clocks[process_id].clone()
    }
}

/// Check whether vector v1 is strictly less than v2 (element-wise).
/// Used in tests to verify vector clock causality properties.
#[allow(dead_code)]
pub(crate) fn vector_less_than(v1: &[u64], v2: &[u64]) -> bool {
    assert_eq!(v1.len(), v2.len());
    let mut strictly_less = false;
    for (a, b) in v1.iter().zip(v2.iter()) {
        if a > b {
            return false;
        }
        if a < b {
            strictly_less = true;
        }
    }
    strictly_less
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lamport_causality_but_no_concurrency_detection() {
        let mut sim = ClockSimulator::new(3, ClockType::Lamport);

        // P0 local event.
        sim.local_event(0, 100);
        let c0 = sim.get_clock_value(0);
        assert_eq!(c0.logical, 1);

        // P0 sends to P1.
        sim.send_message(0, 1, 200);
        let c0_after_send = sim.get_clock_value(0);
        assert_eq!(c0_after_send.logical, 2);

        // P1 receives from P0.
        sim.receive_message(0, 1, 210);
        let c1 = sim.get_clock_value(1);
        // Lamport: max(0, 2) + 1 = 3.
        assert_eq!(c1.logical, 3);

        // Causality: send < receive.
        assert!(c0_after_send.logical < c1.logical);

        // Now show Lamport cannot detect concurrency:
        // P2 has an independent event.
        sim.local_event(2, 150);
        let c2 = sim.get_clock_value(2);
        assert_eq!(c2.logical, 1);

        // Lamport says c1 > c2 (3 > 1), but P1's event does NOT
        // happen-before P2's event. Lamport cannot distinguish this
        // from true causality.
        assert!(c1.logical > c2.logical);
        // This is the limitation: Lamport ordering does not imply
        // happened-before for events on different processes.
    }

    #[test]
    fn test_vector_clocks_distinguish_concurrent_events() {
        let mut sim = ClockSimulator::new(3, ClockType::Vector(3));

        // P0 and P2 both have independent local events.
        sim.local_event(0, 100);
        sim.local_event(2, 150);

        let v0 = sim.get_clock_value(0);
        let v2 = sim.get_clock_value(2);

        // v0 = [1, 0, 0], v2 = [0, 0, 1].
        // Neither is less than the other -- they are concurrent.
        assert!(!vector_less_than(&v0.vector, &v2.vector));
        assert!(!vector_less_than(&v2.vector, &v0.vector));

        // P0 sends to P1.
        sim.send_message(0, 1, 200);
        sim.receive_message(0, 1, 210);

        let v1 = sim.get_clock_value(1);
        // P0's local_event set v0 to [1,0,0], then send_message incremented
        // to [2,0,0]. Receiver merges [2,0,0] and increments index 1 -> [2,1,0].
        assert_eq!(v1.vector, vec![2, 1, 0]);

        // P0 -> P1 happened-before, so v0 < v1.
        assert!(vector_less_than(&v0.vector, &v1.vector));

        // But v2 and v1 are still concurrent (no message between them).
        assert!(!vector_less_than(&v2.vector, &v1.vector));
        assert!(!vector_less_than(&v1.vector, &v2.vector));
    }

    #[test]
    fn test_hlc_stays_close_to_physical_time() {
        let mut sim = ClockSimulator::new(2, ClockType::HLC);

        // P0 event at wall clock 1000.
        sim.local_event(0, 1000);
        let c0 = sim.get_clock_value(0);
        assert_eq!(c0.hlc_physical, 1000);
        assert_eq!(c0.hlc_logical, 0);

        // P0 event at same time.
        sim.local_event(0, 1000);
        let c0_2 = sim.get_clock_value(0);
        assert_eq!(c0_2.hlc_physical, 1000);
        assert_eq!(c0_2.hlc_logical, 1);

        // P0 event at later time.
        sim.local_event(0, 1500);
        let c0_3 = sim.get_clock_value(0);
        assert_eq!(c0_3.hlc_physical, 1500);
        assert_eq!(c0_3.hlc_logical, 0);

        // P0 sends to P1 at time 2000.
        sim.send_message(0, 1, 2000);
        sim.receive_message(0, 1, 2050);
        let c1 = sim.get_clock_value(1);

        // HLC physical >= max(local, remote) >= 2000.
        assert!(c1.hlc_physical >= 2000);

        // HLC stays close to wall clock -- physical component
        // never drifts more than the max clock skew.
        let skew = if c1.hlc_physical > 2050 {
            c1.hlc_physical - 2050
        } else {
            2050 - c1.hlc_physical
        };
        assert!(skew <= 20, "HLC physical {} too far from wall clock 2050", c1.hlc_physical);
    }

    #[test]
    fn test_truetime_provides_uncertainty_bounds() {
        let mut sim = ClockSimulator::new(2, ClockType::TrueTime);

        sim.local_event(0, 1000);
        let c0 = sim.get_clock_value(0);

        // TrueTime interval contains the wall clock.
        assert!(c0.tt_earliest <= 1000);
        assert!(c0.tt_latest >= 1000);

        // The interval width equals the uncertainty.
        let interval_width = c0.tt_latest - c0.tt_earliest;
        assert_eq!(interval_width, 10); // default uncertainty

        // After a receive, the interval widens or merges.
        sim.send_message(0, 1, 1100);
        sim.receive_message(0, 1, 1100);
        let c1 = sim.get_clock_value(1);

        // Wall clock is within the interval.
        assert!(c1.tt_earliest <= 1100);
        assert!(c1.tt_latest >= 1100);
    }

    #[test]
    fn test_physical_clock_tracks_wall_clock_exactly() {
        let mut sim = ClockSimulator::new(2, ClockType::Physical);

        sim.local_event(0, 500);
        assert_eq!(sim.get_clock_value(0).physical, 500);

        sim.local_event(0, 1000);
        assert_eq!(sim.get_clock_value(0).physical, 1000);

        sim.send_message(0, 1, 1500);
        assert_eq!(sim.get_clock_value(0).physical, 1500);

        sim.receive_message(0, 1, 1600);
        assert_eq!(sim.get_clock_value(1).physical, 1600);

        // Physical clock is exact but provides no causality info.
        // P1's physical time (1600) > P0's (1500), but that does not
        // necessarily mean P0 -> P1 in a system with clock skew.
    }

    #[test]
    fn test_lamport_clocks_monotonically_increase() {
        let mut sim = ClockSimulator::new(2, ClockType::Lamport);

        sim.local_event(0, 100);
        sim.local_event(0, 100);
        sim.local_event(0, 100);

        let c = sim.get_clock_value(0);
        // Lamport clock increments on every event.
        assert_eq!(c.logical, 3);
        assert!(c.logical > 0);
    }
}
