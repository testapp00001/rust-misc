//! # Exercise: 2PC Coordinator Crash
//!
//! ## Theory
//!
//! The blocking problem of 2PC is its critical weakness. When the coordinator crashes
//! after phase 1 (collecting votes) but before phase 2 (sending the decision),
//! participants are left in an ambiguous state:
//!
//! - Participants that voted YES are BLOCKED.
//! - They cannot commit (because the coordinator might have decided to abort).
//! - They cannot abort (because they already voted YES and the coordinator might
//!   have decided to commit).
//! - They must hold their locks until the coordinator recovers.
//!
//! ## Proof / Intuition
//!
//! Consider this scenario:
//!
//! 1. Coordinator sends PREPARE to P1 and P2.
//! 2. P1 votes YES, P2 votes YES.
//! 3. Coordinator receives both votes and decides COMMIT.
//! 4. Coordinator writes COMMIT to its WAL.
//! 5. Coordinator crashes before sending COMMIT to P1 and P2.
//!
//! Now:
//! - P1 has voted YES but doesn't know the decision. It's blocked.
//! - P2 has voted YES but doesn't know the decision. It's blocked.
//! - Both hold locks, preventing other transactions from accessing the data.
//!
//! The coordinator MUST recover for the protocol to complete. There is no timeout
//! or failure detector that can safely resolve this -- any decision might violate
//! atomicity.
//!
//! ## Implementation Task
//!
//! Simulate coordinator crash in 2PC:
//!
//! - `CrashableCoordinator`: A coordinator that can crash at a configured point.
//! - `BlockingParticipant`: A participant that blocks after voting YES.
//! - Simulate: crash after collecting votes, before sending decision.
//! - Simulate recovery: coordinator restarts, reads WAL, sends decision.
//!
//! ## Verification
//!
//! - Verify participants block after coordinator crash.
//! - Verify recovery after coordinator restarts resolves the block.
//! - Verify WAL enables correct recovery.

use std::collections::HashMap;

/// Reuse the 2PC types from p01.
use crate::p01_two_phase_commit::{
    CoordinatorState, Participant, ParticipantState, TwoPcResult, Vote, WalEntry,
};

/// The point at which the coordinator crashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashPoint {
    /// Never crash.
    Never,
    /// Crash after collecting all votes but before deciding.
    AfterCollectingVotes,
    /// Crash after writing decision to WAL but before sending.
    AfterDecision,
}

/// A coordinator that can crash at a specific point in the protocol.
pub struct CrashableCoordinator {
    /// Base coordinator state.
    pub participant_ids: Vec<u32>,
    pub state: CoordinatorState,
    pub wal: Vec<WalEntry>,
    pub votes: HashMap<u32, Vote>,
    pub decision: Option<TwoPcResult>,
    /// When to crash.
    pub crash_point: CrashPoint,
    /// Whether the coordinator has crashed.
    pub crashed: bool,
    /// Phase counter for tracking crash point.
    phase_counter: u32,
}

impl CrashableCoordinator {
    /// Creates a new crashable coordinator.
    pub fn new(participant_ids: Vec<u32>, crash_point: CrashPoint) -> Self {
        Self {
            participant_ids,
            state: CoordinatorState::Idle,
            wal: Vec::new(),
            votes: HashMap::new(),
            decision: None,
            crash_point,
            crashed: false,
            phase_counter: 0,
        }
    }

    /// Check if the coordinator should crash at this point.
    fn should_crash(&mut self) -> bool {
        self.phase_counter += 1;
        match self.crash_point {
            CrashPoint::AfterCollectingVotes if self.phase_counter == 1 => {
                self.crashed = true;
                true
            }
            CrashPoint::AfterDecision if self.phase_counter == 2 => {
                self.crashed = true;
                true
            }
            _ => false,
        }
    }

    /// Phase 1: Collect votes from participants.
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

        if self.should_crash() {
            return votes; // Return votes but crash before deciding.
        }

        votes
    }

    /// Phase 2: Send decision. Returns `Err` if coordinator has crashed.
    pub fn phase2_commit(
        &mut self,
        transaction_id: u64,
        participants: &mut [Participant],
    ) -> Result<TwoPcResult, String> {
        if self.crashed {
            return Err("Coordinator has crashed".to_string());
        }

        let all_yes = self.votes.values().all(|v| *v == Vote::Yes);

        let decision = if all_yes {
            self.wal.push(WalEntry::Commit { transaction_id });
            TwoPcResult::Commit
        } else {
            self.wal.push(WalEntry::Abort { transaction_id });
            TwoPcResult::Abort
        };

        if self.should_crash() {
            self.decision = Some(decision.clone());
            return Err("Coordinator crashed after writing WAL".to_string());
        }

        for participant in participants.iter_mut() {
            match &decision {
                TwoPcResult::Commit => participant.commit(transaction_id),
                TwoPcResult::Abort => participant.abort(transaction_id),
            }
        }

        self.decision = Some(decision.clone());
        self.state = CoordinatorState::Decided;
        Ok(decision)
    }

    /// Simulate recovery: read WAL and determine the decision.
    pub fn recover(&mut self) -> Option<TwoPcResult> {
        self.crashed = false;
        // Find the latest commit or abort entry in WAL.
        for entry in self.wal.iter().rev() {
            match entry {
                WalEntry::Commit { .. } => {
                    self.decision = Some(TwoPcResult::Commit);
                    self.state = CoordinatorState::Decided;
                    return Some(TwoPcResult::Commit);
                }
                WalEntry::Abort { .. } => {
                    self.decision = Some(TwoPcResult::Abort);
                    self.state = CoordinatorState::Decided;
                    return Some(TwoPcResult::Abort);
                }
                _ => {}
            }
        }
        None
    }

    /// After recovery, send the decision to participants.
    pub fn send_decision_after_recovery(
        &self,
        transaction_id: u64,
        participants: &mut [Participant],
    ) -> Result<TwoPcResult, String> {
        match &self.decision {
            Some(decision) => {
                for participant in participants.iter_mut() {
                    match decision {
                        TwoPcResult::Commit => participant.commit(transaction_id),
                        TwoPcResult::Abort => participant.abort(transaction_id),
                    }
                }
                Ok(decision.clone())
            }
            None => Err("No decision found in WAL".to_string()),
        }
    }
}

/// Check if any participant is in the Prepared (blocked) state.
pub fn any_participant_blocked(participants: &[Participant]) -> bool {
    participants.iter().any(|p| p.state == ParticipantState::Prepared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p01_two_phase_commit::Coordinator;

    #[test]
    fn participants_block_after_coordinator_crash() {
        let mut coordinator = CrashableCoordinator::new(
            vec![0, 1, 2],
            CrashPoint::AfterCollectingVotes,
        );
        let mut participants = vec![
            Participant::new(0, true),
            Participant::new(1, true),
            Participant::new(2, true),
        ];

        coordinator.phase1_prepare(1, &mut participants);

        // Coordinator crashed before deciding.
        assert!(coordinator.crashed, "Coordinator should have crashed");

        // Participants that voted YES are blocked.
        assert!(
            any_participant_blocked(&participants),
            "Participants should be blocked after coordinator crash"
        );
    }

    #[test]
    fn recovery_resolves_blocked_participants() {
        let mut coordinator = CrashableCoordinator::new(
            vec![0, 1],
            CrashPoint::AfterCollectingVotes,
        );
        let mut participants = vec![Participant::new(0, true), Participant::new(1, true)];

        coordinator.phase1_prepare(1, &mut participants);
        assert!(coordinator.crashed);

        // Participants are blocked.
        assert!(any_participant_blocked(&participants));

        // Simulate recovery: write decision to WAL manually.
        coordinator.wal.push(WalEntry::Commit { transaction_id: 1 });
        let recovered_decision = coordinator.recover();

        assert_eq!(recovered_decision, Some(TwoPcResult::Commit));

        // Send decision after recovery.
        let result = coordinator.send_decision_after_recovery(1, &mut participants);
        assert!(result.is_ok());

        // Participants should no longer be blocked.
        assert!(
            !any_participant_blocked(&participants),
            "Participants should be unblocked after coordinator recovery"
        );
        for p in &participants {
            assert_eq!(p.state, ParticipantState::Committed);
        }
    }

    #[test]
    fn wal_enables_correct_recovery() {
        let mut coordinator = CrashableCoordinator::new(
            vec![0, 1],
            CrashPoint::AfterDecision,
        );
        let mut participants = vec![Participant::new(0, true), Participant::new(1, false)];

        let _ = coordinator.phase1_prepare(1, &mut participants);
        let _ = coordinator.phase2_commit(1, &mut participants);

        // Coordinator crashed after writing WAL but before sending.
        assert!(coordinator.crashed);

        // Recovery reads WAL.
        let decision = coordinator.recover();
        assert_eq!(decision, Some(TwoPcResult::Abort), "WAL should show ABORT since one voted NO");
    }

    #[test]
    fn normal_2pc_still_works() {
        let mut coordinator = Coordinator::new(vec![0, 1, 2]);
        let mut participants = vec![
            Participant::new(0, true),
            Participant::new(1, true),
            Participant::new(2, true),
        ];

        let result = coordinator.execute(1, &mut participants);
        assert_eq!(result, TwoPcResult::Commit);
        assert!(!any_participant_blocked(&participants));
    }

    #[test]
    fn single_participant_crash_blocks() {
        let mut coordinator = CrashableCoordinator::new(
            vec![0],
            CrashPoint::AfterCollectingVotes,
        );
        let mut participants = vec![Participant::new(0, true)];

        coordinator.phase1_prepare(1, &mut participants);
        assert!(coordinator.crashed);
        assert!(any_participant_blocked(&participants));
    }
}
