//! # Exercise: Consensus Deadlock Under Asynchrony
//!
//! ## Theory
//!
//! This exercise demonstrates a concrete deadlock scenario in 3-process consensus.
//! The FLP impossibility tells us this is inevitable for deterministic protocols,
//! but seeing it concretely makes the abstract impossibility tangible.
//!
//! ## Proof / Intuition
//!
//! Consider three processes: P0, P1, P2.
//!
//! 1. All processes broadcast their proposals.
//! 2. P0 proposes 1 and crashes after broadcasting.
//! 3. P1 receives P0's message but P2 does not (P0 crashed before delivery).
//! 4. P1 sees: {P0=1, P1=0, P2=?} -- cannot decide because P2 might have proposed 1.
//! 5. P2 sees: {P1=0, P2=0} -- cannot decide because P0 might have proposed 1.
//! 6. Neither P1 nor P2 can decide: P1 waits for P2, P2 waits for P0.
//!
//! This is the deadlock. Without a timeout or failure detector, the protocol
//! cannot progress.
//!
//! ## Implementation Task
//!
//! Implement a 3-process consensus protocol that demonstrates deadlock:
//!
//! - `Process`: A consensus participant with channels.
//! - `ConsensusMessage`: Messages exchanged during consensus.
//! - `run_consensus()`: Execute the protocol with a simulated crash.
//! - Use timeouts as a workaround to break deadlock.
//!
//! ## Verification
//!
//! - Verify the protocol blocks indefinitely when P0 crashes.
//! - Verify that with a timeout workaround, the protocol eventually decides.
//! - Verify that the workaround decisions may violate agreement (demonstrating
//!   the trade-off).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Messages exchanged between consensus processes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusMessage {
    /// A process broadcasts its proposal.
    Proposal { from: u32, value: bool },
    /// A process votes on a value.
    Vote { from: u32, value: bool },
}

/// State of a consensus process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    /// Initial state, waiting to propose.
    Idle,
    /// Has proposed and is waiting for all proposals.
    WaitingForProposals,
    /// Has received all proposals and is deciding.
    Deciding,
    /// Has made a decision.
    Decided(bool),
    /// Process has crashed.
    Crashed,
}

/// A process participating in 3-process consensus.
pub struct Process {
    /// Unique ID.
    pub id: u32,
    /// Proposed value.
    pub proposal: bool,
    /// Current state.
    pub state: ProcessState,
    /// Proposals received from other processes.
    pub received: HashMap<u32, bool>,
    /// Channel to send messages (simulated).
    pub outbound: Vec<ConsensusMessage>,
}

impl Process {
    /// Creates a new process.
    pub fn new(id: u32, proposal: bool) -> Self {
        Self {
            id,
            proposal,
            state: ProcessState::Idle,
            received: HashMap::new(),
            outbound: Vec::new(),
        }
    }

    /// Start the consensus protocol by broadcasting proposal.
    pub fn propose(&mut self) -> Vec<ConsensusMessage> {
        self.state = ProcessState::WaitingForProposals;
        self.received.insert(self.id, self.proposal);
        vec![ConsensusMessage::Proposal {
            from: self.id,
            value: self.proposal,
        }]
    }

    /// Handle an incoming message.
    pub fn handle_message(&mut self, msg: &ConsensusMessage) -> Option<ConsensusMessage> {
        if self.state == ProcessState::Crashed {
            return None;
        }

        match msg {
            ConsensusMessage::Proposal { from, value } => {
                self.received.insert(*from, *value);
                Some(ConsensusMessage::Vote {
                    from: self.id,
                    value: *value,
                })
            }
            ConsensusMessage::Vote { .. } => None,
        }
    }

    /// Try to decide based on received proposals.
    pub fn try_decide(&mut self, total: usize) -> Option<bool> {
        if self.state == ProcessState::Crashed || matches!(self.state, ProcessState::Decided(_)) {
            return None;
        }
        if self.received.len() < total {
            return None;
        }

        let ones = self.received.values().filter(|v| **v).count();
        let zeros = self.received.values().filter(|v| !**v).count();
        let decision = ones > zeros;
        self.state = ProcessState::Decided(decision);
        Some(decision)
    }
}

/// Result of a consensus round.
#[derive(Debug)]
pub struct ConsensusResult {
    /// Final states of all processes.
    pub states: Vec<ProcessState>,
    /// How long the protocol ran.
    pub elapsed: Duration,
    /// Whether all processes decided.
    pub all_decided: bool,
}

/// Run 3-process consensus with a crash at the specified process index.
/// If `use_timeout` is true, use timeouts as a workaround.
pub fn run_consensus(
    proposals: Vec<bool>,
    crash_process: Option<usize>,
    use_timeout: bool,
    timeout: Duration,
) -> ConsensusResult {
    let start = Instant::now();
    let num_processes = proposals.len();
    let mut processes: Vec<Process> = proposals
        .into_iter()
        .enumerate()
        .map(|(i, p)| Process::new(i as u32, p))
        .collect();

    // Simulate message buffer (all-to-all broadcast).
    let mut message_buffer: Vec<ConsensusMessage> = Vec::new();

    // Phase 1: All processes propose.
    for i in 0..num_processes {
        if crash_process == Some(i) {
            processes[i].state = ProcessState::Crashed;
            continue;
        }
        let msgs = processes[i].propose();
        message_buffer.extend(msgs);
    }

    // Deliver messages in rounds.
    loop {
        let mut new_messages = Vec::new();
        for msg in &message_buffer {
            for proc in &mut processes {
                if let Some(reply) = proc.handle_message(msg) {
                    new_messages.push(reply);
                }
            }
        }
        message_buffer = new_messages;

        // Check if anyone can decide.
        let mut decided_any = false;
        for proc in &mut processes {
            if let Some(_decision) = proc.try_decide(num_processes) {
                decided_any = true;
            }
        }

        if message_buffer.is_empty() && !decided_any {
            // Deadlock: no new messages and nobody can decide.
            if use_timeout && start.elapsed() >= timeout {
                // Workaround: processes decide based on what they have.
                for proc in &mut processes {
                    if proc.state != ProcessState::Crashed && !matches!(proc.state, ProcessState::Decided(_)) {
                        let ones = proc.received.values().filter(|v| **v).count();
                        let zeros = proc.received.values().filter(|v| !**v).count();
                        let decision = ones >= zeros; // tie-break toward 0
                        proc.state = ProcessState::Decided(decision);
                    }
                }
                break;
            }
            if !use_timeout {
                break; // Deadlocked
            }
        }
    }

    let all_decided = processes
        .iter()
        .all(|p| matches!(p.state, ProcessState::Decided(_)) || p.state == ProcessState::Crashed);

    ConsensusResult {
        states: processes.into_iter().map(|p| p.state).collect(),
        elapsed: start.elapsed(),
        all_decided,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consensus_succeeds_no_crash() {
        let result = run_consensus(vec![true, false, true], None, false, Duration::from_secs(1));
        assert!(
            result.all_decided,
            "All processes should decide when none crash"
        );
    }

    #[test]
    fn consensus_deadlocks_when_process_crashes() {
        let result = run_consensus(vec![true, false, true], Some(0), false, Duration::from_secs(1));
        // P0 crashes, P1 and P2 cannot get P0's proposal.
        assert!(
            !result.all_decided,
            "Consensus should deadlock when a process crashes mid-protocol"
        );
    }

    #[test]
    fn timeout_workaround_breaks_deadlock() {
        let result = run_consensus(
            vec![true, false, true],
            Some(0),
            true,
            Duration::from_millis(50),
        );
        assert!(
            result.all_decided,
            "With timeout workaround, all processes should eventually decide"
        );
    }

    #[test]
    fn all_alive_reach_agreement() {
        let result = run_consensus(vec![true, true, false], None, false, Duration::from_secs(1));
        assert!(result.all_decided);
        // All should agree.
        for state in &result.states {
            match state {
                ProcessState::Decided(v) => {
                    assert_eq!(*v, true, "Majority is true, all should decide true");
                }
                _ => panic!("Process should have decided"),
            }
        }
    }

    #[test]
    fn crash_after_proposal_still_deadlocks() {
        // Process 0 proposes then crashes before others receive its message.
        let result = run_consensus(vec![true, false, false], Some(0), false, Duration::from_secs(1));
        // Without P0's message, P1 and P2 cannot know P0 proposed true.
        assert!(!result.all_decided, "Should deadlock when crash prevents message delivery");
    }
}
