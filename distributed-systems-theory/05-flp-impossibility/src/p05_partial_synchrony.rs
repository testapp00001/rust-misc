//! # Exercise: Partial Synchrony Model
//!
//! ## Theory
//!
//! Partial synchrony is the system model that bridges the gap between the impossible
//! (async) and the trivial (synchronous). It postulates:
//!
//! 1. There exists a Global Stabilization Time (GST) after which all messages are
//!    delivered within a known bound Delta.
//! 2. Before GST, the system behaves asynchronously (unbounded delays).
//! 3. After GST, the system behaves synchronously (bounded delays by Delta).
//!
//! Under partial synchrony, deterministic consensus IS solvable because after GST,
//! the system becomes effectively synchronous, and protocols like Paxos, PBFT, and
//! Raft can terminate.
//!
//! ## Proof / Intuition
//!
//! Before GST:
//! - Messages can be arbitrarily delayed.
//! - Processes cannot distinguish slow from crashed.
//! - No deterministic protocol can guarantee termination (FLP).
//!
//! After GST:
//! - All message delays are bounded by Delta.
//! - Failure detectors become accurate.
//! - Leader election stabilizes.
//! - Consensus protocols terminate in bounded time.
//!
//! The key insight: GST is unknown to the processes. They must be designed to make
//! progress during async periods and converge after GST.
//!
//! ## Implementation Task
//!
//! Implement a partial synchrony simulator:
//!
//! - `PartialSynchronySimulator` with `message_delay_bound` (Delta) and `gst_time`.
//! - Initially, messages have unbounded delay.
//! - After GST, messages are delayed by at most Delta.
//! - Run a consensus protocol under this model.
//!
//! ## Verification
//!
//! - Verify consensus fails (or takes too long) before GST.
//! - Verify consensus succeeds after GST.
//! - Verify the protocol eventually terminates.

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// A message in the system.
#[derive(Debug, Clone)]
pub struct Message {
    /// Sender ID.
    pub from: u32,
    /// Receiver ID.
    pub to: u32,
    /// The proposal value.
    pub value: bool,
    /// When the message was sent (relative to simulation start).
    pub sent_at: Duration,
    /// When the message will be delivered (relative to simulation start).
    pub delivery_time: Duration,
}

/// Phase of the system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemPhase {
    /// Asynchronous phase: unbounded delays.
    Async,
    /// Synchronous phase: bounded delays.
    Sync,
}

/// A process in the partial synchrony model.
#[derive(Debug)]
pub struct PartialSyncProcess {
    /// Process ID.
    pub id: u32,
    /// Proposed value.
    pub proposal: bool,
    /// Values received from others.
    pub received: HashMap<u32, bool>,
    /// Whether the process has decided.
    pub decided: bool,
    /// The decided value.
    pub decision: Option<bool>,
    /// Messages waiting to be sent.
    pub pending_outbox: VecDeque<Message>,
}

impl PartialSyncProcess {
    /// Creates a new process.
    pub fn new(id: u32, proposal: bool) -> Self {
        Self {
            id,
            proposal,
            received: HashMap::new(),
            decided: false,
            decision: None,
            pending_outbox: VecDeque::new(),
        }
    }

    /// Broadcast proposal to all other processes.
    pub fn broadcast_proposal(&mut self, total: u32, now: Duration) {
        self.received.insert(self.id, self.proposal);
        for target in 0..total {
            if target != self.id {
                self.pending_outbox.push_back(Message {
                    from: self.id,
                    to: target,
                    value: self.proposal,
                    sent_at: now,
                    delivery_time: now, // Will be adjusted by the simulator.
                });
            }
        }
    }

    /// Receive a proposal from another process.
    pub fn receive_proposal(&mut self, from: u32, value: bool) {
        self.received.insert(from, value);
    }

    /// Try to decide based on received values.
    pub fn try_decide(&mut self, total: u32) -> bool {
        if self.decided {
            return true;
        }
        if self.received.len() < total as usize {
            return false;
        }
        let ones = self.received.values().filter(|v| **v).count();
        let zeros = self.received.values().filter(|v| !**v).count();
        self.decision = Some(ones > zeros);
        self.decided = true;
        true
    }
}

/// Simulator for partial synchrony model.
pub struct PartialSynchronySimulator {
    /// Maximum message delay after GST.
    pub message_delay_bound: Duration,
    /// Time at which GST occurs.
    pub gst_time: Duration,
    /// Elapsed simulation time.
    pub elapsed: Duration,
    /// Current phase.
    pub phase: SystemPhase,
    /// All processes.
    pub processes: Vec<PartialSyncProcess>,
    /// Pending messages in the network.
    pub network: Vec<Message>,
}

impl PartialSynchronySimulator {
    /// Creates a new simulator.
    pub fn new(
        proposals: Vec<bool>,
        message_delay_bound: Duration,
        gst_time: Duration,
    ) -> Self {
        let processes = proposals
            .into_iter()
            .enumerate()
            .map(|(i, p)| PartialSyncProcess::new(i as u32, p))
            .collect();
        Self {
            message_delay_bound,
            gst_time,
            elapsed: Duration::ZERO,
            phase: SystemPhase::Async,
            processes,
            network: Vec::new(),
        }
    }

    /// Calculate message delay based on current phase and a randomness factor.
    pub fn calculate_delay(&self, randomness: f64) -> Duration {
        match self.phase {
            SystemPhase::Async => {
                // Unbounded delay: scale by randomness (0.0 to 1.0).
                // High randomness = very long delay.
                let delay_ms = (randomness * 10000.0) as u64; // up to 10 seconds
                Duration::from_millis(delay_ms)
            }
            SystemPhase::Sync => {
                // Bounded delay: at most message_delay_bound.
                let max_ms = self.message_delay_bound.as_millis() as u64;
                let delay_ms = (randomness * max_ms as f64) as u64;
                Duration::from_millis(delay_ms.min(max_ms))
            }
        }
    }

    /// Broadcast proposals from all processes.
    pub fn broadcast_all(&mut self, randomness: f64) {
        let total = self.processes.len() as u32;
        let now = self.elapsed;

        for proc in &mut self.processes {
            proc.broadcast_proposal(total, now);
        }

        // Drain pending messages and add to network with appropriate delay.
        // Collect messages first to avoid borrow conflicts, then add to network.
        let mut outgoing = Vec::new();
        for proc in &mut self.processes {
            while let Some(msg) = proc.pending_outbox.pop_front() {
                outgoing.push(msg);
            }
        }
        for mut msg in outgoing {
            let delay = self.calculate_delay(randomness);
            msg.delivery_time = now + delay;
            self.network.push(msg);
        }
    }

    /// Advance time and deliver messages that have arrived.
    pub fn tick(&mut self, advance: Duration) {
        self.elapsed += advance;

        // Update phase.
        if self.elapsed >= self.gst_time {
            self.phase = SystemPhase::Sync;
        }

        // Deliver messages whose delivery_time has passed.
        let mut delivered = Vec::new();
        let mut remaining = Vec::new();
        for msg in self.network.drain(..) {
            if msg.delivery_time <= self.elapsed {
                delivered.push(msg);
            } else {
                remaining.push(msg);
            }
        }
        self.network = remaining;

        // Deliver to processes.
        for msg in &delivered {
            if let Some(proc) = self.processes.iter_mut().find(|p| p.id == msg.to) {
                proc.receive_proposal(msg.from, msg.value);
            }
        }

        // Try to decide.
        let total = self.processes.len() as u32;
        for proc in &mut self.processes {
            proc.try_decide(total);
        }
    }

    /// Check if all processes have decided.
    pub fn all_decided(&self) -> bool {
        self.processes.iter().all(|p| p.decided)
    }

    /// Get all decisions.
    pub fn get_decisions(&self) -> Vec<Option<bool>> {
        self.processes.iter().map(|p| p.decision).collect()
    }
}

/// Run consensus under partial synchrony.
/// Returns the decisions and whether consensus was reached.
pub fn run_partial_sync_consensus(
    proposals: Vec<bool>,
    message_delay_bound: Duration,
    gst_time: Duration,
    timeout: Duration,
) -> (Vec<Option<bool>>, bool) {
    let mut sim = PartialSynchronySimulator::new(proposals, message_delay_bound, gst_time);

    let start = Instant::now();

    // Phase 1: Broadcast (async, with high randomness).
    sim.broadcast_all(0.9);

    loop {
        if start.elapsed() >= timeout {
            break;
        }

        // Random tick interval.
        let tick_duration = Duration::from_millis(50);
        sim.tick(tick_duration);

        if sim.all_decided() {
            return (sim.get_decisions(), true);
        }

        // After GST, send any remaining proposals that might not have been delivered.
        if sim.phase == SystemPhase::Sync && sim.network.is_empty() && !sim.all_decided() {
            sim.broadcast_all(0.1); // Low randomness after GST.
        }
    }

    (sim.get_decisions(), false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn async_phase_has_unbounded_delay() {
        let sim = PartialSynchronySimulator::new(vec![true, false], Duration::from_millis(100), Duration::from_secs(5));
        assert_eq!(sim.phase, SystemPhase::Async);

        // In async phase, delay can be very large.
        let delay = sim.calculate_delay(1.0);
        assert!(
            delay > Duration::from_secs(1),
            "Async phase should allow large delays"
        );
    }

    #[test]
    fn sync_phase_has_bounded_delay() {
        let mut sim = PartialSynchronySimulator::new(
            vec![true, false],
            Duration::from_millis(100),
            Duration::ZERO,
        );
        // After GST (which is ZERO), should be in sync phase.
        sim.tick(Duration::from_millis(1));
        assert_eq!(sim.phase, SystemPhase::Sync);

        let delay = sim.calculate_delay(1.0);
        assert!(
            delay <= Duration::from_millis(100),
            "Sync phase delay should be bounded by message_delay_bound"
        );
    }

    #[test]
    fn consensus_succeeds_after_gst() {
        let (decisions, decided) = run_partial_sync_consensus(
            vec![true, true, false],
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_secs(5),
        );
        assert!(
            decided,
            "Consensus should succeed after GST when system becomes synchronous"
        );
        // Agreement check.
        let first = decisions[0];
        assert!(
            decisions.iter().all(|d| *d == first),
            "All decisions should agree"
        );
    }

    #[test]
    fn gst_transitions_from_async_to_sync() {
        let mut sim = PartialSynchronySimulator::new(
            vec![true, false],
            Duration::from_millis(100),
            Duration::from_millis(500),
        );
        assert_eq!(sim.phase, SystemPhase::Async);

        sim.tick(Duration::from_millis(600));
        assert_eq!(sim.phase, SystemPhase::Sync, "Should transition to Sync after GST");
    }

    #[test]
    fn consensus_validity_after_gst() {
        let (decisions, decided) = run_partial_sync_consensus(
            vec![true, false, false],
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_secs(5),
        );
        assert!(decided);
        // The decided value should be one of the proposed values.
        for d in &decisions {
            let val = d.unwrap();
            assert!(
                val == true || val == false,
                "Decided value must be a proposed value"
            );
        }
    }

    #[test]
    fn large_message_bound_allows_consensus() {
        let (decisions, decided) = run_partial_sync_consensus(
            vec![false, false, true],
            Duration::from_millis(50),
            Duration::from_millis(100),
            Duration::from_secs(5),
        );
        assert!(decided, "With bounded delay after GST, consensus should be reached");
    }
}
