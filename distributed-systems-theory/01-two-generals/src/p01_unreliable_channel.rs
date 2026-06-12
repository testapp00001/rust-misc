//! # Exercise: Unreliable Channel
//!
//! ## Theory
//!
//! In distributed systems, network communication is inherently unreliable. Messages
//! can be lost, duplicated, reordered, or delayed. An unreliable channel models the
//! fundamental constraint that any message sent may fail to arrive at its destination.
//!
//! The loss rate of a channel is the probability that any single message is dropped.
//! A loss rate of 0.0 means perfect delivery; 1.0 means all messages are lost.
//! Real networks typically have loss rates between 0.001 and 0.05 under normal
//! conditions, but can spike under congestion.
//!
//! ## Proof / Intuition
//!
//! The key insight is that **no amount of protocol sophistication can overcome an
//! unreliable channel** -- this is the foundation of the Two Generals' impossibility
//! result. The unreliable channel is the reason the last message in any finite
//! protocol can always be lost.
//!
//! In practice, we approximate reliability through retries, acknowledgements, and
//! idempotency, but never achieve true guaranteed delivery over an unreliable medium.
//!
//! ## Implementation Task
//!
//! Implement `UnreliableChannel<T>` with:
//! - A configurable loss rate (0.0 to 1.0)
//! - `send(&self, msg: T) -> Result<(), SendError>` that randomly drops messages
//! - `recv(&mut self) -> Option<T>` that returns the next undropped message
//!
//! ## Verification
//!
//! Run the tests below to verify:
//! - Loss rate approximately matches the configured rate
//! - 0% loss rate delivers all messages
//! - 100% loss rate delivers no messages
//! - Channel can be created with any message type

use std::sync::{Arc, Mutex};

use rand::Rng;

/// Error type for failed sends (message was dropped by the channel).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendError;

/// An unreliable channel that randomly drops messages based on a configured loss rate.
///
/// Messages that survive the loss filter are queued for the receiver.
/// Thread-safe via `Arc<Mutex<>>` interior mutability on the internal buffer.
pub struct UnreliableChannel<T> {
    loss_rate: f64,
    buffer: Arc<Mutex<Vec<T>>>,
}

impl<T> UnreliableChannel<T> {
    /// Create a new unreliable channel with the given loss rate.
    ///
    /// # Arguments
    /// * `loss_rate` - Probability of dropping a message (0.0 = never drop, 1.0 = always drop)
    ///
    /// # Panics
    /// Panics if `loss_rate` is not in [0.0, 1.0].
    pub fn new(loss_rate: f64) -> Self {
        assert!(
            (0.0..=1.0).contains(&loss_rate),
            "loss_rate must be between 0.0 and 1.0, got {loss_rate}"
        );
        Self {
            loss_rate,
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Attempt to send a message through the channel.
    ///
    /// The message is randomly dropped based on the configured loss rate.
    /// Returns `Err(SendError)` if the message was dropped, `Ok(())` otherwise.
    pub fn send(&self, msg: T) -> Result<(), SendError> {
        let mut rng = rand::thread_rng();
        let roll: f64 = rng.gen();
        if roll < self.loss_rate {
            // Message is dropped
            Err(SendError)
        } else {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.push(msg);
            Ok(())
        }
    }

    /// Receive the next message from the channel, if any.
    ///
    /// Returns `None` if no messages are available.
    pub fn recv(&mut self) -> Option<T> {
        let mut buffer = self.buffer.lock().unwrap();
        if buffer.is_empty() {
            None
        } else {
            Some(buffer.remove(0))
        }
    }

    /// Return the number of messages currently in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    /// Return true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.lock().unwrap().is_empty()
    }
}

impl<T> Clone for UnreliableChannel<T> {
    fn clone(&self) -> Self {
        Self {
            loss_rate: self.loss_rate,
            buffer: Arc::clone(&self.buffer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_loss_delivers_all() {
        let channel = UnreliableChannel::new(0.0);
        let total = 1000;
        let mut sent = 0;
        let mut received = 0;

        for i in 0..total {
            if channel.send(i).is_ok() {
                sent += 1;
            }
        }

        let mut rx = channel;
        while rx.recv().is_some() {
            received += 1;
        }

        assert_eq!(sent, total, "all messages should be sent");
        assert_eq!(received, total, "all messages should be received");
    }

    #[test]
    fn hundred_percent_loss_delivers_none() {
        let channel = UnreliableChannel::new(1.0);
        let total = 1000;

        for i in 0..total {
            assert!(channel.send(i).is_err(), "all sends should fail");
        }

        let mut rx = channel;
        assert!(rx.recv().is_none(), "no messages should be available");
    }

    #[test]
    fn loss_rate_approximately_matches_config() {
        let loss_rate = 0.3;
        let channel = UnreliableChannel::new(loss_rate);
        let total = 10_000;
        let mut failures = 0;

        for i in 0..total {
            if channel.send(i).is_err() {
                failures += 1;
            }
        }

        let actual_loss_rate = failures as f64 / total as f64;
        let tolerance = 0.05;
        assert!(
            (actual_loss_rate - loss_rate).abs() < tolerance,
            "actual loss rate {actual_loss_rate:.4} should be within {tolerance} of configured {loss_rate}"
        );
    }

    #[test]
    fn channel_works_with_string_type() {
        let channel = UnreliableChannel::<String>::new(0.0);
        channel
            .send("hello".to_string())
            .unwrap();
        channel
            .send("world".to_string())
            .unwrap();

        let mut rx = channel;
        assert_eq!(rx.recv(), Some("hello".to_string()));
        assert_eq!(rx.recv(), Some("world".to_string()));
        assert!(rx.recv().is_none());
    }

    #[test]
    fn cloned_channels_share_buffer() {
        let ch1 = UnreliableChannel::new(0.0);
        let mut ch2 = ch1.clone();

        ch1.send(42).unwrap();
        ch1.send(43).unwrap();

        assert_eq!(ch2.len(), 2);
        assert_eq!(ch2.recv(), Some(42));
    }
}
