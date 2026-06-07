//! # Lesson 9: Error Testing
//!
//! Testing error paths is just as important as testing success paths.
//! This lesson covers testing error variants, error chain inspection,
//! assert_matches patterns, and test helpers for error scenarios.

use std::fmt;

// ---------------------------------------------------------------------------
// Error type for testing
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ServiceError {
    NotFound { resource: String, id: String },
    Validation { field: String, message: String },
    Internal { source: Box<dyn std::error::Error + Send + Sync> },
    RateLimited { retry_after_secs: u64 },
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::NotFound { resource, id } => {
                write!(f, "{} '{}' not found", resource, id)
            }
            ServiceError::Validation { field, message } => {
                write!(f, "validation error on '{}': {}", field, message)
            }
            ServiceError::Internal { source } => {
                write!(f, "internal error: {}", source)
            }
            ServiceError::RateLimited { retry_after_secs } => {
                write!(f, "rate limited, retry after {}s", retry_after_secs)
            }
        }
    }
}

impl std::error::Error for ServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ServiceError::Internal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Functions under test
// ---------------------------------------------------------------------------

pub fn find_user(id: u64) -> Result<User, ServiceError> {
    if id == 0 {
        return Err(ServiceError::Validation {
            field: "id".into(),
            message: "id must be positive".into(),
        });
    }
    if id == 999 {
        return Err(ServiceError::NotFound {
            resource: "User".into(),
            id: id.to_string(),
        });
    }
    if id == 666 {
        return Err(ServiceError::Internal {
            source: Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "database connection lost",
            )),
        });
    }
    if id == 429 {
        return Err(ServiceError::RateLimited {
            retry_after_secs: 60,
        });
    }
    Ok(User {
        id,
        name: format!("User {}", id),
    })
}

#[derive(Debug, PartialEq)]
pub struct User {
    pub id: u64,
    pub name: String,
}

// ---------------------------------------------------------------------------
// Test helper macros
// ---------------------------------------------------------------------------

/// Assert that a Result is an error matching a specific variant.
macro_rules! assert_err_variant {
    ($expr:expr, $variant:pat) => {
        match $expr {
            Err($variant) => {}
            Err(other) => panic!(
                "expected error variant matching '{}', got: {:?}",
                stringify!($variant),
                other
            ),
            Ok(val) => panic!(
                "expected error, got Ok({:?})",
                val
            ),
        }
    };
}

/// Assert that a Result is Ok and the value satisfies a predicate.
macro_rules! assert_ok_with {
    ($expr:expr, $pred:expr) => {
        match $expr {
            Ok(val) => {
                if !$pred(&val) {
                    panic!("Ok value {:?} did not satisfy predicate", val);
                }
            }
            Err(e) => panic!("expected Ok, got Err: {}", e),
        }
    };
}

// ---------------------------------------------------------------------------
// Error chain test helpers
// ---------------------------------------------------------------------------

/// Walk an error chain and collect all messages.
pub fn collect_error_chain(err: &dyn std::error::Error) -> Vec<String> {
    let mut messages = vec![err.to_string()];
    let mut current = err.source();
    while let Some(source) = current {
        messages.push(source.to_string());
        current = source.source();
    }
    messages
}

/// Check if an error chain contains a message matching a pattern.
pub fn chain_contains(err: &dyn std::error::Error, pattern: &str) -> bool {
    collect_error_chain(err).iter().any(|m| m.contains(pattern))
}

// ---------------------------------------------------------------------------
// Error test builder
// ---------------------------------------------------------------------------

/// A builder for constructing test scenarios that produce errors.
pub struct ErrorTestScenario {
    pub name: String,
    pub input: u64,
    pub expected_error_type: String,
    pub expected_message_contains: Option<String>,
    pub expected_source_contains: Option<String>,
}

impl ErrorTestScenario {
    pub fn new(name: &str, input: u64, error_type: &str) -> Self {
        Self {
            name: name.into(),
            input,
            expected_error_type: error_type.into(),
            expected_message_contains: None,
            expected_source_contains: None,
        }
    }

    pub fn with_message(mut self, pattern: &str) -> Self {
        self.expected_message_contains = Some(pattern.into());
        self
    }

    pub fn with_source(mut self, pattern: &str) -> Self {
        self.expected_source_contains = Some(pattern.into());
        self
    }

    /// Run the scenario and verify expectations.
    pub fn run(&self) {
        let result = find_user(self.input);
        let err = result.expect_err(&format!("scenario '{}': expected error", self.name));

        // Verify message contains expected pattern
        if let Some(ref pattern) = self.expected_message_contains {
            let msg = err.to_string();
            assert!(
                msg.contains(pattern),
                "scenario '{}': error '{}' does not contain '{}'",
                self.name, msg, pattern
            );
        }

        // Verify source contains expected pattern
        if let Some(ref pattern) = self.expected_source_contains {
            assert!(
                chain_contains(&err, pattern),
                "scenario '{}': error chain does not contain '{}'",
                self.name, pattern
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_find_user_success() {
        let user = find_user(1).unwrap();
        assert_eq!(user.id, 1);
        assert_eq!(user.name, "User 1");
    }

    #[test]
    fn test_find_user_not_found() {
        let err = find_user(999).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_find_user_validation_error() {
        let err = find_user(0).unwrap_err();
        assert!(err.to_string().contains("id must be positive"));
    }

    // Using assert_err_variant macro
    #[test]
    fn test_assert_err_variant_not_found() {
        assert_err_variant!(
            find_user(999),
            ServiceError::NotFound { .. }
        );
    }

    #[test]
    fn test_assert_err_variant_validation() {
        assert_err_variant!(
            find_user(0),
            ServiceError::Validation { .. }
        );
    }

    #[test]
    fn test_assert_err_variant_rate_limited() {
        assert_err_variant!(
            find_user(429),
            ServiceError::RateLimited { .. }
        );
    }

    // Using assert_ok_with macro
    #[test]
    fn test_assert_ok_with() {
        assert_ok_with!(find_user(1), |u: &User| u.id == 1);
    }

    #[test]
    #[should_panic]
    fn test_assert_ok_with_failure() {
        assert_ok_with!(find_user(999), |_: &User| true);
    }

    // Matching on specific error fields
    #[test]
    fn test_match_not_found_fields() {
        match find_user(999) {
            Err(ServiceError::NotFound { resource, id }) => {
                assert_eq!(resource, "User");
                assert_eq!(id, "999");
            }
            _ => panic!("expected NotFound"),
        }
    }

    #[test]
    fn test_match_validation_fields() {
        match find_user(0) {
            Err(ServiceError::Validation { field, message }) => {
                assert_eq!(field, "id");
                assert!(message.contains("positive"));
            }
            _ => panic!("expected Validation"),
        }
    }

    // Error chain inspection
    #[test]
    fn test_error_chain_internal() {
        let err = find_user(666).unwrap_err();
        let chain = collect_error_chain(&err);
        assert!(chain.len() >= 2);
        assert!(chain[0].contains("internal error"));
        assert!(chain[1].contains("database connection lost"));
    }

    #[test]
    fn test_chain_contains() {
        let err = find_user(666).unwrap_err();
        assert!(chain_contains(&err, "database connection"));
        assert!(chain_contains(&err, "internal error"));
        assert!(!chain_contains(&err, "not found"));
    }

    // Error test scenarios using builder
    #[test]
    fn test_error_scenarios() {
        let scenarios = vec![
            ErrorTestScenario::new("zero_id", 0, "Validation")
                .with_message("id must be positive"),
            ErrorTestScenario::new("not_found", 999, "NotFound")
                .with_message("not found"),
            ErrorTestScenario::new("internal", 666, "Internal")
                .with_message("internal error")
                .with_source("database connection"),
            ErrorTestScenario::new("rate_limited", 429, "RateLimited")
                .with_message("rate limited"),
        ];

        for scenario in &scenarios {
            scenario.run();
        }
    }

    // Testing error equality via string comparison
    #[test]
    fn test_error_messages() {
        let cases: Vec<(u64, &str)> = vec![
            (0, "id must be positive"),
            (999, "not found"),
            (666, "internal error"),
            (429, "rate limited"),
        ];

        for (input, expected_msg) in cases {
            let err = find_user(input).unwrap_err();
            assert!(
                err.to_string().contains(expected_msg),
                "For input {}: expected '{}' in '{}'",
                input,
                expected_msg,
                err
            );
        }
    }

    // Testing that errors implement the Error trait
    #[test]
    fn test_implements_error_trait() {
        let err = find_user(999).unwrap_err();
        let _: &dyn std::error::Error = &err;
    }

    // Testing Debug output
    #[test]
    fn test_error_debug() {
        let err = find_user(999).unwrap_err();
        let debug = format!("{:?}", err);
        assert!(debug.contains("NotFound"));
        assert!(debug.contains("User"));
    }

    // Testing that internal errors have a source
    #[test]
    fn test_internal_error_has_source() {
        let err = find_user(666).unwrap_err();
        assert!(err.source().is_some());
    }

    #[test]
    fn test_non_internal_errors_no_source() {
        let err = find_user(999).unwrap_err();
        assert!(err.source().is_none());
    }
}
