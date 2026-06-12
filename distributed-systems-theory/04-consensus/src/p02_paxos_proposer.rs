//! # Exercise: Paxos Proposer
//!
//! ## Theory
//!
//! The Paxos proposer is responsible for driving the consensus protocol. It
//! operates in two phases:
//!
//! **Phase 1a (Prepare):** The proposer selects a unique proposal number `n`
//! (higher than any it has used before) and sends a `Prepare(n)` message to
//! a majority of acceptors. This probes the acceptors to learn about any
//! previously accepted values.
//!
//! **Phase 2a (Accept):** After receiving a majority of promises, the proposer
//! sends an `Accept(n, v)` message. The value `v` is:
//! - The value from the promise with the highest proposal number (if any),
//!   OR
//! - The proposer's own proposed value (if no acceptor has accepted anything).
//!
//! This ensures that if any value was previously accepted, the proposer
//! continues that value rather than introducing a new one (preserving
//! the safety property).
//!
//! ## Proof / Intuition
//!
//! The key insight of Paxos is that proposal numbers provide a total order on
//! proposals. By always choosing the value from the highest-numbered promise,
//! the proposer ensures that it doesn't "override" a previously accepted
//! value. This is what makes Paxos safe: once a value is accepted by a
//! majority, any subsequent proposer will learn about it and continue it.
//!
//! ## Implementation Task
//!
//! Implement a `Proposer` that:
//! - Generates unique proposal numbers
//! - Creates `PrepareMessage` for Phase 1a
//! - Processes `PromiseMessage` responses for Phase 2a
//! - Selects the correct value for the Accept message
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Prepare messages have correct proposal numbers
//! - Promise handling selects the highest-numbered accepted value
//! - Fallback to own value when no promises include accepted values

/// A prepare message sent in Phase 1a.
#[derive(Debug, Clone)]
pub struct PrepareMessage {
    pub proposal_number: u64,
    pub proposer_id: usize,
}

/// A promise message returned by an acceptor in Phase 1b.
#[derive(Debug, Clone)]
pub struct PromiseMessage {
    pub proposal_number: u64,
    /// The highest-numbered proposal this acceptor has accepted, if any.
    pub accepted_proposal: Option<(u64, u64)>,
}

/// An accept message sent in Phase 2a.
#[derive(Debug, Clone)]
pub struct AcceptMessage {
    pub proposal_number: u64,
    pub value: u64,
    pub proposer_id: usize,
}

/// A Paxos proposer.
///
/// Drives the two-phase protocol by sending Prepare and Accept messages.
#[derive(Debug)]
pub struct Proposer {
    pub id: usize,
    /// Current proposal number (incremented for each new round).
    proposal_number: u64,
    /// The value this proposer wants to propose.
    value: u64,
    /// Number of acceptors in the cluster.
    num_acceptors: usize,
}

impl Proposer {
    /// Create a new proposer.
    ///
    /// # Arguments
    /// * `id` - Unique identifier for this proposer
    /// * `value` - The value to propose
    /// * `num_acceptors` - Total number of acceptors in the cluster
    pub fn new(id: usize, value: u64, num_acceptors: usize) -> Self {
        Self {
            id,
            proposal_number: 0,
            value,
            num_acceptors,
        }
    }

    /// Generate the next proposal number.
    ///
    /// Proposal numbers must be monotonically increasing and unique.
    /// We use a simple scheme: `base * num_nodes + node_id`.
    fn next_proposal_number(&mut self) -> u64 {
        self.proposal_number += 1;
        self.proposal_number * self.num_acceptors as u64 + self.id as u64
    }

    /// Phase 1a: Create a Prepare message.
    ///
    /// This is the first step of a Paxos round. The proposer sends this
    /// to a majority of acceptors.
    pub fn prepare(&mut self) -> PrepareMessage {
        let n = self.next_proposal_number();
        PrepareMessage {
            proposal_number: n,
            proposer_id: self.id,
        }
    }

    /// Phase 2a: Process promise messages and create an Accept message.
    ///
    /// After receiving promises from a majority of acceptors, the proposer
    /// determines the value for the Accept message:
    /// - If any promise includes a previously accepted value, use the one
    ///   with the highest proposal number.
    /// - Otherwise, use the proposer's own value.
    ///
    /// Returns `None` if we didn't receive enough promises (no quorum).
    pub fn handle_promise(
        &mut self,
        promises: Vec<PromiseMessage>,
    ) -> Option<AcceptMessage> {
        let quorum = self.num_acceptors / 2 + 1;
        if promises.len() < quorum {
            return None;
        }

        // Find the promise with the highest accepted proposal number
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

        // Use the value from the highest-numbered accepted proposal,
        // or fall back to our own value
        let value = best_proposal
            .map(|(_, v)| v)
            .unwrap_or(self.value);

        let proposal_number = promises[0].proposal_number;

        Some(AcceptMessage {
            proposal_number,
            value,
            proposer_id: self.id,
        })
    }

    /// Get the proposer's original proposed value.
    pub fn proposed_value(&self) -> u64 {
        self.value
    }

    /// Get the current proposal number.
    pub fn current_proposal_number(&self) -> u64 {
        self.proposal_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_sends_correct_proposal_number() {
        let mut proposer = Proposer::new(0, 42, 3);
        let msg = proposer.prepare();
        assert!(msg.proposal_number > 0);
        assert_eq!(msg.proposer_id, 0);
    }

    #[test]
    fn proposal_numbers_are_increasing() {
        let mut proposer = Proposer::new(1, 10, 3);
        let m1 = proposer.prepare();
        let m2 = proposer.prepare();
        let m3 = proposer.prepare();
        assert!(m2.proposal_number > m1.proposal_number);
        assert!(m3.proposal_number > m2.proposal_number);
    }

    #[test]
    fn handle_promise_with_no_accepted_values() {
        let mut proposer = Proposer::new(0, 42, 3);
        let prepare = proposer.prepare();

        let promises = vec![
            PromiseMessage {
                proposal_number: prepare.proposal_number,
                accepted_proposal: None,
            },
            PromiseMessage {
                proposal_number: prepare.proposal_number,
                accepted_proposal: None,
            },
        ];

        let accept = proposer.handle_promise(promises).unwrap();
        // Should use own value since no acceptor has accepted anything
        assert_eq!(accept.value, 42);
        assert_eq!(accept.proposal_number, prepare.proposal_number);
    }

    #[test]
    fn handle_promise_selects_highest_accepted() {
        let mut proposer = Proposer::new(0, 42, 3);
        let prepare = proposer.prepare();

        let promises = vec![
            PromiseMessage {
                proposal_number: prepare.proposal_number,
                accepted_proposal: Some((10, 99)), // Lower proposal number
            },
            PromiseMessage {
                proposal_number: prepare.proposal_number,
                accepted_proposal: Some((20, 77)), // Higher proposal number
            },
        ];

        let accept = proposer.handle_promise(promises).unwrap();
        // Should use value from the highest-numbered accepted proposal
        assert_eq!(accept.value, 77);
    }

    #[test]
    fn handle_promise_returns_none_without_quorum() {
        let mut proposer = Proposer::new(0, 42, 5);
        let _prepare = proposer.prepare();

        let promises = vec![
            PromiseMessage {
                proposal_number: 1,
                accepted_proposal: None,
            },
            // Only 1 promise -- need 3 for quorum with 5 acceptors
        ];

        assert!(proposer.handle_promise(promises).is_none());
    }

    #[test]
    fn proposal_number_includes_node_id() {
        let mut p0 = Proposer::new(0, 1, 3);
        let mut p1 = Proposer::new(1, 2, 3);
        let m0 = p0.prepare();
        let m1 = p1.prepare();
        // Different proposers should have different proposal numbers
        assert_ne!(m0.proposal_number, m1.proposal_number);
    }
}
