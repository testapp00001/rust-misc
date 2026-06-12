//! # Exercise 07: Backpressure Channel
//!
//! ## Learning Objective
//! Understand how backpressure propagates through a system using Tokio's
//! bounded channels. Learn the difference between `try_send` (non-blocking
//! reject), `send` (blocking wait), and `send` with timeout (bounded wait).
//!
//! ## Flash Sale Context
//! When the admission controller admits a request, it needs to forward it to
//! the order processing pipeline. If the pipeline is slow, the channel between
//! them creates backpressure: the sender either waits, times out, or drops
//! the request. This prevents unbounded queue growth that would exhaust memory.
//!
//! ## Instructions
//! 1. Implement `create_backpressure_channel(capacity)` -- create a bounded
//!    Tokio mpsc channel
//! 2. Implement `try_send_request()` -- attempt to send without blocking,
//!    returning an error if the channel is full
//! 3. Implement `send_with_timeout()` -- attempt to send with a deadline,
//!    returning an error if the timeout expires
//! 4. Demonstrate backpressure propagation through a multi-stage pipeline
//!
//! ## Hints
//! - `tokio::sync::mpsc::channel(capacity)` creates a bounded channel
//! - `try_send()` returns `Err(TrySendError::Full)` when the channel is full
//! - `tokio::time::timeout()` wraps a future with a deadline
//! - Backpressure propagates when receivers are slow

use tokio::sync::mpsc;
use std::time::Duration;

/// A request in the flash sale pipeline.
#[derive(Debug, Clone)]
pub struct PipelineRequest {
    pub id: u64,
    pub account_id: String,
    pub product_id: String,
}

/// Result of a send attempt.
#[derive(Debug, PartialEq)]
pub enum SendResult {
    /// Request was sent successfully.
    Sent,
    /// Channel is full, request was rejected.
    ChannelFull,
    /// Send timed out.
    TimedOut,
}

/// Create a bounded backpressure channel.
///
/// Returns a (sender, receiver) pair with the specified capacity.
pub fn create_backpressure_channel(
    capacity: usize,
) -> (mpsc::Sender<PipelineRequest>, mpsc::Receiver<PipelineRequest>) {
    todo!("Create a bounded Tokio mpsc channel")
}

/// Attempt to send a request without blocking.
///
/// Returns `SendResult::Sent` on success, `SendResult::ChannelFull` if
/// the channel is at capacity.
pub async fn try_send_request(
    tx: &mpsc::Sender<PipelineRequest>,
    request: PipelineRequest,
) -> SendResult {
    todo!("Implement non-blocking send using try_send")
}

/// Attempt to send a request with a timeout.
///
/// Waits up to `timeout_duration` for space in the channel. Returns
/// `SendResult::Sent` on success, `SendResult::TimedOut` if the deadline
/// expires.
pub async fn send_with_timeout(
    tx: &mpsc::Sender<PipelineRequest>,
    request: PipelineRequest,
    timeout_duration: Duration,
) -> SendResult {
    todo!("Implement timeout-bounded send")
}

/// Demonstrate backpressure propagation through a multi-stage pipeline.
///
/// Creates a pipeline: stage1 -> stage2 -> sink
/// Each stage has a bounded channel. Returns the number of requests that
/// were successfully processed and the number that were rejected.
pub async fn demonstrate_backpressure(
    total_requests: usize,
    stage_capacity: usize,
    processing_delay: Duration,
) -> (usize, usize) {
    todo!("Implement multi-stage backpressure demonstration")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_channel_full_behavior() {
        let (tx, mut rx) = create_backpressure_channel(2);

        // Fill the channel
        let r1 = try_send_request(&tx, PipelineRequest {
            id: 1, account_id: "a".into(), product_id: "p".into(),
        }).await;
        assert_eq!(r1, SendResult::Sent);

        let r2 = try_send_request(&tx, PipelineRequest {
            id: 2, account_id: "a".into(), product_id: "p".into(),
        }).await;
        assert_eq!(r2, SendResult::Sent);

        // Channel should be full now
        let r3 = try_send_request(&tx, PipelineRequest {
            id: 3, account_id: "a".into(), product_id: "p".into(),
        }).await;
        assert_eq!(r3, SendResult::ChannelFull);

        // Drain one to verify channel works again
        rx.recv().await;
        let r4 = try_send_request(&tx, PipelineRequest {
            id: 4, account_id: "a".into(), product_id: "p".into(),
        }).await;
        assert_eq!(r4, SendResult::Sent);
    }

    #[tokio::test]
    async fn test_send_with_timeout_success() {
        let (tx, _rx) = create_backpressure_channel(2);
        let result = send_with_timeout(
            &tx,
            PipelineRequest { id: 1, account_id: "a".into(), product_id: "p".into() },
            Duration::from_millis(100),
        ).await;
        assert_eq!(result, SendResult::Sent);
    }

    #[tokio::test]
    async fn test_send_with_timeout_expired() {
        let (tx, _rx) = create_backpressure_channel(1);
        // Fill the channel
        tx.send(PipelineRequest { id: 1, account_id: "a".into(), product_id: "p".into() })
            .await
            .unwrap();

        // This should time out since the channel is full
        let result = send_with_timeout(
            &tx,
            PipelineRequest { id: 2, account_id: "a".into(), product_id: "p".into() },
            Duration::from_millis(50),
        ).await;
        assert_eq!(result, SendResult::TimedOut);
    }

    #[tokio::test]
    async fn test_graceful_degradation() {
        let (processed, rejected) = demonstrate_backpressure(
            20,     // total requests
            5,      // stage capacity
            Duration::from_millis(10), // processing delay
        ).await;
        // Some should be processed, some rejected due to backpressure
        assert!(processed > 0, "Some requests should be processed");
        assert!(
            processed + rejected <= 20,
            "Total should not exceed input"
        );
    }
}
