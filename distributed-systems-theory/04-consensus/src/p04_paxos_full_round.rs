//! # Exercise: Paxos Full Round
//!
//! ## Theory
//!
//! A full Paxos round involves multiple proposers and acceptors working
//! together to agree on a single value. The protocol proceeds through
//! Phase 1 (Prepare/Promise) and Phase 2 (Accept/Accepted), with the
//! possibility of conflicts when multiple proposers are active.
//!
//! In the presence of conflicting proposals:
//! - A proposer with a higher proposal number will preempt a lower one.
//! - The lower-numbered proposer must restart with a new, higher number.
//! - Eventually, one proposer will achieve a majority and its value will
//!   be decided.
//!
//! ## Proof / Intuition
//!
//! Safety is guaranteed because:
//! 1. An acceptor only accepts proposals with numbers >= its promised number.
//! 2. A proposer learns about previously accepted values and continues them.
//! 3. Any two majorities overlap, so a newly accepted value cannot contradict
//!    a previously accepted value.
//!
//! Liveness requires that eventually a single proposer is able to complete
//! both phases without interference. In practice, leaders (Multi-Paxos)
//! reduce contention.
//!
//! ## Implementation Task
//!
//! Simulate a full Paxos round:
//! - 3 proposers, 5 acceptors
//! - Proposers send Prepare and Accept messages
//! - Handle conflicting proposals
//! - Verify that a single value is decided
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Majority quorum is needed for decision
//! - Only one value is decided
//! - Conflicting proposals are resolved

use super::p03_paxos_acceptor::{
    AcceptMessage as AcceptorAcceptMessage,
    AcceptResult,
    Acceptor,
    PrepareMessage as AcceptorPrepareMessage,
    PrepareResult,
};

/// Result of a Paxos round.
#[derive(Debug, Clone)]
pub struct PaxosResult {
    /// The value that was decided, if any.
    pub decided_value: Option<u64>,
    /// Number of rounds needed to reach decision.
    pub rounds: usize,
}

/// Simple proposer for a full-round simulation.
/// This is a local type that works directly with the acceptor's message types.
pub struct LocalProposer {
    id: usize,
    proposal_number: u64,
    value: u64,
    num_acceptors: usize,
}

impl LocalProposer {
    pub fn new(id: usize, value: u64, num_acceptors: usize) -> Self {
        Self {
            id,
            proposal_number: 0,
            value,
            num_acceptors,
        }
    }

    fn next_proposal_number(&mut self) -> u64 {
        self.proposal_number += 1;
        self.proposal_number * self.num_acceptors as u64 + self.id as u64
    }

    /// Phase 1a: Create a Prepare message.
    fn prepare(&mut self) -> AcceptorPrepareMessage {
        let n = self.next_proposal_number();
        AcceptorPrepareMessage {
            proposal_number: n,
            proposer_id: self.id,
        }
    }

    /// Phase 2a: Process promise messages and create an Accept message.
    fn handle_promise(
        &mut self,
        promises: Vec<super::p03_paxos_acceptor::PromiseMessage>,
    ) -> Option<AcceptorAcceptMessage> {
        let quorum = self.num_acceptors / 2 + 1;
        if promises.len() < quorum {
            return None;
        }

        let mut best_proposal: Option<(u64, u64)> = None;
        for promise in &promises {
            if let Some((acc_n, acc_v)) = promise.accepted_proposal {
                match best_proposal {
                    None => best_proposal = Some((acc_n, acc_v)),
                    Some((best_n, _)) if acc_n > best_n => {
                        best_proposal = Some((acc_n, acc_v));
                    }
                    _ => {}
                }
            }
        }

        let value = best_proposal.map(|(_, v)| v).unwrap_or(self.value);
        let proposal_number = promises[0].proposal_number;

        Some(AcceptorAcceptMessage {
            proposal_number,
            value,
            proposer_id: self.id,
        })
    }
}

/// Simulate a full Paxos round with one proposer against multiple acceptors.
pub fn run_paxos_round(
    proposer: &mut LocalProposer,
    acceptors: &mut [Acceptor],
) -> PaxosResult {
    let mut rounds = 0;

    loop {
        rounds += 1;
        if rounds > 10 {
            return PaxosResult {
                decided_value: None,
                rounds,
            };
        }

        // Phase 1a: Send Prepare
        let prepare = proposer.prepare();

        // Phase 1b: Collect promises
        let mut promises = Vec::new();
        for acceptor in acceptors.iter_mut() {
            match acceptor.handle_prepare(AcceptorPrepareMessage {
                proposal_number: prepare.proposal_number,
                proposer_id: prepare.proposer_id,
            }) {
                PrepareResult::Promise(p) => promises.push(p),
                PrepareResult::Nack(_) => {}
            }
        }

        // Phase 2a: Send Accept (if we have a quorum)
        let accept_msg = match proposer.handle_promise(promises) {
            Some(msg) => msg,
            None => continue,
        };

        // Phase 2b: Collect accepted responses
        let mut accepted_count = 0;
        for acceptor in acceptors.iter_mut() {
            match acceptor.handle_accept(AcceptorAcceptMessage {
                proposal_number: accept_msg.proposal_number,
                value: accept_msg.value,
                proposer_id: accept_msg.proposer_id,
            }) {
                AcceptResult::Accepted(_) => accepted_count += 1,
                AcceptResult::Nack(_) => {}
            }
        }

        let quorum = acceptors.len() / 2 + 1;
        if accepted_count >= quorum {
            return PaxosResult {
                decided_value: Some(accept_msg.value),
                rounds,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_proposer_decides() {
        let mut acceptors: Vec<Acceptor> = (0..5).map(Acceptor::new).collect();
        let mut proposer = LocalProposer::new(0, 42, 5);
        let result = run_paxos_round(&mut proposer, &mut acceptors);

        assert!(result.decided_value.is_some());
        assert_eq!(result.decided_value, Some(42));
    }

    #[test]
    fn majority_quorum_needed() {
        let mut acceptors: Vec<Acceptor> = (0..5).map(Acceptor::new).collect();
        let mut proposer = LocalProposer::new(0, 42, 5);

        // Only give 2 promises (not enough for quorum of 3)
        let prepare = proposer.prepare();
        let mut promises = Vec::new();
        for a in acceptors[..2].iter_mut() {
            if let PrepareResult::Promise(p) = a.handle_prepare(AcceptorPrepareMessage {
                proposal_number: prepare.proposal_number,
                proposer_id: 0,
            }) {
                promises.push(p);
            }
        }

        assert!(promises.len() < 3);
        assert!(proposer.handle_promise(promises).is_none());
    }

    #[test]
    fn conflicting_proposals_resolved() {
        let mut acceptors: Vec<Acceptor> = (0..5).map(Acceptor::new).collect();

        // Proposer 0 tries first
        let mut p0 = LocalProposer::new(0, 10, 5);
        let prepare0 = p0.prepare();
        for a in acceptors.iter_mut() {
            a.handle_prepare(AcceptorPrepareMessage {
                proposal_number: prepare0.proposal_number,
                proposer_id: 0,
            });
        }

        // Proposer 1 preempts with higher number
        let mut p1 = LocalProposer::new(1, 20, 5);
        let prepare1 = p1.prepare();
        let mut promises1 = Vec::new();
        for a in acceptors.iter_mut() {
            if let PrepareResult::Promise(p) = a.handle_prepare(AcceptorPrepareMessage {
                proposal_number: prepare1.proposal_number,
                proposer_id: 1,
            }) {
                promises1.push(p);
            }
        }

        let accept = p1.handle_promise(promises1).unwrap();
        let mut accepted = 0;
        for a in acceptors.iter_mut() {
            if let AcceptResult::Accepted(_) = a.handle_accept(AcceptorAcceptMessage {
                proposal_number: accept.proposal_number,
                value: accept.value,
                proposer_id: 1,
            }) {
                accepted += 1;
            }
        }
        assert!(accepted >= 3);
        assert_eq!(accept.value, 20);
    }

    #[test]
    fn single_value_decided() {
        let mut acceptors: Vec<Acceptor> = (0..5).map(Acceptor::new).collect();
        let mut proposer = LocalProposer::new(0, 99, 5);
        let result = run_paxos_round(&mut proposer, &mut acceptors);

        let accepted_values: Vec<u64> = acceptors
            .iter()
            .filter_map(|a| a.accepted_value())
            .collect();

        if !accepted_values.is_empty() {
            let first = accepted_values[0];
            assert!(
                accepted_values.iter().all(|&v| v == first),
                "all accepted values should be the same: {accepted_values:?}"
            );
        }
        assert_eq!(result.decided_value, Some(99));
    }

    #[test]
    fn decide_with_three_node_cluster() {
        let mut acceptors: Vec<Acceptor> = (0..3).map(Acceptor::new).collect();
        let mut proposer = LocalProposer::new(0, 77, 3);
        let result = run_paxos_round(&mut proposer, &mut acceptors);
        assert!(result.decided_value.is_some());
        assert_eq!(result.decided_value, Some(77));
    }
}
