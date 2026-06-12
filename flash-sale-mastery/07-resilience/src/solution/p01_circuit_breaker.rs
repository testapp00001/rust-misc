//! # Solution 01: Circuit Breaker
//!
//! Complete implementation of the circuit breaker pattern with state transitions:
//! Closed -> Open -> HalfOpen -> Closed (or back to Open on probe failure).

use std::sync::Mutex;
use std::time::{Duration, Instant};
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

/// Internal mutable state of the circuit breaker, protected by a Mutex.
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: usize,
    last_failure_time: Option<Instant>,
}

/// Circuit breaker that protects downstream services from cascading failures.
pub struct CircuitBreaker {
    state: Mutex<CircuitBreakerState>,
    threshold: usize,
    recovery_timeout: Duration,
}

impl CircuitBreaker {
    /// Create a new circuit breaker in the Closed state.
    pub fn new(threshold: usize, recovery_timeout: Duration) -> Self {
        Self {
            state: Mutex::new(CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                last_failure_time: None,
            }),
            threshold,
            recovery_timeout,
        }
    }

    /// Execute an async operation through the circuit breaker.
    pub async fn call<F, T, E>(&self, op: F) -> Result<T, CircuitBreakerError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // Phase 1: Check state and decide whether to proceed
        let should_execute = {
            let mut state = self.state.lock().unwrap();
            match &state.state {
                CircuitState::Closed => true,
                CircuitState::Open => {
                    // Check if recovery timeout has elapsed
                    if let Some(last_failure) = state.last_failure_time {
                        if last_failure.elapsed() >= self.recovery_timeout {
                            // Transition to HalfOpen and allow probe
                            state.state = CircuitState::HalfOpen;
                            true
                        } else {
                            false // Still in timeout window
                        }
                    } else {
                        false
                    }
                }
                CircuitState::HalfOpen => true, // Allow probe request
            }
        };

        if !should_execute {
            return Err(CircuitBreakerError::CircuitOpen);
        }

        // Phase 2: Execute the operation (lock is dropped)
        let result = op.await;

        // Phase 3: Update state based on result
        {
            let mut state = self.state.lock().unwrap();
            match result {
                Ok(value) => {
                    match &state.state {
                        CircuitState::Closed => {
                            // Reset failure count on success
                            state.failure_count = 0;
                        }
                        CircuitState::HalfOpen => {
                            // Probe succeeded -- close the circuit
                            state.state = CircuitState::Closed;
                            state.failure_count = 0;
                            state.last_failure_time = None;
                        }
                        CircuitState::Open => {
                            // Shouldn't happen, but handle gracefully
                            state.state = CircuitState::Closed;
                            state.failure_count = 0;
                        }
                    }
                    Ok(value)
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    match &state.state {
                        CircuitState::Closed => {
                            state.failure_count += 1;
                            state.last_failure_time = Some(Instant::now());
                            if state.failure_count >= self.threshold {
                                state.state = CircuitState::Open;
                            }
                        }
                        CircuitState::HalfOpen => {
                            // Probe failed -- reopen the circuit
                            state.state = CircuitState::Open;
                            state.last_failure_time = Some(Instant::now());
                        }
                        CircuitState::Open => {
                            // Already open, just update timestamp
                            state.last_failure_time = Some(Instant::now());
                        }
                    }
                    Err(CircuitBreakerError::OperationFailed(error_msg))
                }
            }
        }
    }

    /// Get the current state of the circuit breaker.
    pub fn state(&self) -> CircuitState {
        self.state.lock().unwrap().state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_normal_operation_closed_state() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100));
        let result = cb.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_failures_trigger_open_state() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(500));
        for _ in 0..3 {
            let _ = cb.call(async { Err::<String, _>("failure") }).await;
        }
        assert_eq!(cb.state(), CircuitState::Open);
        let result = cb.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timeout_triggers_half_open() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        for _ in 0..2 {
            let _ = cb.call(async { Err::<String, _>("failure") }).await;
        }
        assert_eq!(cb.state(), CircuitState::Open);
        tokio::time::sleep(Duration::from_millis(60)).await;
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
        let result = cb.call(async { Err::<String, _>("still broken") }).await;
        assert!(result.is_err());
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[tokio::test]
    async fn test_success_resets_failure_count() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(500));
        let _ = cb.call(async { Err::<String, _>("fail") }).await;
        let _ = cb.call(async { Err::<String, _>("fail") }).await;
        let _ = cb.call(async { Ok::<_, String>("ok") }).await;
        let _ = cb.call(async { Err::<String, _>("fail") }).await;
        let _ = cb.call(async { Err::<String, _>("fail") }).await;
        assert_eq!(cb.state(), CircuitState::Closed);
    }
}
