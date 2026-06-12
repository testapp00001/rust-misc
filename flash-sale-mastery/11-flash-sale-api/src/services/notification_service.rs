//! # Notification Service
//!
//! Best-effort notifications for purchase confirmations and sold-out alerts.
//! In production these would be sent via email/SMS/push; here we log them.
//!
//! ## Exercise
//!
//! 1. Implement `send_purchase_notification` that logs the notification.
//! 2. Implement `send_sold_out_notification` that logs the alert.
//! 3. Ensure notifications never block the hot path (fire-and-forget).
//! 4. Write tests verifying that notifications are dispatched without errors.

/// Errors from the notification service (all are non-fatal).
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("Notification delivery failed: {0}")]
    DeliveryFailed(String),
}

/// Best-effort notification service.
#[derive(Clone)]
pub struct NotificationService {
    /// In production, this would hold an HTTP client, email client, etc.
    enabled: bool,
}

impl NotificationService {
    /// Create a new notification service.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Send a purchase confirmation notification.
    ///
    /// This is fire-and-forget: failures are logged but never propagated.
    #[cfg(feature = "solution")]
    pub async fn send_purchase_notification(
        &self,
        account_id: &str,
        voucher_code: &str,
    ) -> Result<(), NotificationError> {
        if !self.enabled {
            return Ok(());
        }

        // In production: send email/SMS/push notification.
        tracing::info!(
            account_id = account_id,
            voucher_code = voucher_code,
            "Sending purchase confirmation notification"
        );

        // Simulate async delivery
        tokio::task::yield_now().await;

        tracing::debug!(
            account_id = account_id,
            "Purchase notification dispatched"
        );

        Ok(())
    }

    #[cfg(not(feature = "solution"))]
    pub async fn send_purchase_notification(
        &self,
        account_id: &str,
        voucher_code: &str,
    ) -> Result<(), NotificationError> {
        todo!("Log/send a purchase confirmation notification")
    }

    /// Send a sold-out alert (typically to operations team).
    #[cfg(feature = "solution")]
    pub async fn send_sold_out_notification(
        &self,
        product_id: &str,
    ) -> Result<(), NotificationError> {
        if !self.enabled {
            return Ok(());
        }

        tracing::warn!(
            product_id = product_id,
            "Sending sold-out alert notification"
        );

        // In production: alert ops team via PagerDuty/Slack/etc.
        tokio::task::yield_now().await;

        tracing::debug!(
            product_id = product_id,
            "Sold-out notification dispatched"
        );

        Ok(())
    }

    #[cfg(not(feature = "solution"))]
    pub async fn send_sold_out_notification(
        &self,
        product_id: &str,
    ) -> Result<(), NotificationError> {
        todo!("Log/send a sold-out alert notification")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_purchase_notification_succeeds() {
        let svc = NotificationService::new(true);
        let result = svc
            .send_purchase_notification("acct-1", "VS-TEST-123")
            .await;
        assert!(result.is_ok(), "Notification should succeed");
    }

    #[tokio::test]
    async fn test_sold_out_notification_succeeds() {
        let svc = NotificationService::new(true);
        let result = svc.send_sold_out_notification("prod-1").await;
        assert!(result.is_ok(), "Sold-out notification should succeed");
    }

    #[tokio::test]
    async fn test_disabled_service_skips() {
        let svc = NotificationService::new(false);
        let result = svc
            .send_purchase_notification("acct-1", "VS-TEST-123")
            .await;
        assert!(result.is_ok(), "Disabled service should still return Ok");
    }
}
