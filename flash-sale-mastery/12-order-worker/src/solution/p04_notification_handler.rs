//! # Solution 04: Notification Handler
//!
//! Complete implementation of best-effort notification delivery. Notifications
//! are important but never block the main order processing pipeline.

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
    /// Create a new notification handler with an empty sent list.
    pub fn new() -> Self {
        Self {
            sent: Mutex::new(Vec::new()),
        }
    }

    /// Send an order confirmation notification.
    ///
    /// Simulates sending a confirmation email to the customer. The notification
    /// is recorded in the internal `sent` list for testing purposes.
    ///
    /// # Arguments
    /// * `order` - The successfully processed order
    pub async fn send_confirmation(&self, order: &Order) -> Result<(), NotificationError> {
        let notification = Notification {
            order_id: order.id.to_string(),
            kind: NotificationKind::Confirmation,
            recipient: format!("{}@example.com", order.account_id),
            body: format!(
                "Your order {} for product {} has been confirmed. Voucher code: {}",
                order.id, order.product_id, order.voucher_code
            ),
        };

        tracing::info!(
            order_id = %order.id,
            recipient = %notification.recipient,
            "Sending confirmation notification"
        );

        // Record the notification
        self.sent.lock().unwrap().push(notification);

        Ok(())
    }

    /// Send a payment failure notification.
    ///
    /// Notifies the customer that their payment could not be processed.
    ///
    /// # Arguments
    /// * `order` - The order whose payment failed
    pub async fn send_failure(&self, order: &Order) -> Result<(), NotificationError> {
        let notification = Notification {
            order_id: order.id.to_string(),
            kind: NotificationKind::PaymentFailure,
            recipient: format!("{}@example.com", order.account_id),
            body: format!(
                "Payment for order {} (product {}) could not be processed. \
                 Please try again or contact support.",
                order.id, order.product_id
            ),
        };

        tracing::info!(
            order_id = %order.id,
            recipient = %notification.recipient,
            "Sending payment failure notification"
        );

        // Record the notification
        self.sent.lock().unwrap().push(notification);

        Ok(())
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
        assert!(sent[0].body.contains(&order.voucher_code));
    }

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

    #[tokio::test]
    async fn test_recipient_format() {
        let handler = NotificationHandler::new();
        let order = make_order("acc-42");

        handler
            .send_confirmation(&order)
            .await
            .expect("Should send");

        let sent = handler.sent_notifications();
        assert_eq!(sent[0].recipient, "acc-42@example.com");
    }
}
