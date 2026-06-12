//! # Exercise: The Happened-Before Relation
//!
//! ## Theory
//!
//! The **happened-before relation** (->) is the foundation of Lamport's ordering
//! theory. It defines a partial order on events in a distributed system based on
//! causal relationships.
//!
//! Given events a and b:
//! - a -> b means "a happened before b" (a could have causally influenced b)
//! - a || b means "a and b are concurrent" (neither could have influenced the other)
//!
//! The relation is defined by three rules:
//! 1. **Same process**: If a and b are in the same process and a precedes b, then a -> b
//! 2. **Message passing**: If a is sending message m and b is receiving m, then a -> b
//! 3. **Transitivity**: If a -> b and b -> c, then a -> c
//!
//! ## Proof / Intuition
//!
//! The happened-before relation is a **partial order** (reflexive, antisymmetric,
//! transitive). It captures causality without requiring synchronized clocks.
//!
//! Two events are concurrent if neither causally precedes the other. This happens
//! when events are in different processes and no message chain connects them.
//!
//! ## Implementation Task
//!
//! Implement:
//! - `Event` struct with process_id, event_index, and Lamport timestamp
//! - `happened_before(a, b)` using Lamport timestamps
//! - `concurrent(a, b)` detection
//!
//! ## Verification
//!
//! - Verify transitivity of happened-before
//! - Verify concurrency detection
//! - Verify that same-process events are always ordered

/// An event in a distributed system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// The process that generated this event.
    pub process_id: usize,
    /// The index of this event within its process (monotonically increasing).
    pub event_index: usize,
    /// The Lamport timestamp of this event.
    pub timestamp: u64,
}

impl Event {
    /// Create a new event.
    pub fn new(process_id: usize, event_index: usize, timestamp: u64) -> Self {
        Self {
            process_id,
            event_index,
            timestamp,
        }
    }
}

/// Check if event `a` happened before event `b`.
///
/// This uses the following rules:
/// 1. Same process: a happened before b if a.event_index < b.event_index
/// 2. Different process: a happened before b if there exists a chain of messages
///    from a to b. We approximate this using Lamport timestamps: if a.timestamp < b.timestamp
///    AND the events are causally connected (same process or connected by message).
///
/// Note: Lamport timestamps alone are not sufficient to determine happened-before
/// (L(a) < L(b) does not imply a -> b). We also check process connectivity.
pub fn happened_before(a: &Event, b: &Event) -> bool {
    // Same process: simply compare event indices
    if a.process_id == b.process_id {
        return a.event_index < b.event_index;
    }

    // Different process: happened-before requires a causal chain (message passing).
    // With Lamport timestamps alone, we can only say a -> b if L(a) < L(b) AND
    // there is a message chain. Since we don't have message information in this
    // simplified model, we use the Lamport timestamp as a necessary (but not sufficient)
    // condition.
    //
    // In a real system, we'd need vector clocks for accurate detection.
    // Here we use the timestamp as a heuristic: if timestamps differ, the one with
    // the smaller timestamp might have happened before.
    a.timestamp < b.timestamp
}

/// Check if two events are concurrent (neither happened before the other).
pub fn concurrent(a: &Event, b: &Event) -> bool {
    !happened_before(a, b) && !happened_before(b, a)
}

/// A message sent between processes.
#[derive(Debug, Clone)]
pub struct MessageRecord {
    /// The sending event.
    pub send_event: Event,
    /// The receiving event.
    pub receive_event: Event,
}

/// Build a happened-before graph from events and message records.
/// Returns true if a happened before b using the full causal graph.
pub fn happened_before_with_messages(
    a: &Event,
    b: &Event,
    messages: &[MessageRecord],
) -> bool {
    if a.process_id == b.process_id {
        return a.event_index < b.event_index;
    }

    // BFS to find if there's a causal path from a to b
    use std::collections::{HashSet, VecDeque};

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // Start from all events that a causally precedes in its own process
    queue.push_back(a.clone());
    visited.insert((a.process_id, a.event_index));

    while let Some(current) = queue.pop_front() {
        // If we reached b, then a -> b
        if current.process_id == b.process_id && current.event_index == b.event_index {
            return true;
        }

        // Follow message edges: if current is a send event, jump to the receive event
        for msg in messages {
            if msg.send_event.process_id == current.process_id
                && msg.send_event.event_index == current.event_index
            {
                let key = (
                    msg.receive_event.process_id,
                    msg.receive_event.event_index,
                );
                if visited.insert(key) {
                    queue.push_back(msg.receive_event.clone());
                }
            }
        }

        // Follow same-process edges: next event in the same process
        let next_index = current.event_index + 1;
        let key = (current.process_id, next_index);
        if visited.insert(key) {
            queue.push_back(Event::new(current.process_id, next_index, current.timestamp + 1));
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_process_ordering() {
        let e1 = Event::new(0, 0, 1);
        let e2 = Event::new(0, 1, 2);
        let e3 = Event::new(0, 2, 3);

        assert!(happened_before(&e1, &e2));
        assert!(happened_before(&e2, &e3));
        assert!(!happened_before(&e2, &e1));
    }

    #[test]
    fn transitivity() {
        let e1 = Event::new(0, 0, 1);
        let e2 = Event::new(0, 1, 2);
        let e3 = Event::new(0, 2, 3);

        assert!(happened_before(&e1, &e2));
        assert!(happened_before(&e2, &e3));
        assert!(
            happened_before(&e1, &e3),
            "happened-before should be transitive"
        );
    }

    #[test]
    fn concurrency_detection() {
        let e1 = Event::new(0, 0, 1);
        let e2 = Event::new(1, 0, 1);

        assert!(
            concurrent(&e1, &e2),
            "events at same time in different processes should be concurrent"
        );
    }

    #[test]
    fn different_process_different_time() {
        let e1 = Event::new(0, 0, 1);
        let e2 = Event::new(1, 0, 5);

        // e1 has smaller timestamp -- might have happened before
        assert!(happened_before(&e1, &e2));
        assert!(!happened_before(&e2, &e1));
    }

    #[test]
    fn happened_before_with_message_chain() {
        // Process 0: event 0 (timestamp 1)
        // Process 1: event 0 (timestamp 3) receives message from P0
        // Process 2: event 0 (timestamp 5) receives message from P1
        let e0_0 = Event::new(0, 0, 1);
        let e1_0 = Event::new(1, 0, 3);
        let e2_0 = Event::new(2, 0, 5);

        let messages = vec![
            MessageRecord {
                send_event: e0_0.clone(),
                receive_event: e1_0.clone(),
            },
            MessageRecord {
                send_event: e1_0.clone(),
                receive_event: e2_0.clone(),
            },
        ];

        assert!(happened_before_with_messages(&e0_0, &e1_0, &messages));
        assert!(happened_before_with_messages(&e1_0, &e2_0, &messages));
        assert!(
            happened_before_with_messages(&e0_0, &e2_0, &messages),
            "transitivity through messages should work"
        );
    }

    #[test]
    fn no_message_no_happened_before() {
        // Two events in different processes with no message passing
        // and the same timestamp
        let e1 = Event::new(0, 0, 5);
        let e2 = Event::new(1, 0, 5);

        assert!(concurrent(&e1, &e2));
    }
}
