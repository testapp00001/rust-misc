/// Problem: API Design
///
/// Master API design in Rust.
///
/// Key Concepts:
/// - Ergonomics
/// - Type safety
/// - Error handling
/// - Documentation
/// - Backward compatibility

/// Problem 1: Builder API
/// Design builder API
pub struct RequestBuilder {
    url: Option<String>,
    method: String,
    headers: Vec<(String, String)>,
}

impl RequestBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            method: "GET".to_string(),
            headers: Vec::new(),
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn method(mut self, method: &str) -> Self {
        self.method = method.to_string();
        self
    }

    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key.to_string(), value.to_string()));
        self
    }

    pub fn build(self) -> Result<Request, String> {
        Ok(Request {
            url: self.url.ok_or("URL required")?,
            method: self.method,
            headers: self.headers,
        })
    }
}

#[derive(Debug)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
}

/// Problem 2: Fluent API
/// Design fluent API
pub struct Query {
    table: String,
    conditions: Vec<String>,
    limit: Option<usize>,
}

impl Query {
    pub fn select(table: &str) -> Self {
        Self {
            table: table.to_string(),
            conditions: Vec::new(),
            limit: None,
        }
    }

    pub fn where_clause(mut self, condition: &str) -> Self {
        self.conditions.push(condition.to_string());
        self
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn build(self) -> String {
        let mut query = format!("SELECT * FROM {}", self.table);
        if !self.conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.conditions.join(" AND "));
        }
        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }
        query
    }
}

/// Problem 3: Type-safe API
/// Use types for safety
#[derive(Debug, Clone)]
pub struct Email(String);

impl Email {
    pub fn new(email: &str) -> Result<Self, String> {
        if email.contains('@') {
            Ok(Self(email.to_string()))
        } else {
            Err("Invalid email".to_string())
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Age(u32);

impl Age {
    pub fn new(age: u32) -> Result<Self, String> {
        if age > 0 && age < 150 {
            Ok(Self(age))
        } else {
            Err("Invalid age".to_string())
        }
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Problem 4: Error handling API
/// Design error handling
#[derive(Debug)]
pub enum ApiError {
    NotFound(String),
    Unauthorized,
    Internal(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::Internal(msg) => write!(f, "Internal: {}", msg),
        }
    }
}

/// Problem 5: Iterator API
/// Design iterator API
pub struct Range {
    start: i32,
    end: i32,
    current: i32,
}

impl Range {
    pub fn new(start: i32, end: i32) -> Self {
        Self { start, end, current: start }
    }
}

impl Iterator for Range {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.end {
            let value = self.current;
            self.current += 1;
            Some(value)
        } else {
            None
        }
    }
}

/// Problem 6: Collection API
/// Design collection API
pub struct Stack<T> {
    data: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn push(&mut self, item: T) {
        self.data.push(item);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    pub fn peek(&self) -> Option<&T> {
        self.data.last()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

/// Problem 7: Configuration API
/// Design configuration API
pub struct ConfigBuilder {
    host: String,
    port: u16,
    debug: bool,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
            debug: false,
        }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn build(self) -> Config {
        Config {
            host: self.host,
            port: self.port,
            debug: self.debug,
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
}

/// Problem 8: Validation API
/// Design validation API
pub struct Validator<T> {
    rules: Vec<Box<dyn Fn(&T) -> bool>>,
}

impl<T> Validator<T> {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Fn(&T) -> bool>) {
        self.rules.push(rule);
    }

    pub fn validate(&self, item: &T) -> bool {
        self.rules.iter().all(|rule| rule(item))
    }
}

/// Problem 9: Serialization API
/// Design serialization API
pub trait Serialize {
    fn serialize(&self) -> String;
}

pub trait Deserialize: Sized {
    fn deserialize(data: &str) -> Result<Self, String>;
}

/// Problem 10: Event API
/// Design event API
pub trait EventEmitter {
    fn on(&mut self, event: &str, callback: Box<dyn Fn(&str)>);
    fn emit(&self, event: &str, data: &str);
}

/// Problem 11: Middleware API
/// Design middleware API
pub trait Middleware {
    fn handle(&self, request: &str) -> String;
}

pub struct Pipeline {
    middlewares: Vec<Box<dyn Middleware>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    pub fn add(&mut self, middleware: Box<dyn Middleware>) {
        self.middlewares.push(middleware);
    }

    pub fn execute(&self, request: &str) -> String {
        let mut result = request.to_string();
        for middleware in &self.middlewares {
            result = middleware.handle(&result);
        }
        result
    }
}

/// Problem 12: Repository API
/// Design repository API
pub trait Repository<T> {
    fn find_by_id(&self, id: u32) -> Option<&T>;
    fn find_all(&self) -> Vec<&T>;
    fn save(&mut self, entity: T) -> T;
    fn delete(&mut self, id: u32) -> bool;
}

/// Problem 13: Cache API
/// Design cache API
pub trait Cache<K, V> {
    fn get(&self, key: &K) -> Option<&V>;
    fn set(&mut self, key: K, value: V);
    fn remove(&mut self, key: &K) -> Option<V>;
    fn clear(&mut self);
}

/// Problem 14: Logger API
/// Design logger API
pub trait Logger {
    fn log(&self, level: &str, message: &str);
    fn info(&self, message: &str) {
        self.log("INFO", message);
    }
    fn error(&self, message: &str) {
        self.log("ERROR", message);
    }
}

/// Problem 15: Async API
/// Design async API
pub trait AsyncProcessor {
    fn process(&self, input: &str) -> String;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_api() {
        let request = RequestBuilder::new()
            .url("https://example.com")
            .method("POST")
            .header("Content-Type", "application/json")
            .build()
            .unwrap();
        assert_eq!(request.url, "https://example.com");
    }

    #[test]
    fn test_fluent_api() {
        let query = Query::select("users")
            .where_clause("age > 18")
            .limit(10)
            .build();
        assert!(query.contains("users"));
    }

    #[test]
    fn test_type_safe_api() {
        assert!(Email::new("test@example.com").is_ok());
        assert!(Email::new("invalid").is_err());
        assert!(Age::new(25).is_ok());
        assert!(Age::new(0).is_err());
    }

    #[test]
    fn test_range_api() {
        let range: Vec<i32> = Range::new(0, 5).collect();
        assert_eq!(range, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_stack_api() {
        let mut stack = Stack::new();
        stack.push(1);
        stack.push(2);
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.peek(), Some(&1));
    }

    #[test]
    fn test_config_api() {
        let config = ConfigBuilder::new()
            .host("example.com")
            .port(3000)
            .debug(true)
            .build();
        assert_eq!(config.host, "example.com");
    }

    #[test]
    fn test_validator_api() {
        let mut validator = Validator::new();
        validator.add_rule(Box::new(|x: &i32| *x > 0));
        assert!(validator.validate(&42));
        assert!(!validator.validate(&-1));
    }

    #[test]
    fn test_pipeline_api() {
        let mut pipeline = Pipeline::new();
        // Would add middleware here
        assert_eq!(pipeline.execute("test"), "test");
    }
}
