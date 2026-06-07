/// Problem: Strategy Pattern
///
/// Master the strategy pattern in Rust.
///
/// Key Concepts:
/// - Strategy trait
/// - Context
/// - Algorithm encapsulation
/// - Runtime strategy selection
/// - Strategy composition

/// Problem 1: Basic strategy
/// Create basic strategy
pub trait SortStrategy {
    fn sort(&self, data: &mut Vec<i32>);
}

pub struct BubbleSort;
pub struct QuickSort;

impl SortStrategy for BubbleSort {
    fn sort(&self, data: &mut Vec<i32>) {
        let n = data.len();
        for i in 0..n {
            for j in 0..n - 1 - i {
                if data[j] > data[j + 1] {
                    data.swap(j, j + 1);
                }
            }
        }
    }
}

impl SortStrategy for QuickSort {
    fn sort(&self, data: &mut Vec<i32>) {
        data.sort();
    }
}

pub struct Sorter {
    strategy: Box<dyn SortStrategy>,
}

impl Sorter {
    pub fn new(strategy: Box<dyn SortStrategy>) -> Self {
        Self { strategy }
    }

    pub fn sort(&self, data: &mut Vec<i32>) {
        self.strategy.sort(data);
    }
}

/// Problem 2: Strategy with state
/// Strategy with internal state
pub trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Vec<u8>;
    fn decompress(&self, data: &[u8]) -> Vec<u8>;
}

pub struct NoCompression;
pub struct RunLengthEncoding;

impl CompressionStrategy for NoCompression {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }

    fn decompress(&self, data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }
}

impl CompressionStrategy for RunLengthEncoding {
    fn compress(&self, data: &[u8]) -> Vec<u8> {
        let mut compressed = Vec::new();
        let mut i = 0;
        while i < data.len() {
            let mut count = 1;
            while i + count < data.len() && data[i] == data[i + count] {
                count += 1;
            }
            compressed.push(count as u8);
            compressed.push(data[i]);
            i += count;
        }
        compressed
    }

    fn decompress(&self, data: &[u8]) -> Vec<u8> {
        let mut decompressed = Vec::new();
        let mut i = 0;
        while i < data.len() {
            let count = data[i] as usize;
            let value = data[i + 1];
            for _ in 0..count {
                decompressed.push(value);
            }
            i += 2;
        }
        decompressed
    }
}

/// Problem 3: Strategy with generics
/// Generic strategy
pub trait FilterStrategy<T> {
    fn filter(&self, item: &T) -> bool;
}

pub struct PositiveFilter;
pub struct EvenFilter;

impl FilterStrategy<i32> for PositiveFilter {
    fn filter(&self, item: &i32) -> bool {
        *item > 0
    }
}

impl FilterStrategy<i32> for EvenFilter {
    fn filter(&self, item: &i32) -> bool {
        *item % 2 == 0
    }
}

pub struct Filter<T> {
    strategy: Box<dyn FilterStrategy<T>>,
}

impl<T> Filter<T> {
    pub fn new(strategy: Box<dyn FilterStrategy<T>>) -> Self {
        Self { strategy }
    }

    pub fn apply<'a>(&self, items: &'a [T]) -> Vec<&'a T> {
        items.iter().filter(|item| self.strategy.filter(item)).collect()
    }
}

/// Problem 4: Strategy with enum
/// Use enum for strategies
#[derive(Debug, Clone)]
pub enum PricingStrategy {
    Standard,
    Premium,
    Discount(f64),
}

impl PricingStrategy {
    pub fn calculate(&self, base_price: f64) -> f64 {
        match self {
            PricingStrategy::Standard => base_price,
            PricingStrategy::Premium => base_price * 1.5,
            PricingStrategy::Discount(percent) => base_price * (1.0 - percent / 100.0),
        }
    }
}

/// Problem 5: Strategy composition
/// Compose strategies
pub trait Transform {
    fn apply(&self, value: i32) -> i32;
}

pub struct Double;
pub struct AddOne;
pub struct Negate;

impl Transform for Double {
    fn apply(&self, value: i32) -> i32 {
        value * 2
    }
}

impl Transform for AddOne {
    fn apply(&self, value: i32) -> i32 {
        value + 1
    }
}

impl Transform for Negate {
    fn apply(&self, value: i32) -> i32 {
        -value
    }
}

pub struct Pipeline {
    transforms: Vec<Box<dyn Transform>>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
        }
    }

    pub fn add(&mut self, transform: Box<dyn Transform>) {
        self.transforms.push(transform);
    }

    pub fn execute(&self, value: i32) -> i32 {
        self.transforms.iter().fold(value, |acc, t| t.apply(acc))
    }
}

/// Problem 6: Strategy with configuration
/// Strategy based on configuration
#[derive(Debug, Clone)]
pub enum SerializationFormat {
    Json,
    Xml,
    Yaml,
}

impl SerializationFormat {
    pub fn serialize(&self, data: &str) -> String {
        match self {
            SerializationFormat::Json => format!("{{\"data\":\"{}\"}}", data),
            SerializationFormat::Xml => format!("<data>{}</data>", data),
            SerializationFormat::Yaml => format!("data: {}", data),
        }
    }
}

/// Problem 7: Strategy with validation
/// Validation strategy
pub trait ValidationStrategy {
    fn validate(&self, input: &str) -> Result<(), String>;
}

pub struct EmailValidator;
pub struct PasswordValidator;

impl ValidationStrategy for EmailValidator {
    fn validate(&self, input: &str) -> Result<(), String> {
        if input.contains('@') {
            Ok(())
        } else {
            Err("Invalid email".to_string())
        }
    }
}

impl ValidationStrategy for PasswordValidator {
    fn validate(&self, input: &str) -> Result<(), String> {
        if input.len() >= 8 {
            Ok(())
        } else {
            Err("Password too short".to_string())
        }
    }
}

pub struct Validator {
    strategies: Vec<Box<dyn ValidationStrategy>>,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn add(&mut self, strategy: Box<dyn ValidationStrategy>) {
        self.strategies.push(strategy);
    }

    pub fn validate(&self, input: &str) -> Result<(), String> {
        for strategy in &self.strategies {
            strategy.validate(input)?;
        }
        Ok(())
    }
}

/// Problem 8: Strategy with caching
/// Caching strategy
pub trait CacheStrategy {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
}

pub struct NoCache;
pub struct MemoryCache {
    data: std::collections::HashMap<String, String>,
}

impl CacheStrategy for NoCache {
    fn get(&self, _key: &str) -> Option<String> {
        None
    }

    fn set(&mut self, _key: &str, _value: &str) {}
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }
}

impl CacheStrategy for MemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
}

/// Problem 9: Strategy with retry
/// Retry strategy
pub trait RetryStrategy {
    fn should_retry(&self, attempt: u32) -> bool;
    fn delay(&self, attempt: u32) -> u32;
}

pub struct ExponentialBackoff {
    max_attempts: u32,
    base_delay: u32,
}

impl ExponentialBackoff {
    pub fn new(max_attempts: u32, base_delay: u32) -> Self {
        Self {
            max_attempts,
            base_delay,
        }
    }
}

impl RetryStrategy for ExponentialBackoff {
    fn should_retry(&self, attempt: u32) -> bool {
        attempt < self.max_attempts
    }

    fn delay(&self, attempt: u32) -> u32 {
        self.base_delay * 2u32.pow(attempt)
    }
}

/// Problem 10: Strategy with logging
/// Logging strategy
pub trait LogStrategy {
    fn log(&self, message: &str);
}

pub struct ConsoleLogger;
pub struct FileLogger {
    logs: Vec<String>,
}

impl LogStrategy for ConsoleLogger {
    fn log(&self, message: &str) {
        println!("{}", message);
    }
}

impl FileLogger {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    pub fn get_logs(&self) -> &[String] {
        &self.logs
    }
}

impl LogStrategy for FileLogger {
    fn log(&self, message: &str) {
        // In real implementation, you'd write to file
        let _ = message;
    }
}

/// Problem 11: Strategy with authentication
/// Authentication strategy
pub trait AuthStrategy {
    fn authenticate(&self, credentials: &str) -> Result<String, String>;
}

pub struct BasicAuth;
pub struct TokenAuth;

impl AuthStrategy for BasicAuth {
    fn authenticate(&self, credentials: &str) -> Result<String, String> {
        if credentials.contains(':') {
            Ok("authenticated".to_string())
        } else {
            Err("Invalid credentials".to_string())
        }
    }
}

impl AuthStrategy for TokenAuth {
    fn authenticate(&self, token: &str) -> Result<String, String> {
        if token.starts_with("Bearer ") {
            Ok("authenticated".to_string())
        } else {
            Err("Invalid token".to_string())
        }
    }
}

/// Problem 12: Strategy with routing
/// Routing strategy
pub trait RouteStrategy {
    fn find_route(&self, from: &str, to: &str) -> Vec<String>;
}

pub struct ShortestRoute;
pub struct FastestRoute;

impl RouteStrategy for ShortestRoute {
    fn find_route(&self, from: &str, to: &str) -> Vec<String> {
        vec![from.to_string(), to.to_string()]
    }
}

impl RouteStrategy for FastestRoute {
    fn find_route(&self, from: &str, to: &str) -> Vec<String> {
        vec![from.to_string(), "highway".to_string(), to.to_string()]
    }
}

/// Problem 13: Strategy with rendering
/// Rendering strategy
pub trait RenderStrategy {
    fn render(&self, content: &str) -> String;
}

pub struct HtmlRenderer;
pub struct MarkdownRenderer;

impl RenderStrategy for HtmlRenderer {
    fn render(&self, content: &str) -> String {
        format!("<p>{}</p>", content)
    }
}

impl RenderStrategy for MarkdownRenderer {
    fn render(&self, content: &str) -> String {
        format!("**{}**", content)
    }
}

/// Problem 14: Strategy with serialization
/// Serialization strategy
pub trait SerializeStrategy {
    fn serialize(&self, data: &str) -> String;
    fn deserialize(&self, data: &str) -> String;
}

pub struct JsonSerializer;
pub struct XmlSerializer;

impl SerializeStrategy for JsonSerializer {
    fn serialize(&self, data: &str) -> String {
        format!("{{\"data\":\"{}\"}}", data)
    }

    fn deserialize(&self, data: &str) -> String {
        data.replace("{\"data\":\"", "").replace("\"}", "")
    }
}

impl SerializeStrategy for XmlSerializer {
    fn serialize(&self, data: &str) -> String {
        format!("<data>{}</data>", data)
    }

    fn deserialize(&self, data: &str) -> String {
        data.replace("<data>", "").replace("</data>", "")
    }
}

/// Problem 15: Strategy with execution
/// Execution strategy
pub trait ExecutionStrategy {
    fn execute(&self, code: &str) -> Result<String, String>;
}

pub struct DirectExecution;
pub struct SandboxedExecution;

impl ExecutionStrategy for DirectExecution {
    fn execute(&self, code: &str) -> Result<String, String> {
        Ok(format!("Executed: {}", code))
    }
}

impl ExecutionStrategy for SandboxedExecution {
    fn execute(&self, code: &str) -> Result<String, String> {
        if code.contains("unsafe") {
            Err("Unsafe code not allowed".to_string())
        } else {
            Ok(format!("Executed safely: {}", code))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_strategy() {
        let sorter = Sorter::new(Box::new(BubbleSort));
        let mut data = vec![3, 1, 4, 1, 5, 9, 2, 6];
        sorter.sort(&mut data);
        assert_eq!(data, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_compression_strategy() {
        let compressor = RunLengthEncoding;
        let data = vec![1, 1, 1, 2, 2, 3];
        let compressed = compressor.compress(&data);
        let decompressed = compressor.decompress(&compressed);
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_filter_strategy() {
        let filter = Filter::new(Box::new(PositiveFilter));
        let data = vec![-1, 2, -3, 4, -5];
        let result = filter.apply(&data);
        assert_eq!(result, vec![&2, &4]);
    }

    #[test]
    fn test_pricing_strategy() {
        let strategy = PricingStrategy::Discount(10.0);
        assert_eq!(strategy.calculate(100.0), 90.0);
    }

    #[test]
    fn test_pipeline() {
        let mut pipeline = Pipeline::new();
        pipeline.add(Box::new(Double));
        pipeline.add(Box::new(AddOne));
        assert_eq!(pipeline.execute(5), 11);
    }

    #[test]
    fn test_serialization_format() {
        let format = SerializationFormat::Json;
        assert_eq!(format.serialize("test"), "{\"data\":\"test\"}");
    }

    #[test]
    fn test_validation_strategy() {
        let mut validator = Validator::new();
        validator.add(Box::new(EmailValidator));
        assert!(validator.validate("test@example.com").is_ok());
        assert!(validator.validate("invalid").is_err());
    }

    #[test]
    fn test_cache_strategy() {
        let mut cache = MemoryCache::new();
        cache.set("key", "value");
        assert_eq!(cache.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_retry_strategy() {
        let strategy = ExponentialBackoff::new(3, 100);
        assert!(strategy.should_retry(0));
        assert!(!strategy.should_retry(3));
        assert_eq!(strategy.delay(1), 200);
    }

    #[test]
    fn test_auth_strategy() {
        let auth = BasicAuth;
        assert!(auth.authenticate("user:pass").is_ok());
        assert!(auth.authenticate("invalid").is_err());
    }

    #[test]
    fn test_route_strategy() {
        let router = ShortestRoute;
        let route = router.find_route("A", "B");
        assert_eq!(route, vec!["A", "B"]);
    }

    #[test]
    fn test_render_strategy() {
        let renderer = HtmlRenderer;
        assert_eq!(renderer.render("test"), "<p>test</p>");
    }

    #[test]
    fn test_serialize_strategy() {
        let serializer = JsonSerializer;
        let data = serializer.serialize("test");
        assert_eq!(serializer.deserialize(&data), "test");
    }

    #[test]
    fn test_execution_strategy() {
        let executor = SandboxedExecution;
        assert!(executor.execute("safe code").is_ok());
        assert!(executor.execute("unsafe code").is_err());
    }
}
