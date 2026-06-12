//! # Exercise: Three-Phase Commit (3PC)
//!
//! ## Theory
//!
//! Three-Phase Commit (3PC) attempts to solve the blocking problem of 2PC by adding
//! a pre-commit phase. The idea is that after participants acknowledge the pre-commit,
//! they know that ALL participants voted YES and the coordinator decided to commit.
//!
//! If the coordinator crashes after the pre-commit phase, participants can proceed
//! with the commit independently.
//!
//! ## Proof / Intuition
//!
//! The protocol has three phases:
//!
//! **Phase 1 (Prepare):** Same as 2PC.
//! 1. Coordinator sends PREPARE to all participants.
//! 2. Participants vote YES/NO.
//!
//! **Phase 2 (Pre-commit):** New phase.
//! 1. If all voted YES, coordinator sends PRE-COMMIT.
//! 2. Participants acknowledge PRE-COMMIT.
//! 3. At this point, participants know all voted YES.
//!
//! **Phase 3 (Commit):**
//! 1. Coordinator sends COMMIT.
//! 2. Participants commit.
//!
//! If the coordinator crashes after phase 2 (PRE-COMMIT):
//! - Participants that received PRE-COMMIT know all voted YES.
//! - They can safely commit without waiting for the coordinator.
//!
//! **Limitation:** 3PC assumes no network partitions. Under partitions, it can violate
//! consistency (see exercise p04).
//!
//! ## Implementation Task
//!
//! Implement 3PC:
//!
//! - `ParticipantState3Pc`: Init, Prepared, PreCommitted, Committed, Aborted.
//! - `Participant3Pc`: with prepare(), pre_commit(), commit(), abort().
//! - `ThreePhaseCoordinator`: with phase1_prepare(), phase2_pre_commit(), phase3_commit().
//! - `Crashable3PcCoordinator`: coordinator that can crash at each phase.
//!
//! ## Verification
//!
//! - Verify normal case works (all commit).
//! - Verify non-blocking when coordinator crashes after pre-commit.
//! - Verify abort when any participant votes NO.

/// State of a participant in 3PC.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParticipantState3Pc {
    Init,
    Prepared,
    PreCommitted,
    Committed,
    Aborted,
}

/// A participant in 3PC.
#[derive(Debug)]
pub struct Participant3Pc {
    pub id: u32,
    pub state: ParticipantState3Pc,
    pub will_vote_yes: bool,
    pub log: Vec<String>,
}

impl Participant3Pc {
    pub fn new(id: u32, will_vote_yes: bool) -> Self {
        Self {
            id,
            state: ParticipantState3Pc::Init,
            will_vote_yes,
            log: Vec::new(),
        }
    }

    /// Phase 1: Vote YES or NO.
    pub fn prepare(&mut self, tx_id: u64) -> bool {
        if self.will_vote_yes {
            self.log.push(format!("PREPARE_YES({})", tx_id));
            self.state = ParticipantState3Pc::Prepared;
            true
        } else {
            self.log.push(format!("PREPARE_NO({})", tx_id));
            self.state = ParticipantState3Pc::Aborted;
            false
        }
    }

    /// Phase 2: Acknowledge pre-commit.
    pub fn pre_commit(&mut self, tx_id: u64) {
        self.log.push(format!("PRE_COMMIT({})", tx_id));
        self.state = ParticipantState3Pc::PreCommitted;
    }

    /// Phase 3: Commit.
    pub fn commit(&mut self, tx_id: u64) {
        self.log.push(format!("COMMIT({})", tx_id));
        self.state = ParticipantState3Pc::Committed;
    }

    /// Abort the transaction.
    pub fn abort(&mut self, tx_id: u64) {
        self.log.push(format!("ABORT({})", tx_id));
        self.state = ParticipantState3Pc::Aborted;
    }
}

/// 3PC coordinator.
#[derive(Debug)]
pub struct ThreePhaseCoordinator {
    pub participant_ids: Vec<u32>,
    pub votes: Vec<(u32, bool)>,
    pub decision: Option<String>,
}

impl ThreePhaseCoordinator {
    pub fn new(participant_ids: Vec<u32>) -> Self {
        Self {
            participant_ids,
            votes: Vec::new(),
            decision: None,
        }
    }

    /// Phase 1: Collect votes.
    pub fn phase1_prepare(
        &mut self,
        tx_id: u64,
        participants: &mut [Participant3Pc],
    ) -> Vec<(u32, bool)> {
        self.votes.clear();
        for p in participants.iter_mut() {
            let vote = p.prepare(tx_id);
            self.votes.push((p.id, vote));
        }
        self.votes.clone()
    }

    /// Phase 2: Send pre-commit if all voted YES.
    /// Returns `Ok(true)` if pre-commit sent, `Ok(false)` if aborted.
    pub fn phase2_pre_commit(
        &mut self,
        tx_id: u64,
        participants: &mut [Participant3Pc],
    ) -> Result<bool, String> {
        let all_yes = self.votes.iter().all(|(_, v)| *v);

        if all_yes {
            for p in participants.iter_mut() {
                if p.state == ParticipantState3Pc::Prepared {
                    p.pre_commit(tx_id);
                }
            }
            Ok(true)
        } else {
            for p in participants.iter_mut() {
                p.abort(tx_id);
            }
            self.decision = Some("ABORT".to_string());
            Ok(false)
        }
    }

    /// Phase 3: Send commit.
    pub fn phase3_commit(
        &mut self,
        tx_id: u64,
        participants: &mut [Participant3Pc],
    ) {
        self.decision = Some("COMMIT".to_string());
        for p in participants.iter_mut() {
            if p.state == ParticipantState3Pc::PreCommitted {
                p.commit(tx_id);
            }
        }
    }
}

/// Crash point for the 3PC coordinator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashPhase {
    Never,
    AfterPrepare,
    AfterPreCommit,
    AfterCommit,
}

/// A coordinator that can crash at a configured phase.
pub struct Crashable3PcCoordinator {
    pub inner: ThreePhaseCoordinator,
    pub crash_phase: CrashPhase,
    pub crashed: bool,
    phase_counter: u32,
}

impl Crashable3PcCoordinator {
    pub fn new(participant_ids: Vec<u32>, crash_phase: CrashPhase) -> Self {
        Self {
            inner: ThreePhaseCoordinator::new(participant_ids),
            crash_phase,
            crashed: false,
            phase_counter: 0,
        }
    }

    fn maybe_crash(&mut self, after_phase: CrashPhase) -> bool {
        self.phase_counter += 1;
        if self.crash_phase == after_phase {
            self.crashed = true;
            return true;
        }
        false
    }

    pub fn phase1_prepare(
        &mut self,
        tx_id: u64,
        participants: &mut [Participant3Pc],
    ) -> Vec<(u32, bool)> {
        let votes = self.inner.phase1_prepare(tx_id, participants);
        self.maybe_crash(CrashPhase::AfterPrepare);
        votes
    }

    pub fn phase2_pre_commit(
        &mut self,
        tx_id: u64,
        participants: &mut [Participant3Pc],
    ) -> Result<bool, String> {
        if self.crashed {
            return Err("Coordinator crashed".to_string());
        }
        let result = self.inner.phase2_pre_commit(tx_id, participants);
        self.maybe_crash(CrashPhase::AfterPreCommit);
        result
    }

    pub fn phase3_commit(
        &mut self,
        tx_id: u64,
        participants: &mut [Participant3Pc],
    ) -> Result<(), String> {
        if self.crashed {
            return Err("Coordinator crashed".to_string());
        }
        self.inner.phase3_commit(tx_id, participants);
        self.maybe_crash(CrashPhase::AfterCommit);
        Ok(())
    }

    /// Participants decide on their own after pre-commit timeout.
    pub fn participant_independent_commit(
        participants: &mut [Participant3Pc],
        tx_id: u64,
    ) {
        for p in participants.iter_mut() {
            if p.state == ParticipantState3Pc::PreCommitted {
                p.commit(tx_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_pc_normal_case_commits() {
        let mut coordinator = ThreePhaseCoordinator::new(vec![0, 1, 2]);
        let mut participants = vec![
            Participant3Pc::new(0, true),
            Participant3Pc::new(1, true),
            Participant3Pc::new(2, true),
        ];

        coordinator.phase1_prepare(1, &mut participants);
        coordinator.phase2_pre_commit(1, &mut participants).unwrap();
        coordinator.phase3_commit(1, &mut participants);

        for p in &participants {
            assert_eq!(p.state, ParticipantState3Pc::Committed);
        }
    }

    #[test]
    fn three_pc_abort_on_vote_no() {
        let mut coordinator = ThreePhaseCoordinator::new(vec![0, 1]);
        let mut participants = vec![Participant3Pc::new(0, true), Participant3Pc::new(1, false)];

        coordinator.phase1_prepare(1, &mut participants);
        let pre_commit_result = coordinator.phase2_pre_commit(1, &mut participants).unwrap();

        assert!(!pre_commit_result, "Should abort when any vote is NO");
        for p in &participants {
            assert_eq!(p.state, ParticipantState3Pc::Aborted);
        }
    }

    #[test]
    fn non_blocking_after_pre_commit_crash() {
        let mut coordinator = Crashable3PcCoordinator::new(
            vec![0, 1, 2],
            CrashPhase::AfterPreCommit,
        );
        let mut participants = vec![
            Participant3Pc::new(0, true),
            Participant3Pc::new(1, true),
            Participant3Pc::new(2, true),
        ];

        coordinator.phase1_prepare(1, &mut participants);
        let result = coordinator.phase2_pre_commit(1, &mut participants);
        assert!(result.is_ok());

        // Coordinator crashes after pre-commit.
        assert!(coordinator.crashed);

        // Phase 3 fails because coordinator crashed.
        let phase3_result = coordinator.phase3_commit(1, &mut participants);
        assert!(phase3_result.is_err(), "Phase 3 should fail after coordinator crash");

        // Participants that received PRE-COMMIT can commit independently.
        Crashable3PcCoordinator::participant_independent_commit(&mut participants, 1);

        for p in &participants {
            assert_eq!(
                p.state,
                ParticipantState3Pc::Committed,
                "Participants should be able to commit after pre-commit timeout"
            );
        }
    }

    #[test]
    fn blocking_still_possible_if_crash_before_pre_commit() {
        let mut coordinator = Crashable3PcCoordinator::new(
            vec![0, 1],
            CrashPhase::AfterPrepare,
        );
        let mut participants = vec![Participant3Pc::new(0, true), Participant3Pc::new(1, true)];

        coordinator.phase1_prepare(1, &mut participants);
        assert!(coordinator.crashed);

        // Phase 2 fails because coordinator crashed before sending pre-commit.
        let result = coordinator.phase2_pre_commit(1, &mut participants);
        assert!(result.is_err());

        // Participants are in Prepared state -- they cannot commit safely.
        for p in &participants {
            assert_eq!(
                p.state,
                ParticipantState3Pc::Prepared,
                "Participants should still be in Prepared state"
            );
        }
    }

    #[test]
    fn three_pc_log_entries_recorded() {
        let mut coordinator = ThreePhaseCoordinator::new(vec![0]);
        let mut participants = vec![Participant3Pc::new(0, true)];

        coordinator.phase1_prepare(1, &mut participants);
        coordinator.phase2_pre_commit(1, &mut participants).unwrap();
        coordinator.phase3_commit(1, &mut participants);

        assert_eq!(participants[0].log.len(), 3);
        assert!(participants[0].log[0].contains("PREPARE_YES"));
        assert!(participants[0].log[1].contains("PRE_COMMIT"));
        assert!(participants[0].log[2].contains("COMMIT"));
    }
}
