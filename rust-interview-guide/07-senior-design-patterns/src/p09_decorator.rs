/// Problem: Decorator Pattern
///
/// Master the decorator pattern in Rust.
///
/// Key Concepts:
/// - Component trait
/// - Concrete component
/// - Decorator
/// - Wrapping
/// - Behavior extension

/// Problem 1: Basic decorator
/// Create basic decorator
pub trait Component {
    fn operation(&self) -> String;
}

pub struct ConcreteComponent {
    name: String,
}

impl ConcreteComponent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl Component for ConcreteComponent {
    fn operation(&self) -> String {
        format!("Component: {}", self.name)
    }
}

pub struct Decorator {
    wrapped: Box<dyn Component>,
    prefix: String,
}

impl Decorator {
    pub fn new(wrapped: Box<dyn Component>, prefix: &str) -> Self {
        Self {
            wrapped,
            prefix: prefix.to_string(),
        }
    }
}

impl Component for Decorator {
    fn operation(&self) -> String {
        format!("{} {}", self.prefix, self.wrapped.operation())
    }
}

/// Problem 2: Logging decorator
/// Add logging to component
pub struct LoggingDecorator {
    wrapped: Box<dyn Component>,
    log: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl LoggingDecorator {
    pub fn new(
        wrapped: Box<dyn Component>,
        log: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    ) -> Self {
        Self { wrapped, log }
    }
}

impl Component for LoggingDecorator {
    fn operation(&self) -> String {
        let result = self.wrapped.operation();
        self.log.lock().unwrap().push(format!("Called: {}", result));
        result
    }
}

/// Problem 3: Timing decorator
/// Add timing to component
pub struct TimingDecorator {
    wrapped: Box<dyn Component>,
    duration: std::sync::Mutex<Option<std::time::Duration>>,
}

impl TimingDecorator {
    pub fn new(wrapped: Box<dyn Component>) -> Self {
        Self {
            wrapped,
            duration: std::sync::Mutex::new(None),
        }
    }

    pub fn duration(&self) -> Option<std::time::Duration> {
        let result = *self.duration.lock().unwrap(); result
    }
}

impl Component for TimingDecorator {
    fn operation(&self) -> String {
        let start = std::time::Instant::now();
        let result = self.wrapped.operation();
        *self.duration.lock().unwrap() = Some(start.elapsed());
        result
    }
}

/// Problem 4: Caching decorator
/// Add caching to component
pub struct CachingDecorator {
    wrapped: Box<dyn Component>,
    cache: std::sync::Mutex<Option<String>>,
}

impl CachingDecorator {
    pub fn new(wrapped: Box<dyn Component>) -> Self {
        Self {
            wrapped,
            cache: std::sync::Mutex::new(None),
        }
    }
}

impl Component for CachingDecorator {
    fn operation(&self) -> String {
        let mut cache = self.cache.lock().unwrap();
        if let Some(ref result) = *cache {
            return result.clone();
        }
        let result = self.wrapped.operation();
        *cache = Some(result.clone());
        result
    }
}

/// Problem 5: Validation decorator
/// Add validation to component
pub struct ValidationDecorator {
    wrapped: Box<dyn Component>,
    validator: Box<dyn Fn(&str) -> bool>,
}

impl ValidationDecorator {
    pub fn new(wrapped: Box<dyn Component>, validator: Box<dyn Fn(&str) -> bool>) -> Self {
        Self { wrapped, validator }
    }
}

impl Component for ValidationDecorator {
    fn operation(&self) -> String {
        let result = self.wrapped.operation();
        if (self.validator)(&result) {
            result
        } else {
            "Validation failed".to_string()
        }
    }
}

/// Problem 6: Transform decorator
/// Transform component output
pub struct TransformDecorator {
    wrapped: Box<dyn Component>,
    transform: Box<dyn Fn(String) -> String>,
}

impl TransformDecorator {
    pub fn new(wrapped: Box<dyn Component>, transform: Box<dyn Fn(String) -> String>) -> Self {
        Self { wrapped, transform }
    }
}

impl Component for TransformDecorator {
    fn operation(&self) -> String {
        let result = self.wrapped.operation();
        (self.transform)(result)
    }
}

/// Problem 7: Retry decorator
/// Add retry logic
pub struct RetryDecorator {
    wrapped: Box<dyn Component>,
    max_retries: u32,
}

impl RetryDecorator {
    pub fn new(wrapped: Box<dyn Component>, max_retries: u32) -> Self {
        Self { wrapped, max_retries }
    }
}

impl Component for RetryDecorator {
    fn operation(&self) -> String {
        for _ in 0..self.max_retries {
            let result = self.wrapped.operation();
            if !result.contains("error") {
                return result;
            }
        }
        "Failed after retries".to_string()
    }
}

/// Problem 8: Rate limiting decorator
/// Add rate limiting
pub struct RateLimitDecorator {
    wrapped: Box<dyn Component>,
    last_call: std::sync::Mutex<std::time::Instant>,
    interval: std::time::Duration,
}

impl RateLimitDecorator {
    pub fn new(wrapped: Box<dyn Component>, interval: std::time::Duration) -> Self {
        Self {
            wrapped,
            last_call: std::sync::Mutex::new(std::time::Instant::now() - interval),
            interval,
        }
    }
}

impl Component for RateLimitDecorator {
    fn operation(&self) -> String {
        let mut last_call = self.last_call.lock().unwrap();
        let now = std::time::Instant::now();
        if now.duration_since(*last_call) < self.interval {
            return "Rate limited".to_string();
        }
        *last_call = now;
        self.wrapped.operation()
    }
}

/// Problem 9: Circuit breaker decorator
/// Add circuit breaker
pub struct CircuitBreakerDecorator {
    wrapped: Box<dyn Component>,
    failures: std::sync::Mutex<u32>,
    threshold: u32,
    state: std::sync::Mutex<CircuitState>,
}

#[derive(Debug, Clone, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreakerDecorator {
    pub fn new(wrapped: Box<dyn Component>, threshold: u32) -> Self {
        Self {
            wrapped,
            failures: std::sync::Mutex::new(0),
            threshold,
            state: std::sync::Mutex::new(CircuitState::Closed),
        }
    }
}

impl Component for CircuitBreakerDecorator {
    fn operation(&self) -> String {
        let state = self.state.lock().unwrap().clone();
        match state {
            CircuitState::Open => "Circuit open".to_string(),
            CircuitState::HalfOpen | CircuitState::Closed => {
                let result = self.wrapped.operation();
                if result.contains("error") {
                    let mut failures = self.failures.lock().unwrap();
                    *failures += 1;
                    if *failures >= self.threshold {
                        *self.state.lock().unwrap() = CircuitState::Open;
                    }
                } else {
                    *self.failures.lock().unwrap() = 0;
                    *self.state.lock().unwrap() = CircuitState::Closed;
                }
                result
            }
        }
    }
}

/// Problem 10: Metrics decorator
/// Add metrics collection
pub struct MetricsDecorator {
    wrapped: Box<dyn Component>,
    call_count: std::sync::Mutex<u32>,
    error_count: std::sync::Mutex<u32>,
}

impl MetricsDecorator {
    pub fn new(wrapped: Box<dyn Component>) -> Self {
        Self {
            wrapped,
            call_count: std::sync::Mutex::new(0),
            error_count: std::sync::Mutex::new(0),
        }
    }

    pub fn call_count(&self) -> u32 {
        let result = *self.call_count.lock().unwrap(); result
    }

    pub fn error_count(&self) -> u32 {
        let result = *self.error_count.lock().unwrap(); result
    }
}

impl Component for MetricsDecorator {
    fn operation(&self) -> String {
        *self.call_count.lock().unwrap() += 1;
        let result = self.wrapped.operation();
        if result.contains("error") {
            *self.error_count.lock().unwrap() += 1;
        }
        result
    }
}

/// Problem 11: Authentication decorator
/// Add authentication
pub struct AuthDecorator {
    wrapped: Box<dyn Component>,
    authenticated: bool,
}

impl AuthDecorator {
    pub fn new(wrapped: Box<dyn Component>, authenticated: bool) -> Self {
        Self {
            wrapped,
            authenticated,
        }
    }
}

impl Component for AuthDecorator {
    fn operation(&self) -> String {
        if self.authenticated {
            self.wrapped.operation()
        } else {
            "Unauthorized".to_string()
        }
    }
}

/// Problem 12: Compression decorator
/// Add compression (simulated)
pub struct CompressionDecorator {
    wrapped: Box<dyn Component>,
}

impl CompressionDecorator {
    pub fn new(wrapped: Box<dyn Component>) -> Self {
        Self { wrapped }
    }
}

impl Component for CompressionDecorator {
    fn operation(&self) -> String {
        let result = self.wrapped.operation();
        format!("Compressed({})", result)
    }
}

/// Problem 13: Encryption decorator
/// Add encryption (simulated)
pub struct EncryptionDecorator {
    wrapped: Box<dyn Component>,
}

impl EncryptionDecorator {
    pub fn new(wrapped: Box<dyn Component>) -> Self {
        Self { wrapped }
    }
}

impl Component for EncryptionDecorator {
    fn operation(&self) -> String {
        let result = self.wrapped.operation();
        format!("Encrypted({})", result)
    }
}

/// Problem 14: Serialization decorator
/// Add serialization
pub struct SerializationDecorator {
    wrapped: Box<dyn Component>,
    format: String,
}

impl SerializationDecorator {
    pub fn new(wrapped: Box<dyn Component>, format: &str) -> Self {
        Self {
            wrapped,
            format: format.to_string(),
        }
    }
}

impl Component for SerializationDecorator {
    fn operation(&self) -> String {
        let result = self.wrapped.operation();
        format!("Serialized({}, {})", self.format, result)
    }
}

/// Problem 15: Chaining decorators
/// Chain multiple decorators
pub struct ChainedDecorator {
    decorators: Vec<Box<dyn Component>>,
}

impl ChainedDecorator {
    pub fn new(decorators: Vec<Box<dyn Component>>) -> Self {
        Self { decorators }
    }
}

impl Component for ChainedDecorator {
    fn operation(&self) -> String {
        let mut result = String::new();
        for decorator in &self.decorators {
            result = decorator.operation();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = Decorator::new(Box::new(component), "Prefix");
        assert!(decorator.operation().contains("Prefix"));
    }

    #[test]
    fn test_logging_decorator() {
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let component = ConcreteComponent::new("test");
        let decorator = LoggingDecorator::new(Box::new(component), log.clone());
        decorator.operation();
        assert_eq!(log.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_timing_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = TimingDecorator::new(Box::new(component));
        decorator.operation();
        assert!(decorator.duration().is_some());
    }

    #[test]
    fn test_caching_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = CachingDecorator::new(Box::new(component));
        let result1 = decorator.operation();
        let result2 = decorator.operation();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_validation_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = ValidationDecorator::new(
            Box::new(component),
            Box::new(|s| s.contains("test")),
        );
        assert!(decorator.operation().contains("test"));
    }

    #[test]
    fn test_transform_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = TransformDecorator::new(
            Box::new(component),
            Box::new(|s| s.to_uppercase()),
        );
        assert!(decorator.operation().contains("TEST"));
    }

    #[test]
    fn test_retry_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = RetryDecorator::new(Box::new(component), 3);
        assert!(decorator.operation().contains("test"));
    }

    #[test]
    fn test_rate_limit_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = RateLimitDecorator::new(
            Box::new(component),
            std::time::Duration::from_secs(1),
        );
        assert!(decorator.operation().contains("test"));
    }

    #[test]
    fn test_circuit_breaker_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = CircuitBreakerDecorator::new(Box::new(component), 3);
        assert!(decorator.operation().contains("test"));
    }

    #[test]
    fn test_metrics_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = MetricsDecorator::new(Box::new(component));
        decorator.operation();
        assert_eq!(decorator.call_count(), 1);
    }

    #[test]
    fn test_auth_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = AuthDecorator::new(Box::new(component), true);
        assert!(decorator.operation().contains("test"));
    }

    #[test]
    fn test_compression_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = CompressionDecorator::new(Box::new(component));
        assert!(decorator.operation().contains("Compressed"));
    }

    #[test]
    fn test_encryption_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = EncryptionDecorator::new(Box::new(component));
        assert!(decorator.operation().contains("Encrypted"));
    }

    #[test]
    fn test_serialization_decorator() {
        let component = ConcreteComponent::new("test");
        let decorator = SerializationDecorator::new(Box::new(component), "json");
        assert!(decorator.operation().contains("json"));
    }

    #[test]
    fn test_chained_decorator() {
        let component = ConcreteComponent::new("test");
        let decorators: Vec<Box<dyn Component>> = vec![
            Box::new(CompressionDecorator::new(Box::new(component))),
        ];
        let chained = ChainedDecorator::new(decorators);
        assert!(chained.operation().contains("Compressed"));
    }
}
