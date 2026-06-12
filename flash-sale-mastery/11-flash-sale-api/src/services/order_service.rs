//! # Order Service
//!
//! Queues order creation asynchronously. In the real system, orders are written
//! to a Redis Stream and consumed by the order worker (Module 12). Here we use
//! an in-memory queue for simplicity.
//!
//! ## Exercise
//!
//! 1. Implement `queue_order` to push an order into the async channel.
//! 2. Implement `process_pending_orders` to drain the channel and log processing.
//! 3. Write tests for queuing and processing.

use tokio::sync::mpsc;

/// An order waiting to be persisted.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PendingOrder {
    pub order_id: String,
    pub product_id: String,
    pub account_id: String,
    pub voucher_code: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Errors from the order service.
#[derive(Debug, thiserror::Error)]
pub enum OrderServiceError {
    #[error("Order queue is full")]
    QueueFull,

    #[error("Order queue is closed")]
    QueueClosed,
}

/// Async order queue backed by a Tokio MPSC channel.
#[derive(Clone)]
pub struct OrderQueue {
    tx: mpsc::Sender<PendingOrder>,
}

/// The receiving side of the order queue (consumed by the worker).
pub struct OrderReceiver {
    rx: mpsc::Receiver<PendingOrder>,
}

/// Create an order queue with the given buffer capacity.
pub fn create_order_queue(buffer: usize) -> (OrderQueue, OrderReceiver) {
    let (tx, rx) = mpsc::channel(buffer);
    (OrderQueue { tx }, OrderReceiver { rx })
}

impl OrderQueue {
    /// Enqueue an order for async processing.
    #[cfg(feature = "solution")]
    pub async fn queue_order(&self, order: PendingOrder) -> Result<(), OrderServiceError> {
        self.tx
            .send(order)
            .await
            .map_err(|_| OrderServiceError::QueueClosed)
    }

    #[cfg(not(feature = "solution"))]
    pub async fn queue_order(&self, order: PendingOrder) -> Result<(), OrderServiceError> {
        todo!("Send the order through the MPSC channel")
    }
}

impl OrderReceiver {
    /// Process all currently pending orders (non-blocking drain).
    #[cfg(feature = "solution")]
    pub async fn process_pending_orders(&mut self) -> Vec<PendingOrder> {
        let mut processed = Vec::new();
        while let Ok(order) = self.rx.try_recv() {
            tracing::info!(
                order_id = %order.order_id,
                product_id = %order.product_id,
                account_id = %order.account_id,
                "Processed order"
            );
            processed.push(order);
        }
        processed
    }

    #[cfg(not(feature = "solution"))]
    pub async fn process_pending_orders(&mut self) -> Vec<PendingOrder> {
        todo!("Drain the channel and collect pending orders")
    }

    /// Wait for and process a single order.
    #[cfg(feature = "solution")]
    pub async fn recv_order(&mut self) -> Option<PendingOrder> {
        self.rx.recv().await
    }

    #[cfg(not(feature = "solution"))]
    pub async fn recv_order(&mut self) -> Option<PendingOrder> {
        todo!("Receive a single order from the channel")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_order_queuing() {
        let (queue, mut rx) = create_order_queue(100);

        let order = PendingOrder {
            order_id: uuid::Uuid::new_v4().to_string(),
            product_id: "prod-1".to_string(),
            account_id: "acct-1".to_string(),
            voucher_code: "VS-TEST-123".to_string(),
            created_at: chrono::Utc::now(),
        };

        queue.queue_order(order.clone()).await.unwrap();

        let processed = rx.process_pending_orders().await;
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].order_id, order.order_id);
    }

    #[tokio::test]
    async fn test_order_processing_drains_queue() {
        let (queue, mut rx) = create_order_queue(100);

        for i in 0..5 {
            let order = PendingOrder {
                order_id: format!("order-{i}"),
                product_id: "prod-1".to_string(),
                account_id: format!("acct-{i}"),
                voucher_code: format!("VS-{i}"),
                created_at: chrono::Utc::now(),
            };
            queue.queue_order(order).await.unwrap();
        }

        let processed = rx.process_pending_orders().await;
        assert_eq!(processed.len(), 5);

        // Queue should now be empty
        let empty = rx.process_pending_orders().await;
        assert!(empty.is_empty());
    }

    #[tokio::test]
    async fn test_recv_single_order() {
        let (queue, mut rx) = create_order_queue(100);

        let order = PendingOrder {
            order_id: "single-order".to_string(),
            product_id: "prod-1".to_string(),
            account_id: "acct-1".to_string(),
            voucher_code: "VS-SINGLE".to_string(),
            created_at: chrono::Utc::now(),
        };

        queue.queue_order(order).await.unwrap();
        let received = rx.recv_order().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().order_id, "single-order");
    }
}
