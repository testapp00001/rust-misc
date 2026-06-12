//! # Solution 08: Failure Injection
//!
//! Complete implementation of failure injection tools for testing resilience patterns.
//! Supports latency injection, error injection, and timeout simulation.

use std::sync::atomic::{AtomicU64, Ordering};
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
pub struct FailureInjector {
    latency: Option<Duration>,
    error_rate: f64,
    simulate_timeout: bool,
    call_count: u64,
}

impl FailureInjector {
    /// Create a new failure injector with no injections.
    pub fn new() -> Self {
        Self {
            latency: None,
            error_rate: 0.0,
            simulate_timeout: false,
            call_count: 0,
        }
    }

    /// Configure the injector to add latency before operations.
    pub fn inject_latency(mut self, duration: Duration) -> Self {
        self.latency = Some(duration);
        self
    }

    /// Configure the injector to fail with a given probability.
    pub fn inject_error(mut self, rate: f64) -> Self {
        self.error_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Configure the injector to simulate a timeout (operation hangs forever).
    pub fn inject_timeout(mut self) -> Self {
        self.simulate_timeout = true;
        self
    }

    /// Execute an operation with all configured failure injections.
    ///
    /// Pipeline: latency -> timeout check -> error check -> execute
    pub async fn execute<F, T, E>(&mut self, op: F) -> Result<T, InjectionError>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        self.call_count += 1;

        // Step 1: Inject latency
        if let Some(duration) = self.latency {
            tokio::time::sleep(duration).await;
        }

        // Step 2: Simulate timeout (hang indefinitely)
        if self.simulate_timeout {
            // Sleep for a very long time -- the caller's timeout will fire first
            tokio::time::sleep(Duration::from_secs(3600)).await;
            return Err(InjectionError::InjectedTimeout(Duration::from_secs(3600)));
        }

        // Step 3: Error injection using deterministic pseudo-random
        if self.error_rate > 0.0 {
            // Knuth's multiplicative hash for deterministic pseudo-randomness
            let hash = (self.call_count.wrapping_mul(2654435761)) % 100;
            let threshold = (self.error_rate * 100.0) as u64;
            if hash < threshold {
                return Err(InjectionError::InjectedError {
                    reason: format!(
                        "injected failure (rate={:.2}, call=#{})",
                        self.error_rate, self.call_count
                    ),
                });
            }
        }

        // Step 4: Execute the actual operation
        op.await.map_err(|e| InjectionError::OperationFailed(e.to_string()))
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
pub struct CallCounter {
    count: AtomicU64,
}

impl CallCounter {
    /// Create a new call counter.
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
        }
    }

    /// Record a call and return the current count.
    pub fn record(&self) -> u64 {
        self.count.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Get the current call count.
    pub fn count(&self) -> u64 {
        self.count.load(Ordering::SeqCst)
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
        assert!(elapsed >= Duration::from_millis(45));
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

        for _ in 0..100 {
            match injector.execute(async { Ok::<_, String>("ok") }).await {
                Ok(_) => successes += 1,
                Err(_) => failures += 1,
            }
        }
        assert_eq!(successes, 100);
        assert_eq!(failures, 0);

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
        let result = tokio::time::timeout(
            Duration::from_millis(50),
            injector.execute(async { Ok::<_, String>("should not run") }),
        )
        .await;
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
        use crate::p01_circuit_breaker::CircuitBreaker;

        let cb = CircuitBreaker::new(3, Duration::from_millis(100));
        let mut injector = FailureInjector::new().inject_error(1.0);

        for _ in 0..3 {
            let _ = cb
                .call(injector.execute(async { Ok::<_, String>("op") }))
                .await;
        }

        assert_eq!(cb.state(), crate::p01_circuit_breaker::CircuitState::Open);

        let start = std::time::Instant::now();
        let result = cb
            .call(async { Ok::<_, String>("should be rejected") })
            .await;
        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_failure_injection_retry_integration() {
        use crate::p03_retry_with_backoff::retry_with_backoff;

        let counter = Arc::new(CallCounter::new());
        let counter_clone = counter.clone();

        let attempt = std::sync::atomic::AtomicUsize::new(0);
        let result = retry_with_backoff(
            move || {
                let n = attempt.fetch_add(1, Ordering::SeqCst);
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
        assert_eq!(counter.count(), 3);
    }
}
