//! # Exercise: Deterministic Consensus and Deadlock
//!
//! ## Theory
//!
//! In a deterministic consensus protocol, each process proposes a value and the
//! processes communicate via message passing to agree on a single decision value.
//!
//! The three properties of consensus are:
//! - **Agreement:** All correct processes decide the same value.
//! - **Validity:** The decided value must have been proposed by some process.
//! - **Termination:** Every correct process eventually decides.
//!
//! FLP proves that in an asynchronous system with crash failures, no deterministic
//! protocol can satisfy all three simultaneously.
//!
//! ## Proof / Intuition
//!
//! Consider a simple 3-process consensus protocol:
//! 1. Each process broadcasts its proposal to all others.
//! 2. Each process waits for proposals from all other processes.
//! 3. If all proposals match, decide on that value.
//! 4. If proposals differ, the process with the lowest ID decides on the majority.
//!
//! This protocol works when all processes are correct and responsive. However:
//! - If one process crashes (or is arbitrarily slow), other processes wait forever
//!   for its proposal.
//! - Since the system is asynchronous, there is no timeout to break the deadlock.
//! - The process cannot decide "0" because the crashed process might have proposed "1"
//!   and is simply delayed.
//!
//! ## Implementation Task
//!
//! Implement a deterministic consensus protocol:
//!
//! - `Proposal`: The value a process proposes.
//! - `ConsensusProcess`: A process with an ID, proposal, and message buffer.
//! - `DeterministicConsensus`: The protocol that coordinates processes.
//! - `simulate_consensus()`: Run the protocol and return results.
//!
//! The protocol should:
//! - Collect proposals from all processes.
//! - Wait for all responses (demonstrating the blocking behavior).
//! - Decide based on majority vote when all responses arrive.
//!
//! ## Verification
//!
//! - Verify consensus succeeds when all processes are responsive.
//! - Verify the protocol deadlocks when one process is slow/crashed.
//! - Verify that the decision value satisfies agreement and validity.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// A proposal value for consensus.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Proposal {
    Zero,
    One,
}

impl Proposal {
    /// Returns true if this is the One proposal.
    pub fn is_one(&self) -> bool {
        matches!(self, Proposal::One)
    }
}

/// A process participating in consensus.
#[derive(Debug)]
pub struct ConsensusProcess {
    /// The unique ID of this process.
    pub id: u32,
    /// The value this process proposes.
    pub proposal: Proposal,
    /// Messages received from other processes.
    pub received_proposals: HashMap<u32, Proposal>,
    /// Whether this process has decided.
    pub decided: bool,
    /// The decided value, if any.
    pub decision: Option<Proposal>,
}

impl ConsensusProcess {
    /// Creates a new process with the given ID and proposal.
    pub fn new(id: u32, proposal: Proposal) -> Self {
        Self {
            id,
            proposal,
            received_proposals: HashMap::new(),
            decided: false,
            decision: None,
        }
    }

    /// Record receiving a proposal from another process.
    pub fn receive_proposal(&mut self, from_id: u32, proposal: Proposal) {
        self.received_proposals.insert(from_id, proposal);
    }

    /// Check if this process has received proposals from all expected processes.
    pub fn has_all_proposals(&self, total_processes: usize) -> bool {
        self.received_proposals.len() >= total_processes - 1
    }

    /// Attempt to decide based on the majority of proposals received (including own).
    /// Returns `true` if a decision was made.
    pub fn try_decide(&mut self, total_processes: usize) -> bool {
        if self.decided {
            return true;
        }
        if !self.has_all_proposals(total_processes) {
            return false;
        }

        // Count votes.
        let mut ones = 0u32;
        let mut zeros = 0u32;

        if self.proposal.is_one() {
            ones += 1;
        } else {
            zeros += 1;
        }

        for p in self.received_proposals.values() {
            if p.is_one() {
                ones += 1;
            } else {
                zeros += 1;
            }
        }

        // Simple majority decision.
        if ones > zeros {
            self.decision = Some(Proposal::One);
        } else {
            self.decision = Some(Proposal::Zero);
        }
        self.decided = true;
        true
    }
}

/// Shared state for a consensus round.
pub struct ConsensusRound {
    /// All processes in this round.
    pub processes: Vec<Arc<Mutex<ConsensusProcess>>>,
    /// Number of processes.
    pub num_processes: usize,
}

impl ConsensusRound {
    /// Creates a new consensus round with the given proposals.
    pub fn new(proposals: Vec<Proposal>) -> Self {
        let processes: Vec<Arc<Mutex<ConsensusProcess>>> = proposals
            .into_iter()
            .enumerate()
            .map(|(i, p)| Arc::new(Mutex::new(ConsensusProcess::new(i as u32, p))))
            .collect();
        let num_processes = processes.len();
        Self {
            processes,
            num_processes,
        }
    }

    /// Simulate one round of message exchange.
    /// Returns the number of processes that have decided.
    pub fn exchange_messages(&self) -> usize {
        // Collect all proposals.
        let proposals: Vec<(u32, Proposal)> = self
            .processes
            .iter()
            .map(|p| {
                let proc = p.lock().unwrap();
                (proc.id, proc.proposal.clone())
            })
            .collect();

        // Deliver all proposals to all processes.
        for proc_arc in &self.processes {
            let mut proc = proc_arc.lock().unwrap();
            for (id, proposal) in &proposals {
                if *id != proc.id {
                    proc.receive_proposal(*id, proposal.clone());
                }
            }
        }

        // Attempt decision.
        let mut decided_count = 0;
        for proc_arc in &self.processes {
            let mut proc = proc_arc.lock().unwrap();
            if proc.try_decide(self.num_processes) {
                decided_count += 1;
            }
        }
        decided_count
    }

    /// Check if all processes have decided.
    pub fn all_decided(&self) -> bool {
        self.processes
            .iter()
            .all(|p| p.lock().unwrap().decided)
    }

    /// Get all decisions.
    pub fn get_decisions(&self) -> Vec<Option<Proposal>> {
        self.processes
            .iter()
            .map(|p| p.lock().unwrap().decision.clone())
            .collect()
    }
}

/// Simulate the deterministic consensus protocol.
///
/// Returns `Ok(decisions)` if all processes decided, or `Err(timeout)` if the
/// protocol deadlocks within the given time limit.
pub fn simulate_consensus(
    proposals: Vec<Proposal>,
    timeout: Duration,
) -> Result<Vec<Option<Proposal>>, Duration> {
    let round = ConsensusRound::new(proposals);
    let start = Instant::now();

    loop {
        if round.all_decided() {
            return Ok(round.get_decisions());
        }
        if start.elapsed() >= timeout {
            return Err(start.elapsed());
        }

        let decided = round.exchange_messages();
        if decided == 0 && round.processes.iter().any(|p| !p.lock().unwrap().has_all_proposals(round.num_processes)) {
            // No progress -- protocol is deadlocked.
            return Err(start.elapsed());
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consensus_all_agree_one() {
        let proposals = vec![Proposal::One, Proposal::One, Proposal::One];
        let result = simulate_consensus(proposals, Duration::from_secs(5));
        assert!(result.is_ok(), "Consensus should succeed when all agree");
        let decisions = result.unwrap();
        assert!(
            decisions.iter().all(|d| *d == Some(Proposal::One)),
            "All processes should decide One"
        );
    }

    #[test]
    fn consensus_all_agree_zero() {
        let proposals = vec![Proposal::Zero, Proposal::Zero, Proposal::Zero];
        let result = simulate_consensus(proposals, Duration::from_secs(5));
        assert!(result.is_ok(), "Consensus should succeed when all agree");
        let decisions = result.unwrap();
        assert!(
            decisions.iter().all(|d| *d == Some(Proposal::Zero)),
            "All processes should decide Zero"
        );
    }

    #[test]
    fn consensus_majority_one() {
        let proposals = vec![Proposal::One, Proposal::One, Proposal::Zero];
        let result = simulate_consensus(proposals, Duration::from_secs(5));
        assert!(result.is_ok(), "Consensus should succeed with majority");
        let decisions = result.unwrap();
        assert!(
            decisions.iter().all(|d| *d == Some(Proposal::One)),
            "All processes should decide One (majority)"
        );
    }

    #[test]
    fn consensus_agreement_holds() {
        let proposals = vec![Proposal::Zero, Proposal::One, Proposal::One];
        let result = simulate_consensus(proposals, Duration::from_secs(5));
        assert!(result.is_ok());
        let decisions = result.unwrap();
        let first = decisions[0].clone().unwrap();
        assert!(
            decisions.iter().all(|d| *d == Some(first.clone())),
            "Agreement property violated: not all decisions match"
        );
    }

    #[test]
    fn consensus_validity_holds() {
        let proposals = vec![Proposal::One, Proposal::Zero, Proposal::One];
        let result = simulate_consensus(proposals.clone(), Duration::from_secs(5));
        assert!(result.is_ok());
        let decisions = result.unwrap();
        let proposed_values: Vec<&Proposal> = proposals.iter().collect();
        for decision in &decisions {
            let d = decision.as_ref().unwrap();
            assert!(
                proposed_values.contains(&d),
                "Validity violated: decided value was not proposed by anyone"
            );
        }
    }

    #[test]
    fn single_process_decides_immediately() {
        let proposals = vec![Proposal::One];
        let result = simulate_consensus(proposals, Duration::from_secs(1));
        assert!(result.is_ok());
        let decisions = result.unwrap();
        assert_eq!(decisions, vec![Some(Proposal::One)]);
    }
}
