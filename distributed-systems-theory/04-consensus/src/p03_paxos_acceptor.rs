//! # Exercise: Paxos Acceptor
//!
//! ## Theory
//!
//! The Paxos acceptor is the voting component of the protocol. It responds
//! to two types of messages:
//!
//! **Phase 1b (Promise):** When an acceptor receives a `Prepare(n)` message:
//! - If `n > promised_n`, the acceptor promises not to accept any proposal
//!   with a number less than `n`. It responds with a `Promise` containing
//!   the highest-numbered proposal it has previously accepted (if any).
//! - If `n <= promised_n`, the acceptor rejects the prepare (sends a nack).
//!
//! **Phase 2b (Accepted):** When an acceptor receives an `Accept(n, v)` message:
//! - If `n >= promised_n`, the acceptor accepts the proposal, records it,
//!   and responds with `Accepted(n, v)`.
//! - If `n < promised_n`, the acceptor rejects the accept (sends a nack).
//!
//! ## Proof / Intuition
//!
//! The acceptor's role is simple but critical: by tracking the highest
//! promised proposal number, it ensures that once a value is accepted by a
//! majority, no lower-numbered proposal can override it. This is the
//! safety mechanism that prevents conflicting decisions.
//!
//! The acceptor does not need to be deterministic -- multiple acceptors may
//! accept different proposals. Safety comes from the proposer's obligation
//! to learn and continue accepted values.
//!
//! ## Implementation Task
//!
//! Implement an `Acceptor` that:
//! - Handles Prepare messages (Phase 1b)
//! - Handles Accept messages (Phase 2b)
//! - Tracks promised and accepted proposal numbers
//! - Returns previously accepted values in promises
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Promise includes previously accepted value
//! - Lower proposal numbers are rejected
//! - Accept only succeeds when n >= promised_n

/// A prepare message (Phase 1a).
#[derive(Debug, Clone)]
pub struct PrepareMessage {
    pub proposal_number: u64,
    pub proposer_id: usize,
}

/// A promise message (Phase 1b).
#[derive(Debug, Clone)]
pub struct PromiseMessage {
    pub proposal_number: u64,
    pub accepted_proposal: Option<(u64, u64)>,
    pub acceptor_id: usize,
}

/// A nack (negative acknowledgement) for rejected prepares.
#[derive(Debug, Clone)]
pub struct PrepareNack {
    pub proposal_number: u64,
    pub promised_number: u64,
    pub acceptor_id: usize,
}

/// An accept message (Phase 2a).
#[derive(Debug, Clone)]
pub struct AcceptMessage {
    pub proposal_number: u64,
    pub value: u64,
    pub proposer_id: usize,
}

/// An accepted message (Phase 2b).
#[derive(Debug, Clone)]
pub struct AcceptedMessage {
    pub proposal_number: u64,
    pub value: u64,
    pub acceptor_id: usize,
}

/// A nack for rejected accepts.
#[derive(Debug, Clone)]
pub struct AcceptNack {
    pub proposal_number: u64,
    pub promised_number: u64,
    pub acceptor_id: usize,
}

/// Result of handling a prepare message.
#[derive(Debug, Clone)]
pub enum PrepareResult {
    Promise(PromiseMessage),
    Nack(PrepareNack),
}

/// Result of handling an accept message.
#[derive(Debug, Clone)]
pub enum AcceptResult {
    Accepted(AcceptedMessage),
    Nack(AcceptNack),
}

/// A Paxos acceptor.
///
/// Tracks promise and acceptance state, and responds to proposer messages
/// according to the Paxos protocol.
#[derive(Debug)]
pub struct Acceptor {
    pub id: usize,
    /// The highest proposal number this acceptor has promised to consider.
    promised_n: u64,
    /// The proposal number of the most recently accepted proposal.
    accepted_n: Option<u64>,
    /// The value of the most recently accepted proposal.
    accepted_value: Option<u64>,
}

impl Acceptor {
    /// Create a new acceptor with no prior promises or acceptances.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            promised_n: 0,
            accepted_n: None,
            accepted_value: None,
        }
    }

    /// Phase 1b: Handle a Prepare message.
    ///
    /// If the proposal number is higher than the current promise, promise
    /// to consider it and respond with any previously accepted value.
    /// Otherwise, reject.
    pub fn handle_prepare(&mut self, msg: PrepareMessage) -> PrepareResult {
        if msg.proposal_number > self.promised_n {
            self.promised_n = msg.proposal_number;
            PrepareResult::Promise(PromiseMessage {
                proposal_number: msg.proposal_number,
                accepted_proposal: match (self.accepted_n, self.accepted_value) {
                    (Some(n), Some(v)) => Some((n, v)),
                    _ => None,
                },
                acceptor_id: self.id,
            })
        } else {
            PrepareResult::Nack(PrepareNack {
                proposal_number: msg.proposal_number,
                promised_number: self.promised_n,
                acceptor_id: self.id,
            })
        }
    }

    /// Phase 2b: Handle an Accept message.
    ///
    /// If the proposal number is at least as high as the current promise,
    /// accept the proposal. Otherwise, reject.
    pub fn handle_accept(&mut self, msg: AcceptMessage) -> AcceptResult {
        if msg.proposal_number >= self.promised_n {
            self.promised_n = msg.proposal_number;
            self.accepted_n = Some(msg.proposal_number);
            self.accepted_value = Some(msg.value);
            AcceptResult::Accepted(AcceptedMessage {
                proposal_number: msg.proposal_number,
                value: msg.value,
                acceptor_id: self.id,
            })
        } else {
            AcceptResult::Nack(AcceptNack {
                proposal_number: msg.proposal_number,
                promised_number: self.promised_n,
                acceptor_id: self.id,
            })
        }
    }

    /// Get the currently accepted value, if any.
    pub fn accepted_value(&self) -> Option<u64> {
        self.accepted_value
    }

    /// Get the currently promised proposal number.
    pub fn promised_number(&self) -> u64 {
        self.promised_n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promise_includes_previously_accepted_value() {
        let mut acceptor = Acceptor::new(0);

        // First, accept a proposal
        let accept_msg = AcceptMessage {
            proposal_number: 1,
            value: 42,
            proposer_id: 0,
        };
        acceptor.handle_accept(accept_msg);

        // Now handle a prepare with a higher number
        let prepare = PrepareMessage {
            proposal_number: 2,
            proposer_id: 1,
        };
        match acceptor.handle_prepare(prepare) {
            PrepareResult::Promise(p) => {
                assert_eq!(p.accepted_proposal, Some((1, 42)));
            }
            _ => panic!("expected promise"),
        }
    }

    #[test]
    fn reject_lower_proposal_number() {
        let mut acceptor = Acceptor::new(0);

        // Promise to proposal 5
        let prepare = PrepareMessage {
            proposal_number: 5,
            proposer_id: 0,
        };
        acceptor.handle_prepare(prepare);

        // Try to prepare with a lower number
        let prepare2 = PrepareMessage {
            proposal_number: 3,
            proposer_id: 1,
        };
        match acceptor.handle_prepare(prepare2) {
            PrepareResult::Nack(n) => {
                assert_eq!(n.promised_number, 5);
            }
            _ => panic!("expected nack"),
        }
    }

    #[test]
    fn accept_succeeds_when_n_ge_promised() {
        let mut acceptor = Acceptor::new(0);

        let accept_msg = AcceptMessage {
            proposal_number: 5,
            value: 99,
            proposer_id: 0,
        };
        match acceptor.handle_accept(accept_msg) {
            AcceptResult::Accepted(a) => {
                assert_eq!(a.value, 99);
                assert_eq!(a.proposal_number, 5);
            }
            _ => panic!("expected accepted"),
        }
    }

    #[test]
    fn accept_rejects_lower_than_promised() {
        let mut acceptor = Acceptor::new(0);

        // Promise to 10
        acceptor.handle_prepare(PrepareMessage {
            proposal_number: 10,
            proposer_id: 0,
        });

        // Try to accept with number 5
        let result = acceptor.handle_accept(AcceptMessage {
            proposal_number: 5,
            value: 42,
            proposer_id: 1,
        });
        assert!(matches!(result, AcceptResult::Nack(_)));
    }

    #[test]
    fn initially_no_accepted_value() {
        let acceptor = Acceptor::new(0);
        assert_eq!(acceptor.accepted_value(), None);
        assert_eq!(acceptor.promised_number(), 0);
    }

    #[test]
    fn accept_updates_state() {
        let mut acceptor = Acceptor::new(0);
        acceptor.handle_accept(AcceptMessage {
            proposal_number: 3,
            value: 77,
            proposer_id: 0,
        });
        assert_eq!(acceptor.accepted_value(), Some(77));
        assert_eq!(acceptor.promised_number(), 3);
    }
}
