/// Problem: Reliability
///
/// Master reliability in Rust.
///
/// Key Concepts:
/// - Error handling
/// - Retry logic
/// - Circuit breakers
/// - Health checks
/// - Graceful degradation

/// Problem 1: Error handling
/// Handle errors gracefully
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Unauthorized,
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Internal(msg) => write!(f, "Internal: {}", msg),
        }
    }
}

/// Problem 2: Retry logic
/// Implement retry logic
pub struct RetryPolicy {
    max_attempts: u32,
    delay: std::time::Duration,
}

impl RetryPolicy {
    pub fn new(max_attempts: u32, delay: std::time::Duration) -> Self {
        Self { max_attempts, delay }
    }

    pub fn execute<F, T>(&self, mut f: F) -> Result<T, String>
    where
        F: FnMut() -> Result<T, String>,
    {
        for attempt in 0..self.max_attempts {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == self.max_attempts - 1 {
                        return Err(e);
                    }
                    std::thread::sleep(self.delay);
                }
            }
        }
        Err("Max attempts reached".to_string())
    }
}

/// Problem 3: Circuit breaker
/// Implement circuit breaker
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreaker {
    state: CircuitState,
    failures: u32,
    threshold: u32,
    timeout: std::time::Duration,
    last_failure: Option<std::time::Instant>,
}

impl CircuitBreaker {
    pub fn new(threshold: u32, timeout: std::time::Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failures: 0,
            threshold,
            timeout,
            last_failure: None,
        }
    }

    pub fn record_success(&mut self) {
        self.failures = 0;
        self.state = CircuitState::Closed;
    }

    pub fn record_failure(&mut self) {
        self.failures += 1;
        self.last_failure = Some(std::time::Instant::now());
        if self.failures >= self.threshold {
            self.state = CircuitState::Open;
        }
    }

    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure {
                    if last.elapsed() >= self.timeout {
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    pub fn state(&self) -> &CircuitState {
        &self.state
    }
}

/// Problem 4: Health check
/// Implement health checks
pub struct HealthCheck {
    checks: Vec<Box<dyn Fn() -> bool>>,
}

impl HealthCheck {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn add_check(&mut self, check: Box<dyn Fn() -> bool>) {
        self.checks.push(check);
    }

    pub fn is_healthy(&self) -> bool {
        self.checks.iter().all(|check| check())
    }
}

/// Problem 5: Graceful degradation
/// Implement graceful degradation
pub struct Service {
    primary: bool,
    fallback: String,
}

impl Service {
    pub fn new(fallback: &str) -> Self {
        Self {
            primary: true,
            fallback: fallback.to_string(),
        }
    }

    pub fn set_primary(&mut self, available: bool) {
        self.primary = available;
    }

    pub fn get_data(&self) -> String {
        if self.primary {
            "Primary data".to_string()
        } else {
            self.fallback.clone()
        }
    }
}

/// Problem 6: Timeout handling
/// Implement timeout handling
pub struct TimeoutHandler {
    timeout: std::time::Duration,
}

impl TimeoutHandler {
    pub fn new(timeout: std::time::Duration) -> Self {
        Self { timeout }
    }

    pub fn execute<F, T>(&self, f: F) -> Result<T, String>
    where
        F: FnOnce() -> T,
    {
        let start = std::time::Instant::now();
        let result = f();
        if start.elapsed() > self.timeout {
            Err("Timeout".to_string())
        } else {
            Ok(result)
        }
    }
}

/// Problem 7: Bulkhead pattern
/// Implement bulkhead
pub struct Bulkhead {
    max_concurrent: usize,
    current: usize,
}

impl Bulkhead {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            current: 0,
        }
    }

    pub fn execute<F, T>(&mut self, f: F) -> Result<T, String>
    where
        F: FnOnce() -> T,
    {
        if self.current >= self.max_concurrent {
            return Err("Bulkhead full".to_string());
        }
        self.current += 1;
        let result = f();
        self.current -= 1;
        Ok(result)
    }
}

/// Problem 8: Fallback pattern
/// Implement fallback
pub struct FallbackService {
    primary: Box<dyn Fn() -> Result<String, String>>,
    fallback: Box<dyn Fn() -> String>,
}

impl FallbackService {
    pub fn new(
        primary: Box<dyn Fn() -> Result<String, String>>,
        fallback: Box<dyn Fn() -> String>,
    ) -> Self {
        Self { primary, fallback }
    }

    pub fn execute(&self) -> String {
        (self.primary)().unwrap_or_else(|_| (self.fallback)())
    }
}

/// Problem 9: Retry with backoff
/// Implement retry with backoff
pub struct BackoffRetry {
    max_attempts: u32,
    base_delay: std::time::Duration,
}

impl BackoffRetry {
    pub fn new(max_attempts: u32, base_delay: std::time::Duration) -> Self {
        Self { max_attempts, base_delay }
    }

    pub fn execute<F, T>(&self, mut f: F) -> Result<T, String>
    where
        F: FnMut() -> Result<T, String>,
    {
        for attempt in 0..self.max_attempts {
            match f() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempt == self.max_attempts - 1 {
                        return Err(e);
                    }
                    let delay = self.base_delay * 2u32.pow(attempt);
                    std::thread::sleep(delay);
                }
            }
        }
        Err("Max attempts reached".to_string())
    }
}

/// Problem 10: Idempotency
/// Implement idempotency
pub struct IdempotentService {
    processed: std::collections::HashSet<String>,
}

impl IdempotentService {
    pub fn new() -> Self {
        Self {
            processed: std::collections::HashSet::new(),
        }
    }

    pub fn process(&mut self, id: &str) -> bool {
        if self.processed.contains(id) {
            false // Already processed
        } else {
            self.processed.insert(id.to_string());
            true // Newly processed
        }
    }

    pub fn is_processed(&self, id: &str) -> bool {
        self.processed.contains(id)
    }
}

/// Problem 11: Dead letter queue
/// Implement dead letter queue
pub struct DeadLetterQueue {
    queue: Vec<String>,
}

impl DeadLetterQueue {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn add(&mut self, message: String) {
        self.queue.push(message);
    }

    pub fn drain(&mut self) -> Vec<String> {
        std::mem::take(&mut self.queue)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

/// Problem 12: Health endpoint
/// Implement health endpoint
pub struct HealthEndpoint {
    healthy: bool,
}

impl HealthEndpoint {
    pub fn new() -> Self {
        Self { healthy: true }
    }

    pub fn set_healthy(&mut self, healthy: bool) {
        self.healthy = healthy;
    }

    pub fn check(&self) -> (u16, String) {
        if self.healthy {
            (200, "OK".to_string())
        } else {
            (503, "Service Unavailable".to_string())
        }
    }
}

/// Problem 13: Graceful shutdown
/// Implement graceful shutdown
pub struct GracefulShutdown {
    shutdown_requested: bool,
}

impl GracefulShutdown {
    pub fn new() -> Self {
        Self {
            shutdown_requested: false,
        }
    }

    pub fn request_shutdown(&mut self) {
        self.shutdown_requested = true;
    }

    pub fn should_shutdown(&self) -> bool {
        self.shutdown_requested
    }

    pub fn shutdown(&mut self) {
        // Cleanup resources
        self.shutdown_requested = false;
    }
}

/// Problem 14: Rate limiting
/// Implement rate limiting
pub struct RateLimiter {
    requests: std::collections::HashMap<String, Vec<std::time::Instant>>,
    max_requests: usize,
    window: std::time::Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: std::time::Duration) -> Self {
        Self {
            requests: std::collections::HashMap::new(),
            max_requests,
            window,
        }
    }

    pub fn allow(&mut self, key: &str) -> bool {
        let now = std::time::Instant::now();
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        requests.retain(|t| now.duration_since(*t) < self.window);
        if requests.len() < self.max_requests {
            requests.push(now);
            true
        } else {
            false
        }
    }
}

/// Problem 15: Monitoring
/// Implement monitoring
pub struct Monitor {
    metrics: std::collections::HashMap<String, u64>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            metrics: std::collections::HashMap::new(),
        }
    }

    pub fn increment(&mut self, name: &str) {
        *self.metrics.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn get(&self, name: &str) -> u64 {
        self.metrics.get(name).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy() {
        let policy = RetryPolicy::new(3, std::time::Duration::from_millis(10));
        let mut attempts = 0;
        let result = policy.execute(|| {
            attempts += 1;
            if attempts < 3 {
                Err("Not ready".to_string())
            } else {
                Ok(42)
            }
        });
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_circuit_breaker() {
        let mut cb = CircuitBreaker::new(3, std::time::Duration::from_secs(1));
        assert!(cb.allow_request());
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_health_check() {
        let mut health = HealthCheck::new();
        health.add_check(Box::new(|| true));
        assert!(health.is_healthy());
    }

    #[test]
    fn test_graceful_degradation() {
        let mut service = Service::new("fallback");
        assert_eq!(service.get_data(), "Primary data");
        service.set_primary(false);
        assert_eq!(service.get_data(), "fallback");
    }

    #[test]
    fn test_timeout_handler() {
        let handler = TimeoutHandler::new(std::time::Duration::from_secs(1));
        let result = handler.execute(|| 42);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_bulkhead() {
        let mut bulkhead = Bulkhead::new(2);
        assert!(bulkhead.execute(|| 42).is_ok());
        assert!(bulkhead.execute(|| 42).is_ok());
    }

    #[test]
    fn test_fallback_service() {
        let service = FallbackService::new(
            Box::new(|| Err("Error".to_string())),
            Box::new(|| "fallback".to_string()),
        );
        assert_eq!(service.execute(), "fallback");
    }

    #[test]
    fn test_backoff_retry() {
        let retry = BackoffRetry::new(3, std::time::Duration::from_millis(10));
        let mut attempts = 0;
        let result = retry.execute(|| {
            attempts += 1;
            if attempts < 3 {
                Err("Not ready".to_string())
            } else {
                Ok(42)
            }
        });
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_idempotent_service() {
        let mut service = IdempotentService::new();
        assert!(service.process("id1"));
        assert!(!service.process("id1"));
    }

    #[test]
    fn test_dead_letter_queue() {
        let mut dlq = DeadLetterQueue::new();
        dlq.add("message".to_string());
        assert_eq!(dlq.len(), 1);
        let messages = dlq.drain();
        assert_eq!(messages, vec!["message"]);
    }

    #[test]
    fn test_health_endpoint() {
        let mut endpoint = HealthEndpoint::new();
        assert_eq!(endpoint.check(), (200, "OK".to_string()));
        endpoint.set_healthy(false);
        assert_eq!(endpoint.check(), (503, "Service Unavailable".to_string()));
    }

    #[test]
    fn test_graceful_shutdown() {
        let mut shutdown = GracefulShutdown::new();
        assert!(!shutdown.should_shutdown());
        shutdown.request_shutdown();
        assert!(shutdown.should_shutdown());
        shutdown.shutdown();
        assert!(!shutdown.should_shutdown());
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3, std::time::Duration::from_secs(1));
        assert!(limiter.allow("user1"));
        assert!(limiter.allow("user1"));
        assert!(limiter.allow("user1"));
        assert!(!limiter.allow("user1"));
    }

    #[test]
    fn test_monitor() {
        let mut monitor = Monitor::new();
        monitor.increment("requests");
        monitor.increment("requests");
        assert_eq!(monitor.get("requests"), 2);
    }
}
