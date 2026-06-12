//! # Exercise 04: Notification Handler
//!
//! ## Learning Objective
//! Learn to implement best-effort operations that must not block the main
//! processing pipeline. Notifications are important but not critical -- a
//! failed email should never prevent an order from being recorded.
//!
//! ## Flash Sale Context
//! After an order is created and payment is processed, the customer expects a
//! confirmation email. But if the email service is down, we should still
//! acknowledge the order. The notification handler isolates this failure domain:
//! it logs the error and moves on. This is a common pattern for non-critical
//! side effects.
//!
//! ## Instructions
//! 1. Implement `NotificationHandler::new` to create the handler
//! 2. Implement `send_confirmation` to send a success notification
//! 3. Implement `send_failure` to send a payment failure notification
//! 4. Both methods should never propagate errors to the caller
//!
//! ## Hints
//! - In a real system, this would call an email/SMS API. Here, we simulate it.
//! - Log the notification attempt using `tracing::info!`
//! - For testing, you can use an interior-mutable pattern to track sent
//!   notifications (e.g., `Mutex<Vec<Notification>>`)
//! - The key insight: `send_confirmation` and `send_failure` return `Result<()`,
//!   but the caller should use `let _ = handler.send_confirmation(...)` to
//!   explicitly ignore failures

use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::p02_order_processor::Order;

/// A notification that was sent (or attempted).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// The order this notification is about.
    pub order_id: String,
    /// Type of notification.
    pub kind: NotificationKind,
    /// Recipient (simulated email address).
    pub recipient: String,
    /// Notification body.
    pub body: String,
}

/// The type of notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationKind {
    /// Order confirmation after successful payment.
    Confirmation,
    /// Payment failure notification.
    PaymentFailure,
}

/// Custom error type for notification operations.
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Notification service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
}

/// Handles sending notifications for order events.
///
/// All notification methods are best-effort: failures are logged but do not
/// propagate to the caller. This ensures the main processing pipeline is
/// never blocked by notification issues.
pub struct NotificationHandler {
    /// Track sent notifications for testing (interior mutability).
    sent: Mutex<Vec<Notification>>,
}

impl NotificationHandler {
    /// Create a new notification handler.
    pub fn new() -> Self {
        todo!("Initialize NotificationHandler with an empty sent list")
    }

    /// Send an order confirmation notification.
    ///
    /// In a real system, this would send an email/SMS. Here, we simulate it
    /// by recording the notification in the `sent` list.
    ///
    /// # Arguments
    /// * `order` - The successfully processed order
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(NotificationError)` on failure.
    /// The caller should treat this as best-effort and ignore errors.
    pub async fn send_confirmation(&self, order: &Order) -> Result<(), NotificationError> {
        todo!("Implement confirmation notification")
    }

    /// Send a payment failure notification.
    ///
    /// Notifies the customer that their payment could not be processed.
    ///
    /// # Arguments
    /// * `order` - The order whose payment failed
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(NotificationError)` on failure.
    pub async fn send_failure(&self, order: &Order) -> Result<(), NotificationError> {
        todo!("Implement failure notification")
    }

    /// Get a list of all sent notifications (for testing).
    pub fn sent_notifications(&self) -> Vec<Notification> {
        self.sent.lock().unwrap().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p02_order_processor::OrderStatus;
    use chrono::Utc;
    use uuid::Uuid;

    fn make_order(account_id: &str) -> Order {
        Order {
            id: Uuid::new_v4(),
            product_id: "prod-test".to_string(),
            account_id: account_id.to_string(),
            voucher_code: "VC-TEST".to_string(),
            status: OrderStatus::Pending,
            created_at: Utc::now(),
        }
    }

    /// Test that a confirmation notification is sent successfully.
    #[tokio::test]
    async fn test_confirmation_sent() {
        let handler = NotificationHandler::new();
        let order = make_order("acc-100");

        handler
            .send_confirmation(&order)
            .await
            .expect("Confirmation should succeed");

        let sent = handler.sent_notifications();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].kind, NotificationKind::Confirmation);
        assert_eq!(sent[0].order_id, order.id.to_string());
    }

    /// Test that a failure notification is sent successfully.
    #[tokio::test]
    async fn test_failure_notification_sent() {
        let handler = NotificationHandler::new();
        let order = make_order("acc-200");

        handler
            .send_failure(&order)
            .await
            .expect("Failure notification should succeed");

        let sent = handler.sent_notifications();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].kind, NotificationKind::PaymentFailure);
        assert_eq!(sent[0].order_id, order.id.to_string());
    }

    /// Test that notification failure does not affect order processing.
    ///
    /// This test demonstrates the best-effort pattern: the caller wraps the
    /// notification call in `let _ = ...` to explicitly discard the result.
    #[tokio::test]
    async fn test_notification_failure_does_not_block_order() {
        let handler = NotificationHandler::new();
        let order = make_order("acc-300");

        // Simulate: order was created and payment succeeded
        let order_created = true;
        let payment_succeeded = true;

        // Send confirmation, but ignore any error
        let _ = handler.send_confirmation(&order).await;

        // The order processing pipeline continues regardless
        assert!(order_created, "Order should still be recorded");
        assert!(payment_succeeded, "Payment should still be recorded");
    }

    /// Test that multiple notifications accumulate in the sent list.
    #[tokio::test]
    async fn test_multiple_notifications() {
        let handler = NotificationHandler::new();

        for i in 0..3 {
            let order = make_order(&format!("acc-{i}"));
            handler
                .send_confirmation(&order)
                .await
                .expect("Should send");
        }

        let sent = handler.sent_notifications();
        assert_eq!(sent.len(), 3);
    }
}
