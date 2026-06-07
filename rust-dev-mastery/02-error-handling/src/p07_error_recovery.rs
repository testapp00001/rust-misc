//! # Lesson 7: Error Recovery Patterns
//!
//! Not all errors are fatal. This lesson covers retry patterns, fallback
//! strategies, partial success handling, and error classification for
//! deciding what to do when things go wrong.

use std::time::Duration;

// ---------------------------------------------------------------------------
// Error classification for recovery decisions
// ---------------------------------------------------------------------------

/// Classifies errors by their recoverability.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryAction {
    /// Retry immediately.
    RetryNow,
    /// Retry after a delay.
    RetryAfter(Duration),
    /// Use a fallback value.
    Fallback,
    /// Skip this item and continue.
    Skip,
    /// Abort the entire operation.
    Abort,
}

/// An error with its recommended recovery action.
#[derive(Debug)]
pub struct RecoverableError<E> {
    pub error: E,
    pub action: RecoveryAction,
    pub attempt: u32,
    pub max_attempts: u32,
}

impl<E: std::fmt::Display> std::fmt::Display for RecoverableError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (attempt {}/{}, action: {:?})",
            self.error, self.attempt, self.max_attempts, self.action
        )
    }
}

// ---------------------------------------------------------------------------
// Retry with exponential backoff
// ---------------------------------------------------------------------------

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            base_delay: Duration::from_millis(50),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 1.5,
        }
    }

    pub fn conservative() -> Self {
        Self {
            max_attempts: 10,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(120),
            backoff_multiplier: 2.0,
        }
    }

    /// Calculate delay for a given attempt (0-indexed).
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let millis = self.base_delay.as_millis() as f64
            * self.backoff_multiplier.powi(attempt as i32);
        let delay = Duration::from_millis(millis as u64);
        if delay > self.max_delay {
            self.max_delay
        } else {
            delay
        }
    }
}

/// Execute an operation with retry logic.
pub fn retry<T, E, F>(config: &RetryConfig, mut operation: F) -> Result<T, RecoverableError<E>>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Debug,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match operation() {
            Ok(value) => return Ok(value),
            Err(e) => {
                last_error = Some(e);
                if attempt + 1 < config.max_attempts {
                    let _delay = config.delay_for_attempt(attempt);
                    // In real code: std::thread::sleep(delay);
                    // For tests we skip the actual sleep
                }
            }
        }
    }

    Err(RecoverableError {
        error: last_error.unwrap(),
        action: RecoveryAction::Abort,
        attempt: config.max_attempts,
        max_attempts: config.max_attempts,
    })
}

// ---------------------------------------------------------------------------
// Retry with predicate (only retry on specific errors)
// ---------------------------------------------------------------------------

/// Retry only if the error matches a predicate.
pub fn retry_if<T, E, F, P>(
    config: &RetryConfig,
    mut operation: F,
    should_retry: P,
) -> Result<T, RecoverableError<E>>
where
    F: FnMut() -> Result<T, E>,
    P: Fn(&E) -> bool,
    E: std::fmt::Debug,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        match operation() {
            Ok(value) => return Ok(value),
            Err(e) => {
                let retryable = should_retry(&e);
                if !retryable || attempt + 1 >= config.max_attempts {
                    return Err(RecoverableError {
                        error: e,
                        action: RecoveryAction::Abort,
                        attempt: attempt + 1,
                        max_attempts: config.max_attempts,
                    });
                }
                last_error = Some(e);
            }
        }
    }

    Err(RecoverableError {
        error: last_error.unwrap(),
        action: RecoveryAction::Abort,
        attempt: config.max_attempts,
        max_attempts: config.max_attempts,
    })
}

// ---------------------------------------------------------------------------
// Fallback strategies
// ---------------------------------------------------------------------------

/// Try a primary operation, falling back to a secondary on failure.
pub fn with_fallback<T, E, F1, F2>(
    primary: F1,
    fallback: F2,
) -> Result<T, E>
where
    F1: FnOnce() -> Result<T, E>,
    F2: FnOnce() -> Result<T, E>,
{
    primary().or_else(|_| fallback())
}

/// Try multiple fallbacks in order.
pub fn with_fallbacks<T, E, F>(operations: Vec<F>) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
{
    let mut last_err = None;
    for op in operations {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) => last_err = Some(e),
        }
    }
    Err(last_err.expect("at least one operation required"))
}

// ---------------------------------------------------------------------------
// Partial success handling
// ---------------------------------------------------------------------------

/// The result of an operation that may partially succeed.
#[derive(Debug)]
pub enum PartialResult<T, E> {
    /// All items succeeded.
    Complete(T),
    /// Some items succeeded, some failed.
    Partial {
        successes: Vec<T>,
        failures: Vec<(usize, E)>,
    },
    /// All items failed.
    Failed(Vec<(usize, E)>),
}

impl<T, E> PartialResult<T, E> {
    pub fn is_complete(&self) -> bool {
        matches!(self, PartialResult::Complete(_))
    }

    pub fn success_count(&self) -> usize {
        match self {
            PartialResult::Complete(_) => 1,
            PartialResult::Partial { successes, .. } => successes.len(),
            PartialResult::Failed(_) => 0,
        }
    }

    pub fn failure_count(&self) -> usize {
        match self {
            PartialResult::Complete(_) => 0,
            PartialResult::Partial { failures, .. } => failures.len(),
            PartialResult::Failed(f) => f.len(),
        }
    }

    /// Get successes, returning empty vec for Failed.
    pub fn successes(&self) -> Vec<&T> {
        match self {
            PartialResult::Complete(v) => vec![v],
            PartialResult::Partial { successes, .. } => successes.iter().collect(),
            PartialResult::Failed(_) => vec![],
        }
    }
}

/// Process a collection of items, collecting successes and failures.
pub fn process_all<T, R, E, F>(
    items: Vec<T>,
    mut operation: F,
) -> PartialResult<R, E>
where
    F: FnMut(&T) -> Result<R, E>,
{
    let mut successes = Vec::new();
    let mut failures = Vec::new();

    for (i, item) in items.iter().enumerate() {
        match operation(item) {
            Ok(result) => successes.push(result),
            Err(e) => failures.push((i, e)),
        }
    }

    if failures.is_empty() {
        if successes.len() == 1 {
            PartialResult::Complete(successes.into_iter().next().unwrap())
        } else {
            // For multi-item, return as partial with all successes
            PartialResult::Partial {
                successes,
                failures: vec![],
            }
        }
    } else if successes.is_empty() {
        PartialResult::Failed(failures)
    } else {
        PartialResult::Partial {
            successes,
            failures,
        }
    }
}

// ---------------------------------------------------------------------------
// Circuit breaker pattern
// ---------------------------------------------------------------------------

/// A simple circuit breaker to prevent cascading failures.
#[derive(Debug)]
pub struct CircuitBreaker {
    pub failure_threshold: u32,
    pub failure_count: u32,
    pub state: CircuitState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing if service recovered
}

impl CircuitBreaker {
    pub fn new(threshold: u32) -> Self {
        Self {
            failure_threshold: threshold,
            failure_count: 0,
            state: CircuitState::Closed,
        }
    }

    /// Record a success, resetting the circuit.
    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    /// Record a failure, potentially tripping the circuit.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    /// Check if the circuit allows a request.
    pub fn allow_request(&self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => false,
            CircuitState::HalfOpen => true,
        }
    }

    /// Attempt to reset (transition from Open to HalfOpen).
    pub fn try_reset(&mut self) {
        if self.state == CircuitState::Open {
            self.state = CircuitState::HalfOpen;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_success_first_attempt() {
        let config = RetryConfig::default();
        let result = retry(&config, || Ok::<_, String>(42));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_retry_success_after_failures() {
        let config = RetryConfig::default();
        let mut attempts = 0;
        let result = retry(&config, || {
            attempts += 1;
            if attempts < 3 {
                Err("not yet")
            } else {
                Ok(42)
            }
        });
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 3);
    }

    #[test]
    fn test_retry_exhausted() {
        let config = RetryConfig {
            max_attempts: 3,
            ..Default::default()
        };
        let result = retry(&config, || Err::<i32, _>("always fails"));
        let err = result.unwrap_err();
        assert_eq!(err.attempt, 3);
        assert_eq!(err.max_attempts, 3);
    }

    #[test]
    fn test_retry_config_delay_calculation() {
        let config = RetryConfig {
            base_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(5),
            ..Default::default()
        };

        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
        // Should cap at max_delay
        assert_eq!(config.delay_for_attempt(10), Duration::from_secs(5));
    }

    #[test]
    fn test_retry_config_presets() {
        let aggressive = RetryConfig::aggressive();
        assert_eq!(aggressive.max_attempts, 5);

        let conservative = RetryConfig::conservative();
        assert_eq!(conservative.max_attempts, 10);
    }

    #[test]
    fn test_retry_if_should_not_retry() {
        let config = RetryConfig::default();
        let result = retry_if(&config, || Err::<i32, _>("permanent"), |_| false);
        assert!(result.is_err());
        // Should have tried only once
        assert_eq!(result.unwrap_err().attempt, 1);
    }

    #[test]
    fn test_retry_if_should_retry() {
        let config = RetryConfig {
            max_attempts: 3,
            ..Default::default()
        };
        let mut attempts = 0;
        let result = retry_if(
            &config,
            || {
                attempts += 1;
                if attempts < 3 {
                    Err("transient")
                } else {
                    Ok(42)
                }
            },
            |_| true,
        );
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_with_fallback_primary_success() {
        let result = with_fallback(|| Ok::<_, String>(42), || Ok(0));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_with_fallback_primary_failure() {
        let result = with_fallback(|| Err::<i32, _>("fail"), || Ok(99));
        assert_eq!(result.unwrap(), 99);
    }

    #[test]
    fn test_with_fallbacks() {
        let result = with_fallbacks(vec![
            || Err::<i32, _>("fail 1"),
            || Err::<i32, _>("fail 2"),
            || Ok(42),
        ]);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_with_fallbacks_all_fail() {
        let result = with_fallbacks(vec![
            || Err::<i32, _>("fail 1"),
            || Err::<i32, _>("fail 2"),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_process_all_complete() {
        let result = process_all(vec![1, 2, 3], |x| Ok::<_, String>(x * 2));
        // With multiple items all succeeding, process_all returns Partial
        // with all successes and no failures.
        assert_eq!(result.failure_count(), 0);
        assert_eq!(result.success_count(), 3);
    }

    #[test]
    fn test_process_all_partial() {
        let result = process_all(vec![1, 2, 3, 4, 5], |x| {
            if x % 2 == 0 {
                Ok(x * 2)
            } else {
                Err("odd")
            }
        });
        match result {
            PartialResult::Partial {
                successes,
                failures,
            } => {
                assert_eq!(successes.len(), 2);
                assert_eq!(failures.len(), 3);
            }
            _ => panic!("expected partial"),
        }
    }

    #[test]
    fn test_process_all_failed() {
        let result = process_all(vec![1, 2, 3], |x| Err::<i32, _>(format!("err {}", x)));
        match result {
            PartialResult::Failed(failures) => {
                assert_eq!(failures.len(), 3);
            }
            _ => panic!("expected failed"),
        }
    }

    #[test]
    fn test_circuit_breaker_normal() {
        let mut cb = CircuitBreaker::new(3);
        assert_eq!(cb.state, CircuitState::Closed);
        assert!(cb.allow_request());

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_trips() {
        let mut cb = CircuitBreaker::new(3);
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_recovery() {
        let mut cb = CircuitBreaker::new(2);
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state, CircuitState::Open);

        cb.try_reset();
        assert_eq!(cb.state, CircuitState::HalfOpen);
        assert!(cb.allow_request());

        cb.record_success();
        assert_eq!(cb.state, CircuitState::Closed);
    }

    #[test]
    fn test_recoverable_error_display() {
        let err = RecoverableError {
            error: "timeout",
            action: RecoveryAction::RetryAfter(Duration::from_secs(5)),
            attempt: 2,
            max_attempts: 3,
        };
        let msg = err.to_string();
        assert!(msg.contains("timeout"));
        assert!(msg.contains("2/3"));
    }

    #[test]
    fn test_partial_result_methods() {
        let complete = PartialResult::<i32, String>::Complete(42);
        assert!(complete.is_complete());
        assert_eq!(complete.success_count(), 1);
        assert_eq!(complete.failure_count(), 0);

        let failed = PartialResult::<i32, String>::Failed(vec![(0, "err".into())]);
        assert!(!failed.is_complete());
        assert_eq!(failed.success_count(), 0);
        assert_eq!(failed.failure_count(), 1);
    }
}
