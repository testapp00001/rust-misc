//! # Exercise 01: Circuit Breaker
//!
//! ## Learning Objective
//!
//! Implement the Circuit Breaker pattern to prevent cascading failures. When a
//! downstream service fails repeatedly, the circuit breaker "opens" and immediately
//! rejects further calls, giving the failing service time to recover.
//!
//! ## Flash Sale Context
//!
//! During a flash sale, your order service calls a payment gateway. If the payment
//! gateway becomes overwhelmed and starts timing out, every request will hang for
//! the full timeout duration, consuming threads and memory. A circuit breaker detects
//! the pattern of failures and short-circuits new requests immediately, returning a
//! fast error instead of waiting. After a recovery period, it allows a probe request
//! through to test if the service has recovered.
//!
//! ## Instructions
//!
//! 1. Define a `CircuitState` enum with variants: `Closed`, `Open`, `HalfOpen`
//! 2. Define a `CircuitBreaker` struct with state tracking fields
//! 3. Implement `new(threshold, recovery_timeout)` constructor
//! 4. Implement `call<F, T>(op: F)` that wraps an async operation with circuit breaker logic
//!    - In Closed state: execute the operation, track failures
//!    - In Open state: reject immediately if timeout hasn't elapsed, transition to HalfOpen otherwise
//!    - In HalfOpen state: allow one probe request, transition based on result
//!
//! ## Hints
//!
//! - Use `std::sync::Mutex` for interior mutability (state updates are fast, no await inside lock)
//! - Use `std::time::Instant` for tracking failure timestamps
//! - The `call` method takes `&self` -- use a Mutex to mutate state through a shared reference
//! - When transitioning from Open to HalfOpen, let the probe request through

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during circuit breaker operations.
#[derive(Debug, Error)]
pub enum CircuitBreakerError {
    #[error("circuit breaker is open")]
    CircuitOpen,
    #[error("operation failed: {0}")]
    OperationFailed(String),
}

/// The current state of the circuit breaker.
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    /// Normal operation -- requests pass through.
    Closed,
    /// Too many failures -- requests are rejected immediately.
    Open,
    /// Testing if the service has recovered -- one probe request allowed.
    HalfOpen,
}

/// Circuit breaker that protects downstream services from cascading failures.
///
/// Tracks failure count and transitions between states:
/// - Closed -> Open: when failures reach the threshold
/// - Open -> HalfOpen: when recovery timeout has elapsed
/// - HalfOpen -> Closed: when the probe request succeeds
/// - HalfOpen -> Open: when the probe request fails
pub struct CircuitBreaker {
    // TODO: Add fields for:
    //   - state: CircuitState (inside a Mutex for interior mutability)
    //   - failure_count: usize
    //   - threshold: usize (max failures before opening)
    //   - recovery_timeout: Duration (how long to wait before half-open)
    //   - last_failure_time: Option<Instant>
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// # Arguments
    /// * `threshold` - Number of consecutive failures before the circuit opens
    /// * `recovery_timeout` - Duration to wait in Open state before transitioning to HalfOpen
    ///
    /// # Returns
    /// A new `CircuitBreaker` in the Closed state
    pub fn new(threshold: usize, recovery_timeout: Duration) -> Self {
        // TODO: Initialize the circuit breaker in the Closed state
        todo!("Implement CircuitBreaker::new")
    }

    /// Execute an async operation through the circuit breaker.
    ///
    /// If the circuit is Open, returns `Err(CircuitBreakerError::CircuitOpen)` immediately.
    /// If the circuit is Closed or HalfOpen, executes the operation and updates state
    /// based on the result.
    ///
    /// # Arguments
    /// * `op` - An async closure that returns `Result<T, E>` where E can be converted to a string
    ///
    /// # Returns
    /// `Ok(T)` if the operation succeeds, or a `CircuitBreakerError` if it fails
    pub async fn call<F, T, E>(&self, op: F) -> Result<T, CircuitBreakerError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // TODO: Implement circuit breaker logic:
        //   1. Lock the state mutex
        //   2. Check current state:
        //      - Closed: proceed with operation
        //      - Open: check if recovery_timeout has elapsed
        //        - If yes: transition to HalfOpen, proceed with operation
        //        - If no: return Err(CircuitOpen)
        //      - HalfOpen: proceed with operation (probe)
        //   3. Drop the lock before executing the async operation
        //   4. Execute the operation
        //   5. Lock again and update state based on result:
        //      - Success in Closed: reset failure_count to 0
        //      - Success in HalfOpen: transition to Closed, reset failure_count
        //      - Failure in Closed: increment failure_count, transition to Open if >= threshold
        //      - Failure in HalfOpen: transition to Open
        todo!("Implement CircuitBreaker::call")
    }

    /// Get the current state of the circuit breaker.
    pub fn state(&self) -> CircuitState {
        // TODO: Return the current state (clone it out of the Mutex)
        todo!("Implement CircuitBreaker::state")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_normal_operation_closed_state() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100));
        // A successful operation should stay in Closed state
        let result = cb.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_failures_trigger_open_state() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(500));
        // Cause 3 failures to trip the circuit
        for _ in 0..3 {
            let _ = cb.call(async { Err::<String, _>("failure") }).await;
        }
        assert_eq!(cb.state(), CircuitState::Open);
        // Next call should be rejected immediately
        let result = cb.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout_triggers_half_open() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        // Trip the circuit
        for _ in 0..2 {
            let _ = cb.call(async { Err::<String, _>("failure") }).await;
        }
        assert_eq!(cb.state(), CircuitState::Open);
        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(60)).await;
        // Next call should transition to HalfOpen and execute
        let result = cb.call(async { Ok::<_, String>("recovered") }).await;
        assert!(result.is_ok());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_half_open_success_closes() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        for _ in 0..2 {
            let _ = cb.call(async { Err::<String, _>("failure") }).await;
        }
        tokio::time::sleep(Duration::from_millis(60)).await;
        // Probe with success
        let result = cb.call(async { Ok::<_, String>("ok") }).await;
        assert!(result.is_ok());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_half_open_failure_reopens() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        for _ in 0..2 {
            let _ = cb.call(async { Err::<String, _>("failure") }).await;
        }
        tokio::time::sleep(Duration::from_millis(60)).await;
        // Probe with failure -- should reopen
        let result = cb.call(async { Err::<String, _>("still broken") }).await;
        assert!(result.is_err());
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_success_resets_failure_count() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(500));
        // Two failures (below threshold)
        let _ = cb.call(async { Err::<String, _>("fail") }).await;
        let _ = cb.call(async { Err::<String, _>("fail") }).await;
        // One success resets the counter
        let _ = cb.call(async { Ok::<_, String>("ok") }).await;
        // Two more failures should NOT trip the circuit (count was reset)
        let _ = cb.call(async { Err::<String, _>("fail") }).await;
        let _ = cb.call(async { Err::<String, _>("fail") }).await;
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
