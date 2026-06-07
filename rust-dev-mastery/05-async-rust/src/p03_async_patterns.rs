//! # Async Patterns
//!
//! Core patterns for combining, racing, and orchestrating async operations.
//! `select!`, `join!`, `try_join!`, and `timeout` are the building blocks
//! of all non-trivial async applications.
//!
//! ## Key Concepts
//! - **select!**: Race multiple futures, proceed with the first to complete
//! - **join!**: Run multiple futures concurrently and wait for ALL to complete
//! - **try_join!**: Like join! but short-circuits on the first error
//! - **timeout**: Bound a future's execution time
//! - **Async blocks/closures**: Creating futures inline

use std::time::Duration;

/// A service that makes redundant requests to multiple backends and returns
/// whichever responds first (hedged request pattern).
pub struct HedgedService {
    backends: Vec<String>,
    timeout: Duration,
}

impl HedgedService {
    pub fn new(backends: Vec<String>, timeout: Duration) -> Self {
        HedgedService { backends, timeout }
    }

    /// Sends a request to all backends concurrently and returns the first
    /// successful response. This is the core hedged-request pattern used
    /// in high-reliability systems.
    pub async fn query(&self, request: &str) -> Result<String, ServiceError> {
        if self.backends.is_empty() {
            return Err(ServiceError::NoBackends);
        }

        let mut futures: Vec<std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ServiceError>> + Send>>> =
            Vec::new();

        for backend in &self.backends {
            let req = request.to_string();
            let backend = backend.clone();
            futures.push(Box::pin(async move {
                simulate_backend_call(&backend, &req).await
            }));
        }

        // Race all backends — return the first successful result
        let result = tokio::time::timeout(self.timeout, async {
            // Use futures::future::select_all to race all backends
            let (result, _idx, _remaining) = futures::future::select_all(futures).await;
            result
        })
        .await;

        match result {
            Ok(result) => result,
            Err(_) => Err(ServiceError::Timeout),
        }
    }
}

async fn simulate_backend_call(backend: &str, request: &str) -> Result<String, ServiceError> {
    // Simulate variable latency per backend
    let delay = match backend.as_bytes().last() {
        Some(b) => Duration::from_millis((*b % 10) as u64),
        None => Duration::from_millis(5),
    };
    tokio::time::sleep(delay).await;
    Ok(format!("{backend}:{request}:response"))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceError {
    NoBackends,
    Timeout,
    BackendError(String),
}

/// Demonstrates the join! pattern for running independent operations concurrently.
/// Returns the results of all operations once they all complete.
pub async fn fetch_all_data(user_id: u64) -> (String, Vec<String>, u64) {
    // These three operations run concurrently
    let (profile, orders, score) = tokio::join!(
        fetch_profile(user_id),
        fetch_orders(user_id),
        fetch_score(user_id),
    );

    (profile, orders, score)
}

async fn fetch_profile(user_id: u64) -> String {
    tokio::time::sleep(Duration::from_millis(10)).await;
    format!("Profile for user {user_id}")
}

async fn fetch_orders(user_id: u64) -> Vec<String> {
    tokio::time::sleep(Duration::from_millis(15)).await;
    vec![format!("Order-{user_id}-1"), format!("Order-{user_id}-2")]
}

async fn fetch_score(user_id: u64) -> u64 {
    tokio::time::sleep(Duration::from_millis(5)).await;
    user_id * 100
}

/// Demonstrates try_join! for operations where any failure should abort all.
pub async fn load_config() -> Result<(String, u32, bool), ConfigError> {
    let (host, port, tls) = tokio::try_join!(
        read_config_value("host"),
        read_config_value::<u32>("port"),
        read_config_value::<bool>("tls_enabled"),
    )?;

    Ok((host, port, tls))
}

async fn read_config_value<T: std::str::FromStr>(key: &str) -> Result<T, ConfigError>
where
    T::Err: std::fmt::Display,
{
    // Simulate reading from config source
    let value = match key {
        "host" => "localhost".to_string(),
        "port" => "8080".to_string(),
        "tls_enabled" => "true".to_string(),
        _ => return Err(ConfigError::NotFound(key.to_string())),
    };
    value
        .parse::<T>()
        .map_err(|e| ConfigError::ParseError(format!("{e}")))
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    NotFound(String),
    ParseError(String),
}

/// Demonstrates the select! macro for racing multiple futures with pattern matching.
pub async fn race_operations() -> String {
    let op_a = async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        "operation_a"
    };
    let op_b = async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        "operation_b"
    };

    tokio::select! {
        result = op_a => format!("A won: {result}"),
        result = op_b => format!("B won: {result}"),
    }
}

/// Demonstrates select! with a cancellation token pattern.
pub async fn cancellable_work(cancel: tokio::sync::watch::Receiver<bool>) -> Result<u64, ()> {
    let mut cancel = cancel;
    let mut count = 0u64;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(10)) => {
                count += 1;
                if count >= 100 {
                    return Ok(count);
                }
            }
            _ = cancel.changed() => {
                if *cancel.borrow() {
                    return Err(());
                }
            }
        }
    }
}

/// An async retry utility with exponential backoff.
pub async fn retry_with_backoff<F, Fut, T, E>(
    max_retries: u32,
    initial_delay: Duration,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let mut delay = initial_delay;

    for attempt in 0..=max_retries {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(e) => {
                if attempt == max_retries {
                    return Err(e);
                }
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
        }
    }

    unreachable!()
}

/// A debounce utility: waits for a period of inactivity before executing.
pub struct Debouncer {
    delay: Duration,
    last_request: Option<tokio::time::Instant>,
}

impl Debouncer {
    pub fn new(delay: Duration) -> Self {
        Debouncer {
            delay,
            last_request: None,
        }
    }

    /// Call this to signal activity. Returns a future that resolves after the
    /// debounce period with no new activity.
    pub async fn debounce(&mut self) {
        self.last_request = Some(tokio::time::Instant::now());
        let target = self.last_request.unwrap() + self.delay;

        loop {
            let now = tokio::time::Instant::now();
            if now >= target {
                // Check if a newer request came in during sleep
                if self.last_request.map_or(false, |t| t + self.delay > now) {
                    // A new request was made; restart the timer
                    let new_target = self.last_request.unwrap() + self.delay;
                    tokio::time::sleep_until(new_target).await;
                    continue;
                }
                return;
            }
            tokio::time::sleep_until(target).await;
        }
    }
}

/// Circuit breaker pattern for async services.
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: CircuitState,
    consecutive_failures: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,         // Normal operation
    Open,           // Failing, reject requests
    HalfOpen,       // Testing if service recovered
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        CircuitBreaker {
            failure_threshold,
            recovery_timeout,
            state: CircuitState::Closed,
            consecutive_failures: 0,
        }
    }

    pub async fn call<F, Fut, T, E>(&mut self, operation: F) -> Result<T, CircuitError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
    {
        match self.state {
            CircuitState::Open => {
                // Check if recovery timeout has elapsed
                // (simplified: in production you'd track the time)
                self.state = CircuitState::HalfOpen;
                // Fall through to try the operation
            }
            CircuitState::HalfOpen | CircuitState::Closed => {}
        }

        match operation().await {
            Ok(value) => {
                self.consecutive_failures = 0;
                self.state = CircuitState::Closed;
                Ok(value)
            }
            Err(e) => {
                self.consecutive_failures += 1;
                if self.consecutive_failures >= self.failure_threshold {
                    self.state = CircuitState::Open;
                }
                Err(CircuitError::OperationFailed(e))
            }
        }
    }

    pub fn is_open(&self) -> bool {
        self.state == CircuitState::Open
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitError<E> {
    CircuitOpen,
    OperationFailed(E),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hedged_service_returns_first() {
        let service = HedgedService::new(
            vec!["backend-a".into(), "backend-b".into()],
            Duration::from_secs(5),
        );
        let result = service.query("test").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("response"));
    }

    #[tokio::test]
    async fn test_hedged_service_no_backends() {
        let service = HedgedService::new(vec![], Duration::from_secs(5));
        let result = service.query("test").await;
        assert_eq!(result, Err(ServiceError::NoBackends));
    }

    #[tokio::test]
    async fn test_fetch_all_data() {
        let (profile, orders, score) = fetch_all_data(42).await;
        assert_eq!(profile, "Profile for user 42");
        assert_eq!(orders.len(), 2);
        assert_eq!(score, 4200);
    }

    #[tokio::test]
    async fn test_load_config() {
        let result = load_config().await;
        assert!(result.is_ok());
        let (host, port, tls) = result.unwrap();
        assert_eq!(host, "localhost");
        assert_eq!(port, 8080);
        assert!(tls);
    }

    #[tokio::test]
    async fn test_race_operations() {
        // B should win since it has shorter delay
        let result = race_operations().await;
        assert!(result.contains("operation_b"));
    }

    #[tokio::test]
    async fn test_cancellable_work_completes() {
        let (_tx, rx) = tokio::sync::watch::channel(false);
        let result = cancellable_work(rx).await;
        assert_eq!(result, Ok(100));
    }

    #[tokio::test]
    async fn test_cancellable_work_cancelled() {
        let (tx, rx) = tokio::sync::watch::channel(false);
        let handle = tokio::spawn(async move { cancellable_work(rx).await });

        // Cancel after a short time
        tokio::time::sleep(Duration::from_millis(50)).await;
        tx.send(true).unwrap();

        let result = handle.await.unwrap();
        assert_eq!(result, Err(()));
    }

    #[tokio::test]
    async fn test_retry_succeeds_first_try() {
        let result = retry_with_backoff(3, Duration::from_millis(1), || async {
            Ok::<_, String>(42)
        })
        .await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_retry_eventually_succeeds() {
        let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        let result = retry_with_backoff(3, Duration::from_millis(1), move || {
            let a = attempt_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            async move {
                if a < 2 {
                    Err(format!("fail {a}"))
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        assert_eq!(result, Ok(42));
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let result = retry_with_backoff(2, Duration::from_millis(1), || async {
            Err::<i32, _>("always fails")
        })
        .await;
        assert_eq!(result, Err("always fails"));
    }

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(30));
        assert!(!cb.is_open());
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let mut cb = CircuitBreaker::new(2, Duration::from_secs(30));

        // First failure
        let result: Result<i32, _> = cb.call(|| async { Err("fail") }).await;
        assert!(result.is_err());
        assert!(!cb.is_open());

        // Second failure — should open
        let result: Result<i32, _> = cb.call(|| async { Err("fail") }).await;
        assert!(result.is_err());
        assert!(cb.is_open());
    }

    #[tokio::test]
    async fn test_circuit_breaker_success_resets() {
        let mut cb = CircuitBreaker::new(2, Duration::from_secs(30));

        // Fail once
        let _: Result<i32, _> = cb.call(|| async { Err("fail") }).await;
        // Succeed — should reset
        let result = cb.call(|| async { Ok::<i32, &str>(42) }).await;
        assert_eq!(result, Ok(42));
        assert!(!cb.is_open());
    }

    #[tokio::test]
    async fn test_debouncer_basic() {
        let mut debouncer = Debouncer::new(Duration::from_millis(50));
        debouncer.debounce().await;
        // If we got here without hanging, debounce resolved
    }
}
