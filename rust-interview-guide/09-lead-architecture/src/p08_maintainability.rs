/// Problem: Maintainability
///
/// Master maintainability in Rust.
///
/// Key Concepts:
/// - Code organization
/// - Documentation
/// - Testing
/// - Refactoring
/// - Clean code

/// Problem 1: Code organization
/// Organize code properly
pub mod user {
    pub struct User {
        pub name: String,
        pub email: String,
    }

    impl User {
        pub fn new(name: &str, email: &str) -> Self {
            Self {
                name: name.to_string(),
                email: email.to_string(),
            }
        }
    }
}

pub mod auth {
    pub fn authenticate(username: &str, password: &str) -> bool {
        !username.is_empty() && !password.is_empty()
    }
}

/// Problem 2: Documentation
/// Write good documentation
/// # Examples
///
/// ```
/// use lead_architecture::p08_maintainability::add;
///
/// assert_eq!(add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// # Errors
///
/// Returns error if divisor is zero.
pub fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

/// Problem 3: Testing
/// Write good tests

/// Problem 4: Error handling
/// Handle errors properly
#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    InvalidInput(String),
    Internal(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal: {}", msg),
        }
    }
}

/// Problem 5: Clean functions
/// Write clean functions
pub fn process_user(name: &str, email: &str) -> Result<user::User, AppError> {
    if name.is_empty() {
        return Err(AppError::InvalidInput("Name required".to_string()));
    }
    if !email.contains('@') {
        return Err(AppError::InvalidInput("Invalid email".to_string()));
    }
    Ok(user::User::new(name, email))
}

/// Problem 6: Single responsibility
/// Follow single responsibility
pub struct UserValidator;

impl UserValidator {
    pub fn validate_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            Err("Name required".to_string())
        } else {
            Ok(())
        }
    }

    pub fn validate_email(email: &str) -> Result<(), String> {
        if email.contains('@') {
            Ok(())
        } else {
            Err("Invalid email".to_string())
        }
    }
}

/// Problem 7: DRY (Don't Repeat Yourself)
/// Avoid code duplication
pub fn validate_required(value: &str, field_name: &str) -> Result<(), String> {
    if value.is_empty() {
        Err(format!("{} required", field_name))
    } else {
        Ok(())
    }
}

/// Problem 8: Meaningful names
/// Use meaningful names
pub struct OrderProcessor {
    max_items: usize,
}

impl OrderProcessor {
    pub fn new(max_items: usize) -> Self {
        Self { max_items }
    }

    pub fn can_process(&self, item_count: usize) -> bool {
        item_count <= self.max_items
    }
}

/// Problem 9: Small functions
/// Keep functions small
pub fn calculate_total(price: f64, quantity: u32, tax_rate: f64) -> f64 {
    let subtotal = calculate_subtotal(price, quantity);
    let tax = calculate_tax(subtotal, tax_rate);
    subtotal + tax
}

fn calculate_subtotal(price: f64, quantity: u32) -> f64 {
    price * quantity as f64
}

fn calculate_tax(amount: f64, rate: f64) -> f64 {
    amount * rate
}

/// Problem 10: Avoid magic numbers
/// Use constants
const MAX_RETRIES: u32 = 3;
const TIMEOUT_MS: u64 = 5000;

pub fn retry_operation() -> u32 {
    MAX_RETRIES
}

/// Problem 11: Error messages
/// Provide good error messages
pub fn parse_port(port_str: &str) -> Result<u16, String> {
    port_str
        .parse::<u16>()
        .map_err(|_| format!("Invalid port '{}': must be a number between 0 and 65535", port_str))
}

/// Problem 12: Immutability
/// Prefer immutability
pub fn process_data(data: &[i32]) -> i32 {
    data.iter().sum()
}

/// Problem 13: Composition over inheritance
/// Use composition
pub struct Logger {
    prefix: String,
}

impl Logger {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    pub fn log(&self, message: &str) {
        println!("[{}] {}", self.prefix, message);
    }
}

pub struct Application {
    logger: Logger,
}

impl Application {
    pub fn new(logger: Logger) -> Self {
        Self { logger }
    }

    pub fn run(&self) {
        self.logger.log("Application started");
    }
}

/// Problem 14: Interface segregation
/// Keep interfaces small
pub trait Reader {
    fn read(&self) -> String;
}

pub trait Writer {
    fn write(&self, data: &str);
}

pub struct FileReader {
    path: String,
}

impl Reader for FileReader {
    fn read(&self) -> String {
        format!("Reading from {}", self.path)
    }
}

/// Problem 15: Dependency injection
/// Use dependency injection
pub trait DataStore {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: &str, value: &str);
}

pub struct Service {
    store: Box<dyn DataStore>,
}

impl Service {
    pub fn new(store: Box<dyn DataStore>) -> Self {
        Self { store }
    }

    pub fn get_data(&self, key: &str) -> Option<String> {
        self.store.get(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user() {
        let user = user::User::new("Alice", "alice@example.com");
        assert_eq!(user.name, "Alice");
    }

    #[test]
    fn test_auth() {
        assert!(auth::authenticate("user", "pass"));
        assert!(!auth::authenticate("", "pass"));
    }

    #[test]
    fn test_process_user() {
        assert!(process_user("Alice", "alice@example.com").is_ok());
        assert!(process_user("", "alice@example.com").is_err());
    }

    #[test]
    fn test_validator() {
        assert!(UserValidator::validate_name("Alice").is_ok());
        assert!(UserValidator::validate_name("").is_err());
    }

    #[test]
    fn test_calculate_total() {
        let total = calculate_total(10.0, 2, 0.1);
        assert!((total - 22.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_port() {
        assert_eq!(parse_port("8080"), Ok(8080));
        assert!(parse_port("invalid").is_err());
    }

    #[test]
    fn test_process_data() {
        assert_eq!(process_data(&[1, 2, 3, 4, 5]), 15);
    }

    #[test]
    fn test_composition() {
        let logger = Logger::new("APP");
        let app = Application::new(logger);
        app.run();
    }

    #[test]
    fn test_retry_operation() {
        assert_eq!(retry_operation(), 3);
    }
}
