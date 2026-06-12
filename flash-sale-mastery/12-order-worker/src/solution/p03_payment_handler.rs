//! # Solution 03: Payment Handler with Circuit Breaker
//!
//! Complete implementation of mock payment processing with a circuit breaker
//! pattern that protects against cascading failures.

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

/// Internal circuit breaker state.
struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    cooldown: Duration,
    opened_at: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, cooldown: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            cooldown,
            opened_at: None,
        }
    }

    /// Check if a request is allowed through the circuit breaker.
    ///
    /// Returns `Ok(())` if the request should proceed, `Err(())` if rejected.
    /// Transitions from Open to HalfOpen after the cooldown period.
    fn check(&mut self) -> Result<(), ()> {
        match self.state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                // Check if cooldown has elapsed
                if let Some(opened) = self.opened_at {
                    if opened.elapsed() >= self.cooldown {
                        self.state = CircuitState::HalfOpen;
                        return Ok(());
                    }
                }
                Err(())
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }

    /// Record a successful request.
    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
        self.opened_at = None;
    }

    /// Record a failed request.
    fn record_failure(&mut self) {
        self.failure_count += 1;

        match self.state {
            CircuitState::HalfOpen => {
                // Trial request failed -- re-open the circuit
                self.state = CircuitState::Open;
                self.opened_at = Some(Instant::now());
            }
            CircuitState::Closed => {
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                    self.opened_at = Some(Instant::now());
                }
            }
            CircuitState::Open => {
                // Already open, keep counting
            }
        }
    }
}

/// Handles payment processing with a built-in circuit breaker.
pub struct PaymentHandler {
    /// Circuit breaker state (protected by Mutex for thread safety).
    circuit: Mutex<CircuitBreaker>,
}

impl PaymentHandler {
    /// Create a new payment handler with the circuit breaker in Closed state.
    ///
    /// The circuit breaker trips after 5 consecutive failures and has a
    /// 30-second cooldown period.
    pub fn new() -> Self {
        Self {
            circuit: Mutex::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        }
    }

    /// Process a payment for an order.
    ///
    /// Simulation rules:
    /// - `account_id` ending in "999" => Declined
    /// - `account_id` ending in "888" => Timeout
    /// - All others => Success
    ///
    /// The circuit breaker wraps this logic:
    /// - **Closed**: Process normally. Failures increment the counter.
    /// - **Open**: Reject immediately with `CircuitOpen`.
    /// - **HalfOpen**: Allow one trial. Success closes the circuit;
    ///   failure re-opens it.
    pub async fn process_payment(&self, order: &Order) -> Result<PaymentResult, PaymentError> {
        // Check circuit breaker
        {
            let mut circuit = self.circuit.lock().unwrap();
            if circuit.check().is_err() {
                return Err(PaymentError::CircuitOpen);
            }
        }

        // Simulate payment processing
        let result = simulate_payment(order);

        // Update circuit breaker based on result
        {
            let mut circuit = self.circuit.lock().unwrap();
            match &result {
                Ok(_) => circuit.record_success(),
                Err(_) => circuit.record_failure(),
            }
        }

        result
    }
}

/// Simulate an external payment service.
///
/// - `account_id` ending in "999" => Declined
/// - `account_id` ending in "888" => Timeout
/// - All others => Success
fn simulate_payment(order: &Order) -> Result<PaymentResult, PaymentError> {
    if order.account_id.ends_with("999") {
        Err(PaymentError::Declined {
            order_id: order.id.to_string(),
        })
    } else if order.account_id.ends_with("888") {
        Err(PaymentError::Timeout {
            order_id: order.id.to_string(),
        })
    } else {
        Ok(PaymentResult::Success)
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
    async fn test_payment_success() {
        let handler = PaymentHandler::new();
        let order = make_order("acc-100");

        let result = handler
            .process_payment(&order)
            .await
            .expect("Payment should succeed");
        assert_eq!(result, PaymentResult::Success);
    }

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

    #[tokio::test]
    async fn test_payment_timeout() {
        let handler = PaymentHandler::new();
        let order = make_order("acc-888");

        let result = handler.process_payment(&order).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            PaymentError::Timeout { .. } => {}
            other => panic!("Expected Timeout, got: {other}"),
        }
    }

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
            PaymentError::CircuitOpen => {}
            other => panic!("Expected CircuitOpen, got: {other}"),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_success() {
        // Use a handler with a very short cooldown for testing
        let handler = PaymentHandler {
            circuit: Mutex::new(CircuitBreaker::new(3, Duration::from_millis(100))),
        };

        // Trip the circuit with 3 failures
        for _ in 0..3 {
            let order = make_order("acc-888");
            let _ = handler.process_payment(&order).await;
        }

        // Verify circuit is open
        let order = make_order("acc-100");
        let result = handler.process_payment(&order).await;
        assert!(matches!(result, Err(PaymentError::CircuitOpen)));

        // Wait for cooldown
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Circuit should be half-open, allowing a trial request
        let order = make_order("acc-100"); // this one succeeds
        let result = handler.process_payment(&order).await;
        assert!(result.is_ok(), "Half-open trial should succeed");

        // Circuit should now be closed -- another request should work
        let order = make_order("acc-200");
        let result = handler.process_payment(&order).await;
        assert!(result.is_ok(), "Circuit should be closed after successful trial");
    }

    #[tokio::test]
    async fn test_circuit_breaker_half_open_failure() {
        let handler = PaymentHandler {
            circuit: Mutex::new(CircuitBreaker::new(3, Duration::from_millis(100))),
        };

        // Trip the circuit
        for _ in 0..3 {
            let order = make_order("acc-999");
            let _ = handler.process_payment(&order).await;
        }

        // Wait for cooldown
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Half-open trial fails
        let order = make_order("acc-999"); // this one fails
        let _ = handler.process_payment(&order).await;

        // Circuit should be open again
        let order = make_order("acc-100");
        let result = handler.process_payment(&order).await;
        assert!(
            matches!(result, Err(PaymentError::CircuitOpen)),
            "Circuit should re-open after failed trial"
        );
    }
}
