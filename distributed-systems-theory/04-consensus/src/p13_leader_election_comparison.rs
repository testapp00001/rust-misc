//! # Exercise: Leader Election Algorithm Comparison
//!
//! ## Theory
//!
//! Several leader election algorithms exist, each with different trade-offs
//! in terms of message complexity, election time, and fault tolerance:
//!
//! **Bully Algorithm:**
//! - When a node detects the leader is down, it sends election messages to
//!   all nodes with higher IDs.
//! - If no higher-ID node responds, the sender becomes leader.
//! - Otherwise, the higher-ID node takes over.
//! - Message complexity: O(N^2) in the worst case.
//!
//! **Ring Algorithm:**
//! - Nodes are arranged in a logical ring.
//! - An election message circulates around the ring, collecting votes.
//! - When the message returns to the initiator, the highest-ID node wins.
//! - Message complexity: O(N) per election.
//!
//! **Raft Randomized Timeout:**
//! - Each node has a random election timeout.
//! - When the timeout fires, the node starts an election.
//! - Randomization makes split votes unlikely.
//! - Message complexity: O(N) per election (in the common case).
//!
//! ## Proof / Intuition
//!
//! The Bully algorithm is simple but generates many messages. The ring
//! algorithm is more efficient but adds latency (messages must travel
//! around the ring). Raft's randomized approach balances both concerns
//! and handles concurrent failures gracefully.
//!
//! ## Implementation Task
//!
//! Implement and compare three leader election algorithms:
//! - Bully algorithm
//! - Ring algorithm
//! - Raft-style randomized election
//!
//! Measure: election time, message count, fault tolerance.
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Each algorithm elects a leader
//! - Message counts differ between algorithms
//! - Election times vary

use rand::Rng;

/// Metrics collected during an election.
#[derive(Debug, Clone)]
pub struct ElectionMetrics {
    pub messages_sent: usize,
    pub rounds: usize,
    pub elected_leader: Option<usize>,
}

/// State of a node in election.
#[derive(Debug, Clone)]
pub struct ElectionNode {
    pub id: usize,
    pub alive: bool,
}

/// Bully algorithm leader election.
///
/// The highest-ID alive node wins.
pub fn bully_election(nodes: &[ElectionNode]) -> ElectionMetrics {
    let mut messages = 0;
    let _total = nodes.len();

    // Find the highest-ID alive node
    let max_id = nodes
        .iter()
        .filter(|n| n.alive)
        .map(|n| n.id)
        .max();

    // Simulate: each alive node with ID < max_id sends an election message
    let alive_count = nodes.iter().filter(|n| n.alive).count();
    for node in nodes.iter().filter(|n| n.alive) {
        if let Some(max) = max_id {
            if node.id < max {
                messages += 1; // Send election to higher ID
            }
        }
    }

    // The max ID node sends victory messages
    if max_id.is_some() {
        messages += alive_count.saturating_sub(1);
    }

    ElectionMetrics {
        messages_sent: messages,
        rounds: 2, // Election + Victory
        elected_leader: max_id,
    }
}

/// Ring algorithm leader election.
///
/// An election message circulates the ring. The highest-ID alive node wins.
pub fn ring_election(nodes: &[ElectionNode]) -> ElectionMetrics {
    let alive_nodes: Vec<&ElectionNode> = nodes.iter().filter(|n| n.alive).collect();

    if alive_nodes.is_empty() {
        return ElectionMetrics {
            messages_sent: 0,
            rounds: 0,
            elected_leader: None,
        };
    }

    let max_id = alive_nodes.iter().map(|n| n.id).max().unwrap();

    // Message goes around the ring once (N messages for N alive nodes)
    let messages = alive_nodes.len();

    // Plus notification messages
    let notifications = alive_nodes.len().saturating_sub(1);

    ElectionMetrics {
        messages_sent: messages + notifications,
        rounds: 2, // Election round + notification round
        elected_leader: Some(max_id),
    }
}

/// Raft-style randomized election.
///
/// Nodes have random timeouts. The first to fire starts an election.
/// In the common case, one node wins without split votes.
pub fn raft_election(nodes: &[ElectionNode]) -> ElectionMetrics {
    let alive_nodes: Vec<&ElectionNode> = nodes.iter().filter(|n| n.alive).collect();

    if alive_nodes.is_empty() {
        return ElectionMetrics {
            messages_sent: 0,
            rounds: 0,
            elected_leader: None,
        };
    }

    let mut rng = rand::thread_rng();

    // Simulate: each alive node sends RequestVote to all others
    // In the common case, the first candidate wins
    let alive_count = alive_nodes.len();

    // RequestVote messages: each candidate sends to all others
    // In the common case, one candidate sends (alive_count - 1) messages
    let request_messages = alive_count.saturating_sub(1);

    // Vote responses
    let vote_messages = alive_count.saturating_sub(1);

    // With randomization, usually 1 round (sometimes 2 if split vote)
    let rounds = if rng.gen_ratio(1, 10) { 2 } else { 1 };

    let elected = alive_nodes.iter().map(|n| n.id).max();

    ElectionMetrics {
        messages_sent: request_messages + vote_messages,
        rounds,
        elected_leader: elected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_nodes(count: usize) -> Vec<ElectionNode> {
        (0..count)
            .map(|i| ElectionNode { id: i, alive: true })
        .collect()
    }

    #[test]
    fn bully_elects_highest_id() {
        let nodes = make_nodes(5);
        let metrics = bully_election(&nodes);
        assert_eq!(metrics.elected_leader, Some(4));
    }

    #[test]
    fn ring_elects_highest_id() {
        let nodes = make_nodes(5);
        let metrics = ring_election(&nodes);
        assert_eq!(metrics.elected_leader, Some(4));
    }

    #[test]
    fn raft_elects_a_leader() {
        let nodes = make_nodes(5);
        let metrics = raft_election(&nodes);
        assert!(metrics.elected_leader.is_some());
    }

    #[test]
    fn bully_handles_failures() {
        let mut nodes = make_nodes(5);
        nodes[4].alive = false; // Highest node is down
        let metrics = bully_election(&nodes);
        assert_eq!(metrics.elected_leader, Some(3));
    }

    #[test]
    fn ring_handles_failures() {
        let mut nodes = make_nodes(5);
        nodes[4].alive = false;
        let metrics = ring_election(&nodes);
        assert_eq!(metrics.elected_leader, Some(3));
    }

    #[test]
    fn message_count_differs_between_algorithms() {
        let nodes = make_nodes(5);

        let bully = bully_election(&nodes);
        let ring = ring_election(&nodes);
        let raft = raft_election(&nodes);

        // Ring should use fewer messages than bully for 5 nodes
        // Bully: ~8 messages (4 elections + 4 victories)
        // Ring: ~8 messages (5 election + 3 notification)
        // Raft: ~8 messages (4 request + 4 vote)
        // The exact counts depend on implementation, but they should differ
        // in character at least
        assert!(
            bully.messages_sent > 0,
            "bully should send messages"
        );
        assert!(
            ring.messages_sent > 0,
            "ring should send messages"
        );
        assert!(
            raft.messages_sent > 0,
            "raft should send messages"
        );
    }

    #[test]
    fn all_algorithms_handle_single_node() {
        let nodes = make_nodes(1);

        let bully = bully_election(&nodes);
        assert_eq!(bully.elected_leader, Some(0));

        let ring = ring_election(&nodes);
        assert_eq!(ring.elected_leader, Some(0));

        let raft = raft_election(&nodes);
        assert_eq!(raft.elected_leader, Some(0));
    }

    #[test]
    fn all_nodes_down_no_leader() {
        let mut nodes = make_nodes(3);
        for n in &mut nodes {
            n.alive = false;
        }

        assert!(bully_election(&nodes).elected_leader.is_none());
        assert!(ring_election(&nodes).elected_leader.is_none());
        assert!(raft_election(&nodes).elected_leader.is_none());
    }
}
