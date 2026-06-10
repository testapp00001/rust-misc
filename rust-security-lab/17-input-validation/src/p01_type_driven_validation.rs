//! # Lesson 01: Type-Driven Validation
//!
//! ## The Problem
//!
//! Most security bugs stem from a simple cause: the program accepts data that
//! violates its assumptions. A function that expects an email address receives
//! a SQL injection payload. A function that expects a positive integer receives
//! -1. A function that expects a filename receives `../../etc/passwd`.
//!
//! In dynamically-typed languages, you must remember to validate at every call
//! site. In Rust, you can encode validation in the type system so that it is
//! **impossible** to construct an invalid value.
//!
//! ## The Newtype Pattern
//!
//! A newtype is a single-field struct that wraps a primitive type:
//!
//! ```ignore
//! struct Email(String);
//! struct Username(String);
//! struct Port(u16);
//! ```
//!
//! The key insight: if the only way to construct an `Email` is through a
//! validation function, then every `Email` in your program is guaranteed valid.
//!
//! ## The Builder Pattern for Complex Validation
//!
//! When multiple fields must be validated together (e.g., a date where the
//! day must be valid for the given month), use a builder that validates on
//! `.build()`:
//!
//! ```ignore
//! let user = UserBuilder::new()
//!     .email("alice@example.com")?
//!     .username("alice")?
//!     .age(25)?
//!     .build()?;
//! ```
//!
//! ## Attack Scenario
//!
//! Without type-driven validation, a developer might write:
//!
//! ```ignore
//! fn send_email(to: &str, subject: &str) {
//!     // to might be "admin@example.com; DROP TABLE users;--"
//!     // subject might contain XSS payloads
//! }
//! ```
//!
//! With newtypes, the compiler forces validation before the function can be called.

use std::fmt;

/// A validation error with a descriptive message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error on '{}': {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Exercise 1: Implement an `Email` newtype with validation.
///
/// An email is valid if:
/// - It contains exactly one `@` character
/// - The part before `@` is non-empty
/// - The part after `@` contains at least one `.`
/// - No whitespace characters
///
/// The `new()` method should return `Err` for invalid emails.
/// The `as_str()` method should return the inner string.
///
/// Hints:
/// - Use `.find('@')` or `.matches('@').count()`
/// - Split on '@' and validate each part
/// - Check for whitespace with `.chars().any(|c| c.is_whitespace())`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn new(raw: &str) -> Result<Self, ValidationError> {
        todo!("Implement Email validation")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exercise 2: Implement a `Username` newtype with validation.
///
/// A username is valid if:
/// - Length is between 3 and 32 characters (inclusive)
/// - Contains only alphanumeric characters, underscores, or hyphens
/// - Starts with an alphabetic character
///
/// Hints:
/// - Use `.len()` for length check
/// - Use `.chars()` and check each character
/// - Use `.starts_with()` or check `first char`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Username(String);

impl Username {
    pub fn new(raw: &str) -> Result<Self, ValidationError> {
        todo!("Implement Username validation")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Exercise 3: Implement a `Port` newtype that rejects port 0 and common
/// dangerous ports.
///
/// A port is valid if:
/// - It is not 0
/// - It is not in the blocked list: [22, 25, 445, 3389] (SSH, SMTP, SMB, RDP)
///
/// Hints:
/// - Store as `u16`
/// - Use a const array or `matches!` for the blocked list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(u16);

impl Port {
    pub fn new(raw: u16) -> Result<Self, ValidationError> {
        todo!("Implement Port validation")
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

/// Exercise 4: Implement a `UserConfig` builder with cross-field validation.
///
/// A `UserConfig` has:
/// - `email: Email` (validated)
/// - `username: Username` (validated)
/// - `display_name: String` (1-64 chars, non-empty after trimming)
/// - `age: u8` (must be 13-150, COPPA compliance)
///
/// The builder collects values and validates on `.build()`.
///
/// Hints:
/// - Store each field as `Option<T>` in the builder
/// - On `build()`, check that all required fields are `Some`
/// - Reuse the `Email::new()` and `Username::new()` validators
#[derive(Debug, Clone)]
pub struct UserConfig {
    pub email: Email,
    pub username: Username,
    pub display_name: String,
    pub age: u8,
}

pub struct UserConfigBuilder {
    email: Option<Email>,
    username: Option<Username>,
    display_name: Option<String>,
    age: Option<u8>,
}

impl UserConfigBuilder {
    pub fn new() -> Self {
        todo!("Initialize builder with all None fields")
    }

    pub fn email(mut self, raw: &str) -> Result<Self, ValidationError> {
        todo!("Validate and store email")
    }

    pub fn username(mut self, raw: &str) -> Result<Self, ValidationError> {
        todo!("Validate and store username")
    }

    pub fn display_name(mut self, raw: &str) -> Result<Self, ValidationError> {
        todo!("Validate and store display name")
    }

    pub fn age(mut self, raw: u8) -> Result<Self, ValidationError> {
        todo!("Validate and store age")
    }

    pub fn build(self) -> Result<UserConfig, ValidationError> {
        todo!("Build UserConfig, ensuring all fields are present and valid")
    }
}

/// Exercise 5: Implement a `SanitizedString` newtype that strips dangerous
/// characters on construction.
///
/// A SanitizedString:
/// - Has a maximum length of 1000 characters
/// - Removes null bytes (`\0`)
/// - Removes control characters (except newline and tab)
/// - Trims leading/trailing whitespace
///
/// Hints:
/// - Use `.chars().filter(...)` to remove bad characters
/// - Use `.trim()` for whitespace
/// - Check `.len()` after cleaning
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedString(String);

impl SanitizedString {
    pub fn new(raw: &str) -> Result<Self, ValidationError> {
        todo!("Implement SanitizedString with cleaning and validation")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_valid() {
        let email = Email::new("alice@example.com").unwrap();
        assert_eq!(email.as_str(), "alice@example.com");
    }

    #[test]
    fn test_email_no_at() {
        assert!(Email::new("aliceexample.com").is_err());
    }

    #[test]
    fn test_email_no_domain_dot() {
        assert!(Email::new("alice@localhost").is_err());
    }

    #[test]
    fn test_email_empty_local() {
        assert!(Email::new("@example.com").is_err());
    }

    #[test]
    fn test_email_whitespace() {
        assert!(Email::new("alice @example.com").is_err());
    }

    #[test]
    fn test_username_valid() {
        let user = Username::new("alice_99").unwrap();
        assert_eq!(user.as_str(), "alice_99");
    }

    #[test]
    fn test_username_too_short() {
        assert!(Username::new("ab").is_err());
    }

    #[test]
    fn test_username_starts_with_number() {
        assert!(Username::new("1alice").is_err());
    }

    #[test]
    fn test_username_special_chars() {
        assert!(Username::new("alice@home").is_err());
    }

    #[test]
    fn test_port_valid() {
        let port = Port::new(8080).unwrap();
        assert_eq!(port.value(), 8080);
    }

    #[test]
    fn test_port_zero() {
        assert!(Port::new(0).is_err());
    }

    #[test]
    fn test_port_blocked() {
        assert!(Port::new(22).is_err());
        assert!(Port::new(445).is_err());
    }

    #[test]
    fn test_builder_valid() {
        let config = UserConfigBuilder::new()
            .email("alice@example.com").unwrap()
            .username("alice").unwrap()
            .display_name("Alice").unwrap()
            .age(25).unwrap()
            .build().unwrap();
        assert_eq!(config.email.as_str(), "alice@example.com");
        assert_eq!(config.username.as_str(), "alice");
        assert_eq!(config.age, 25);
    }

    #[test]
    fn test_builder_missing_field() {
        let result = UserConfigBuilder::new()
            .email("alice@example.com").unwrap()
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_underage() {
        let result = UserConfigBuilder::new()
            .email("child@example.com").unwrap()
            .username("child").unwrap()
            .display_name("Child").unwrap()
            .age(10).unwrap()
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitized_strips_null_bytes() {
        let s = SanitizedString::new("hello\0world").unwrap();
        assert_eq!(s.as_str(), "helloworld");
    }

    #[test]
    fn test_sanitized_trims_whitespace() {
        let s = SanitizedString::new("  hello  ").unwrap();
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_sanitized_too_long() {
        let long = "a".repeat(1001);
        assert!(SanitizedString::new(&long).is_err());
    }
}
