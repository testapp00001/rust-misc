//! # Exercise: PBFT Key-Value Store
//!
//! ## Theory
//!
//! Practical Byzantine Fault Tolerance (PBFT) is a consensus algorithm designed
//! to work correctly even when some nodes are Byzantine (arbitrarily faulty).
//! PBFT tolerates f Byzantine faults with 3f+1 nodes total.
//!
//! The PBFT protocol has three phases:
//! 1. **Pre-prepare**: The leader assigns a sequence number and broadcasts a
//!    pre-prepare message.
//! 2. **Prepare**: Each node broadcasts a prepare message. After receiving 2f+1
//!    matching prepare messages, the node is "prepared."
//! 3. **Commit**: Each node broadcasts a commit message. After receiving 2f+1
//!    matching commit messages, the node commits the operation.
//!
//! ## Proof / Intuition
//!
//! The safety property of PBFT: if a correct node commits an operation at
//! sequence number s, then no correct node commits a different operation at s.
//!
//! The liveness property: if at most f nodes are Byzantine, the protocol
//! eventually commits operations.
//!
//! The 2f+1 quorum ensures that at least one honest node's prepare/commit
//! is counted, preventing a Byzantine leader from forging consensus.
//!
//! With n=3f+1 nodes:
//! - Prepare quorum: 2f+1 nodes
//! - Commit quorum: 2f+1 nodes
//! - Total messages per phase: O(n^2) in worst case
//!
//! ## Implementation Task
//!
//! 1. Implement PBFT message types and the message structure
//! 2. Implement a PBFT node that can be honest or Byzantine
//! 3. Implement the three-phase protocol (pre-prepare, prepare, commit)
//! 4. Test that consensus is reached with honest nodes
//! 5. Test that Byzantine nodes cannot break consensus with 3f+1 honest nodes
//!
//! ## Verification
//!
//! - Test correct operation with all honest nodes
//! - Test that a Byzantine node sending conflicting messages doesn't break consensus
//! - Test that quorum is required before committing

use std::collections::{HashMap, HashSet};

/// The type of PBFT message.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageType {
    Preprepare,
    Prepare,
    Commit,
    Request,
}

/// A PBFT consensus message.
#[derive(Debug, Clone)]
pub struct PBFTMessage {
    /// Type of message.
    pub msg_type: MessageType,
    /// Current view number.
    pub view: usize,
    /// Sequence number for the operation.
    pub sequence: usize,
    /// ID of the node that sent this message.
    pub sender_id: usize,
    /// Digest (hash) of the data for integrity verification.
    pub digest: u64,
    /// The actual data (serialized command).
    pub data: String,
}

impl PBFTMessage {
    /// Create a new PBFT message.
    pub fn new(
        msg_type: MessageType,
        view: usize,
        sequence: usize,
        sender_id: usize,
        data: &str,
    ) -> Self {
        let digest = simple_digest(data);
        Self {
            msg_type,
            view,
            sequence,
            sender_id,
            digest,
            data: data.to_string(),
        }
    }

    /// Create a message with a specific digest (for Byzantine behavior).
    pub fn with_digest(
        msg_type: MessageType,
        view: usize,
        sequence: usize,
        sender_id: usize,
        data: &str,
        digest: u64,
    ) -> Self {
        Self {
            msg_type,
            view,
            sequence,
            sender_id,
            digest,
            data: data.to_string(),
        }
    }
}

/// Simple digest function for message integrity.
fn simple_digest(data: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// A PBFT node that participates in consensus.
pub struct PBFTNode {
    /// Unique node ID.
    pub id: usize,
    /// The node's key-value state machine.
    state: HashMap<String, String>,
    /// Current view number.
    view: usize,
    /// Next sequence number to propose.
    sequence_number: usize,
    /// Map from "view-seq-digest" -> set of sender IDs who prepared.
    prepare_count: HashMap<String, HashSet<usize>>,
    /// Map from "view-seq-digest" -> set of sender IDs who committed.
    commit_count: HashMap<String, HashSet<usize>>,
    /// Map from "view-seq-digest" -> the pre-prepare message.
    preprepare_received: HashMap<String, PBFTMessage>,
    /// Whether this node is Byzantine (for testing).
    is_byzantine: bool,
    /// Total number of nodes in the system.
    total_nodes: usize,
    /// Messages generated for broadcasting.
    pub pending_messages: Vec<PBFTMessage>,
}

impl PBFTNode {
    /// Create a new honest PBFT node.
    pub fn new(id: usize, total_nodes: usize) -> Self {
        Self {
            id,
            state: HashMap::new(),
            view: 0,
            sequence_number: 0,
            prepare_count: HashMap::new(),
            commit_count: HashMap::new(),
            preprepare_received: HashMap::new(),
            is_byzantine: false,
            total_nodes,
            pending_messages: Vec::new(),
        }
    }

    /// Create a new Byzantine PBFT node (for testing faults).
    pub fn new_byzantine(id: usize, total_nodes: usize) -> Self {
        Self {
            id,
            state: HashMap::new(),
            view: 0,
            sequence_number: 0,
            prepare_count: HashMap::new(),
            commit_count: HashMap::new(),
            preprepare_received: HashMap::new(),
            is_byzantine: true,
            total_nodes,
            pending_messages: Vec::new(),
        }
    }

    /// Get the quorum size (2f+1 where f = (n-1)/3).
    fn quorum(&self) -> usize {
        let f = (self.total_nodes - 1) / 3;
        2 * f + 1
    }

    /// Send a request to the leader (this node proposes an operation).
    pub fn send_request(&mut self, data: &str) -> PBFTMessage {
        let msg = PBFTMessage::new(
            MessageType::Request,
            self.view,
            0, // sequence assigned by leader
            self.id,
            data,
        );
        self.pending_messages.push(msg.clone());
        msg
    }

    /// Handle a pre-prepare message from the leader.
    /// Returns true if the pre-prepare was accepted.
    pub fn handle_preprepare(&mut self, msg: &PBFTMessage) -> bool {
        if self.is_byzantine {
            return false;
        }

        let key = format!("{}-{}-{}", msg.view, msg.sequence, msg.digest);
        self.preprepare_received.insert(key.clone(), msg.clone());

        // Send a prepare message
        let prepare = PBFTMessage::new(
            MessageType::Prepare,
            msg.view,
            msg.sequence,
            self.id,
            &msg.data,
        );
        self.pending_messages.push(prepare);
        true
    }

    /// Handle a prepare message from another node.
    /// Returns true if the node is now "prepared" (has quorum of prepare messages).
    pub fn handle_prepare(&mut self, msg: &PBFTMessage) -> bool {
        if self.is_byzantine {
            return false;
        }

        let key = format!("{}-{}-{}", msg.view, msg.sequence, msg.digest);

        // Check if we have a matching pre-prepare
        if !self.preprepare_received.contains_key(&key) {
            return false;
        }

        self.prepare_count
            .entry(key.clone())
            .or_insert_with(HashSet::new)
            .insert(msg.sender_id);

        let count = self.prepare_count[&key].len();

        if count >= self.quorum() {
            // Node is now prepared, send commit
            let commit = PBFTMessage::new(
                MessageType::Commit,
                msg.view,
                msg.sequence,
                self.id,
                &msg.data,
            );
            self.pending_messages.push(commit);
            return true;
        }

        false
    }

    /// Handle a commit message from another node.
    /// Returns true if the node has committed (reached commit quorum).
    pub fn handle_commit(&mut self, msg: &PBFTMessage) -> bool {
        if self.is_byzantine {
            return false;
        }

        let key = format!("{}-{}-{}", msg.view, msg.sequence, msg.digest);

        self.commit_count
            .entry(key.clone())
            .or_insert_with(HashSet::new)
            .insert(msg.sender_id);

        let count = self.commit_count[&key].len();

        if count >= self.quorum() {
            // Commit the operation
            self.execute_if_committed(&msg.data);
            return true;
        }

        false
    }

    /// Execute a committed command on the state machine.
    pub fn execute_if_committed(&mut self, data: &str) {
        // Parse the command: "PUT key value" or "GET key" or "DELETE key"
        let parts: Vec<&str> = data.splitn(3, ' ').collect();
        if parts.len() >= 2 {
            match parts[0] {
                "PUT" => {
                    if parts.len() >= 3 {
                        self.state
                            .insert(parts[1].to_string(), parts[2].to_string());
                    }
                }
                "DELETE" => {
                    self.state.remove(parts[1]);
                }
                _ => {}
            }
        }
    }

    /// Get a value from the state machine.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.state.get(key).map(|s| s.as_str())
    }

    /// Drain pending messages (messages to broadcast).
    pub fn drain_messages(&mut self) -> Vec<PBFTMessage> {
        std::mem::take(&mut self.pending_messages)
    }

    /// Check if this node is Byzantine.
    pub fn is_byzantine(&self) -> bool {
        self.is_byzantine
    }

    /// Get the current view.
    pub fn view(&self) -> usize {
        self.view
    }

    /// Get the number of prepared messages for a given key.
    pub fn prepare_count(&self, key: &str) -> usize {
        self.prepare_count.get(key).map_or(0, |s| s.len())
    }

    /// Get the number of committed messages for a given key.
    pub fn commit_count(&self, key: &str) -> usize {
        self.commit_count.get(key).map_or(0, |s| s.len())
    }
}

/// Simulate a full PBFT round across all nodes.
/// Returns true if consensus was reached.
pub fn simulate_pbft_round(
    nodes: &mut Vec<PBFTNode>,
    command: &str,
) -> bool {
    // Step 1: Leader (node 0) broadcasts pre-prepare
    let view = nodes[0].view();
    let preprepare = PBFTMessage::new(
        MessageType::Preprepare,
        view,
        1,
        0,
        command,
    );

    // Leader also processes its own pre-prepare
    nodes[0].handle_preprepare(&preprepare);

    // All other nodes handle pre-prepare
    for node in nodes.iter_mut().skip(1) {
        node.handle_preprepare(&preprepare);
    }

    // Collect all prepare messages
    let mut prepare_msgs = Vec::new();
    for node in nodes.iter_mut() {
        let msgs = node.drain_messages();
        for msg in msgs {
            if msg.msg_type == MessageType::Prepare {
                prepare_msgs.push(msg);
            }
        }
    }

    // Broadcast prepare messages to all nodes
    for msg in &prepare_msgs {
        for node in nodes.iter_mut() {
            if node.id != msg.sender_id && !node.is_byzantine {
                node.handle_prepare(msg);
            }
        }
    }

    // Collect all commit messages
    let mut commit_msgs = Vec::new();
    for node in nodes.iter_mut() {
        let msgs = node.drain_messages();
        for msg in msgs {
            if msg.msg_type == MessageType::Commit {
                commit_msgs.push(msg);
            }
        }
    }

    // Broadcast commit messages to all nodes
    for msg in &commit_msgs {
        for node in nodes.iter_mut() {
            if node.id != msg.sender_id && !node.is_byzantine {
                node.handle_commit(msg);
            }
        }
    }

    // Check if consensus was reached: all honest nodes should have committed
    let key = command.split(' ').nth(1).unwrap_or("");
    nodes.iter().all(|n| n.get(key).is_some() || n.is_byzantine())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_honest_nodes_reach_consensus() {
        let mut nodes: Vec<PBFTNode> = (0..4).map(|i| PBFTNode::new(i, 4)).collect();

        let command = "PUT key1 value1";
        let reached = simulate_pbft_round(&mut nodes, command);

        assert!(reached, "Consensus should be reached with 4 honest nodes");
        // All honest nodes should have the value
        for node in &nodes {
            assert_eq!(node.get("key1"), Some("value1"));
        }
    }

    #[test]
    fn test_byzantine_node_doesnt_break_consensus() {
        let mut nodes: Vec<PBFTNode> = Vec::new();

        // 4 honest nodes + 1 Byzantine node = 5 total (3f+1 where f=1)
        for i in 0..4 {
            nodes.push(PBFTNode::new(i, 5));
        }
        nodes.push(PBFTNode::new_byzantine(4, 5));

        let command = "PUT secret hidden";

        // Leader sends pre-prepare
        let leader = &nodes[0];
        let preprepare = PBFTMessage::new(
            MessageType::Preprepare,
            leader.view(),
            1,
            0,
            command,
        );

        // Leader handles its own pre-prepare
        nodes[0].handle_preprepare(&preprepare);

        // Non-leader honest nodes handle pre-prepare
        for node in nodes.iter_mut().skip(1) {
            if !node.is_byzantine() {
                node.handle_preprepare(&preprepare);
            }
        }

        // Collect and broadcast prepare messages
        let mut prepare_msgs = Vec::new();
        for node in nodes.iter_mut() {
            let msgs = node.drain_messages();
            for msg in msgs {
                if msg.msg_type == MessageType::Prepare {
                    prepare_msgs.push(msg);
                }
            }
        }

        for msg in &prepare_msgs {
            for node in nodes.iter_mut() {
                if node.id != msg.sender_id && !node.is_byzantine() {
                    node.handle_prepare(msg);
                }
            }
        }

        // Collect and broadcast commit messages
        let mut commit_msgs = Vec::new();
        for node in nodes.iter_mut() {
            let msgs = node.drain_messages();
            for msg in msgs {
                if msg.msg_type == MessageType::Commit {
                    commit_msgs.push(msg);
                }
            }
        }

        for msg in &commit_msgs {
            for node in nodes.iter_mut() {
                if node.id != msg.sender_id && !node.is_byzantine() {
                    node.handle_commit(msg);
                }
            }
        }

        // Honest nodes should reach consensus despite Byzantine node
        for node in nodes.iter() {
            if !node.is_byzantine() {
                assert_eq!(
                    node.get("secret"),
                    Some("hidden"),
                    "Honest node {} should have committed",
                    node.id
                );
            }
        }
    }

    #[test]
    fn test_quorum_required() {
        let mut nodes: Vec<PBFTNode> = (0..4).map(|i| PBFTNode::new(i, 4)).collect();

        // Simulate a partial round where only 2 prepare messages arrive
        // (below quorum of 3 for 4 nodes)
        let preprepare = PBFTMessage::new(
            MessageType::Preprepare,
            0,
            1,
            0,
            "PUT incomplete round",
        );

        // Only nodes 1 and 2 handle pre-prepare (node 3 is "offline")
        nodes[1].handle_preprepare(&preprepare);
        nodes[2].handle_preprepare(&preprepare);

        let mut prepare_msgs = Vec::new();
        for node in nodes.iter_mut().skip(1).take(2) {
            let msgs = node.drain_messages();
            for msg in msgs {
                if msg.msg_type == MessageType::Prepare {
                    prepare_msgs.push(msg);
                }
            }
        }

        // Broadcast prepare to all
        for msg in &prepare_msgs {
            for node in nodes.iter_mut() {
                if node.id != msg.sender_id && !node.is_byzantine() {
                    node.handle_prepare(msg);
                }
            }
        }

        // Collect any commit messages
        let mut commit_msgs = Vec::new();
        for node in nodes.iter_mut() {
            let msgs = node.drain_messages();
            for msg in msgs {
                if msg.msg_type == MessageType::Commit {
                    commit_msgs.push(msg);
                }
            }
        }

        // With only 2 prepare messages, quorum (3) is not reached
        // So no commit messages should be generated
        assert!(
            commit_msgs.is_empty(),
            "No commit messages should be sent without quorum"
        );

        // State should not be committed
        assert_eq!(nodes[0].get("incomplete"), None);
    }
}
