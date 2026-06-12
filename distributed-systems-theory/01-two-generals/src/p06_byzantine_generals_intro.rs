//! # Exercise: Byzantine Generals Introduction
//!
//! ## Theory
//!
//! The **Byzantine Generals Problem** (Lamport, Shostak, Pease, 1982) extends the
//! Two Generals' Problem to multiple generals where some may be **traitors** -- they
//! send conflicting or incorrect messages.
//!
//! A system is **Byzantine fault-tolerant** if it can reach agreement even when some
//! generals are traitors. The key result is:
//!
//! > A system with *n* generals can tolerate at most *f* traitors where **n >= 3f + 1**.
//!
//! This means:
//! - 3 generals can tolerate 0 traitors (impossible with 1 traitor)
//! - 4 generals can tolerate 1 traitor
//! - 7 generals can tolerate 2 traitors
//!
//! ## Proof / Intuition
//!
//! Consider 3 generals (A, B, C) where C is a traitor:
//! - C tells A: "Attack at dawn"
//! - C tells B: "Retreat at dawn"
//!
//! A and B exchange messages:
//! - A says: "C told me to attack"
//! - B says: "C told me to retreat"
//!
//! A and B now have conflicting information about C's message, and they cannot
//! determine which one is truthful. With only 3 generals, there is no way to
//! detect or resolve this inconsistency.
//!
//! The proof uses an information-theoretic argument: with f traitors, honest
//! generals need enough redundant information to detect contradictions, which
//! requires n >= 3f + 1 total generals.
//!
//! ## Implementation Task
//!
//! Implement:
//! - A simulation of 3 generals where one is a traitor
//! - The traitor sends conflicting messages (attack to some, retreat to others)
//! - Show that honest generals cannot always reach agreement
//!
//! ## Verification
//!
//! - Demonstrate that with 3 generals and 1 traitor, agreement is impossible
//! - Show the traitor successfully creates inconsistency

use std::collections::HashMap;

/// A decision that a general can make.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Decision {
    Attack,
    Retreat,
}

impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Decision::Attack => write!(f, "Attack"),
            Decision::Retreat => write!(f, "Retreat"),
        }
    }
}

/// A message from one general to another.
#[derive(Debug, Clone)]
pub struct Message {
    /// Who sent this message.
    pub from: usize,
    /// Who this message is intended for.
    pub to: usize,
    /// The decision being communicated.
    pub decision: Decision,
}

/// A general in the Byzantine Generals Problem.
#[derive(Debug, Clone)]
pub struct General {
    /// Unique ID of this general.
    pub id: usize,
    /// Whether this general is a traitor.
    pub is_traitor: bool,
    /// The general's own decision.
    pub own_decision: Decision,
    /// Messages received from other generals.
    pub received_messages: Vec<Message>,
}

impl General {
    /// Create a new honest general.
    pub fn honest(id: usize, decision: Decision) -> Self {
        Self {
            id,
            is_traitor: false,
            own_decision: decision,
            received_messages: Vec::new(),
        }
    }

    /// Create a new traitor general.
    pub fn traitor(id: usize, decision: Decision) -> Self {
        Self {
            id,
            is_traitor: true,
            own_decision: decision,
            received_messages: Vec::new(),
        }
    }

    /// If this general is honest, they broadcast their true decision to all others.
    /// If this general is a traitor, they send conflicting messages.
    pub fn broadcast(&self, total_generals: usize) -> Vec<Message> {
        let mut messages = Vec::new();

        if self.is_traitor {
            // Traitor sends conflicting messages
            for target in 0..total_generals {
                if target == self.id {
                    continue;
                }
                // Send attack to some, retreat to others
                let decision = if target % 2 == 0 {
                    Decision::Attack
                } else {
                    Decision::Retreat
                };
                messages.push(Message {
                    from: self.id,
                    to: target,
                    decision,
                });
            }
        } else {
            // Honest general sends their true decision to everyone
            for target in 0..total_generals {
                if target == self.id {
                    continue;
                }
                messages.push(Message {
                    from: self.id,
                    to: target,
                    decision: self.own_decision,
                });
            }
        }

        messages
    }

    /// Process received messages and determine the general's conclusion.
    ///
    /// Uses majority voting among received decisions (including own).
    pub fn decide(&self) -> Decision {
        let mut votes = HashMap::new();
        *votes.entry(self.own_decision).or_insert(0) += 1;

        for msg in &self.received_messages {
            *votes.entry(msg.decision).or_insert(0) += 1;
        }

        // Return the majority decision
        votes
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(decision, _)| decision)
            .unwrap_or(self.own_decision)
    }
}

/// Simulate a round of the Byzantine Generals Protocol.
///
/// Returns a vector of (general_id, decided_value) tuples.
pub fn simulate_byzantine_round(generals: &[General]) -> Vec<(usize, Decision)> {
    let n = generals.len();

    // Phase 1: All generals broadcast
    let mut all_messages: Vec<Message> = Vec::new();
    for general in generals {
        let msgs = general.broadcast(n);
        all_messages.extend(msgs);
    }

    // Phase 2: Deliver messages to recipients
    let mut updated_generals: Vec<General> = generals.to_vec();
    for msg in &all_messages {
        if let Some(recipient) = updated_generals.iter_mut().find(|g| g.id == msg.to) {
            recipient.received_messages.push(msg.clone());
        }
    }

    // Phase 3: Each general decides
    updated_generals
        .iter()
        .map(|g| (g.id, g.decide()))
        .collect()
}

/// Check if all honest generals reached the same decision.
pub fn all_honest_agree(
    generals: &[General],
    decisions: &[(usize, Decision)],
) -> bool {
    let honest_decisions: Vec<Decision> = decisions
        .iter()
        .filter(|(id, _)| !generals.iter().find(|g| g.id == *id).unwrap().is_traitor)
        .map(|(_, d)| *d)
        .collect();

    if honest_decisions.is_empty() {
        return true;
    }

    honest_decisions.iter().all(|d| *d == honest_decisions[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn honest_generals_broadcast_consistently() {
        let general = General::honest(0, Decision::Attack);
        let messages = general.broadcast(3);

        // Honest general sends the same decision to all others
        for msg in &messages {
            assert_eq!(msg.decision, Decision::Attack);
        }
    }

    #[test]
    fn traitor_broadcasts_conflicting_messages() {
        let general = General::traitor(2, Decision::Attack);
        let messages = general.broadcast(3);

        // Traitor should send different decisions to different generals
        let decisions: Vec<Decision> = messages.iter().map(|m| m.decision).collect();
        assert!(
            decisions.contains(&Decision::Attack) && decisions.contains(&Decision::Retreat),
            "traitor should send conflicting messages, got: {decisions:?}"
        );
    }

    #[test]
    fn three_generals_one_traitor_cannot_agree() {
        // With 3 generals and 1 traitor, the traitor can always cause disagreement
        // when honest generals have different initial opinions.
        //
        // General 0 (honest, Attack), General 1 (honest, Retreat), General 2 (traitor)
        // The traitor sends Attack to General 0 and Retreat to General 1.
        //
        // General 0 votes: own=Attack, from 1=Retreat, from 2=Attack -> Attack wins 2-1
        // General 1 votes: own=Retreat, from 0=Attack, from 2=Retreat -> Retreat wins 2-1
        // They disagree!
        let generals = vec![
            General::honest(0, Decision::Attack),
            General::honest(1, Decision::Retreat),
            General::traitor(2, Decision::Attack),
        ];

        let mut disagreement_count = 0;
        let trials = 50;

        for _ in 0..trials {
            let decisions = simulate_byzantine_round(&generals);
            if !all_honest_agree(&generals, &decisions) {
                disagreement_count += 1;
            }
        }

        // With 3 generals and 1 traitor, the traitor should be able to cause
        // disagreement (honest generals have different initial opinions)
        assert!(
            disagreement_count > 0,
            "with 3 generals and 1 traitor, agreement should sometimes fail"
        );
    }

    #[test]
    fn four_generals_one_traitor_can_agree() {
        // With n=4 and f=1, Byzantine agreement is possible
        // Using a more sophisticated protocol (simplified version)
        let generals = vec![
            General::honest(0, Decision::Attack),
            General::honest(1, Decision::Attack),
            General::honest(2, Decision::Attack),
            General::traitor(3, Decision::Retreat),
        ];

        // With 4 generals, honest majority can outvote the traitor
        let decisions = simulate_byzantine_round(&generals);

        // Count honest decisions
        let mut attack_count = 0;
        let mut retreat_count = 0;
        for (id, decision) in &decisions {
            let general = generals.iter().find(|g| g.id == *id).unwrap();
            if !general.is_traitor {
                match decision {
                    Decision::Attack => attack_count += 1,
                    Decision::Retreat => retreat_count += 1,
                }
            }
        }

        // Honest majority should agree on Attack
        assert!(
            attack_count >= 2,
            "honest majority should agree on Attack, got {attack_count} attacks, {retreat_count} retreats"
        );
    }

    #[test]
    fn display_decision() {
        assert_eq!(format!("{}", Decision::Attack), "Attack");
        assert_eq!(format!("{}", Decision::Retreat), "Retreat");
    }
}
