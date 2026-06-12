//! # Exercise 03: Payment Handler with Circuit Breaker
//!
//! ## Learning Objective
//! Learn to integrate with an unreliable downstream service using the circuit
//! breaker pattern. The circuit breaker prevents cascading failures by
//! fast-failing when the downstream service is unhealthy.
//!
//! ## Flash Sale Context
//! After an order is created, the worker must charge the customer. An external
//! payment service handles the actual transaction. Under flash sale load, this
//! service may become overwhelmed. Rather than queuing thousands of requests
//! that will all time out, the circuit breaker detects repeated failures and
//! stops sending requests for a cooldown period.
//!
//! ## Instructions
//! 1. Implement `PaymentHandler::new` to create the handler
//! 2. Implement `process_payment` to simulate payment processing
//! 3. Implement the circuit breaker with three states: Closed, Open, HalfOpen
//! 4. Simulate payment outcomes deterministically based on order properties
//!
//! ## Hints
//! - The circuit breaker has a failure threshold (e.g., 5 failures) that trips
//!   it from Closed to Open
//! - In Open state, reject all requests immediately for a cooldown period
//! - In HalfOpen state, allow one trial request. If it succeeds, close the
//!   circuit; if it fails, re-open it
//! - Use `std::time::Instant` to track when the circuit opened
//! - For simulation: orders with account_id ending in "999" fail, those ending
//!   in "888" time out, and all others succeed

use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::p02_order_processor::Order;

/// Result of a payment attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaymentResult {
    /// Payment completed successfully.
    Success,
    /// Payment was declined.
    Declined,
    /// Payment service timed out.
    Timeout,
}

/// Circuit breaker states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation -- requests flow through.
    Closed,
    /// Too many failures -- requests are rejected immediately.
    Open,
    /// Cooldown expired -- one trial request is allowed.
    HalfOpen,
}

/// Custom error type for payment operations.
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Payment declined for order {order_id}")]
    Declined { order_id: String },

    #[error("Payment service timeout for order {order_id}")]
    Timeout { order_id: String },

    #[error("Circuit breaker is open -- payment service unavailable")]
    CircuitOpen,
}

/// Handles payment processing with a built-in circuit breaker.
pub struct PaymentHandler {
    /// Circuit breaker state (protected by Mutex for thread safety).
    circuit: Mutex<CircuitBreaker>,
}

struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    cooldown: Duration,
    opened_at: Option<Instant>,
}

impl PaymentHandler {
    /// Create a new payment handler with the circuit breaker in Closed state.
    ///
    /// The circuit breaker trips after 5 consecutive failures and has a
    /// 30-second cooldown period.
    pub fn new() -> Self {
        todo!("Initialize PaymentHandler with circuit breaker in Closed state")
    }

    /// Process a payment for an order.
    ///
    /// This simulates an external payment service:
    /// - Orders with `account_id` ending in "999" => `Declined`
    /// - Orders with `account_id` ending in "888" => `Timeout`
    /// - All other orders => `Success`
    ///
    /// The circuit breaker wraps this logic:
    /// - **Closed**: Process normally. On failure, increment failure count.
    ///   If threshold reached, trip to Open.
    /// - **Open**: Reject immediately with `CircuitOpen`. After cooldown,
    ///   transition to HalfOpen.
    /// - **HalfOpen**: Allow one trial request. Success => Closed.
    ///   Failure => Open.
    ///
    /// # Arguments
    /// * `order` - The order to charge
    ///
    /// # Returns
    /// The `PaymentResult` on success, or a `PaymentError`.
    pub async fn process_payment(&self, order: &Order) -> Result<PaymentResult, PaymentError> {
        todo!("Implement payment processing with circuit breaker logic")
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

    /// Test successful payment processing.
    #[tokio::test]
    async fn test_payment_success() {
        let handler = PaymentHandler::new();
        let order = make_order("acc-100");

        let result = handler
            .process_payment(&order)
            .await
            .expect("Payment should succeed");
        assert_eq!(result, PaymentResult::Success);
    }

    /// Test payment decline.
    #[tokio::test]
    async fn test_payment_declined() {
        let handler = PaymentHandler::new();
        let order = make_order("acc-999");

        let result = handler.process_payment(&order).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PaymentError::Declined { order_id } => {
                assert_eq!(order_id, order.id.to_string());
            }
            other => panic!("Expected Declined, got: {other}"),
        }
    }

    /// Test payment timeout.
    #[tokio::test]
    async fn test_payment_timeout() {
        let handler = PaymentHandler::new();
        let order = make_order("acc-888");

        let result = handler.process_payment(&order).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PaymentError::Timeout { .. } => {} // expected
            other => panic!("Expected Timeout, got: {other}"),
        }
    }

    /// Test circuit breaker trips after repeated failures.
    #[tokio::test]
    async fn test_circuit_breaker_trips() {
        let handler = PaymentHandler::new();

        // Cause 5 failures to trip the circuit breaker
        for _ in 0..5 {
            let order = make_order("acc-999");
            let _ = handler.process_payment(&order).await;
        }

        // Next request should be rejected immediately (circuit open)
        let order = make_order("acc-100"); // would normally succeed
        let result = handler.process_payment(&order).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PaymentError::CircuitOpen => {} // expected
            other => panic!("Expected CircuitOpen, got: {other}"),
        }
    }

    /// Test that the circuit breaker recovers after cooldown.
    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let handler = PaymentHandler::new();

        // Trip the circuit
        for _ in 0..5 {
            let order = make_order("acc-888");
            let _ = handler.process_payment(&order).await;
        }

        // Verify circuit is open
        let order = make_order("acc-100");
        let result = handler.process_payment(&order).await;
        assert!(matches!(result, Err(PaymentError::CircuitOpen)));

        // Wait for cooldown (30 seconds is long, so this test may be slow)
        // In a real implementation, the cooldown would be injectable for testing.
        // For now, this test verifies the circuit is open.
        // A production test would mock time or use a short cooldown.
    }
}
