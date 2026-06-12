//! # Circuit Breaker
//!
//! Prevents cascading failures by short-circuiting calls to a failing service.
//!
//! States: Closed (normal) -> Open (rejecting) -> HalfOpen (probing) -> Closed

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Circuit breaker states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation -- requests pass through.
    Closed,
    /// Service is failing -- requests are rejected immediately.
    Open,
    /// Testing whether the service has recovered.
    HalfOpen,
}

/// Per-service state tracked by the circuit breaker.
#[derive(Debug, Clone)]
struct ServiceState {
    state: CircuitState,
    failure_count: u32,
    last_failure: Option<Instant>,
    success_count: u32,
}

/// A simple circuit breaker that tracks failure rates per named service.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    inner: Arc<Mutex<CircuitBreakerInner>>,
    failure_threshold: u32,
    recovery_timeout: Duration,
    half_open_success_threshold: u32,
}

#[derive(Debug)]
struct CircuitBreakerInner {
    services: HashMap<String, ServiceState>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    ///
    /// - `failure_threshold`: consecutive failures before opening.
    /// - `how long to wait in Open state before moving to HalfOpen.
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            inner: Arc::new(Mutex::new(CircuitBreakerInner {
                services: HashMap::new(),
            })),
            failure_threshold,
            recovery_timeout,
            half_open_success_threshold: 2,
        }
    }

    /// Check whether a request to `service` is allowed.
    pub fn can_execute(&self, service: &str) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let entry = inner.services.entry(service.to_string()).or_insert_with(|| ServiceState {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: None,
            success_count: 0,
        });

        match entry.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last) = entry.last_failure {
                    if last.elapsed() >= self.recovery_timeout {
                        entry.state = CircuitState::HalfOpen;
                        entry.success_count = 0;
                        tracing::info!(service = service, "Circuit breaker -> HalfOpen");
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful call to `service`.
    pub fn record_success(&self, service: &str) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(entry) = inner.services.get_mut(service) {
            match entry.state {
                CircuitState::Closed => {
                    entry.failure_count = 0;
                }
                CircuitState::HalfOpen => {
                    entry.success_count += 1;
                    if entry.success_count >= self.half_open_success_threshold {
                        entry.state = CircuitState::Closed;
                        entry.failure_count = 0;
                        entry.success_count = 0;
                        tracing::info!(service = service, "Circuit breaker -> Closed");
                    }
                }
                CircuitState::Open => {
                    // Should not happen (we don't execute in Open), but reset anyway.
                    entry.state = CircuitState::Closed;
                    entry.failure_count = 0;
                }
            }
        }
    }

    /// Record a failed call to `service`.
    pub fn record_failure(&self, service: &str) {
        let mut inner = self.inner.lock().unwrap();
        let entry = inner.services.entry(service.to_string()).or_insert_with(|| ServiceState {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure: None,
            success_count: 0,
        });

        entry.failure_count += 1;
        entry.last_failure = Some(Instant::now());

        match entry.state {
            CircuitState::Closed => {
                if entry.failure_count >= self.failure_threshold {
                    entry.state = CircuitState::Open;
                    tracing::warn!(
                        service = service,
                        failures = entry.failure_count,
                        "Circuit breaker -> Open"
                    );
                }
            }
            CircuitState::HalfOpen => {
                // A single failure in HalfOpen trips back to Open.
                entry.state = CircuitState::Open;
                entry.success_count = 0;
                tracing::warn!(service = service, "Circuit breaker HalfOpen -> Open");
            }
            CircuitState::Open => {
                // Already open, just update the timestamp.
            }
        }
    }

    /// Get the current state of a service's circuit breaker.
    pub fn state(&self, service: &str) -> CircuitState {
        let inner = self.inner.lock().unwrap();
        inner
            .services
            .get(service)
            .map(|s| s.state)
            .unwrap_or(CircuitState::Closed)
    }

    /// Get the current failure count for a service.
    pub fn failure_count(&self, service: &str) -> u32 {
        let inner = self.inner.lock().unwrap();
        inner
            .services
            .get(service)
            .map(|s| s.failure_count)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_closed() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(5));
        assert_eq!(cb.state("redis"), CircuitState::Closed);
        assert!(cb.can_execute("redis"));
    }

    #[test]
    fn test_opens_after_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(5));
        cb.record_failure("redis");
        cb.record_failure("redis");
        assert_eq!(cb.state("redis"), CircuitState::Closed);
        assert!(cb.can_execute("redis"));

        cb.record_failure("redis");
        assert_eq!(cb.state("redis"), CircuitState::Open);
        assert!(!cb.can_execute("redis"));
    }

    #[test]
    fn test_transitions_to_half_open_after_timeout() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(50));
        cb.record_failure("redis");
        cb.record_failure("redis");
        assert_eq!(cb.state("redis"), CircuitState::Open);

        // Not enough time has passed
        assert!(!cb.can_execute("redis"));

        std::thread::sleep(Duration::from_millis(60));
        assert!(cb.can_execute("redis"));
        assert_eq!(cb.state("redis"), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_to_closed_on_success() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure("redis");
        cb.record_failure("redis");
        std::thread::sleep(Duration::from_millis(15));
        assert!(cb.can_execute("redis")); // -> HalfOpen

        cb.record_success("redis");
        assert_eq!(cb.state("redis"), CircuitState::HalfOpen); // need 2 successes

        cb.record_success("redis");
        assert_eq!(cb.state("redis"), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_to_open_on_failure() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(10));
        cb.record_failure("redis");
        cb.record_failure("redis");
        std::thread::sleep(Duration::from_millis(15));
        assert!(cb.can_execute("redis")); // -> HalfOpen

        cb.record_failure("redis");
        assert_eq!(cb.state("redis"), CircuitState::Open);
    }

    #[test]
    fn test_success_resets_failure_count() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(5));
        cb.record_failure("redis");
        cb.record_failure("redis");
        cb.record_success("redis");
        assert_eq!(cb.failure_count("redis"), 0);
    }

    #[test]
    fn test_independent_services() {
        let cb = CircuitBreaker::new(2, Duration::from_secs(5));
        cb.record_failure("redis");
        cb.record_failure("redis");
        assert_eq!(cb.state("redis"), CircuitState::Open);

        // DB is independent
        assert_eq!(cb.state("db"), CircuitState::Closed);
        assert!(cb.can_execute("db"));
    }
}
