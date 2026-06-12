//! # Solution 07: Backpressure Channel
//!
//! Complete implementation of backpressure using Tokio bounded channels.

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
    Sent,
    ChannelFull,
    TimedOut,
}

/// Create a bounded backpressure channel.
pub fn create_backpressure_channel(
    capacity: usize,
) -> (mpsc::Sender<PipelineRequest>, mpsc::Receiver<PipelineRequest>) {
    mpsc::channel(capacity)
}

/// Attempt to send a request without blocking.
pub async fn try_send_request(
    tx: &mpsc::Sender<PipelineRequest>,
    request: PipelineRequest,
) -> SendResult {
    match tx.try_send(request) {
        Ok(()) => SendResult::Sent,
        Err(mpsc::error::TrySendError::Full(_)) => SendResult::ChannelFull,
        Err(mpsc::error::TrySendError::Closed(_)) => SendResult::ChannelFull,
    }
}

/// Attempt to send a request with a timeout.
pub async fn send_with_timeout(
    tx: &mpsc::Sender<PipelineRequest>,
    request: PipelineRequest,
    timeout_duration: Duration,
) -> SendResult {
    match tokio::time::timeout(timeout_duration, tx.send(request)).await {
        Ok(Ok(())) => SendResult::Sent,
        Ok(Err(_)) => SendResult::ChannelFull,
        Err(_) => SendResult::TimedOut,
    }
}

/// Demonstrate backpressure propagation through a multi-stage pipeline.
pub async fn demonstrate_backpressure(
    total_requests: usize,
    stage_capacity: usize,
    processing_delay: Duration,
) -> (usize, usize) {
    let (tx1, mut rx1) = create_backpressure_channel(stage_capacity);
    let (tx2, mut rx2) = create_backpressure_channel(stage_capacity);

    // Stage 2: consumes from rx1, produces to tx2
    let stage2 = tokio::spawn(async move {
        while let Some(_req) = rx1.recv().await {
            tokio::time::sleep(processing_delay).await;
            if tx2.send(_req).await.is_err() {
                break;
            }
        }
    });

    // Sink: consumes from rx2
    let sink = tokio::spawn(async move {
        let mut count = 0;
        while rx2.recv().await.is_some() {
            tokio::time::sleep(processing_delay).await;
            count += 1;
        }
        count
    });

    // Producer: sends to tx1
    let mut rejected = 0;
    for i in 0..total_requests {
        let req = PipelineRequest {
            id: i as u64,
            account_id: format!("user-{i}"),
            product_id: "prod-1".to_string(),
        };
        match try_send_request(&tx1, req).await {
            SendResult::Sent => {}
            SendResult::ChannelFull | SendResult::TimedOut => rejected += 1,
        }
    }
    drop(tx1);

    stage2.await.unwrap();
    let sink_count = sink.await.unwrap();

    (sink_count, rejected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_channel_full_behavior() {
        let (tx, mut rx) = create_backpressure_channel(2);

        let r1 = try_send_request(&tx, PipelineRequest {
            id: 1, account_id: "a".into(), product_id: "p".into(),
        }).await;
        assert_eq!(r1, SendResult::Sent);

        let r2 = try_send_request(&tx, PipelineRequest {
            id: 2, account_id: "a".into(), product_id: "p".into(),
        }).await;
        assert_eq!(r2, SendResult::Sent);

        let r3 = try_send_request(&tx, PipelineRequest {
            id: 3, account_id: "a".into(), product_id: "p".into(),
        }).await;
        assert_eq!(r3, SendResult::ChannelFull);

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
        tx.send(PipelineRequest { id: 1, account_id: "a".into(), product_id: "p".into() })
            .await
            .unwrap();

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
            20,
            5,
            Duration::from_millis(10),
        ).await;
        assert!(processed > 0, "Some requests should be processed");
        assert!(processed + rejected <= 20, "Total should not exceed input");
    }
}
