/// Problem: Documentation
///
/// Master documentation in Rust.
///
/// Key Concepts:
/// - Doc comments
/// - Examples
/// - Safety documentation
/// - API documentation
/// - Architecture documentation

/// Problem 1: Basic doc comments
/// Write basic documentation
/// This function adds two numbers.
///
/// # Arguments
///
/// * `a` - First number
/// * `b` - Second number
///
/// # Returns
///
/// The sum of `a` and `b`.
///
/// # Examples
///
/// ```
/// use lead_architecture::p10_documentation::add;
///
/// assert_eq!(add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Problem 2: Safety documentation
/// Document safety requirements
/// # Safety
///
/// The pointer must be valid and aligned.
pub unsafe fn unsafe_function(ptr: *const i32) -> i32 {
    *ptr
}

/// Problem 3: Error documentation
/// Document error conditions
/// # Errors
///
/// Returns `Err` if:
/// - The input is empty
/// - The input is not a valid number
pub fn parse_number(input: &str) -> Result<i32, String> {
    input.parse().map_err(|_| format!("Invalid number: {}", input))
}

/// Problem 4: Panics documentation
/// Document panic conditions
/// # Panics
///
/// Panics if the index is out of bounds.
pub fn get_element(data: &[i32], index: usize) -> i32 {
    data[index]
}

/// Problem 5: Module documentation
/// Document modules
/// # User Module
///
/// This module contains user-related types and functions.
pub mod user {
    /// Represents a user.
    pub struct User {
        /// The user's name.
        pub name: String,
        /// The user's email.
        pub email: String,
    }

    impl User {
        /// Creates a new user.
        ///
        /// # Arguments
        ///
        /// * `name` - The user's name
        /// * `email` - The user's email
        pub fn new(name: &str, email: &str) -> Self {
            Self {
                name: name.to_string(),
                email: email.to_string(),
            }
        }
    }
}

/// Problem 6: Struct documentation
/// Document structs
/// A configuration builder.
///
/// Use this to build configuration objects with a fluent API.
///
/// # Examples
///
/// ```
/// use lead_architecture::p10_documentation::ConfigBuilder;
///
/// let config = ConfigBuilder::new()
///     .host("localhost")
///     .port(8080)
///     .build();
/// ```
pub struct ConfigBuilder {
    host: String,
    port: u16,
}

impl ConfigBuilder {
    /// Creates a new config builder with default values.
    pub fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8080,
        }
    }

    /// Sets the host.
    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    /// Sets the port.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Builds the configuration.
    pub fn build(self) -> Config {
        Config {
            host: self.host,
            port: self.port,
        }
    }
}

/// Configuration object.
#[derive(Debug)]
pub struct Config {
    /// The host address.
    pub host: String,
    /// The port number.
    pub port: u16,
}

/// Problem 7: Enum documentation
/// Document enums
/// Represents application errors.
#[derive(Debug)]
pub enum AppError {
    /// The requested resource was not found.
    NotFound(String),
    /// The request was unauthorized.
    Unauthorized,
    /// An internal error occurred.
    Internal(String),
}

/// Problem 8: Trait documentation
/// Document traits
/// A trait for types that can be validated.
pub trait Validatable {
    /// Validates the value.
    ///
    /// # Returns
    ///
    /// `Ok(())` if valid, `Err(message)` if invalid.
    fn validate(&self) -> Result<(), String>;
}

/// Problem 9: Method documentation
/// Document methods
pub struct Calculator;

impl Calculator {
    /// Adds two numbers.
    ///
    /// # Examples
    ///
    /// ```
    /// use lead_architecture::p10_documentation::Calculator;
    ///
    /// assert_eq!(Calculator::add(2, 3), 5);
    /// ```
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    /// Subtracts two numbers.
    pub fn subtract(a: i32, b: i32) -> i32 {
        a - b
    }
}

/// Problem 10: Type alias documentation
/// Document type aliases
/// A type alias for a callback function.
pub type Callback = Box<dyn Fn(i32) -> i32>;

/// Problem 11: Constant documentation
/// Document constants
/// The maximum number of retries.
pub const MAX_RETRIES: u32 = 3;

/// The default timeout in milliseconds.
pub const DEFAULT_TIMEOUT_MS: u64 = 5000;

/// Problem 12: Feature documentation
/// Document features
/// # Features
///
/// - `json` - Enables JSON serialization
/// - `yaml` - Enables YAML serialization
pub struct Serializer;

/// Problem 13: Architecture documentation
/// Document architecture
/// # Architecture
///
/// The system follows a layered architecture:
///
/// 1. **Presentation Layer** - Handles HTTP requests
/// 2. **Business Layer** - Contains business logic
/// 3. **Data Layer** - Handles data persistence
pub struct System;

/// Problem 14: API documentation
/// Document API
/// # API
///
/// ## Endpoints
///
/// - `GET /users` - List all users
/// - `POST /users` - Create a user
/// - `GET /users/:id` - Get a user
/// - `PUT /users/:id` - Update a user
/// - `DELETE /users/:id` - Delete a user
pub struct Api;

/// Problem 15: Changelog documentation
/// Document changes
/// # Changelog
///
/// ## 1.0.0
///
/// - Initial release
/// - Added user management
/// - Added authentication
pub struct Changelog;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("42"), Ok(42));
        assert!(parse_number("abc").is_err());
    }

    #[test]
    fn test_user() {
        let user = user::User::new("Alice", "alice@example.com");
        assert_eq!(user.name, "Alice");
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .host("example.com")
            .port(3000)
            .build();
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, 3000);
    }

    #[test]
    fn test_calculator() {
        assert_eq!(Calculator::add(2, 3), 5);
        assert_eq!(Calculator::subtract(5, 3), 2);
    }
}
