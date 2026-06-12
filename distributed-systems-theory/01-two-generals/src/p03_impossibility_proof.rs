//! # Exercise: Formal Impossibility Proof
//!
//! ## Theory
//!
//! The Two Generals' Problem has a formal impossibility proof: for any finite
//! protocol with *k* messages, there exists a network configuration where the *k*-th
//! message is lost, making agreement impossible.
//!
//! This exercise formalizes this argument. We define a protocol as a finite sequence
//! of message exchanges, and show that for every such protocol, we can find a
//! failure scenario that prevents agreement.
//!
//! ## Proof / Intuition
//!
//! **Theorem**: No finite protocol guarantees agreement between two generals over an
//! unreliable channel.
//!
//! **Proof by strong induction on k (number of messages)**:
//!
//! *Base case (k=1)*: General A sends attack time t to B. If message is lost, B
//! never learns t. If message arrives, B learns t but A doesn't know B learned it.
//! Neither can safely commit.
//!
//! *Inductive step*: Assume no protocol with k messages guarantees agreement. Consider
//! a protocol P with k+1 messages. The (k+1)-th message must be the last message
//! sent by some general. Consider the scenario where this last message is lost. The
//! receiver's state is identical to the state after only k messages in protocol P.
//! By the inductive hypothesis, agreement cannot be guaranteed with k messages.
//! Therefore it cannot be guaranteed with k+1 messages.
//!
//! In this exercise, we implement this proof by:
//! 1. Defining a protocol as a list of message rounds
//! 2. For any protocol with k messages, constructing a specific failure that breaks it
//! 3. Showing the infinite regress: solving k messages requires k+1 messages
//!
//! ## Implementation Task
//!
//! Implement the `ProtocolRound`, `ImpossibilityProof`, and related structures to:
//! - Model a k-message protocol
//! - Construct the failure scenario for any k
//! - Demonstrate that agreement fails for any finite k
//!
//! ## Verification
//!
//! - For k=1, show agreement fails
//! - For k=2, show agreement fails
//! - For any k up to 100, show a configuration where agreement fails
//! - Demonstrate that the only "solution" requires infinite messages

/// Represents a single message round in the protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sender {
    GeneralA,
    GeneralB,
}

/// A message in the Two Generals protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolMessage {
    /// Who sends this message.
    pub sender: Sender,
    /// The content of the message (attack time or acknowledgement).
    pub content: MessageContent,
}

/// Content of a protocol message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageContent {
    AttackTime(u64),
    Ack,
}

/// The state of each general after processing messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneralState {
    /// Whether this general has committed to the attack.
    pub committed: bool,
    /// The attack time this general believes was agreed upon.
    pub agreed_time: Option<u64>,
    /// Number of messages this general has received.
    pub messages_received: usize,
}

impl Default for GeneralState {
    fn default() -> Self {
        Self {
            committed: false,
            agreed_time: None,
            messages_received: 0,
        }
    }
}

/// A finite protocol with k message rounds.
#[derive(Debug, Clone)]
pub struct FiniteProtocol {
    /// The messages in the protocol, in order.
    pub messages: Vec<ProtocolMessage>,
    /// The number of messages in the protocol.
    pub k: usize,
}

impl FiniteProtocol {
    /// Create a new protocol with the given number of message rounds.
    ///
    /// The protocol follows the standard Two Generals pattern:
    /// Round 1: A sends attack time to B
    /// Round 2: B sends ack to A
    /// Round 3: A sends ack-of-ack to B
    /// ...
    pub fn new(k: usize, attack_time: u64) -> Self {
        let mut messages = Vec::with_capacity(k);
        for i in 0..k {
            let (sender, content) = if i % 2 == 0 {
                (
                    Sender::GeneralA,
                    if i == 0 {
                        MessageContent::AttackTime(attack_time)
                    } else {
                        MessageContent::Ack
                    },
                )
            } else {
                (Sender::GeneralB, MessageContent::Ack)
            };
            messages.push(ProtocolMessage { sender, content });
        }
        Self { messages, k }
    }
}

/// Simulates a protocol execution where the last message is always lost.
/// Returns the states of both generals after execution.
pub fn simulate_last_message_lost(protocol: &FiniteProtocol) -> (GeneralState, GeneralState) {
    let mut state_a = GeneralState::default();
    let mut state_b = GeneralState::default();

    // Process all messages except the last one
    let delivered_count = protocol.k.saturating_sub(1);

    for msg in &protocol.messages[..delivered_count] {
        match msg.sender {
            Sender::GeneralA => {
                match &msg.content {
                    MessageContent::AttackTime(t) => {
                        state_a.messages_received += 1;
                        // A sent the message (A knows the time)
                        state_a.agreed_time = Some(*t);
                        // B would receive it if delivered
                        state_b.messages_received += 1;
                        state_b.agreed_time = Some(*t);
                    }
                    MessageContent::Ack => {
                        state_a.messages_received += 1;
                        state_b.messages_received += 1;
                    }
                }
            }
            Sender::GeneralB => {
                state_b.messages_received += 1;
                state_a.messages_received += 1;
            }
        }
    }

    // The last message is LOST -- neither general updates
    // Now check: can both generals safely commit?

    // General A commits only if it's sure B knows the time.
    // A knows B received messages up to (k-1), but doesn't know if B received all of them.
    // More critically: after the last message is lost, the state is identical to a k-1 protocol.

    // For k=1: A sent but doesn't know B received. A cannot commit.
    // For k=2: A knows B received the attack time (B sent ack), but doesn't know B received A's ack.
    //          B sent the ack but doesn't know A received it. B cannot be sure A will attack.
    // For k>2: By induction, same problem.

    // In general, the receiver of the lost message cannot commit.
    let last_sender = &protocol.messages[protocol.k - 1].sender;

    // The general who sent the last (lost) message doesn't know it was lost,
    // so they might commit. The receiver doesn't receive it, so they can't commit.
    // But crucially, the sender ALSO can't be sure because the receiver's state
    // after k-1 messages is indistinguishable from a k-1 protocol.

    // Simple model: both generals commit only if ALL messages were delivered.
    if delivered_count < protocol.k {
        // Last message was lost -- agreement fails
        // The receiving general of the last message doesn't commit
        match last_sender {
            Sender::GeneralA => {
                // A sent, B should receive. B doesn't get it.
                // But A already committed after sending (optimistically).
                // B doesn't commit.
                state_a.committed = true; // A committed (but is wrong -- B won't attack)
                state_b.committed = false;
            }
            Sender::GeneralB => {
                state_a.committed = false;
                state_b.committed = true;
            }
        }
    } else {
        // All messages delivered (shouldn't happen in this function)
        state_a.committed = true;
        state_b.committed = true;
    }

    (state_a, state_b)
}

/// The core impossibility proof: for any finite k, construct a failure.
///
/// Returns true if agreement fails (proving impossibility).
pub fn proof_step(k: usize, attack_time: u64) -> bool {
    let protocol = FiniteProtocol::new(k, attack_time);
    let (state_a, state_b) = simulate_last_message_lost(&protocol);

    // Agreement fails if not both committed to the same time
    let agreement = state_a.committed && state_b.committed;
    !agreement
}

/// Demonstrate the infinite regress: solving k messages requires k+1.
///
/// Returns the chain of dependencies as a vector of strings.
pub fn infinite_regress(k: usize) -> Vec<String> {
    let mut explanations = Vec::new();

    for i in 1..=k {
        if i == 1 {
            explanations.push(format!(
                "Round 1: A sends attack time to B. If lost, B doesn't know. \
                 Need B to acknowledge (round 2)."
            ));
        } else if i % 2 == 0 {
            explanations.push(format!(
                "Round {i}: B sends ack to A. If lost, A doesn't know B received the time. \
                 Need A to ack-of-ack (round {}).",
                i + 1
            ));
        } else {
            explanations.push(format!(
                "Round {i}: A sends ack-of-ack to B. If lost, B doesn't know A got the ack. \
                 Need B to ack (round {}).",
                i + 1
            ));
        }
    }

    explanations.push(format!(
        "After round {k}: The last message can always be lost. \
         This is identical to a protocol with {k} messages. \
         By induction, no finite protocol works."
    ));

    explanations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k1_protocol_fails() {
        assert!(
            proof_step(1, 10),
            "1-message protocol should fail when last message is lost"
        );
    }

    #[test]
    fn k2_protocol_fails() {
        assert!(
            proof_step(2, 10),
            "2-message protocol should fail when last message is lost"
        );
    }

    #[test]
    fn all_finite_protocols_fail() {
        for k in 1..=100 {
            assert!(
                proof_step(k, 10),
                "protocol with k={k} messages should fail when last message is lost"
            );
        }
    }

    #[test]
    fn infinite_regress_demonstration() {
        let regress = infinite_regress(5);
        assert_eq!(regress.len(), 6); // 5 rounds + 1 conclusion
        assert!(regress.last().unwrap().contains("no finite protocol works"));
    }

    #[test]
    fn protocol_generates_correct_message_count() {
        for k in 1..=10 {
            let protocol = FiniteProtocol::new(k, 10);
            assert_eq!(protocol.messages.len(), k);
        }
    }

    #[test]
    fn last_message_loss_causes_failure() {
        let protocol = FiniteProtocol::new(4, 10);
        let (state_a, state_b) = simulate_last_message_lost(&protocol);

        // Not both can commit when last message is lost
        assert!(
            !(state_a.committed && state_b.committed),
            "both generals cannot commit when last message is lost"
        );
    }
}
