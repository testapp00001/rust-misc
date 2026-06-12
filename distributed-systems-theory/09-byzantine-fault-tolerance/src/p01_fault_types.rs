//! # Exercise: Fault Types
//!
//! ## Theory
//!
//! Distributed systems must be designed to handle various types of faults.
//! Understanding the fault model is critical because it determines which
//! protocols are needed and how many redundant nodes are required.
//!
//! There are three primary fault models, listed from weakest to strongest:
//!
//! 1. **Crash Faults**: A node simply stops functioning. It stops sending
//!    messages, stops responding to requests, and does not corrupt any data.
//!    The node is permanently silent. Protocols like Raft and Paxos tolerate
//!    crash faults with n >= 2f + 1 nodes.
//!
//! 2. **Omission Faults**: A node fails to deliver some of its messages. Unlike
//!    a crash, the node may still process some requests and send some messages,
//!    but selectively (or randomly) drops others. Omission faults are a middle
//!    ground between crash and Byzantine.
//!
//! 3. **Byzantine Faults**: A node can exhibit arbitrary behavior. It may send
//!    conflicting messages to different nodes, fabricate messages, refuse to
//!    respond, or collude with other faulty nodes. Byzantine faults are the
//!    strongest failure model, requiring n >= 3f + 1 nodes for agreement.
//!
//! ## Proof / Intuition
//!
//! The hierarchy of fault severity is: Crash < Omission < Byzantine.
//!
//! - A crashed node is **detectable** via timeouts -- if a node doesn't respond,
//!   other nodes eventually conclude it is dead.
//! - An omitting node is **partially detectable** -- messages are missing, but
//!   the node still appears alive.
//! - A Byzantine node is **undetectable** in isolation -- its messages look
//!   legitimate. Only protocol-level redundancy (quorum checks) can expose
//!   inconsistencies.
//!
//! The key insight is that the more powerful the fault model, the more redundancy
//! (nodes) you need. Crash tolerance needs 2f + 1; Byzantine tolerance needs 3f + 1.
//!
//! ## Implementation Task
//!
//! Implement the following:
//!
//! - `FaultType` enum with variants: `Crash`, `Omission`, `Byzantine`
//! - `FaultyNode` struct with fields:
//!   - `node_id: usize`
//!   - `fault_type: FaultType`
//!   - `is_alive: bool`
//!   - `received_messages: Vec<String>`
//! - Methods:
//!   - `new(node_id, fault_type)` - create a new faulty node
//!   - `send_message(&self, content, target_id) -> Option<(usize, String)>` -
//!     simulate sending a message based on fault type
//!   - `receive_message(&mut self, content)` - record an incoming message
//!   - `is_responsive(&self) -> bool` - check if the node is responsive
//!
//! ## Verification
//!
//! - Verify crash fault stops responding (is_alive becomes false)
//! - Verify omission fault drops messages based on omission_rate
//! - Verify Byzantine fault sends conflicting messages to different targets
//! - Verify a healthy node responds normally

use rand::Rng;

/// The type of fault a node can exhibit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultType {
    /// Node stops functioning entirely -- stops sending and receiving messages.
    Crash,
    /// Node randomly drops some outgoing messages.
    Omission,
    /// Node sends arbitrary or conflicting messages.
    Byzantine,
}

/// A node in the distributed system that can exhibit faulty behavior.
///
/// The fault type determines how the node behaves when sending and receiving
/// messages. A crashed node is permanently unresponsive, an omitting node
/// randomly drops messages, and a Byzantine node sends conflicting information.
#[derive(Debug, Clone)]
pub struct FaultyNode {
    /// Unique identifier for this node.
    pub node_id: usize,
    /// The type of fault this node exhibits.
    pub fault_type: FaultType,
    /// Whether the node is currently alive and responsive.
    pub is_alive: bool,
    /// Messages that have been received by this node.
    pub received_messages: Vec<String>,
    /// Probability of dropping an outgoing message (only used for Omission faults).
    pub omission_rate: f64,
}

impl FaultyNode {
    /// Create a new faulty node with the given ID and fault type.
    ///
    /// The node starts alive. For omission faults, the default omission rate
    /// is 0.5 (50% of messages are dropped).
    pub fn new(node_id: usize, fault_type: FaultType) -> Self {
        Self {
            node_id,
            fault_type,
            is_alive: true,
            received_messages: Vec::new(),
            omission_rate: 0.5,
        }
    }

    /// Create a new faulty node with a custom omission rate.
    ///
    /// Only relevant for `FaultType::Omission`. The rate should be between
    /// 0.0 (no drops) and 1.0 (all drops).
    pub fn new_with_omission_rate(node_id: usize, omission_rate: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&omission_rate),
            "omission_rate must be between 0.0 and 1.0"
        );
        Self {
            node_id,
            fault_type: FaultType::Omission,
            is_alive: true,
            received_messages: Vec::new(),
            omission_rate,
        }
    }

    /// Simulate sending a message to a target node.
    ///
    /// Returns `Some((target_id, content))` if the message is sent, or `None`
    /// if the message is dropped (due to crash or omission).
    ///
    /// - **Crash**: Always returns `None` (node is dead, sends nothing).
    /// - **Omission**: Returns `None` with probability equal to `omission_rate`.
    /// - **Byzantine**: Always returns `Some`, but may modify the content to
    ///   create conflicting messages (different content for different targets).
    pub fn send_message(&self, content: &str, target_id: usize) -> Option<(usize, String)> {
        match self.fault_type {
            FaultType::Crash => {
                // Crashed node sends nothing
                None
            }
            FaultType::Omission => {
                if !self.is_alive {
                    return None;
                }
                let mut rng = rand::thread_rng();
                let roll: f64 = rng.gen();
                if roll < self.omission_rate {
                    // Message is dropped
                    None
                } else {
                    Some((target_id, content.to_string()))
                }
            }
            FaultType::Byzantine => {
                // Byzantine node sends modified messages to create confusion
                // It alternates between sending the true content and a false one
                let modified = if target_id % 2 == 0 {
                    content.to_string()
                } else {
                    format!("CONFLICTING({content})")
                };
                Some((target_id, modified))
            }
        }
    }

    /// Record an incoming message.
    ///
    /// A crashed node ignores incoming messages (it is dead). A live node
    /// records the message in its `received_messages` vector.
    pub fn receive_message(&mut self, content: &str) {
        if self.is_alive {
            self.received_messages.push(content.to_string());
        }
    }

    /// Check if the node is currently responsive.
    ///
    /// Returns `true` if the node is alive. A crashed node always returns `false`.
    /// Omission and Byzantine nodes remain alive but exhibit faulty behavior.
    pub fn is_responsive(&self) -> bool {
        self.is_alive
    }

    /// Simulate a crash fault: mark the node as dead.
    ///
    /// After crashing, the node stops sending and receiving messages.
    /// Only meaningful for crash-type faults, but can be called on any node.
    pub fn crash(&mut self) {
        self.is_alive = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crash_stops_responding() {
        let mut node = FaultyNode::new(0, FaultType::Crash);
        assert!(node.is_responsive(), "node should be alive initially");

        node.crash();
        assert!(
            !node.is_responsive(),
            "crashed node should not be responsive"
        );

        // Crashed node sends nothing
        let result = node.send_message("hello", 1);
        assert!(
            result.is_none(),
            "crashed node should not send messages"
        );

        // Crashed node ignores incoming messages
        node.receive_message("world");
        assert!(
            node.received_messages.is_empty(),
            "crashed node should not record received messages"
        );
    }

    #[test]
    fn omission_drops_messages() {
        let node = FaultyNode::new_with_omission_rate(0, 1.0);
        // With 100% omission rate, all messages should be dropped
        for target in 0..100 {
            let result = node.send_message("hello", target);
            assert!(
                result.is_none(),
                "100% omission should drop all messages"
            );
        }

        let node = FaultyNode::new_with_omission_rate(0, 0.0);
        // With 0% omission rate, no messages should be dropped
        let mut sent = 0;
        for target in 0..100 {
            if node.send_message("hello", target).is_some() {
                sent += 1;
            }
        }
        assert_eq!(sent, 100, "0% omission should deliver all messages");
    }

    #[test]
    fn byzantine_sends_conflicting_messages() {
        let node = FaultyNode::new(0, FaultType::Byzantine);

        // Send to multiple targets and check that at least some are conflicting
        let mut messages = Vec::new();
        for target in 0..10 {
            if let Some((_, content)) = node.send_message("attack", target) {
                messages.push(content);
            }
        }

        // Byzantine node should produce both original and conflicting messages
        let has_original = messages.iter().any(|m| m == "attack");
        let has_conflicting = messages.iter().any(|m| m.contains("CONFLICTING"));
        assert!(
            has_original,
            "Byzantine node should send original messages to some targets"
        );
        assert!(
            has_conflicting,
            "Byzantine node should send conflicting messages to other targets"
        );
    }

    #[test]
    fn healthy_node_responds_normally() {
        let node = FaultyNode::new(0, FaultType::Crash);
        // Before crashing, a crash-type node is still alive
        assert!(node.is_responsive());

        let mut node = FaultyNode::new(1, FaultType::Byzantine);
        assert!(node.is_responsive());
        node.receive_message("test");
        assert_eq!(
            node.received_messages.len(),
            1,
            "Byzantine node should record incoming messages"
        );
    }

    #[test]
    fn omission_receive_message_works() {
        let mut node = FaultyNode::new(0, FaultType::Omission);
        node.receive_message("msg1");
        node.receive_message("msg2");
        assert_eq!(
            node.received_messages.len(),
            2,
            "omission node should record all incoming messages"
        );
    }
}
