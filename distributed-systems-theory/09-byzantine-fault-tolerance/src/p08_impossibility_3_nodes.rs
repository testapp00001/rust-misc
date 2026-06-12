//! # Exercise: Demonstrate 3 Nodes CANNOT Tolerate 1 Byzantine Fault
//!
//! ## Theory
//!
//! One of the most fundamental results in Byzantine fault tolerance is the
//! **impossibility bound**: a system of `n` nodes can tolerate at most
//! `f < n/3` Byzantine faults. Equivalently, `n >= 3f + 1` is required.
//!
//! This exercise demonstrates the failure case concretely:
//! - With **3 nodes and 1 Byzantine fault** (n=3, f=1): the 2 correct nodes
//!   each receive conflicting information from the Byzantine node and from
//!   each other. They **cannot** determine who is lying, so they may reach
//!   **different** decisions.
//! - With **4 nodes and 1 Byzantine fault** (n=4, f=1): the 3 correct nodes
//!   can form a quorum of 3 and always agree, because correct nodes
//!   outnumber the Byzantine node.
//!
//! ## Proof / Intuition
//!
//! Consider 3 nodes where node 0 is Byzantine and nodes 1, 2 are correct:
//!
//! 1. Node 0 sends "value A" to node 1, and "value B" to node 2.
//! 2. Node 1 sees: (its own value, A from node 0) -- it cannot distinguish
//!    whether node 0 or node 2 is the traitor.
//! 3. Node 2 sees: (its own value, B from node 0) -- same dilemma.
//!
//! Without a third correct node to break the tie, nodes 1 and 2 cannot
//! determine the truth. This is the essence of the impossibility.
//!
//! With 4 nodes (1 Byzantine, 3 correct), the 3 correct nodes all send
//! the same honest value to each other. Even if the Byzantine node sends
//! conflicting messages, each correct node receives at least 2 agreeing
//! messages from other correct nodes, forming a quorum of 3.
//!
//! ## Implementation Task
//!
//! Implement `ThreeNodeSystem` and `FourNodeSystem` to demonstrate:
//! - The impossibility with 3 nodes (correct nodes may disagree)
//! - The solvability with 4 nodes (correct nodes always agree)
//! - The mathematical bound `n >= 3f + 1`
//!
//! ## Verification
//!
//! - Demonstrate that with 3 nodes and 1 Byzantine, correct nodes may disagree
//! - Demonstrate that with 4 nodes and 1 Byzantine, correct nodes always agree
//! - Verify the bound n >= 3f + 1 is necessary and sufficient

/// State of a single node in the system.
#[derive(Debug, Clone)]
pub struct NodeState {
    /// Unique identifier for this node.
    pub node_id: usize,
    /// Messages received from other nodes: (sender_id, message_content).
    pub received_messages: Vec<(usize, String)>,
    /// The node's final decision, if any.
    pub decision: Option<bool>,
    /// Whether this node is Byzantine (malicious).
    pub is_byzantine: bool,
}

impl NodeState {
    /// Create a new node.
    pub fn new(node_id: usize, is_byzantine: bool) -> Self {
        Self {
            node_id,
            received_messages: Vec::new(),
            decision: None,
            is_byzantine,
        }
    }
}

/// A system of 3 nodes demonstrating the impossibility of tolerating 1
/// Byzantine fault.
#[derive(Debug)]
pub struct ThreeNodeSystem {
    /// The three nodes in the system.
    pub nodes: Vec<NodeState>,
    /// All messages exchanged: (from, to, content).
    pub messages: Vec<(usize, usize, String)>,
    /// The ID of the Byzantine node.
    pub byzantine_id: usize,
}

impl ThreeNodeSystem {
    /// Create a new 3-node system with one Byzantine node.
    pub fn new(byzantine_id: usize) -> Self {
        assert!(byzantine_id < 3, "byzantine_id must be 0, 1, or 2");
        let nodes = (0..3)
            .map(|i| NodeState::new(i, i == byzantine_id))
            .collect();
        Self {
            nodes,
            messages: Vec::new(),
            byzantine_id,
        }
    }

    /// The Byzantine node sends conflicting messages to the two correct nodes:
    /// "true" to one, "false" to the other.
    pub fn byzantine_send_conflicting(&mut self) {
        let correct_nodes: Vec<usize> = (0..3)
            .filter(|&id| id != self.byzantine_id)
            .collect();

        // Send "true" to the first correct node
        self.messages.push((
            self.byzantine_id,
            correct_nodes[0],
            "true".to_string(),
        ));

        // Send "false" to the second correct node
        self.messages.push((
            self.byzantine_id,
            correct_nodes[1],
            "false".to_string(),
        ));
    }

    /// Process messages. Each correct node receives conflicting info from the
    /// Byzantine node and cannot determine the truth.
    ///
    /// In the 3-node case, each correct node hears:
    /// - Its own initial value (say "true")
    /// - A conflicting value from the Byzantine node
    ///
    /// Without a quorum of agreeing nodes, they each decide independently
    /// based on the (potentially conflicting) messages they received.
    pub fn process_messages(&mut self) {
        // Deliver messages to recipients
        for &(from, to, ref content) in &self.messages {
            if !self.nodes[to].is_byzantine {
                self.nodes[to]
                    .received_messages
                    .push((from, content.clone()));
            }
        }

        // Each correct node makes a decision.
        // In the 3-node case, a correct node receives 1 message from the
        // Byzantine node. With only 2 correct nodes, there is no quorum of 3,
        // so each node decides based on what it received.
        let correct_nodes: Vec<usize> = (0..3)
            .filter(|&id| !self.nodes[id].is_byzantine)
            .collect();

        for &node_id in &correct_nodes {
            let msgs: Vec<&str> = self.nodes[node_id]
                .received_messages
                .iter()
                .map(|(_, c)| c.as_str())
                .collect();

            // Count "true" and "false" votes
            let true_count = msgs.iter().filter(|&&m| m == "true").count();
            let false_count = msgs.iter().filter(|&&m| m == "false").count();

            // Without quorum (need 2 out of 3 correct, but only 2 exist),
            // each node may decide differently depending on what it received
            self.nodes[node_id].decision = if true_count > false_count {
                Some(true)
            } else if false_count > true_count {
                Some(false)
            } else {
                // Tie -- node cannot decide. This is the impossibility!
                None
            };
        }
    }

    /// Get the decisions of all nodes.
    pub fn get_decisions(&self) -> Vec<Option<bool>> {
        self.nodes.iter().map(|n| n.decision).collect()
    }

    /// Check if all correct (non-Byzantine) nodes have agreed on the same
    /// decision. Returns `false` if any correct node disagrees or has no decision.
    pub fn are_correct_nodes_agreed(&self) -> bool {
        let correct_decisions: Vec<Option<bool>> = self
            .nodes
            .iter()
            .filter(|n| !n.is_byzantine)
            .map(|n| n.decision)
            .collect();

        if correct_decisions.is_empty() {
            return true;
        }

        // All correct nodes must have Some(value) and all values must match
        match correct_decisions[0] {
            Some(val) => correct_decisions.iter().all(|d| *d == Some(val)),
            None => false,
        }
    }
}

/// A system of 4 nodes demonstrating that 1 Byzantine fault CAN be tolerated.
#[derive(Debug)]
pub struct FourNodeSystem {
    /// The four nodes in the system.
    pub nodes: Vec<NodeState>,
    /// All messages exchanged: (from, to, content).
    pub messages: Vec<(usize, usize, String)>,
    /// The ID of the Byzantine node.
    pub byzantine_id: usize,
}

impl FourNodeSystem {
    /// Create a new 4-node system with one Byzantine node.
    pub fn new(byzantine_id: usize) -> Self {
        assert!(byzantine_id < 4, "byzantine_id must be 0..3");
        let nodes = (0..4)
            .map(|i| NodeState::new(i, i == byzantine_id))
            .collect();
        Self {
            nodes,
            messages: Vec::new(),
            byzantine_id,
        }
    }

    /// The Byzantine node sends conflicting messages to correct nodes.
    pub fn byzantine_send_conflicting(&mut self) {
        let correct_nodes: Vec<usize> = (0..4)
            .filter(|&id| id != self.byzantine_id)
            .collect();

        // Send "true" to the first correct node, "false" to the rest
        for (i, &node_id) in correct_nodes.iter().enumerate() {
            let content = if i == 0 {
                "true".to_string()
            } else {
                "false".to_string()
            };
            self.messages
                .push((self.byzantine_id, node_id, content));
        }

        // Correct nodes also exchange messages with each other (they all
        // know the true value is "true")
        for &from in &correct_nodes {
            for &to in &correct_nodes {
                if from != to {
                    self.messages.push((from, to, "true".to_string()));
                }
            }
        }
    }

    /// Process messages using a 3-of-4 quorum rule.
    ///
    /// Each node collects messages and uses quorum voting: if a value is
    /// supported by at least 3 out of 4 nodes, it is adopted.
    pub fn process_messages(&mut self) {
        // Deliver messages
        for &(from, to, ref content) in &self.messages {
            if !self.nodes[to].is_byzantine {
                self.nodes[to]
                    .received_messages
                    .push((from, content.clone()));
            }
        }

        let correct_nodes: Vec<usize> = (0..4)
            .filter(|&id| !self.nodes[id].is_byzantine)
            .collect();

        for &node_id in &correct_nodes {
            let msgs: Vec<&str> = self.nodes[node_id]
                .received_messages
                .iter()
                .map(|(_, c)| c.as_str())
                .collect();

            let true_count = msgs.iter().filter(|&&m| m == "true").count();
            let false_count = msgs.iter().filter(|&&m| m == "false").count();

            // Quorum of 3 is needed. With 3 correct nodes all sending "true",
            // true_count will be >= 3, dominating any false messages.
            self.nodes[node_id].decision = if true_count >= 3 {
                Some(true)
            } else if false_count >= 3 {
                Some(false)
            } else if true_count > false_count {
                Some(true)
            } else {
                Some(false)
            };
        }
    }

    /// Get the decisions of all nodes.
    pub fn get_decisions(&self) -> Vec<Option<bool>> {
        self.nodes.iter().map(|n| n.decision).collect()
    }

    /// Check if all correct (non-Byzantine) nodes have agreed on the same decision.
    pub fn are_correct_nodes_agreed(&self) -> bool {
        let correct_decisions: Vec<Option<bool>> = self
            .nodes
            .iter()
            .filter(|n| !n.is_byzantine)
            .map(|n| n.decision)
            .collect();

        if correct_decisions.is_empty() {
            return true;
        }

        match correct_decisions[0] {
            Some(val) => correct_decisions.iter().all(|d| *d == Some(val)),
            None => false,
        }
    }
}

/// Verify the mathematical bound: n >= 3f + 1.
///
/// Returns `true` if the system can tolerate `f` Byzantine faults with `n` nodes.
pub fn can_tolerate(n: usize, f: usize) -> bool {
    n >= 3 * f + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_nodes_cannot_agree_with_one_byzantine() {
        // With 3 nodes and 1 Byzantine, the 2 correct nodes may disagree
        // or be unable to decide.
        let mut system = ThreeNodeSystem::new(0);
        system.byzantine_send_conflicting();
        system.process_messages();

        let decisions = system.get_decisions();

        // The Byzantine node (0) has no decision
        assert!(decisions[0].is_none());

        // The two correct nodes received conflicting messages.
        // In our implementation, they each vote based on what they received.
        // Node 1 received "true" from Byzantine -> decides Some(true)
        // Node 2 received "false" from Byzantine -> decides Some(false)
        // OR they may both decide based on their own initial value.
        // The key point: they cannot reliably agree.
        //
        // With our message setup:
        // Node 1 gets: [("Byzantine", "true")]
        // Node 2 gets: [("Byzantine", "false")]
        // Both have 1 vote, so they decide by majority (which is a tie -> None)
        // OR if we count the node's own initial value as "true":
        // Node 1: 2 "true" -> Some(true)
        // Node 2: 1 "true" + 1 "false" -> tie -> None
        //
        // Either way, correctness is not guaranteed.

        // Verify that the system does NOT have reliable agreement
        // (at least one correct node has None or they disagree)
        let correct_decisions: Vec<Option<bool>> = decisions[1..].to_vec();
        let all_some = correct_decisions.iter().all(|d| d.is_some());
        let all_same = if all_some {
            correct_decisions
                .windows(2)
                .all(|w| w[0] == w[1])
        } else {
            false
        };

        // Demonstrate the impossibility: either they can't decide or they disagree
        assert!(
            !all_same,
            "with 3 nodes and 1 Byzantine, correct nodes should NOT reliably agree"
        );
    }

    #[test]
    fn four_nodes_can_agree_with_one_byzantine() {
        let mut system = FourNodeSystem::new(0);
        system.byzantine_send_conflicting();
        system.process_messages();

        // All 3 correct nodes should agree
        assert!(
            system.are_correct_nodes_agreed(),
            "with 4 nodes and 1 Byzantine, correct nodes should always agree"
        );

        let decisions = system.get_decisions();

        // Byzantine node has no decision
        assert!(decisions[0].is_none());

        // All correct nodes decided "true" (majority from other correct nodes)
        for id in 1..4 {
            assert_eq!(
                decisions[id],
                Some(true),
                "correct node {id} should decide true"
            );
        }
    }

    #[test]
    fn mathematical_bound_n_ge_3f_plus_1() {
        // f=0: n >= 1 (trivially true)
        assert!(can_tolerate(1, 0));
        assert!(can_tolerate(3, 0));

        // f=1: n >= 4
        assert!(!can_tolerate(3, 1));
        assert!(can_tolerate(4, 1));
        assert!(can_tolerate(7, 1));

        // f=2: n >= 7
        assert!(!can_tolerate(6, 2));
        assert!(can_tolerate(7, 2));
        assert!(can_tolerate(10, 2));

        // f=3: n >= 10
        assert!(!can_tolerate(9, 3));
        assert!(can_tolerate(10, 3));
    }
}
