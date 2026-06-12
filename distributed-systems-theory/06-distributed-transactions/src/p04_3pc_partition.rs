//! # Exercise: 3PC Consistency Violation Under Partition
//!
//! ## Theory
//!
//! Despite 3PC's design to eliminate blocking, it can still violate consistency under
//! network partitions. This is why 3PC is rarely used in practice.
//!
//! The vulnerability arises because 3PC allows independent commits after the pre-commit
//! phase. Under a partition, different partitions may reach different decisions.
//!
//! ## Proof / Intuition
//!
//! Consider this scenario with 3 participants:
//!
//! 1. All vote YES in phase 1.
//! 2. Coordinator sends PRE-COMMIT to all.
//! 3. All participants enter PreCommitted state.
//! 4. Network partitions: Coordinator and P1 are in partition A, P2 is in partition B.
//! 5. Coordinator sends COMMIT to P1 (in same partition).
//! 6. P1 commits. P2 does not receive the COMMIT.
//! 7. P2 times out and decides to commit independently (since it's PreCommitted).
//!
//! Now consider a different scenario:
//! 4. Network partitions BEFORE coordinator sends COMMIT.
//! 5. P2 is isolated and times out -- it commits.
//! 6. The coordinator, seeing the partition, decides to ABORT (if any node is unreachable).
//! 7. P1 aborts.
//!
//! Result: P1 is aborted, P2 is committed -- VIOLATION of atomicity.
//!
//! ## Implementation Task
//!
//! Demonstrate the partition scenario:
//!
//! - `PartitionedNetwork`: simulates network partitions.
//! - `PartitionParticipant`: participant that can commit independently.
//! - Simulate the partition scenario that leads to inconsistency.
//!
//! ## Verification
//!
//! - Demonstrate a scenario where 3PC violates consistency under partition.
//! - Show that different participants end up in different final states.

/// State of a participant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionParticipantState {
    Init,
    Prepared,
    PreCommitted,
    Committed,
    Aborted,
}

/// A participant in the partitioned 3PC scenario.
#[derive(Debug)]
pub struct PartitionParticipant {
    pub id: u32,
    pub state: PartitionParticipantState,
    pub will_vote_yes: bool,
    pub reachable: bool,
}

impl PartitionParticipant {
    pub fn new(id: u32, will_vote_yes: bool, reachable: bool) -> Self {
        Self {
            id,
            state: PartitionParticipantState::Init,
            will_vote_yes,
            reachable,
        }
    }

    pub fn prepare(&mut self, tx_id: u64) -> bool {
        if self.will_vote_yes {
            self.state = PartitionParticipantState::Prepared;
            true
        } else {
            self.state = PartitionParticipantState::Aborted;
            false
        }
    }

    pub fn pre_commit(&mut self, tx_id: u64) {
        if self.reachable && self.state == PartitionParticipantState::Prepared {
            self.state = PartitionParticipantState::PreCommitted;
        }
    }

    pub fn commit(&mut self, tx_id: u64) {
        if self.state == PartitionParticipantState::PreCommitted
            || self.state == PartitionParticipantState::Prepared
        {
            self.state = PartitionParticipantState::Committed;
        }
    }

    pub fn abort(&mut self, tx_id: u64) {
        if self.state != PartitionParticipantState::Committed {
            self.state = PartitionParticipantState::Aborted;
        }
    }

    /// Independent commit after timeout: used when coordinator is unreachable.
    pub fn independent_commit(&mut self) {
        if self.state == PartitionParticipantState::PreCommitted {
            self.state = PartitionParticipantState::Committed;
        }
    }
}

/// Simulate a 3PC run with a network partition.
/// Returns the final states of all participants.
pub fn simulate_partition_scenario(
    proposals: Vec<bool>,
    partition_after_pre_commit: bool,
    coordinator_aborts: bool,
) -> Vec<PartitionParticipantState> {
    let tx_id = 1u64;
    let mut participants: Vec<PartitionParticipant> = proposals
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            // All are reachable during phase 1 and 2.
            PartitionParticipant::new(i as u32, p, true)
        })
        .collect();

    // Phase 1: Collect votes.
    let votes: Vec<bool> = participants.iter_mut().map(|p| p.prepare(tx_id)).collect();

    let all_yes = votes.iter().all(|v| *v);
    if !all_yes {
        for p in &mut participants {
            p.abort(tx_id);
        }
        return participants.into_iter().map(|p| p.state).collect();
    }

    // Phase 2: Pre-commit.
    for p in &mut participants {
        p.pre_commit(tx_id);
    }

    if partition_after_pre_commit {
        // Network partition occurs.
        // Some participants become unreachable from the coordinator.
        // Simulate: participant at index 1 is in the other partition.
        if participants.len() > 1 {
            participants[1].reachable = false;
        }

        if coordinator_aborts {
            // Coordinator decides to abort (e.g., because it can't reach all nodes).
            for p in &mut participants.iter_mut() {
                if p.reachable {
                    p.abort(tx_id);
                }
            }
            // Unreachable participant times out and commits independently.
            for p in &mut participants.iter_mut() {
                if !p.reachable {
                    p.independent_commit();
                }
            }
        } else {
            // Coordinator sends commit to reachable participants.
            for p in &mut participants.iter_mut() {
                if p.reachable {
                    p.commit(tx_id);
                }
            }
            // Unreachable participant commits independently.
            for p in &mut participants.iter_mut() {
                if !p.reachable {
                    p.independent_commit();
                }
            }
        }
    } else {
        // No partition: normal commit.
        for p in &mut participants {
            p.commit(tx_id);
        }
    }

    participants.into_iter().map(|p| p.state).collect()
}

/// Check if all participants are in the same final state.
pub fn all_same_state(states: &[PartitionParticipantState]) -> bool {
    states.windows(2).all(|w| w[0] == w[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_3pc_no_partition_all_commit() {
        let states = simulate_partition_scenario(
            vec![true, true, true],
            false, // no partition
            false,
        );
        assert!(
            all_same_state(&states),
            "Without partition, all should commit: {:?}",
            states
        );
        assert!(states.iter().all(|s| *s == PartitionParticipantState::Committed));
    }

    #[test]
    fn partition_can_cause_inconsistency() {
        // Coordinator aborts (can't reach all), unreachable participant commits.
        let states = simulate_partition_scenario(
            vec![true, true, true],
            true,  // partition after pre-commit
            true,  // coordinator aborts
        );

        // Participant 0 and 2 are reachable: they aborted.
        // Participant 1 is unreachable: it committed independently.
        let has_aborted = states.iter().any(|s| *s == PartitionParticipantState::Aborted);
        let has_committed = states.iter().any(|s| *s == PartitionParticipantState::Committed);

        assert!(
            has_aborted && has_committed,
            "Partition should cause inconsistency: some committed, some aborted. Got: {:?}",
            states
        );
    }

    #[test]
    fn consistency_violated_under_partition() {
        let states = simulate_partition_scenario(
            vec![true, true, true],
            true,
            true,
        );

        // The states are NOT all the same -- this is the consistency violation.
        assert!(
            !all_same_state(&states),
            "3PC should violate consistency under partition: {:?}",
            states
        );
    }

    #[test]
    fn no_partition_maintains_consistency() {
        let states = simulate_partition_scenario(
            vec![true, true, true],
            false,
            false,
        );
        assert!(all_same_state(&states));
    }

    #[test]
    fn partition_with_coordinator_commit_maintains_consistency() {
        // If coordinator commits and all reachable participants commit,
        // and unreachable also commits independently, we get consistency.
        let states = simulate_partition_scenario(
            vec![true, true, true],
            true,
            false, // coordinator commits
        );
        assert!(
            all_same_state(&states),
            "If coordinator commits, all should commit: {:?}",
            states
        );
    }
}
