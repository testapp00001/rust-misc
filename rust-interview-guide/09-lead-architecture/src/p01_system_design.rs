/// Problem: System Design
///
/// Master system design in Rust.
///
/// Key Concepts:
/// - Scalability
/// - Reliability
/// - Performance
/// - Maintainability
/// - Trade-offs

/// Problem 1: URL shortener
/// Design a URL shortener
pub struct UrlShortener {
    urls: std::collections::HashMap<String, String>,
    counter: u64,
}

impl UrlShortener {
    pub fn new() -> Self {
        Self {
            urls: std::collections::HashMap::new(),
            counter: 0,
        }
    }

    pub fn shorten(&mut self, url: &str) -> String {
        self.counter += 1;
        let short = format!("short_{}", self.counter);
        self.urls.insert(short.clone(), url.to_string());
        short
    }

    pub fn resolve(&self, short: &str) -> Option<&String> {
        self.urls.get(short)
    }
}

/// Problem 2: Rate limiter
/// Design a rate limiter
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

/// Problem 3: Cache system
/// Design a cache system
pub struct CacheSystem {
    data: std::collections::HashMap<String, (String, std::time::Instant)>,
    ttl: std::time::Duration,
}

impl CacheSystem {
    pub fn new(ttl: std::time::Duration) -> Self {
        Self {
            data: std::collections::HashMap::new(),
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        if let Some((value, time)) = self.data.get(key) {
            if time.elapsed() < self.ttl {
                return Some(value);
            }
        }
        None
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), (value.to_string(), std::time::Instant::now()));
    }
}

/// Problem 4: Message queue
/// Design a message queue
pub struct MessageQueue {
    queues: std::collections::HashMap<String, Vec<String>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        Self {
            queues: std::collections::HashMap::new(),
        }
    }

    pub fn publish(&mut self, topic: &str, message: &str) {
        self.queues
            .entry(topic.to_string())
            .or_insert_with(Vec::new)
            .push(message.to_string());
    }

    pub fn subscribe(&mut self, topic: &str) -> Option<String> {
        self.queues.get_mut(topic).and_then(|q| {
            if q.is_empty() {
                None
            } else {
                Some(q.remove(0))
            }
        })
    }
}

/// Problem 5: Load balancer
/// Design a load balancer
pub struct LoadBalancer {
    servers: Vec<String>,
    current: usize,
}

impl LoadBalancer {
    pub fn new(servers: Vec<String>) -> Self {
        Self { servers, current: 0 }
    }

    pub fn next_server(&mut self) -> &str {
        let server = &self.servers[self.current % self.servers.len()];
        self.current += 1;
        server
    }
}

/// Problem 6: Circuit breaker
/// Design a circuit breaker
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
}

/// Problem 7: Connection pool
/// Design a connection pool
pub struct ConnectionPool {
    connections: Vec<String>,
    max_size: usize,
}

impl ConnectionPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            connections: Vec::new(),
            max_size,
        }
    }

    pub fn get(&mut self) -> String {
        self.connections.pop().unwrap_or_else(|| "new_connection".to_string())
    }

    pub fn release(&mut self, conn: String) {
        if self.connections.len() < self.max_size {
            self.connections.push(conn);
        }
    }
}

/// Problem 8: Event system
/// Design an event system
pub struct EventSystem {
    listeners: std::collections::HashMap<String, Vec<Box<dyn Fn(&str)>>>,
}

impl EventSystem {
    pub fn new() -> Self {
        Self {
            listeners: std::collections::HashMap::new(),
        }
    }

    pub fn on(&mut self, event: &str, callback: Box<dyn Fn(&str)>) {
        self.listeners
            .entry(event.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
    }

    pub fn emit(&self, event: &str, data: &str) {
        if let Some(callbacks) = self.listeners.get(event) {
            for callback in callbacks {
                callback(data);
            }
        }
    }
}

/// Problem 9: Task scheduler
/// Design a task scheduler
pub struct TaskScheduler {
    tasks: Vec<(std::time::Instant, Box<dyn FnOnce()>)>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn schedule(&mut self, at: std::time::Instant, task: Box<dyn FnOnce()>) {
        self.tasks.push((at, task));
        self.tasks.sort_by(|a, b| a.0.cmp(&b.0));
    }

    pub fn run_pending(&mut self) {
        let now = std::time::Instant::now();
        while let Some((time, _)) = self.tasks.first() {
            if *time <= now {
                let (_, task) = self.tasks.remove(0);
                task();
            } else {
                break;
            }
        }
    }
}

/// Problem 10: Configuration manager
/// Design a configuration manager
pub struct ConfigManager {
    config: std::collections::HashMap<String, String>,
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.config.get(key)
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.config.insert(key.to_string(), value.to_string());
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.config.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
}

/// Problem 11: Metrics collector
/// Design a metrics collector
pub struct MetricsCollector {
    counters: std::collections::HashMap<String, u64>,
    gauges: std::collections::HashMap<String, f64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            counters: std::collections::HashMap::new(),
            gauges: std::collections::HashMap::new(),
        }
    }

    pub fn increment(&mut self, name: &str) {
        *self.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.get(name).copied().unwrap_or(0)
    }

    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).copied()
    }
}

/// Problem 12: Logger
/// Design a logger
pub struct Logger {
    entries: Vec<(String, String)>,
}

impl Logger {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    pub fn log(&mut self, level: &str, message: &str) {
        self.entries.push((level.to_string(), message.to_string()));
    }

    pub fn info(&mut self, message: &str) {
        self.log("INFO", message);
    }

    pub fn error(&mut self, message: &str) {
        self.log("ERROR", message);
    }

    pub fn get_entries(&self) -> &[(String, String)] {
        &self.entries
    }
}

/// Problem 13: Validator
/// Design a validator
pub struct Validator {
    rules: Vec<Box<dyn Fn(&str) -> bool>>,
}

impl Validator {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Fn(&str) -> bool>) {
        self.rules.push(rule);
    }

    pub fn validate(&self, input: &str) -> bool {
        self.rules.iter().all(|rule| rule(input))
    }
}

/// Problem 14: Serializer
/// Design a serializer
pub struct Serializer;

impl Serializer {
    pub fn to_json(data: &std::collections::HashMap<String, String>) -> String {
        let entries: Vec<String> = data
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect();
        format!("{{{}}}", entries.join(","))
    }
}

/// Problem 15: Middleware
/// Design middleware
pub struct Middleware {
    handlers: Vec<Box<dyn Fn(&str) -> String>>,
}

impl Middleware {
    pub fn new() -> Self {
        Self { handlers: Vec::new() }
    }

    pub fn add(&mut self, handler: Box<dyn Fn(&str) -> String>) {
        self.handlers.push(handler);
    }

    pub fn execute(&self, input: &str) -> String {
        let mut result = input.to_string();
        for handler in &self.handlers {
            result = handler(&result);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_shortener() {
        let mut shortener = UrlShortener::new();
        let short = shortener.shorten("https://example.com");
        assert_eq!(shortener.resolve(&short), Some(&"https://example.com".to_string()));
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
    fn test_cache_system() {
        let mut cache = CacheSystem::new(std::time::Duration::from_secs(1));
        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_message_queue() {
        let mut queue = MessageQueue::new();
        queue.publish("topic", "message");
        assert_eq!(queue.subscribe("topic"), Some("message".to_string()));
    }

    #[test]
    fn test_load_balancer() {
        let mut lb = LoadBalancer::new(vec!["s1".to_string(), "s2".to_string()]);
        assert_eq!(lb.next_server(), "s1");
        assert_eq!(lb.next_server(), "s2");
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
    fn test_connection_pool() {
        let mut pool = ConnectionPool::new(2);
        let conn = pool.get();
        pool.release(conn);
        assert_eq!(pool.connections.len(), 1);
    }

    #[test]
    fn test_event_system() {
        let mut system = EventSystem::new();
        system.on("test", Box::new(|_| {}));
        system.emit("test", "data");
    }

    #[test]
    fn test_config_manager() {
        let mut config = ConfigManager::new();
        config.set("key", "value");
        assert_eq!(config.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_metrics_collector() {
        let mut metrics = MetricsCollector::new();
        metrics.increment("requests");
        assert_eq!(metrics.get_counter("requests"), 1);
    }

    #[test]
    fn test_logger() {
        let mut logger = Logger::new();
        logger.info("test");
        assert_eq!(logger.get_entries().len(), 1);
    }

    #[test]
    fn test_validator() {
        let mut validator = Validator::new();
        validator.add_rule(Box::new(|s| !s.is_empty()));
        assert!(validator.validate("test"));
        assert!(!validator.validate(""));
    }

    #[test]
    fn test_serializer() {
        let mut data = std::collections::HashMap::new();
        data.insert("key".to_string(), "value".to_string());
        let json = Serializer::to_json(&data);
        assert!(json.contains("key"));
    }

    #[test]
    fn test_middleware() {
        let mut middleware = Middleware::new();
        middleware.add(Box::new(|s| format!("prefix_{}", s)));
        assert_eq!(middleware.execute("test"), "prefix_test");
    }
}
