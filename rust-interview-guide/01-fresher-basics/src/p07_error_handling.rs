/// Problem: Error Handling
///
/// Master Rust's error handling system.
///
/// Key Concepts:
/// - Result<T, E>
/// - Option<T>
/// - ? operator
/// - unwrap and expect
/// - Custom error types
/// - Error conversion with From

use std::num::ParseIntError;

/// Problem 1: Basic Result
/// Parse a string to integer
pub fn parse_int(s: &str) -> Result<i32, ParseIntError> {
    s.parse()
}

/// Problem 2: Result with map
/// Parse and double a number
pub fn parse_and_double(s: &str) -> Result<i32, ParseIntError> {
    s.parse::<i32>().map(|x| x * 2)
}

/// Problem 3: Result with and_then
/// Parse, validate, and double
pub fn parse_validate_double(s: &str) -> Result<i32, String> {
    s.parse::<i32>()
        .map_err(|e| e.to_string())
        .and_then(|x| {
            if x >= 0 {
                Ok(x * 2)
            } else {
                Err("Number must be non-negative".to_string())
            }
        })
}

/// Problem 4: Option basics
/// Find a value in a slice
pub fn find_value(arr: &[i32], target: i32) -> Option<usize> {
    arr.iter().position(|&x| x == target)
}

/// Problem 5: Option with map
/// Get the first element and double it
pub fn first_doubled(arr: &[i32]) -> Option<i32> {
    arr.first().map(|&x| x * 2)
}

/// Problem 6: Option with and_then
/// Get the first element, check if positive, and double it
pub fn first_positive_doubled(arr: &[i32]) -> Option<i32> {
    arr.first()
        .filter(|&&x| x > 0)
        .map(|&x| x * 2)
}

/// Problem 7: ? operator
/// Parse multiple values
pub fn parse_pair(s: &str) -> Result<(i32, i32), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err("Expected two comma-separated values".to_string());
    }

    let a = parts[0].trim().parse::<i32>()
        .map_err(|e| format!("Failed to parse first value: {}", e))?;
    let b = parts[1].trim().parse::<i32>()
        .map_err(|e| format!("Failed to parse second value: {}", e))?;

    Ok((a, b))
}

/// Problem 8: Custom error type
#[derive(Debug, PartialEq)]
pub enum AppError {
    ParseError(String),
    ValidationError(String),
    NotFoundError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::NotFoundError(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::ParseError(e.to_string())
    }
}

/// Problem 9: Using custom error type
pub fn parse_and_validate(s: &str) -> Result<i32, AppError> {
    let n: i32 = s.parse()?; // Uses From<ParseIntError>
    if n < 0 {
        Err(AppError::ValidationError("Number must be non-negative".to_string()))
    } else {
        Ok(n)
    }
}

/// Problem 10: Error handling with multiple error types
pub fn process_input(s: &str) -> Result<i32, AppError> {
    let parts: Vec<&str> = s.split('=').collect();
    if parts.len() != 2 {
        return Err(AppError::ParseError("Expected key=value format".to_string()));
    }

    let key = parts[0].trim();
    let value = parts[1].trim();

    if key != "number" {
        return Err(AppError::NotFoundError(format!("Unknown key: {}", key)));
    }

    let n: i32 = value.parse()?;
    Ok(n)
}

/// Problem 11: Collecting Results
/// Parse a vector of strings to integers
pub fn parse_all(strings: &[&str]) -> Result<Vec<i32>, ParseIntError> {
    strings.iter().map(|s| s.parse()).collect()
}

/// Problem 12: Option and Result conversion
/// Convert Option to Result
pub fn option_to_result<T>(opt: Option<T>, msg: &str) -> Result<T, String> {
    opt.ok_or_else(|| msg.to_string())
}

/// Problem 13: Result and Option conversion
/// Convert Result to Option
pub fn result_to_option<T, E>(res: Result<T, E>) -> Option<T> {
    res.ok()
}

/// Problem 14: Unwrap with context
/// Unwrap with a custom panic message
pub fn unwrap_with_context(opt: Option<i32>) -> i32 {
    opt.expect("Expected a value")
}

/// Problem 15: Early return with ?
/// Use ? for early return
pub fn find_and_double(arr: &[i32], target: i32) -> Result<i32, String> {
    let index = arr.iter()
        .position(|&x| x == target)
        .ok_or_else(|| format!("Value {} not found", target))?;

    Ok(arr[index] * 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int() {
        assert_eq!(parse_int("42"), Ok(42));
        assert!(parse_int("abc").is_err());
    }

    #[test]
    fn test_parse_and_double() {
        assert_eq!(parse_and_double("21"), Ok(42));
        assert!(parse_and_double("abc").is_err());
    }

    #[test]
    fn test_parse_validate_double() {
        assert_eq!(parse_validate_double("21"), Ok(42));
        assert_eq!(
            parse_validate_double("-5"),
            Err("Number must be non-negative".to_string())
        );
    }

    #[test]
    fn test_find_value() {
        assert_eq!(find_value(&[1, 2, 3], 2), Some(1));
        assert_eq!(find_value(&[1, 2, 3], 4), None);
    }

    #[test]
    fn test_first_doubled() {
        assert_eq!(first_doubled(&[5, 2, 3]), Some(10));
        assert_eq!(first_doubled(&[]), None);
    }

    #[test]
    fn test_first_positive_doubled() {
        assert_eq!(first_positive_doubled(&[5, 2, 3]), Some(10));
        assert_eq!(first_positive_doubled(&[-1, 2, 3]), None);
        assert_eq!(first_positive_doubled(&[]), None);
    }

    #[test]
    fn test_parse_pair() {
        assert_eq!(parse_pair("1, 2"), Ok((1, 2)));
        assert!(parse_pair("1").is_err());
        assert!(parse_pair("a, b").is_err());
    }

    #[test]
    fn test_parse_and_validate() {
        assert_eq!(parse_and_validate("42"), Ok(42));
        assert_eq!(
            parse_and_validate("-5"),
            Err(AppError::ValidationError("Number must be non-negative".to_string()))
        );
        assert!(matches!(parse_and_validate("abc"), Err(AppError::ParseError(_))));
    }

    #[test]
    fn test_process_input() {
        assert_eq!(process_input("number=42"), Ok(42));
        assert!(process_input("name=Alice").is_err());
        assert!(process_input("number=abc").is_err());
    }

    #[test]
    fn test_parse_all() {
        assert_eq!(parse_all(&["1", "2", "3"]), Ok(vec![1, 2, 3]));
        assert!(parse_all(&["1", "abc", "3"]).is_err());
    }

    #[test]
    fn test_option_to_result() {
        assert_eq!(option_to_result(Some(42), "error"), Ok(42));
        assert_eq!(option_to_result::<i32>(None, "error"), Err("error".to_string()));
    }

    #[test]
    fn test_result_to_option() {
        assert_eq!(result_to_option(Ok::<i32, String>(42)), Some(42));
        assert_eq!(result_to_option(Err::<i32, String>("error".to_string())), None);
    }

    #[test]
    #[should_panic(expected = "Expected a value")]
    fn test_unwrap_with_context() {
        unwrap_with_context(None);
    }

    #[test]
    fn test_find_and_double() {
        assert_eq!(find_and_double(&[1, 2, 3], 2), Ok(4));
        assert!(find_and_double(&[1, 2, 3], 4).is_err());
    }
}
