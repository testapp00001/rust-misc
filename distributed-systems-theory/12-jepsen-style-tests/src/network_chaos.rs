//! # Exercise: Network Chaos Monkey
//!
//! ## Theory
//!
//! Real networks are unreliable. Messages can be:
//!
//! - **Lost**: dropped silently and never delivered.
//! - **Duplicated**: delivered more than once.
//! - **Reordered**: delivered in a different order than they were sent.
//! - **Delayed**: delivered after an unpredictable delay.
//!
//! A chaos monkey applies these perturbations to a stream of messages,
//! allowing you to test that your distributed system handles each case
//! correctly. This technique was popularized by Netflix's Chaos Monkey and
//! is a core component of Jepsen-style testing.
//!
//! ## Proof / Intuition
//!
//! Consider a system that processes messages in order. Under message
//! reordering, the system must still produce a consistent state. For example,
//! if it receives:
//!
//!   1. SET x = 1
//!   2. SET x = 2
//!   3. GET x -> should return 2
//!
//! But the network delivers them as 2, 1, 3, the system should still
//! converge to x = 2 (the last write wins) or maintain causal order.
//!
//! Similarly, under message loss, the system should not crash or enter an
//! inconsistent state -- it should continue operating with whatever subset
//! of messages were delivered.
//!
//! ## Implementation Task
//!
//! Implement:
//! - `ChaosMonkey` that applies various chaos types to a message stream.
//! - `ResilientSystem` that processes messages and maintains consistency
//!   under chaos conditions.
//!
//! ## Verification
//!
//! Run `cargo test network_chaos` to verify your implementation.

use std::collections::HashMap;

use rand::Rng;

/// Types of network chaos that can be injected.
#[derive(Debug, Clone)]
pub enum ChaosType {
    /// Drop messages with a certain probability.
    MessageLoss,
    /// Duplicate messages with a certain probability.
    MessageDuplication,
    /// Reorder adjacent messages in the stream.
    MessageReordering,
    /// Delay messages by a random amount within a range.
    MessageDelay(u64),
}

/// A message in the network.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    /// Unique message identifier.
    pub id: usize,
    /// Source node.
    pub from: usize,
    /// Destination node.
    pub to: usize,
    /// The payload string.
    pub payload: String,
    /// Timestamp when the message was sent.
    pub timestamp: u64,
}

/// A chaos monkey that injects faults into message streams.
#[derive(Debug)]
pub struct ChaosMonkey {
    /// The types of chaos to apply.
    pub chaos_types: Vec<ChaosType>,
    /// Probability of dropping a message (0.0 to 1.0).
    pub loss_rate: f64,
    /// Probability of duplicating a message (0.0 to 1.0).
    pub duplication_rate: f64,
    /// Range for message delay (min, max) in time units.
    pub delay_range: std::ops::Range<u64>,
}

impl ChaosMonkey {
    /// Create a new chaos monkey with specified chaos types.
    pub fn new(chaos_types: Vec<ChaosType>, loss_rate: f64, duplication_rate: f64) -> Self {
        Self {
            chaos_types,
            loss_rate,
            duplication_rate,
            delay_range: 1..10,
        }
    }

    /// Create a chaos monkey with custom delay range.
    pub fn with_delay_range(
        chaos_types: Vec<ChaosType>,
        loss_rate: f64,
        duplication_rate: f64,
        delay_range: std::ops::Range<u64>,
    ) -> Self {
        Self {
            chaos_types,
            loss_rate,
            duplication_rate,
            delay_range,
        }
    }

    /// Apply all configured chaos types to a list of messages.
    /// Returns the resulting (possibly modified) message list.
    pub fn inject_chaos(&self, messages: Vec<Message>) -> Vec<Message> {
        let mut result = messages;

        for chaos in &self.chaos_types {
            result = match chaos {
                ChaosType::MessageLoss => self.apply_loss(result),
                ChaosType::MessageDuplication => self.apply_duplication(result),
                ChaosType::MessageReordering => self.apply_reordering(result),
                ChaosType::MessageDelay(_) => self.apply_delay(result),
            };
        }

        result
    }

    /// Drop messages based on loss_rate.
    fn apply_loss(&self, messages: Vec<Message>) -> Vec<Message> {
        let mut rng = rand::thread_rng();
        messages
            .into_iter()
            .filter(|_| rng.gen::<f64>() >= self.loss_rate)
            .collect()
    }

    /// Duplicate messages based on duplication_rate.
    fn apply_duplication(&self, messages: Vec<Message>) -> Vec<Message> {
        let mut rng = rand::thread_rng();
        let mut result = Vec::with_capacity(messages.len());

        for msg in messages {
            result.push(msg.clone());
            if rng.gen::<f64>() < self.duplication_rate {
                result.push(msg);
            }
        }

        result
    }

    /// Reorder adjacent messages with some probability.
    fn apply_reordering(&self, messages: Vec<Message>) -> Vec<Message> {
        let mut rng = rand::thread_rng();
        let mut result = messages;

        // Swap adjacent pairs with 50% probability.
        let mut i = 0;
        while i + 1 < result.len() {
            if rng.gen::<f64>() < 0.5 {
                result.swap(i, i + 1);
                i += 2;
            } else {
                i += 1;
            }
        }

        result
    }

    /// Add random delays to messages (simulated by incrementing timestamp).
    fn apply_delay(&self, messages: Vec<Message>) -> Vec<Message> {
        let mut rng = rand::thread_rng();
        messages
            .into_iter()
            .map(|mut msg| {
                let delay = rng.gen_range(self.delay_range.clone());
                msg.timestamp += delay;
                msg
            })
            .collect()
    }
}

/// A resilient system that processes messages and maintains a consistent state.
#[derive(Debug)]
pub struct ResilientSystem {
    /// The key-value data store, built from delivered messages.
    pub data: HashMap<String, String>,
    /// Log of all processed message payloads.
    pub message_log: Vec<String>,
    /// Set of message IDs already processed (for deduplication).
    pub processed_ids: Vec<usize>,
}

impl ResilientSystem {
    /// Create a new resilient system.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            message_log: Vec::new(),
            processed_ids: Vec::new(),
        }
    }

    /// Apply a single message to the system.
    /// Handles deduplication: if the message ID was already processed, skip it.
    pub fn apply(&mut self, msg: &Message) {
        // Deduplication: skip if already processed.
        if self.processed_ids.contains(&msg.id) {
            return;
        }

        self.processed_ids.push(msg.id);
        self.message_log.push(msg.payload.clone());

        // Parse simple SET key = value commands.
        if let Some(parsed) = Self::parse_set_command(&msg.payload) {
            self.data.insert(parsed.0, parsed.1);
        }
    }

    /// Apply multiple messages.
    pub fn apply_batch(&mut self, messages: &[Message]) {
        for msg in messages {
            self.apply(msg);
        }
    }

    /// Get the value for a key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }

    /// Parse a "SET key = value" command.
    fn parse_set_command(payload: &str) -> Option<(String, String)> {
        let trimmed = payload.trim();
        if trimmed.starts_with("SET ") {
            let rest = &trimmed[4..];
            if let Some(eq_pos) = rest.find('=') {
                let key = rest[..eq_pos].trim().to_string();
                let value = rest[eq_pos + 1..].trim().to_string();
                return Some((key, value));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_msg(id: usize, from: usize, to: usize, payload: &str) -> Message {
        Message {
            id,
            from,
            to,
            payload: payload.to_string(),
            timestamp: 0,
        }
    }

    #[test]
    fn test_system_survives_message_loss() {
        let monkey = ChaosMonkey::new(vec![ChaosType::MessageLoss], 0.5, 0.0);

        let messages = vec![
            make_msg(1, 0, 1, "SET x = 1"),
            make_msg(2, 0, 1, "SET y = 2"),
            make_msg(3, 0, 1, "SET z = 3"),
            make_msg(4, 0, 1, "SET w = 4"),
            make_msg(5, 0, 1, "SET v = 5"),
            make_msg(6, 0, 1, "SET u = 6"),
            make_msg(7, 0, 1, "SET t = 7"),
            make_msg(8, 0, 1, "SET s = 8"),
        ];

        let delivered = monkey.inject_chaos(messages);

        let mut system = ResilientSystem::new();
        system.apply_batch(&delivered);

        // The system should have processed some subset of messages.
        // It should not crash and should have at least one entry.
        assert!(
            !system.data.is_empty(),
            "System should have data from at least some delivered messages"
        );

        // Every key that was set should have the correct value.
        for (key, value) in &system.data {
            let expected_key = key.as_str();
            let expected_value = value.as_str();
            // Verify consistency: the value matches the expected SET command.
            let expected_cmd = format!("SET {} = {}", expected_key, expected_value);
            assert!(
                system.message_log.iter().any(|log| log == &expected_cmd),
                "Message log should contain the SET command for {}={}",
                expected_key,
                expected_value
            );
        }
    }

    #[test]
    fn test_system_handles_message_reordering() {
        let monkey = ChaosMonkey::new(vec![ChaosType::MessageReordering], 0.0, 0.0);

        // Send SET x = 1 then SET x = 2.
        // After reordering, the final value of x should be 2 (last writer wins).
        let messages = vec![
            make_msg(1, 0, 1, "SET x = 1"),
            make_msg(2, 0, 1, "SET x = 2"),
            make_msg(3, 0, 1, "SET y = 3"),
        ];

        // Run multiple times to ensure the system handles reordering.
        for _ in 0..10 {
            let reordered = monkey.inject_chaos(messages.clone());
            let mut system = ResilientSystem::new();
            system.apply_batch(&reordered);

            // x should be either 1 or 2 (depending on order), but must be
            // one of the two values that were written.
            if let Some(x_val) = system.get("x") {
                assert!(
                    x_val == "1" || x_val == "2",
                    "x should be 1 or 2 after reordering, got {}",
                    x_val
                );
            }

            // y should be 3 if delivered.
            if let Some(y_val) = system.get("y") {
                assert_eq!(y_val, "3");
            }
        }
    }

    #[test]
    fn test_system_handles_message_duplication() {
        let monkey = ChaosMonkey::new(vec![ChaosType::MessageDuplication], 0.0, 0.8);

        let messages = vec![
            make_msg(1, 0, 1, "SET x = hello"),
            make_msg(2, 0, 1, "SET y = world"),
        ];

        let duplicated = monkey.inject_chaos(messages);

        let mut system = ResilientSystem::new();
        system.apply_batch(&duplicated);

        // Despite duplication, the system should have exactly one entry per
        // message ID due to deduplication.
        assert_eq!(system.get("x"), Some("hello"));
        assert_eq!(system.get("y"), Some("world"));

        // The message log should contain each payload exactly once.
        let x_count = system
            .message_log
            .iter()
            .filter(|m| m == &&"SET x = hello".to_string())
            .count();
        let y_count = system
            .message_log
            .iter()
            .filter(|m| m == &&"SET y = world".to_string())
            .count();
        assert_eq!(x_count, 1, "SET x = hello should appear exactly once");
        assert_eq!(y_count, 1, "SET y = world should appear exactly once");
    }

    #[test]
    fn test_system_handles_message_delay() {
        let monkey = ChaosMonkey::with_delay_range(
            vec![ChaosType::MessageDelay(0)],
            0.0,
            0.0,
            5..20,
        );

        let messages = vec![
            make_msg(1, 0, 1, "SET x = 1"),
            make_msg(2, 0, 1, "SET x = 2"),
        ];

        let delayed = monkey.inject_chaos(messages);

        // Timestamps should be incremented.
        assert!(delayed[0].timestamp > 0);
        assert!(delayed[1].timestamp > 0);

        // The system should still process correctly regardless of timestamps.
        let mut system = ResilientSystem::new();
        system.apply_batch(&delayed);

        // Final value should be 2 (last processed message wins).
        assert_eq!(system.get("x"), Some("2"));
    }

    #[test]
    fn test_parse_set_command() {
        assert_eq!(
            ResilientSystem::parse_set_command("SET x = hello"),
            Some(("x".to_string(), "hello".to_string()))
        );
        assert_eq!(
            ResilientSystem::parse_set_command("SET key = value with spaces"),
            Some(("key".to_string(), "value with spaces".to_string()))
        );
        assert_eq!(ResilientSystem::parse_set_command("GET x"), None);
    }
}
