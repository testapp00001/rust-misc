//! # Exercise: Consensus Definition
//!
//! ## Theory
//!
//! The consensus problem requires a group of processes to agree on a single
//! value. A consensus algorithm must satisfy four properties:
//!
//! - **Agreement:** All correct (non-faulty) processes decide on the same value.
//! - **Validity:** The value decided upon must have been proposed by at least
//!   one process.
//! - **Termination:** All correct processes eventually decide (the algorithm
//!   does not run forever).
//! - **Integrity (Non-triviality):** Each correct process decides at most once.
//!
//! The FLP impossibility result (1985) proves that no deterministic consensus
//! algorithm can guarantee termination in a fully asynchronous system with
//! even one crash-faulty process. Practical algorithms (Paxos, Raft) work
//! around this by assuming partial synchrony or using timeouts.
//!
//! ## Proof / Intuition
//!
//! Consider three processes trying to agree on a value. If process A proposes
//! 0 and process B proposes 1, they must somehow resolve this conflict. The
//! key insight is that a majority (quorum) is required: any two majorities
//! must overlap, ensuring shared information.
//!
//! Validity prevents "deciding" a value that nobody proposed -- the algorithm
//! must be meaningful. Agreement prevents different processes from deciding
//! different values -- the algorithm must be consistent.
//!
//! ## Implementation Task
//!
//! Define the types and trait for consensus:
//! - `ConsensusValue` type alias
//! - `ConsensusError` enum for failure modes
//! - `ConsensusProtocol` trait with `propose()` and `decided()`
//! - `verify_consensus_properties()` to validate a history of events
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - A valid history passes verification
//! - Agreement violations are detected
//! - Validity violations are detected

/// A value that can be proposed for consensus.
pub type ConsensusValue = u64;

/// Errors that can occur during consensus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusError {
    /// The algorithm could not terminate (timeout / livelock).
    Timeout,
    /// The node is no longer part of the consensus group.
    NotParticipating,
    /// A conflicting proposal is in progress.
    Conflict,
}

/// Events that can occur in a consensus history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsensusEvent {
    /// A node proposed a value: (node_id, proposed_value, timestamp).
    Propose(usize, ConsensusValue, u64),
    /// A node decided on a value: (node_id, decided_value, timestamp).
    Decide(usize, ConsensusValue, u64),
}

/// Trait for a consensus protocol implementation.
pub trait ConsensusProtocol {
    /// Propose a value for consensus.
    ///
    /// Returns the decided value on success, or an error if the proposal
    /// cannot be completed.
    fn propose(&self, value: ConsensusValue) -> Result<ConsensusValue, ConsensusError>;

    /// Check if this node has already decided on a value.
    fn decided(&self) -> Option<ConsensusValue>;
}

/// A simple single-decider consensus for testing purposes.
///
/// In a real system, this would involve multiple nodes communicating.
/// Here we simulate consensus with a simple majority-vote mechanism.
pub struct SimpleConsensus {
    node_id: usize,
    decided: Option<ConsensusValue>,
    /// Threshold: how many proposals needed before deciding.
    threshold: usize,
    /// Collected proposals from other nodes.
    proposals: Vec<ConsensusValue>,
}

impl SimpleConsensus {
    pub fn new(node_id: usize, threshold: usize) -> Self {
        Self {
            node_id,
            decided: None,
            threshold,
            proposals: Vec::new(),
        }
    }

    /// Record a proposal from another node.
    pub fn receive_proposal(&mut self, value: ConsensusValue) {
        if self.decided.is_none() {
            self.proposals.push(value);
            // Simple majority: if we have enough proposals, decide
            if self.proposals.len() >= self.threshold {
                // Decide on the most common value (simplified: last one)
                self.decided = Some(value);
            }
        }
    }

    pub fn node_id(&self) -> usize {
        self.node_id
    }
}

impl ConsensusProtocol for SimpleConsensus {
    fn propose(&self, value: ConsensusValue) -> Result<ConsensusValue, ConsensusError> {
        if self.decided.is_some() {
            return Ok(self.decided.unwrap());
        }
        // In a real system, this would broadcast and wait for responses.
        // Here we just return the proposed value as the decided value.
        Ok(value)
    }

    fn decided(&self) -> Option<ConsensusValue> {
        self.decided
    }
}

/// Verify that a history of consensus events satisfies the consensus properties.
///
/// Checks:
/// - **Agreement:** All Decided events for the same timestamp have the same value.
/// - **Validity:** Every decided value must have been proposed by some node.
/// - **Integrity:** No node decides more than once per timestamp.
pub fn verify_consensus_properties(history: &[ConsensusEvent]) -> bool {
    // Collect all proposed values
    let proposed: Vec<ConsensusValue> = history
        .iter()
        .filter_map(|e| match e {
            ConsensusEvent::Propose(_, val, _) => Some(*val),
            _ => None,
        })
        .collect();

    // Collect all decided values grouped by approximate time bucket
    let decided: Vec<(usize, ConsensusValue, u64)> = history
        .iter()
        .filter_map(|e| match e {
            ConsensusEvent::Decide(node, val, ts) => Some((*node, *val, *ts)),
            _ => None,
        })
        .collect();

    // Check agreement: all decided values should be the same
    // (for a single consensus instance)
    if decided.len() > 1 {
        let first_val = decided[0].1;
        for &(_, val, _) in &decided[1..] {
            if val != first_val {
                return false; // Agreement violated
            }
        }
    }

    // Check validity: every decided value must have been proposed
    for &(_, val, _) in &decided {
        if !proposed.contains(&val) {
            return false; // Validity violated
        }
    }

    // Check integrity: no duplicate decisions by the same node at the same time
    let mut seen_decisions = std::collections::HashSet::new();
    for &(node, val, ts) in &decided {
        if !seen_decisions.insert((node, val, ts)) {
            return false; // Integrity violated
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_history_passes() {
        let history = vec![
            ConsensusEvent::Propose(0, 42, 1),
            ConsensusEvent::Propose(1, 42, 2),
            ConsensusEvent::Decide(0, 42, 3),
            ConsensusEvent::Decide(1, 42, 4),
        ];
        assert!(verify_consensus_properties(&history));
    }

    #[test]
    fn agreement_violation_detected() {
        let history = vec![
            ConsensusEvent::Propose(0, 42, 1),
            ConsensusEvent::Propose(1, 99, 2),
            ConsensusEvent::Decide(0, 42, 3),
            ConsensusEvent::Decide(1, 99, 4), // Different value!
        ];
        assert!(!verify_consensus_properties(&history));
    }

    #[test]
    fn validity_violation_detected() {
        let history = vec![
            ConsensusEvent::Propose(0, 42, 1),
            ConsensusEvent::Decide(0, 99, 2), // Decided on a value nobody proposed
        ];
        assert!(!verify_consensus_properties(&history));
    }

    #[test]
    fn empty_history_is_valid() {
        assert!(verify_consensus_properties(&[]));
    }

    #[test]
    fn single_propose_decide_is_valid() {
        let history = vec![
            ConsensusEvent::Propose(0, 7, 1),
            ConsensusEvent::Decide(0, 7, 2),
        ];
        assert!(verify_consensus_properties(&history));
    }

    #[test]
    fn simple_consensus_decides() {
        let mut node = SimpleConsensus::new(0, 2);
        assert!(node.decided().is_none());

        node.receive_proposal(42);
        assert!(node.decided().is_none());

        node.receive_proposal(42);
        assert_eq!(node.decided(), Some(42));
    }

    #[test]
    fn consensus_protocol_trait() {
        let node = SimpleConsensus::new(0, 1);
        let result = node.propose(99).unwrap();
        assert_eq!(result, 99);
    }
}
