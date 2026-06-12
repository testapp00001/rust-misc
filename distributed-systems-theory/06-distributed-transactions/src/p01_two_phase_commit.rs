//! # Exercise: Two-Phase Commit (2PC)
//!
//! ## Theory
//!
//! Two-Phase Commit (2PC) is the foundational distributed atomic commit protocol.
//! It ensures that either all participants commit a transaction or all abort.
//!
//! The protocol involves:
//! - A **coordinator** that orchestrates the protocol.
//! - Multiple **participants** that execute the transaction.
//!
//! ## Proof / Intuition
//!
//! The protocol has two phases:
//!
//! **Phase 1 (Prepare/Vote):**
//! 1. The coordinator sends a PREPARE message to all participants.
//! 2. Each participant executes the transaction up to the commit point (but does
//!    not commit yet).
//! 3. Each participant votes YES (can commit) or NO (must abort).
//! 4. Participants write the vote to a WAL (write-ahead log) before responding.
//!
//! **Phase 2 (Commit/Abort):**
//! 1. If ALL participants vote YES, the coordinator decides COMMIT.
//!    - It writes the decision to its WAL.
//! 2. If ANY participant votes NO, the coordinator decides ABORT.
//! 3. The coordinator sends the decision to all participants.
//! 4. Participants apply the decision (commit or abort) and acknowledge.
//!
//! **Atomicity guarantee:** Since participants record their votes before responding,
//! and the coordinator records the decision before sending it, recovery is possible.
//! A crashed participant can recover by checking its WAL. A crashed coordinator can
//! recover by checking its WAL and contacting participants.
//!
//! **Blocking problem:** After voting YES, a participant is BLOCKED -- it cannot decide
//! the outcome on its own. It must wait for the coordinator's decision. If the
//! coordinator crashes, the participant holds locks indefinitely.
//!
//! ## Implementation Task
//!
//! Implement 2PC:
//!
//! - `Vote`: YES or NO from a participant.
//! - `ParticipantState`: Init, Prepared, Committed, Aborted.
//! - `Participant`: with `prepare()` and `commit()` / `abort()` methods.
//! - `Coordinator`: with `phase1_prepare()` and `phase2_commit()` / `phase2_abort()`.
//!
//! ## Verification
//!
//! - Verify all-commit scenario when all participants vote YES.
//! - Verify abort when any participant votes NO.
//! - Verify WAL entries are written correctly.

use std::collections::HashMap;

/// A participant's vote in phase 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vote {
    Yes,
    No,
}

/// State of a participant in the 2PC protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticipantState {
    /// Initial state.
    Init,
    /// Participant has voted YES and is waiting for decision.
    Prepared,
    /// Participant has committed.
    Committed,
    /// Participant has aborted.
    Aborted,
}

/// Write-ahead log entry for recovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalEntry {
    /// Participant voted YES.
    VoteYes { transaction_id: u64 },
    /// Participant voted NO.
    VoteNo { transaction_id: u64 },
    /// Coordinator decided COMMIT.
    Commit { transaction_id: u64 },
    /// Coordinator decided ABORT.
    Abort { transaction_id: u64 },
}

/// A participant in the 2PC protocol.
#[derive(Debug)]
pub struct Participant {
    /// Unique identifier.
    pub id: u32,
    /// Current state.
    pub state: ParticipantState,
    /// Write-ahead log.
    pub wal: Vec<WalEntry>,
    /// Transaction data (simulated).
    pub data: HashMap<String, String>,
    /// Whether this participant will vote YES.
    pub will_vote_yes: bool,
}

impl Participant {
    /// Creates a new participant.
    pub fn new(id: u32, will_vote_yes: bool) -> Self {
        Self {
            id,
            state: ParticipantState::Init,
            wal: Vec::new(),
            data: HashMap::new(),
            will_vote_yes,
        }
    }

    /// Phase 1: Prepare the transaction.
    /// Returns the participant's vote.
    pub fn prepare(&mut self, transaction_id: u64) -> Vote {
        if self.will_vote_yes {
            self.wal.push(WalEntry::VoteYes { transaction_id });
            self.state = ParticipantState::Prepared;
            Vote::Yes
        } else {
            self.wal.push(WalEntry::VoteNo { transaction_id });
            self.state = ParticipantState::Aborted;
            Vote::No
        }
    }

    /// Phase 2: Commit the transaction.
    pub fn commit(&mut self, transaction_id: u64) {
        self.wal.push(WalEntry::Commit { transaction_id });
        self.state = ParticipantState::Committed;
        self.data
            .insert("status".to_string(), "committed".to_string());
    }

    /// Phase 2: Abort the transaction.
    pub fn abort(&mut self, transaction_id: u64) {
        self.wal.push(WalEntry::Abort { transaction_id });
        self.state = ParticipantState::Aborted;
        self.data
            .insert("status".to_string(), "aborted".to_string());
    }
}

/// Coordinator state in the 2PC protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoordinatorState {
    Idle,
    CollectingVotes,
    Decided,
}

/// Result of the 2PC protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TwoPcResult {
    Commit,
    Abort,
}

/// The 2PC coordinator.
#[derive(Debug)]
pub struct Coordinator {
    /// Participants managed by this coordinator.
    pub participant_ids: Vec<u32>,
    /// Current state.
    pub state: CoordinatorState,
    /// Write-ahead log.
    pub wal: Vec<WalEntry>,
    /// Collected votes.
    pub votes: HashMap<u32, Vote>,
    /// The decided outcome.
    pub decision: Option<TwoPcResult>,
}

impl Coordinator {
    /// Creates a new coordinator.
    pub fn new(participant_ids: Vec<u32>) -> Self {
        Self {
            participant_ids,
            state: CoordinatorState::Idle,
            wal: Vec::new(),
            votes: HashMap::new(),
            decision: None,
        }
    }

    /// Phase 1: Send PREPARE to all participants and collect votes.
    ///
    /// # Arguments
    /// * `transaction_id` - The transaction to prepare.
    /// * `participants` - Mutable references to all participants.
    pub fn phase1_prepare(
        &mut self,
        transaction_id: u64,
        participants: &mut [Participant],
    ) -> Vec<(u32, Vote)> {
        self.state = CoordinatorState::CollectingVotes;
        let mut votes = Vec::new();

        for participant in participants.iter_mut() {
            let vote = participant.prepare(transaction_id);
            self.votes.insert(participant.id, vote.clone());
            votes.push((participant.id, vote));
        }

        votes
    }

    /// Phase 2: Send COMMIT to all participants if all voted YES.
    /// Returns the decision.
    pub fn phase2_commit(
        &mut self,
        transaction_id: u64,
        participants: &mut [Participant],
    ) -> TwoPcResult {
        let all_yes = self.votes.values().all(|v| *v == Vote::Yes);

        let decision = if all_yes {
            self.wal.push(WalEntry::Commit { transaction_id });
            TwoPcResult::Commit
        } else {
            self.wal.push(WalEntry::Abort { transaction_id });
            TwoPcResult::Abort
        };

        for participant in participants.iter_mut() {
            match &decision {
                TwoPcResult::Commit => participant.commit(transaction_id),
                TwoPcResult::Abort => participant.abort(transaction_id),
            }
        }

        self.decision = Some(decision.clone());
        self.state = CoordinatorState::Decided;
        decision
    }

    /// Convenience: run both phases and return the result.
    pub fn execute(
        &mut self,
        transaction_id: u64,
        participants: &mut [Participant],
    ) -> TwoPcResult {
        self.phase1_prepare(transaction_id, participants);
        self.phase2_commit(transaction_id, participants)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_commit_when_all_vote_yes() {
        let mut coordinator = Coordinator::new(vec![0, 1, 2]);
        let mut participants = vec![
            Participant::new(0, true),
            Participant::new(1, true),
            Participant::new(2, true),
        ];

        let result = coordinator.execute(1, &mut participants);

        assert_eq!(result, TwoPcResult::Commit);
        for p in &participants {
            assert_eq!(p.state, ParticipantState::Committed);
        }
    }

    #[test]
    fn abort_when_any_votes_no() {
        let mut coordinator = Coordinator::new(vec![0, 1, 2]);
        let mut participants = vec![
            Participant::new(0, true),
            Participant::new(1, false), // This one votes NO.
            Participant::new(2, true),
        ];

        let result = coordinator.execute(1, &mut participants);

        assert_eq!(result, TwoPcResult::Abort);
        for p in &participants {
            assert_eq!(p.state, ParticipantState::Aborted);
        }
    }

    #[test]
    fn wal_entries_written_correctly() {
        let mut coordinator = Coordinator::new(vec![0, 1]);
        let mut participants = vec![Participant::new(0, true), Participant::new(1, true)];

        coordinator.execute(42, &mut participants);

        // Coordinator should have Commit entry.
        assert!(coordinator.wal.contains(&WalEntry::Commit { transaction_id: 42 }));

        // Participants should have VoteYes and Commit entries.
        assert!(participants[0].wal.contains(&WalEntry::VoteYes { transaction_id: 42 }));
        assert!(participants[0].wal.contains(&WalEntry::Commit { transaction_id: 42 }));
    }

    #[test]
    fn single_participant_commits() {
        let mut coordinator = Coordinator::new(vec![0]);
        let mut participants = vec![Participant::new(0, true)];

        let result = coordinator.execute(1, &mut participants);
        assert_eq!(result, TwoPcResult::Commit);
        assert_eq!(participants[0].state, ParticipantState::Committed);
    }

    #[test]
    fn phase1_returns_all_votes() {
        let mut coordinator = Coordinator::new(vec![0, 1]);
        let mut participants = vec![Participant::new(0, true), Participant::new(1, false)];

        let votes = coordinator.phase1_prepare(1, &mut participants);

        assert_eq!(votes.len(), 2);
        assert!(votes.iter().any(|(id, v)| *id == 0 && *v == Vote::Yes));
        assert!(votes.iter().any(|(id, v)| *id == 1 && *v == Vote::No));
    }

    #[test]
    fn all_abort_when_all_vote_no() {
        let mut coordinator = Coordinator::new(vec![0, 1]);
        let mut participants = vec![Participant::new(0, false), Participant::new(1, false)];

        let result = coordinator.execute(1, &mut participants);
        assert_eq!(result, TwoPcResult::Abort);
    }
}
