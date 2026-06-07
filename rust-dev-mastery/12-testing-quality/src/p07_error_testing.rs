//! # Testing Error Paths
//!
//! Thorough testing of error handling is critical for production code.
//! This lesson covers patterns for testing Result types, panics, and
//! error propagation.

use std::fmt;

/// A custom error type for demonstrating error testing.
#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    NotFound { resource: String, id: String },
    Validation { field: String, message: String },
    Permission { action: String, user: String },
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound { resource, id } => {
                write!(f, "{resource} with id '{id}' not found")
            }
            AppError::Validation { field, message } => {
                write!(f, "validation error on '{field}': {message}")
            }
            AppError::Permission { action, user } => {
                write!(f, "user '{user}' lacks permission for '{action}'")
            }
            AppError::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for AppError {}

/// Demonstrates functions that return Result for testing error paths.
pub fn find_user(id: u32) -> Result<String, AppError> {
    match id {
        1 => Ok("Alice".to_string()),
        2 => Ok("Bob".to_string()),
        _ => Err(AppError::NotFound {
            resource: "User".to_string(),
            id: id.to_string(),
        }),
    }
}

pub fn validate_age(age: i32) -> Result<(), AppError> {
    if age < 0 {
        return Err(AppError::Validation {
            field: "age".to_string(),
            message: "must be non-negative".to_string(),
        });
    }
    if age > 150 {
        return Err(AppError::Validation {
            field: "age".to_string(),
            message: "must be <= 150".to_string(),
        });
    }
    Ok(())
}

pub fn check_permission(user: &str, action: &str) -> Result<(), AppError> {
    let allowed = match (user, action) {
        ("admin", _) => true,
        ("user", "read") => true,
        ("Alice", _) => true,
        _ => false,
    };
    if allowed {
        Ok(())
    } else {
        Err(AppError::Permission {
            action: action.to_string(),
            user: user.to_string(),
        })
    }
}

/// Demonstrates error propagation with the ? operator.
pub fn get_user_name(id: u32) -> Result<String, AppError> {
    let name = find_user(id)?;
    Ok(format!("User: {name}"))
}

/// Demonstrates a function that panics on invalid input.
pub fn divide(a: f64, b: f64) -> f64 {
    assert!(b != 0.0, "division by zero");
    a / b
}

/// Demonstrates testing error chains.
pub fn process_request(user_id: u32, action: &str) -> Result<String, AppError> {
    let user = find_user(user_id)?;
    check_permission(&user, action)?;
    Ok(format!("{user} performed {action}"))
}

/// Demonstrates a function that converts between error types.
#[derive(Debug)]
pub enum ParseError {
    EmptyInput,
    InvalidFormat(String),
    OutOfRange(i64),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::EmptyInput => write!(f, "empty input"),
            ParseError::InvalidFormat(s) => write!(f, "invalid format: {s}"),
            ParseError::OutOfRange(v) => write!(f, "value out of range: {v}"),
        }
    }
}

pub fn parse_port(s: &str) -> Result<u16, ParseError> {
    if s.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    let port: i64 = s.parse().map_err(|_| ParseError::InvalidFormat(s.to_string()))?;
    if port < 0 || port > 65535 {
        return Err(ParseError::OutOfRange(port));
    }
    Ok(port as u16)
}

/// Demonstrates testing multiple error conditions.
pub fn validate_config(config: &Config) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if config.host.is_empty() {
        errors.push("host must not be empty".to_string());
    }
    if config.port == 0 {
        errors.push("port must be non-zero".to_string());
    }
    if config.max_connections == 0 {
        errors.push("max_connections must be non-zero".to_string());
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[derive(Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_user_found() {
        assert_eq!(find_user(1).unwrap(), "Alice");
        assert_eq!(find_user(2).unwrap(), "Bob");
    }

    #[test]
    fn test_find_user_not_found() {
        let err = find_user(99).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
        assert!(err.to_string().contains("99"));
    }

    #[test]
    fn test_validate_age_valid() {
        assert!(validate_age(25).is_ok());
        assert!(validate_age(0).is_ok());
        assert!(validate_age(150).is_ok());
    }

    #[test]
    fn test_validate_age_negative() {
        let err = validate_age(-1).unwrap_err();
        assert!(err.to_string().contains("non-negative"));
    }

    #[test]
    fn test_validate_age_too_high() {
        let err = validate_age(200).unwrap_err();
        assert!(err.to_string().contains("150"));
    }

    #[test]
    fn test_check_permission_allowed() {
        assert!(check_permission("admin", "delete").is_ok());
        assert!(check_permission("user", "read").is_ok());
    }

    #[test]
    fn test_check_permission_denied() {
        let err = check_permission("user", "delete").unwrap_err();
        assert!(matches!(err, AppError::Permission { .. }));
    }

    #[test]
    fn test_error_propagation() {
        // Success case
        assert!(get_user_name(1).is_ok());

        // Error propagated from find_user
        let err = get_user_name(99).unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    #[should_panic(expected = "division by zero")]
    fn test_divide_by_zero_panics() {
        divide(10.0, 0.0);
    }

    #[test]
    fn test_divide_normal() {
        assert_eq!(divide(10.0, 2.0), 5.0);
    }

    #[test]
    fn test_process_request_success() {
        let result = process_request(1, "read");
        assert!(result.unwrap().contains("Alice"));
    }

    #[test]
    fn test_process_request_user_not_found() {
        let err = process_request(99, "read").unwrap_err();
        assert!(matches!(err, AppError::NotFound { .. }));
    }

    #[test]
    fn test_process_request_no_permission() {
        let err = process_request(2, "delete").unwrap_err();
        assert!(matches!(err, AppError::Permission { .. }));
    }

    #[test]
    fn test_parse_port_valid() {
        assert_eq!(parse_port("8080").unwrap(), 8080);
        assert_eq!(parse_port("0").unwrap(), 0);
        assert_eq!(parse_port("65535").unwrap(), 65535);
    }

    #[test]
    fn test_parse_port_empty() {
        let err = parse_port("").unwrap_err();
        assert!(matches!(err, ParseError::EmptyInput));
    }

    #[test]
    fn test_parse_port_invalid_format() {
        let err = parse_port("abc").unwrap_err();
        assert!(matches!(err, ParseError::InvalidFormat(_)));
    }

    #[test]
    fn test_parse_port_out_of_range() {
        let err = parse_port("70000").unwrap_err();
        assert!(matches!(err, ParseError::OutOfRange(70000)));
    }

    #[test]
    fn test_validate_config_valid() {
        let config = Config {
            host: "localhost".to_string(),
            port: 8080,
            max_connections: 100,
        };
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_multiple_errors() {
        let config = Config {
            host: String::new(),
            port: 0,
            max_connections: 0,
        };
        let errors = validate_config(&config).unwrap_err();
        assert_eq!(errors.len(), 3);
    }
}
