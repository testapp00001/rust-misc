//! # Lesson 04: Schema Validation
//!
//! ## The Problem
//!
//! Deserialization checks that the data matches your struct's shape. But shape
//! is not enough. An API might expect `{"age": 25}` and correctly deserialize
//! it, but what about `{"age": -1}` or `{"age": 999999}`?
//!
//! Schema validation goes beyond type checking to enforce:
//! - Value ranges (age is 0-150)
//! - String patterns (email contains @)
//! - Array length limits (no more than 1000 items)
//! - Required vs optional fields
//!
//! ## Why Validate After Deserialization
//!
//! serde handles structural validation (types match). Business validation
//! (values make sense) belongs in a separate layer:
//!
//! ```rust
//! let input: Config = serde_json::from_str(&raw)?;  // structural
//! validate_config(&input)?;                          // business
//! ```
//!
//! This separation makes security auditors happy: they can review the
//! validation function without understanding serde internals.

use serde::Deserialize;

/// A server configuration that must pass strict validation.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub hostname: String,
    pub port: u16,
    pub max_connections: u32,
    pub log_level: String,
}

/// Exercise 1: Validate a ServerConfig.
///
/// Rules:
/// - hostname: must not be empty, must be at most 253 chars (DNS limit)
/// - port: must not be 0 (reserved), must be above 1023 (non-privileged)
/// - max_connections: must be between 1 and 100,000
/// - log_level: must be one of "trace", "debug", "info", "warn", "error"
///
/// Return Ok(()) if valid, Err(message) if invalid.
pub fn validate_server_config(config: &ServerConfig) -> Result<(), String> {
    todo!("Validate ServerConfig fields")
}

/// Exercise 2: Parse and validate a ServerConfig from JSON.
pub fn parse_server_config(json: &str) -> Result<ServerConfig, String> {
    todo!("Parse and validate ServerConfig")
}

/// Exercise 3: An API request with validation.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CreatePostRequest {
    pub title: String,
    pub body: String,
    pub tags: Vec<String>,
}

/// Validate a CreatePostRequest:
/// - title: 1-200 characters
/// - body: 1-10,000 characters
/// - tags: 0-10 tags, each 1-50 characters, all lowercase alphanumeric
pub fn validate_create_post(request: &CreatePostRequest) -> Result<(), String> {
    todo!("Validate CreatePostRequest")
}

/// Exercise 4: Parse and validate a CreatePostRequest from JSON.
pub fn parse_create_post(json: &str) -> Result<CreatePostRequest, String> {
    todo!("Parse and validate CreatePostRequest")
}

/// Exercise 5: Generic value range validator.
///
/// Check that `value` is between `min` and `max` (inclusive).
/// Return Ok(()) if in range, Err(message) if out of range.
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    field_name: &str,
) -> Result<(), String> {
    todo!("Validate numeric range")
}

/// Exercise 6: Validate a string against an allowlist.
///
/// Check that `value` is one of the allowed values (case-sensitive).
/// Return Ok(()) if allowed, Err(message) if not.
pub fn validate_enum(
    value: &str,
    allowed: &[&str],
    field_name: &str,
) -> Result<(), String> {
    todo!("Validate against allowlist")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_config() {
        let config = ServerConfig {
            hostname: "api.example.com".to_string(),
            port: 8080,
            max_connections: 1000,
            log_level: "info".to_string(),
        };
        assert!(validate_server_config(&config).is_ok());
    }

    #[test]
    fn test_empty_hostname() {
        let config = ServerConfig {
            hostname: "".to_string(),
            port: 8080,
            max_connections: 1000,
            log_level: "info".to_string(),
        };
        assert!(validate_server_config(&config).is_err());
    }

    #[test]
    fn test_reserved_port() {
        let config = ServerConfig {
            hostname: "api.example.com".to_string(),
            port: 0,
            max_connections: 1000,
            log_level: "info".to_string(),
        };
        assert!(validate_server_config(&config).is_err());
    }

    #[test]
    fn test_privileged_port() {
        let config = ServerConfig {
            hostname: "api.example.com".to_string(),
            port: 80,
            max_connections: 1000,
            log_level: "info".to_string(),
        };
        assert!(validate_server_config(&config).is_err());
    }

    #[test]
    fn test_invalid_log_level() {
        let config = ServerConfig {
            hostname: "api.example.com".to_string(),
            port: 8080,
            max_connections: 1000,
            log_level: "verbose".to_string(),
        };
        assert!(validate_server_config(&config).is_err());
    }

    #[test]
    fn test_parse_and_validate_config() {
        let json = r#"{
            "hostname": "api.example.com",
            "port": 8080,
            "max_connections": 1000,
            "log_level": "info"
        }"#;
        assert!(parse_server_config(json).is_ok());
    }

    #[test]
    fn test_valid_post_request() {
        let req = CreatePostRequest {
            title: "Hello World".to_string(),
            body: "This is a valid post body.".to_string(),
            tags: vec!["rust".to_string(), "security".to_string()],
        };
        assert!(validate_create_post(&req).is_ok());
    }

    #[test]
    fn test_too_many_tags() {
        let req = CreatePostRequest {
            title: "Post".to_string(),
            body: "Body text here.".to_string(),
            tags: (0..11).map(|i| format!("tag{}", i)).collect(),
        };
        assert!(validate_create_post(&req).is_err());
    }

    #[test]
    fn test_range_validator() {
        assert!(validate_range(50, 0, 100, "score").is_ok());
        assert!(validate_range(101, 0, 100, "score").is_err());
        assert!(validate_range(-1, 0, 100, "score").is_err());
    }

    #[test]
    fn test_enum_validator() {
        let allowed = &["trace", "debug", "info", "warn", "error"];
        assert!(validate_enum("info", allowed, "log_level").is_ok());
        assert!(validate_enum("verbose", allowed, "log_level").is_err());
    }
}
