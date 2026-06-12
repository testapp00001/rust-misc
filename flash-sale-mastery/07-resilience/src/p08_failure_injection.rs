//! # Exercise 08: Failure Injection
//!
//! ## Learning Objective
//!
//! Implement failure injection tools to test resilience patterns. You can't wait
//! for a real outage to discover that your circuit breaker doesn't trip or your
//! fallbacks don't work. Failure injection lets you simulate latency spikes, error
//! bursts, and timeouts in a controlled way, verifying that your resilience patterns
//! behave correctly under failure conditions.
//!
//! ## Flash Sale Context
//!
//! Before a flash sale, you need confidence that your circuit breakers will open
//! when Redis fails, that retry logic handles transient payment gateway errors,
//! and that fallbacks kick in when the notification service is overloaded. Failure
//! injection in integration tests simulates these scenarios without requiring actual
//! infrastructure failures.
//!
//! ## Instructions
//!
//! 1. Implement `FailureInjector` that can inject latency, errors, and timeouts
//! 2. Implement `inject_latency` that adds artificial delay to operations
//! 3. Implement `inject_error` that fails with a configurable probability
//! 4. Implement `inject_timeout` that simulates a hung operation
//! 5. Write integration tests using the injector to verify circuit breaker and retry behavior
//!
//! ## Hints
//!
//! - Use `tokio::time::sleep` for latency injection
//! - For error rate injection, generate a random value and compare to the threshold
//! - Use a deterministic seed for reproducible tests
//! - Combine injectors: `inject_latency(inject_error(op, rate), delay)`

use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during failure injection.
#[derive(Debug, Error)]
pub enum InjectionError {
    #[error("injected error: {reason}")]
    InjectedError { reason: String },
    #[error("injected timeout after {0:?}")]
    InjectedTimeout(Duration),
    #[error("operation failed: {0}")]
    OperationFailed(String),
}

/// Injects failures into async operations for testing resilience patterns.
///
/// Supports three injection modes:
/// - **Latency**: Adds artificial delay before the operation
/// - **Error**: Fails with a configurable probability (0.0 to 1.0)
/// - **Timeout**: Simulates an operation that never completes
pub struct FailureInjector {
    /// Fixed latency to inject before each operation
    latency: Option<Duration>,
    /// Error injection rate (0.0 = never fail, 1.0 = always fail)
    error_rate: f64,
    /// Whether to simulate a timeout (operation hangs forever)
    simulate_timeout: bool,
    /// Counter for deterministic pseudo-random behavior
    call_count: u64,
}

impl FailureInjector {
    /// Create a new failure injector with no injections.
    pub fn new() -> Self {
        // TODO: Initialize with no injections active
        todo!("Implement FailureInjector::new")
    }

    /// Configure the injector to add latency before operations.
    ///
    /// # Arguments
    /// * `duration` - How much artificial delay to add
    ///
    /// # Returns
    /// Self for method chaining
    pub fn inject_latency(mut self, duration: Duration) -> Self {
        // TODO: Store the latency duration
        todo!("Implement FailureInjector::inject_latency")
    }

    /// Configure the injector to fail with a given probability.
    ///
    /// # Arguments
    /// * `rate` - Error probability from 0.0 (never) to 1.0 (always)
    ///
    /// # Returns
    /// Self for method chaining
    pub fn inject_error(mut self, rate: f64) -> Self {
        // TODO: Store the error rate, clamped to [0.0, 1.0]
        todo!("Implement FailureInjector::inject_error")
    }

    /// Configure the injector to simulate a timeout.
    ///
    /// When active, the operation is never executed and the injector
    /// waits indefinitely (until the caller's timeout fires).
    ///
    /// # Returns
    /// Self for method chaining
    pub fn inject_timeout(mut self) -> Self {
        // TODO: Set the timeout simulation flag
        todo!("Implement FailureInjector::inject_timeout")
    }

    /// Execute an operation with all configured failure injections.
    ///
    /// Applies latency injection first, then error injection, then runs
    /// the operation (unless simulating timeout).
    ///
    /// # Arguments
    /// * `op` - An async closure that returns `Result<T, E>`
    ///
    /// # Returns
    /// `Ok(T)` if the operation succeeds, or an `InjectionError` if injection triggers
    pub async fn execute<F, T, E>(&mut self, op: F) -> Result<T, InjectionError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        // TODO: Implement injection pipeline:
        //   1. Increment call_count
        //   2. If latency is set, sleep for that duration
        //   3. If simulate_timeout is set, sleep "forever" (use a very long sleep)
        //   4. If error_rate > 0, check if this call should fail:
        //      Use a deterministic approach: (call_count * 2654435761) % 100 < error_rate * 100
        //      This gives pseudo-random behavior without the rand crate
        //   5. If no injection triggered, execute the operation
        todo!("Implement FailureInjector::execute")
    }

    /// Get the number of times execute has been called.
    pub fn call_count(&self) -> u64 {
        self.call_count
    }
}

impl Default for FailureInjector {
    fn default() -> Self {
        Self::new()
    }
}

/// A helper that counts how many times an operation was called.
///
/// Useful for verifying that retry logic called the operation the expected
/// number of times.
pub struct CallCounter {
    count: std::sync::atomic::AtomicU64,
}

impl CallCounter {
    /// Create a new call counter.
    pub fn new() -> Self {
        Self {
            count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Record a call and return the current count.
    pub fn record(&self) -> u64 {
        self.count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
    }

    /// Get the current call count.
    pub fn count(&self) -> u64 {
        self.count.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for CallCounter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_no_injection_succeeds() {
        let mut injector = FailureInjector::new();
        let result = injector.execute(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_latency_injection() {
        let mut injector = FailureInjector::new().inject_latency(Duration::from_millis(50));
        let start = std::time::Instant::now();
        let result = injector.execute(async { Ok::<_, String>("done") }).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed >= Duration::from_millis(45)); // Allow small timing variance
    }

    #[tokio::test]
    async fn test_error_injection_always_fails() {
        let mut injector = FailureInjector::new().inject_error(1.0);
        let result = injector.execute(async { Ok::<_, String>("success") }).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_error_injection_never_fails() {
        let mut injector = FailureInjector::new().inject_error(0.0);
        let result = injector.execute(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_error_injection_partial_rate() {
        let mut injector = FailureInjector::new().inject_error(0.0);
        let mut successes = 0;
        let mut failures = 0;

        // With rate=0.0, all should succeed
        for _ in 0..100 {
            match injector.execute(async { Ok::<_, String>("ok") }).await {
                Ok(_) => successes += 1,
                Err(_) => failures += 1,
            }
        }
        assert_eq!(successes, 100);
        assert_eq!(failures, 0);

        // With rate=1.0, all should fail
        let mut injector = FailureInjector::new().inject_error(1.0);
        successes = 0;
        failures = 0;
        for _ in 0..100 {
            match injector.execute(async { Ok::<_, String>("ok") }).await {
                Ok(_) => successes += 1,
                Err(_) => failures += 1,
            }
        }
        assert_eq!(successes, 0);
        assert_eq!(failures, 100);
    }

    #[tokio::test]
    async fn test_timeout_injection() {
        let mut injector = FailureInjector::new().inject_timeout();
        // The injector should hang; use tokio timeout to verify
        let result = tokio::time::timeout(
            Duration::from_millis(50),
            injector.execute(async { Ok::<_, String>("should not run") }),
        )
        .await;
        // Should time out because the injector is "hung"
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_counter() {
        let counter = CallCounter::new();
        assert_eq!(counter.count(), 0);
        counter.record();
        counter.record();
        counter.record();
        assert_eq!(counter.count(), 3);
    }

    #[tokio::test]
    async fn test_failure_injection_circuit_breaker_integration() {
        // This test demonstrates using failure injection to verify
        // circuit breaker behavior
        use crate::p01_circuit_breaker::CircuitBreaker;

        let cb = CircuitBreaker::new(3, Duration::from_millis(100));
        let mut injector = FailureInjector::new().inject_error(1.0);

        // Inject failures until circuit opens
        for _ in 0..3 {
            let _ = cb
                .call(injector.execute(async { Ok::<_, String>("op") }))
                .await;
        }

        // Circuit should be open now
        assert_eq!(cb.state(), crate::p01_circuit_breaker::CircuitState::Open);

        // Calls should be rejected immediately
        let start = std::time::Instant::now();
        let result = cb
            .call(async { Ok::<_, String>("should be rejected") })
            .await;
        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_millis(10)); // Fast rejection
    }

    #[tokio::test]
    async fn test_failure_injection_retry_integration() {
        // This test demonstrates using failure injection to verify retry behavior
        use crate::p03_retry_with_backoff::retry_with_backoff;

        let counter = Arc::new(CallCounter::new());
        let counter_clone = counter.clone();

        // Create an injector that fails the first 2 calls, then succeeds
        let attempt = std::sync::atomic::AtomicUsize::new(0);
        let result = retry_with_backoff(
            move || {
                let n = attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                counter_clone.record();
                async move {
                    if n < 2 {
                        Err(format!("fail #{n}"))
                    } else {
                        Ok("recovered")
                    }
                }
            },
            3,
            Duration::from_millis(5),
            Duration::from_millis(50),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "recovered");
        assert_eq!(counter.count(), 3); // 2 failures + 1 success
    }
}
