/// Problem: Testing
///
/// Master testing in Rust.
///
/// Key Concepts:
/// - Unit tests
/// - Integration tests
/// - Test organization
/// - Test helpers
/// - Mocking

/// Problem 1: Basic unit test
/// Write a basic test
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Problem 2: Test with assertions
/// Use different assertions
pub fn is_even(n: i32) -> bool {
    n % 2 == 0
}

/// Problem 3: Test with Result
/// Return Result from test
pub fn parse_number(s: &str) -> Result<i32, String> {
    s.parse().map_err(|_| format!("Failed to parse: {}", s))
}

/// Problem 4: Test organization
/// Organize tests
pub fn factorial(n: u32) -> u64 {
    match n {
        0 => 1,
        _ => n as u64 * factorial(n - 1),
    }
}

/// Problem 5: Test helper
/// Create test helpers
pub fn create_test_user(name: &str) -> String {
    format!("user_{}", name)
}

/// Problem 6: Test with setup
/// Setup test data
pub fn setup_test_data() -> Vec<i32> {
    vec![1, 2, 3, 4, 5]
}

/// Problem 7: Test with teardown
/// Cleanup after test
pub fn teardown_test_data(data: &mut Vec<i32>) {
    data.clear();
}

/// Problem 8: Test with mock (simulated)
/// Simulate mocking
pub trait DataStore {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
}

pub struct MockDataStore {
    data: std::collections::HashMap<String, String>,
}

impl MockDataStore {
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }
}

impl DataStore for MockDataStore {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
}

/// Problem 9: Test with error cases
/// Test error handling
pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

/// Problem 10: Test with edge cases
/// Test edge cases
pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

/// Problem 11: Test with property-based testing (simulated)
/// Simulate property-based testing
pub fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

/// Problem 12: Test with benchmarks (simulated)
/// Simulate benchmark
pub fn sum_vector(v: &[i32]) -> i32 {
    v.iter().sum()
}

/// Problem 13: Test with fixtures
/// Create test fixtures
pub struct TestFixture {
    pub name: String,
    pub value: i32,
}

impl TestFixture {
    pub fn new(name: &str, value: i32) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}

/// Problem 14: Test with snapshots (simulated)
/// Simulate snapshot testing
pub fn format_user(name: &str, age: u32) -> String {
    format!("User: {} (age: {})", name, age)
}

/// Problem 15: Test with coverage (simulated)
/// Simulate code coverage
pub fn complex_function(x: i32) -> i32 {
    if x > 0 {
        x * 2
    } else if x < 0 {
        x * -1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_is_even() {
        assert!(is_even(2));
        assert!(!is_even(3));
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("42"), Ok(42));
        assert!(parse_number("abc").is_err());
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(5), 120);
    }

    #[test]
    fn test_create_test_user() {
        assert_eq!(create_test_user("alice"), "user_alice");
    }

    #[test]
    fn test_setup_test_data() {
        let data = setup_test_data();
        assert_eq!(data.len(), 5);
    }

    #[test]
    fn test_teardown_test_data() {
        let mut data = vec![1, 2, 3];
        teardown_test_data(&mut data);
        assert!(data.is_empty());
    }

    #[test]
    fn test_mock_data_store() {
        let mut store = MockDataStore::new();
        store.set("key", "value");
        assert_eq!(store.get("key"), Some("value".to_string()));
    }

    #[test]
    fn test_divide() {
        assert_eq!(divide(10.0, 2.0), Ok(5.0));
        assert!(divide(10.0, 0.0).is_err());
    }

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
    }

    #[test]
    fn test_reverse_string() {
        assert_eq!(reverse_string("hello"), "olleh");
    }

    #[test]
    fn test_sum_vector() {
        assert_eq!(sum_vector(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_test_fixture() {
        let fixture = TestFixture::new("test", 42);
        assert_eq!(fixture.name, "test");
        assert_eq!(fixture.value, 42);
    }

    #[test]
    fn test_format_user() {
        assert_eq!(format_user("Alice", 30), "User: Alice (age: 30)");
    }

    #[test]
    fn test_complex_function() {
        assert_eq!(complex_function(5), 10);
        assert_eq!(complex_function(-5), 5);
        assert_eq!(complex_function(0), 0);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, -2), -3);
    }

    #[test]
    fn test_factorial_large() {
        assert_eq!(factorial(10), 3628800);
    }

    #[test]
    fn test_divide_negative() {
        assert_eq!(divide(-10.0, 2.0), Ok(-5.0));
    }

    #[test]
    fn test_fibonacci_large() {
        assert_eq!(fibonacci(20), 6765);
    }
}
