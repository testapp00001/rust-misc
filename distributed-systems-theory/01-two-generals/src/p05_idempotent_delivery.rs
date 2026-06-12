//! # Exercise: Idempotent Message Delivery
//!
//! ## Theory
//!
//! When messages are retried over an unreliable channel, duplicates are inevitable.
//! An **idempotent operation** is one that produces the same result regardless of
//! how many times it is applied. By designing receivers to be idempotent, we can
//! safely retry messages without side effects from duplicates.
//!
//! A message has a unique `MessageId`. The receiver tracks which IDs it has already
//! processed and discards duplicates. This is the foundation of at-least-once
//! delivery semantics: the network may deliver a message multiple times, but the
//! application processes it exactly once.
//!
//! ## Proof / Intuition
//!
//! Consider a message queue delivering a payment instruction:
//! - Without idempotency: 3 deliveries = 3 payments (catastrophic)
//! - With idempotency: 3 deliveries = 1 payment (correct)
//!
//! The receiver maintains a set of processed message IDs. On receiving a message:
//! 1. Check if ID is in the set -> if yes, discard (duplicate)
//! 2. If no, process the message and add ID to the set
//!
//! The ordering of delivery does not affect correctness: processing messages out of
//! order still produces the correct result because each message is processed exactly
//! once.
//!
//! ## Implementation Task
//!
//! Implement:
//! - `MessageId` as a unique identifier (u64 sequence number)
//! - `Message<T>` containing an id and payload
//! - `IdempotentReceiver<T>` that deduplicates messages
//!
//! ## Verification
//!
//! - Duplicate messages are detected and discarded
//! - Unique messages all pass through
//! - Delivery order does not affect correctness

use std::collections::{HashMap, HashSet};

/// A unique identifier for a message.
pub type MessageId = u64;

/// A message with a unique identifier and a payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message<T> {
    /// Unique identifier for this message.
    pub id: MessageId,
    /// The payload of the message.
    pub payload: T,
}

/// An idempotent receiver that deduplicates messages by their ID.
///
/// Tracks all seen message IDs and only delivers messages that haven't
/// been seen before.
pub struct IdempotentReceiver<T> {
    /// Set of message IDs that have already been processed.
    seen_ids: HashSet<MessageId>,
    /// Buffered messages that have been received (for ordered processing).
    buffer: Vec<Message<T>>,
    /// Map from message ID to the processed result.
    results: HashMap<MessageId, T>,
}

impl<T: Clone> IdempotentReceiver<T> {
    /// Create a new idempotent receiver.
    pub fn new() -> Self {
        Self {
            seen_ids: HashSet::new(),
            buffer: Vec::new(),
            results: HashMap::new(),
        }
    }

    /// Receive a message. Returns `Some(payload)` if this is a new message,
    /// or `None` if it's a duplicate.
    pub fn receive(&mut self, msg: Message<T>) -> Option<T> {
        if self.seen_ids.contains(&msg.id) {
            // Duplicate -- discard
            None
        } else {
            // New message -- record and deliver
            let payload = msg.payload.clone();
            self.seen_ids.insert(msg.id);
            self.results.insert(msg.id, payload.clone());
            self.buffer.push(msg);
            Some(payload)
        }
    }

    /// Check if a message ID has already been seen.
    pub fn has_seen(&self, id: MessageId) -> bool {
        self.seen_ids.contains(&id)
    }

    /// Return the number of unique messages received.
    pub fn unique_count(&self) -> usize {
        self.seen_ids.len()
    }

    /// Get all processed message IDs.
    pub fn processed_ids(&self) -> &HashSet<MessageId> {
        &self.seen_ids
    }

    /// Get the result for a specific message ID.
    pub fn get_result(&self, id: MessageId) -> Option<&T> {
        self.results.get(&id)
    }

    /// Get all buffered messages in order of receipt.
    pub fn messages(&self) -> &[Message<T>] {
        &self.buffer
    }
}

impl<T: Clone> Default for IdempotentReceiver<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a sequence of messages, some of which are duplicates.
/// Returns (unique_messages, all_messages_with_duplicates).
pub fn generate_messages_with_duplicates(
    count: u64,
    duplicate_rate: f64,
) -> (Vec<Message<String>>, Vec<Message<String>>) {
    use rand::Rng;

    let mut unique_messages = Vec::new();
    let mut all_messages = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 0..count {
        let msg = Message {
            id: i,
            payload: format!("message_{i}"),
        };
        unique_messages.push(msg.clone());
        all_messages.push(msg);

        // Possibly add duplicates
        let num_duplicates = if rng.gen::<f64>() < duplicate_rate {
            rng.gen_range(1..=3)
        } else {
            0
        };

        for _ in 0..num_duplicates {
            all_messages.push(Message {
                id: i,
                payload: format!("message_{i}"),
            });
        }
    }

    (unique_messages, all_messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_messages_are_detected() {
        let mut receiver = IdempotentReceiver::new();

        let msg = Message {
            id: 1,
            payload: "hello".to_string(),
        };

        // First delivery succeeds
        let result1 = receiver.receive(msg.clone());
        assert_eq!(result1, Some("hello".to_string()));

        // Second delivery returns None (duplicate)
        let result2 = receiver.receive(msg);
        assert!(result2.is_none(), "duplicate should return None");
    }

    #[test]
    fn unique_messages_all_pass_through() {
        let mut receiver = IdempotentReceiver::new();

        for i in 0..100 {
            let msg = Message {
                id: i,
                payload: format!("payload_{i}"),
            };
            let result = receiver.receive(msg);
            assert!(
                result.is_some(),
                "unique message {i} should pass through"
            );
            assert_eq!(result.unwrap(), format!("payload_{i}"));
        }

        assert_eq!(receiver.unique_count(), 100);
    }

    #[test]
    fn ordering_does_not_matter() {
        // Deliver messages out of order -- all unique messages still pass through
        let mut receiver = IdempotentReceiver::new();

        let messages = vec![
            Message {
                id: 5,
                payload: "fifth".to_string(),
            },
            Message {
                id: 2,
                payload: "second".to_string(),
            },
            Message {
                id: 8,
                payload: "eighth".to_string(),
            },
            Message {
                id: 1,
                payload: "first".to_string(),
            },
            Message {
                id: 5, // duplicate
                payload: "fifth".to_string(),
            },
        ];

        let results: Vec<_> = messages
            .into_iter()
            .filter_map(|m| receiver.receive(m))
            .collect();

        assert_eq!(results.len(), 4); // 5 messages, 1 duplicate
        assert_eq!(results[0], "fifth");
        assert_eq!(results[1], "second");
        assert_eq!(results[2], "eighth");
        assert_eq!(results[3], "first");
    }

    #[test]
    fn has_seen_tracks_processed_ids() {
        let mut receiver = IdempotentReceiver::new();

        assert!(!receiver.has_seen(1));

        receiver.receive(Message {
            id: 1,
            payload: "test".to_string(),
        });

        assert!(receiver.has_seen(1));
        assert!(!receiver.has_seen(2));
    }

    #[test]
    fn duplicate_rate_experiment() {
        let (unique, all) = generate_messages_with_duplicates(100, 0.3);
        let total = all.len();

        let mut receiver = IdempotentReceiver::new();
        let mut delivered = 0;
        let mut duplicates_caught = 0;

        for msg in all {
            if receiver.receive(msg).is_some() {
                delivered += 1;
            } else {
                duplicates_caught += 1;
            }
        }

        assert_eq!(delivered, unique.len(), "all unique messages should be delivered");
        assert!(duplicates_caught > 0, "some duplicates should have been caught");
        assert_eq!(delivered + duplicates_caught, total);
    }
}
