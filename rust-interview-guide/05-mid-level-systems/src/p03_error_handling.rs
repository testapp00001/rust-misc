/// Problem: Error Handling
///
/// Master Rust's error handling patterns.
///
/// Key Concepts:
/// - Result type
/// - Error types
/// - Error propagation
/// - Custom errors
/// - Error conversion

use std::fmt;
use std::num::ParseIntError;

/// Problem 1: Basic Result
/// Return Result from function
pub fn basic_result() -> Result<i32, String> {
    Ok(42)
}

/// Problem 2: Result with error
/// Return error from function
pub fn result_with_error() -> Result<i32, String> {
    Err("Something went wrong".to_string())
}

/// Problem 3: Result with ? operator
/// Use ? for error propagation
pub fn result_with_question_mark() -> Result<i32, String> {
    let value = basic_result()?;
    Ok(value * 2)
}

/// Problem 4: Custom error type
/// Define custom error type
#[derive(Debug, PartialEq)]
pub enum AppError {
    NotFound(String),
    Unauthorized,
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// Problem 5: Error conversion
/// Implement From for error conversion
impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Internal(e.to_string())
    }
}

pub fn parse_number(s: &str) -> Result<i32, AppError> {
    let n = s.parse()?; // Uses From<ParseIntError>
    Ok(n)
}

/// Problem 6: Error with context
/// Add context to errors
pub fn with_context() -> Result<i32, String> {
    basic_result().map_err(|e| format!("Context: {}", e))
}

/// Problem 7: Error recovery
/// Recover from errors
pub fn error_recovery() -> i32 {
    result_with_error().unwrap_or(0)
}

/// Problem 8: Error with multiple types
/// Handle multiple error types
pub fn multiple_error_types() -> Result<i32, String> {
    let value1 = basic_result()?;
    let value2 = result_with_error().unwrap_or(0);
    Ok(value1 + value2)
}

/// Problem 9: Error with chain
/// Chain error handling
pub fn error_chain() -> Result<i32, String> {
    basic_result()
        .and_then(|v| Ok(v * 2))
        .map_err(|e| format!("Error: {}", e))
}

/// Problem 10: Error with or_else
/// Recover with or_else
pub fn error_or_else() -> Result<i32, String> {
    result_with_error().or_else(|_| Ok(42))
}

/// Problem 11: Error with map_err
/// Map error type
pub fn error_map_err() -> Result<i32, String> {
    result_with_error().map_err(|e| format!("Mapped: {}", e))
}

/// Problem 12: Error with unwrap_or_else
/// Compute default on error
pub fn error_unwrap_or_else() -> i32 {
    result_with_error().unwrap_or_else(|_| 42)
}

/// Problem 13: Error with expect
/// Expect with message
pub fn error_expect() -> i32 {
    basic_result().expect("Should succeed")
}

/// Problem 14: Error with panic
/// Panic on error
pub fn error_panic() -> i32 {
    basic_result().unwrap()
}

/// Problem 15: Error with custom conversion
/// Convert between error types
pub fn error_conversion() -> Result<i32, AppError> {
    let value = parse_number("42")?;
    Ok(value * 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_result() {
        assert_eq!(basic_result(), Ok(42));
    }

    #[test]
    fn test_result_with_error() {
        assert!(result_with_error().is_err());
    }

    #[test]
    fn test_result_with_question_mark() {
        assert_eq!(result_with_question_mark(), Ok(84));
    }

    #[test]
    fn test_custom_error() {
        let error = AppError::NotFound("Resource".to_string());
        assert!(error.to_string().contains("Not found"));
    }

    #[test]
    fn test_error_conversion() {
        assert_eq!(parse_number("42"), Ok(42));
        assert!(parse_number("abc").is_err());
    }

    #[test]
    fn test_with_context() {
        assert_eq!(with_context(), Ok(42));
    }

    #[test]
    fn test_error_recovery() {
        assert_eq!(error_recovery(), 0);
    }

    #[test]
    fn test_multiple_error_types() {
        assert_eq!(multiple_error_types(), Ok(42));
    }

    #[test]
    fn test_error_chain() {
        assert_eq!(error_chain(), Ok(84));
    }

    #[test]
    fn test_error_or_else() {
        assert_eq!(error_or_else(), Ok(42));
    }

    #[test]
    fn test_error_map_err() {
        assert!(error_map_err().is_err());
    }

    #[test]
    fn test_error_unwrap_or_else() {
        assert_eq!(error_unwrap_or_else(), 42);
    }

    #[test]
    fn test_error_expect() {
        assert_eq!(error_expect(), 42);
    }

    #[test]
    fn test_error_panic() {
        assert_eq!(error_panic(), 42);
    }

    #[test]
    fn test_error_conversion_custom() {
        assert_eq!(error_conversion(), Ok(84));
    }
}
