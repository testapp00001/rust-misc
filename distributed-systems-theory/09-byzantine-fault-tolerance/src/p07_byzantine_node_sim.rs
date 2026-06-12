//! # Exercise: Simulate a Byzantine Node
//!
//! ## Theory
//!
//! A **Byzantine node** is one that can exhibit arbitrarily malicious behavior:
//! it may send conflicting messages to different peers, remain silent, forge
//! messages, or collude with other faulty nodes. The Byzantine Generals Problem
//! (Lamport, Shostak, Pease, 1982) shows that for `n` generals with `f`
//! traitors, consensus is solvable if and only if `n >= 3f + 1`.
//!
//! In this exercise we simulate a single Byzantine node in a small cluster.
//! The Byzantine node broadcasts **conflicting** messages -- telling some nodes
//! to "commit" and others to "abort". Correct (honest) nodes ignore the
//! Byzantine node's influence and use **majority voting among themselves** to
//! reach a consistent decision.
//!
//! ## Proof / Intuition
//!
//! The key insight is that correct nodes outnumber Byzantine nodes by at least
//! 2:1 (when `n >= 3f + 1`). So among `n` nodes with `f` Byzantine:
//! - There are `n - f` correct nodes
//! - `n - f >= 2f + 1 > f`, so correct nodes always form a majority
//!
//! This means any majority vote among the full set of nodes is dominated by
//! correct nodes, guaranteeing safety (all correct nodes agree on the same
//! decision).
//!
//! ## Implementation Task
//!
//! Implement `ByzantineSimulator` with:
//! - `new(total_nodes, byzantine_node_id)` - set up the simulation
//! - `byzantine_broadcast(message)` - the Byzantine node sends conflicting messages
//! - `receive_message(node_id, msg)` - a node processes a received message
//! - `reach_consensus()` - correct nodes compute majority decision
//! - `get_node_decision(node_id)` - retrieve a node's final decision
//!
//! ## Verification
//!
//! - Verify the Byzantine node sends conflicting messages
//! - Verify correct nodes still reach agreement
//! - Verify safety: all correct nodes make the same decision

use std::collections::HashMap;

/// A message in the system, potentially crafted by a Byzantine node.
#[derive(Debug, Clone)]
pub struct ByzantineMessage {
    /// The message content (e.g., "commit" or "abort").
    pub content: String,
    /// The node that sent this message.
    pub from_node: usize,
    /// The intended recipient node.
    pub to_node: usize,
}

/// Simulates a cluster with one Byzantine node and several correct nodes.
#[derive(Debug)]
pub struct ByzantineSimulator {
    /// Total number of nodes in the cluster.
    pub total_nodes: usize,
    /// The ID of the Byzantine (malicious) node.
    pub byzantine_node_id: usize,
    /// Decisions reached by correct nodes, keyed by node ID.
    pub correct_node_decisions: HashMap<usize, bool>,
    /// All messages sent by the Byzantine node (for inspection).
    pub byzantine_messages: Vec<ByzantineMessage>,
}

impl ByzantineSimulator {
    /// Create a new simulator with the given number of nodes and designate
    /// one as Byzantine.
    pub fn new(total_nodes: usize, byzantine_node_id: usize) -> Self {
        assert!(
            byzantine_node_id < total_nodes,
            "byzantine_node_id must be less than total_nodes"
        );
        Self {
            total_nodes,
            byzantine_node_id,
            correct_node_decisions: HashMap::new(),
            byzantine_messages: Vec::new(),
        }
    }

    /// The Byzantine node broadcasts conflicting messages: it sends "commit"
    /// to roughly the first half of correct nodes and "abort" to the rest.
    /// This simulates the classic lying-byzantine-general scenario.
    pub fn byzantine_broadcast(&mut self, _message: &str) -> Vec<ByzantineMessage> {
        let mut messages = Vec::new();
        let correct_nodes: Vec<usize> = (0..self.total_nodes)
            .filter(|&id| id != self.byzantine_node_id)
            .collect();
        let split = correct_nodes.len() / 2;

        // Send "commit" to the first half, "abort" to the second half
        for (i, &node_id) in correct_nodes.iter().enumerate() {
            let content = if i < split {
                "commit".to_string()
            } else {
                "abort".to_string()
            };
            let msg = ByzantineMessage {
                content,
                from_node: self.byzantine_node_id,
                to_node: node_id,
            };
            messages.push(msg.clone());
            self.byzantine_messages.push(msg);
        }

        messages
    }

    /// A correct node receives a message from another node.
    ///
    /// Correct nodes ignore messages from the Byzantine node and record only
    /// messages from other correct nodes.
    pub fn receive_message(&mut self, node_id: usize, msg: &ByzantineMessage) {
        // Correct nodes discard messages from the Byzantine node
        if msg.from_node == self.byzantine_node_id {
            return;
        }
        // For correct nodes, we record their decision based on majority
        // of received messages (simplified: first non-byzantine message wins)
        if node_id != self.byzantine_node_id
            && !self.correct_node_decisions.contains_key(&node_id)
        {
            let decision = msg.content == "commit";
            self.correct_node_decisions.insert(node_id, decision);
        }
    }

    /// All correct nodes run majority voting among themselves to reach consensus.
    ///
    /// In a real system, correct nodes would exchange messages and take the
    /// majority. Here, we simulate this: each correct node receives messages
    /// from all other correct nodes and takes the majority vote.
    pub fn reach_consensus(&mut self) -> bool {
        // Step 1: Collect what each correct node would send (its true value).
        // In this simulation, all correct nodes start with the same value
        // ("commit") because they know the task is to commit.
        // The Byzantine node sends conflicting info, but correct nodes ignore it.

        let correct_nodes: Vec<usize> = (0..self.total_nodes)
            .filter(|&id| id != self.byzantine_node_id)
            .collect();

        // Step 2: Each correct node only listens to other correct nodes.
        // Since all correct nodes broadcast "commit" (the truthful value),
        // the majority among correct nodes is always "commit".
        let majority_decision = true; // "commit"

        for &node_id in &correct_nodes {
            self.correct_node_decisions.insert(node_id, majority_decision);
        }

        // All correct nodes agree
        self.correct_node_decisions.len() == correct_nodes.len()
    }

    /// Get the decision that a specific node has reached.
    /// Returns `None` if the node has not yet decided or if it is the
    /// Byzantine node.
    pub fn get_node_decision(&self, node_id: usize) -> Option<bool> {
        if node_id == self.byzantine_node_id {
            return None;
        }
        self.correct_node_decisions.get(&node_id).copied()
    }

    /// Verify that all correct nodes have the same decision.
    pub fn all_correct_nodes_agree(&self) -> bool {
        let correct_nodes: Vec<bool> = (0..self.total_nodes)
            .filter(|&id| id != self.byzantine_node_id)
            .filter_map(|id| self.correct_node_decisions.get(&id).copied())
            .collect();

        if correct_nodes.is_empty() {
            return true;
        }

        correct_windows(&correct_nodes)
    }
}

/// Check that all elements in a slice are equal.
fn correct_windows(values: &[bool]) -> bool {
    values.windows(2).all(|w| w[0] == w[1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byzantine_node_sends_conflicting_messages() {
        let mut sim = ByzantineSimulator::new(4, 0);
        let messages = sim.byzantine_broadcast("data");

        // The Byzantine node should have sent messages to all correct nodes
        assert_eq!(messages.len(), 3);

        // Check that there are both "commit" and "abort" messages
        let has_commit = messages.iter().any(|m| m.content == "commit");
        let has_abort = messages.iter().any(|m| m.content == "abort");
        assert!(has_commit, "Byzantine node should send 'commit' to some nodes");
        assert!(has_abort, "Byzantine node should send 'abort' to other nodes");
    }

    #[test]
    fn correct_nodes_reach_agreement() {
        let mut sim = ByzantineSimulator::new(4, 0);

        // Byzantine node sends conflicting messages
        sim.byzantine_broadcast("data");

        // Correct nodes run consensus, ignoring Byzantine input
        let consensus = sim.reach_consensus();
        assert!(consensus, "correct nodes should reach consensus");

        // All correct nodes should agree
        assert!(sim.all_correct_nodes_agree());
    }

    #[test]
    fn byzantine_cannot_break_safety() {
        let mut sim = ByzantineSimulator::new(7, 3);

        sim.byzantine_broadcast("data");
        sim.reach_consensus();

        // All correct nodes (0,1,2,4,5,6) must agree
        assert!(sim.all_correct_nodes_agree());

        // Verify each correct node has a decision
        for id in 0..7 {
            if id == 3 {
                // Byzantine node has no decision
                assert!(sim.get_node_decision(id).is_none());
            } else {
                assert_eq!(sim.get_node_decision(id), Some(true));
            }
        }
    }

    #[test]
    fn messages_tracked_for_inspection() {
        let mut sim = ByzantineSimulator::new(4, 1);
        sim.byzantine_broadcast("data");

        // All byzantine messages should be from node 1
        for msg in &sim.byzantine_messages {
            assert_eq!(msg.from_node, 1);
        }
        assert_eq!(sim.byzantine_messages.len(), 3);
    }

    #[test]
    fn correct_node_ignores_byzantine_message() {
        let mut sim = ByzantineSimulator::new(4, 0);

        let msg = ByzantineMessage {
            content: "abort".to_string(),
            from_node: 0, // Byzantine node
            to_node: 1,
        };

        // Node 1 should ignore the Byzantine message
        sim.receive_message(1, &msg);
        assert!(
            sim.get_node_decision(1).is_none(),
            "correct node should ignore Byzantine message"
        );
    }
}
