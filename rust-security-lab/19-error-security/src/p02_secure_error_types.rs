//! # Lesson 02: Secure Error Types
//!
//! ## The Problem
//!
//! Most error types have a single `Display` implementation that is shown to both
//! developers and users. If `Display` includes internal details (file paths, SQL
//! queries, stack traces), those details leak to users. If it's generic, developers
//! lose debugging context.
//!
//! ## Solution: Dual-Representation Errors
//!
//! A secure error type provides two views:
//! - **Internal** (`Debug` / internal logging method): Full details for developers
//! - **External** (`Display` / user-facing method): Generic messages for users
//!
//! This is the pattern used by production frameworks like Actix-web, Axum, and Rocket.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │            SecureError                       │
//! │  ┌─────────────────────────────────────────┐ │
//! │  │ Internal (for logging)                  │ │
//! │  │  - Full error message                   │ │
//! │  │  - Source file and line                 │ │
//! │  │  - Database query                       │ │
//! │  │  - Stack trace                          │ │
//! │  └─────────────────────────────────────────┘ │
//! │  ┌─────────────────────────────────────────┐ │
//! │  │ External (for users)                    │ │
//! │  │  - Generic message category             │ │
//! │  │  - Error code (e.g., "AUTH_FAILED")     │ │
//! │  │  - Request ID for correlation           │ │
//! │  └─────────────────────────────────────────┘ │
//! └─────────────────────────────────────────────┘
//! ```

use std::fmt;

/// Error category for user-facing messages.
/// Each category maps to a generic message that reveals nothing about internals.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Generic authentication failure (never distinguish user vs password)
    AuthenticationFailed,
    /// Authorization failure (insufficient permissions)
    AccessDenied,
    /// Input validation failure (don't reveal which field)
    ValidationFailed,
    /// Resource not found (don't reveal whether it exists or not)
    NotFound,
    /// Internal server error (catch-all)
    InternalError,
    /// Rate limit exceeded
    RateLimited,
}

impl ErrorCategory {
    /// Return the user-facing message for this category.
    pub fn user_message(&self) -> &'static str {
        match self {
            ErrorCategory::AuthenticationFailed => "Authentication failed",
            ErrorCategory::AccessDenied => "Access denied",
            ErrorCategory::ValidationFailed => "Invalid request",
            ErrorCategory::NotFound => "Resource not found",
            ErrorCategory::InternalError => "An internal error occurred",
            ErrorCategory::RateLimited => "Too many requests",
        }
    }

    /// Return a machine-readable error code for API responses.
    pub fn error_code(&self) -> &'static str {
        match self {
            ErrorCategory::AuthenticationFailed => "AUTH_FAILED",
            ErrorCategory::AccessDenied => "ACCESS_DENIED",
            ErrorCategory::ValidationFailed => "VALIDATION_FAILED",
            ErrorCategory::NotFound => "NOT_FOUND",
            ErrorCategory::InternalError => "INTERNAL_ERROR",
            ErrorCategory::RateLimited => "RATE_LIMITED",
        }
    }
}

/// A secure error type with dual representation.
///
/// Internal details are stored for logging but NEVER exposed in user-facing output.
#[derive(Debug)]
pub struct SecureError {
    /// User-facing category (determines the generic message)
    pub category: ErrorCategory,
    /// Full internal message for logging (NEVER shown to users)
    pub internal_message: String,
    /// Optional request ID for correlation
    pub request_id: Option<String>,
    /// Optional source error for chaining
    pub source: Option<String>,
}

impl SecureError {
    pub fn new(category: ErrorCategory, internal_message: impl Into<String>) -> Self {
        Self {
            category,
            internal_message: internal_message.into(),
            request_id: None,
            source: None,
        }
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    pub fn with_source(mut self, source: impl fmt::Display) -> Self {
        self.source = Some(source.to_string());
        self
    }
}

/// Exercise 1: Implement Display for SecureError.
///
/// The `Display` implementation is what users see. It must:
/// - Return ONLY the category's user-facing message
/// - Include the error code if available
/// - Include the request_id if available
/// - NEVER include `internal_message` or `source`
///
/// Format: "{user_message} (code: {error_code})"
/// With request_id: "{user_message} (code: {error_code}, request: {request_id})"
impl fmt::Display for SecureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!("Implement Display -- show only user-safe information")
    }
}

/// Exercise 2: Implement the internal logging method.
///
/// This returns the FULL error details for server-side logging.
/// Include: category, internal_message, request_id, source -- everything.
///
/// Format: "[{error_code}] {internal_message}"
/// With source: "[{error_code}] {internal_message} | caused by: {source}"
/// With request_id: "[{error_code}] {internal_message} | request: {request_id}"
impl SecureError {
    pub fn internal_display(&self) -> String {
        todo!("Return full internal details for logging")
    }
}

/// Exercise 3: Create a secure authentication error.
///
/// Return a `SecureError` with category `AuthenticationFailed`.
/// The internal message should contain the REAL reason (e.g., "user 'admin' not found").
/// The external message will automatically be "Authentication failed".
pub fn auth_error(internal_reason: &str, request_id: &str) -> SecureError {
    todo!("Create a SecureError for authentication failures")
}

/// Exercise 4: Create a secure database error.
///
/// Return a `SecureError` with category `InternalError`.
/// The internal message should contain the full DB error.
/// Add the DB error as the source for error chaining.
pub fn db_error(db_message: &str, source: &str, request_id: &str) -> SecureError {
    todo!("Create a SecureError for database failures")
}

/// Exercise 5: Validate that an error response is safe for users.
///
/// Given a user-facing error string (from Display), verify it does NOT contain
/// any of these internal details:
/// - The internal_message content
/// - SQL keywords
/// - File paths
/// - IP addresses
///
/// Return true if the response is safe.
pub fn is_safe_error_response(response: &str, internal_message: &str) -> bool {
    todo!("Verify the error response contains no internal details")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_no_internal_details() {
        let err = SecureError::new(
            ErrorCategory::AuthenticationFailed,
            "user 'admin' not found in database at /app/src/auth.rs:42",
        );
        let display = format!("{}", err);
        assert!(!display.contains("admin"), "Display must not contain username");
        assert!(!display.contains("/app"), "Display must not contain file path");
        assert!(!display.contains("database"), "Display must not contain internal details");
    }

    #[test]
    fn test_display_has_error_code() {
        let err = SecureError::new(ErrorCategory::AuthenticationFailed, "internal");
        let display = format!("{}", err);
        assert!(display.contains("AUTH_FAILED"), "Display should contain error code");
    }

    #[test]
    fn test_display_has_request_id() {
        let err = SecureError::new(ErrorCategory::InternalError, "internal")
            .with_request_id("req_abc123");
        let display = format!("{}", err);
        assert!(display.contains("req_abc123"), "Display should contain request ID");
    }

    #[test]
    fn test_internal_display_has_details() {
        let err = SecureError::new(
            ErrorCategory::AuthenticationFailed,
            "user 'admin' not found",
        ).with_request_id("req_xyz");
        let internal = err.internal_display();
        assert!(internal.contains("admin"), "Internal display should contain username");
        assert!(internal.contains("req_xyz"), "Internal display should contain request ID");
    }

    #[test]
    fn test_internal_display_has_source() {
        let err = SecureError::new(ErrorCategory::InternalError, "query failed")
            .with_source("connection refused");
        let internal = err.internal_display();
        assert!(internal.contains("connection refused"), "Internal display should contain source");
    }

    #[test]
    fn test_auth_error_category() {
        let err = auth_error("user 'admin' wrong password", "req_1");
        assert_eq!(err.category, ErrorCategory::AuthenticationFailed);
    }

    #[test]
    fn test_auth_error_display_safe() {
        let err = auth_error("user 'admin' wrong password", "req_1");
        let display = format!("{}", err);
        assert!(!display.contains("admin"), "Auth error display must not leak username");
        assert!(!display.contains("wrong"), "Auth error display must not leak failure reason");
    }

    #[test]
    fn test_db_error_has_source() {
        let err = db_error("SELECT failed", "connection timeout", "req_2");
        let internal = err.internal_display();
        assert!(internal.contains("connection timeout"), "DB error should chain source");
    }

    #[test]
    fn test_db_error_display_safe() {
        let err = db_error("SELECT * FROM users", "connection timeout", "req_2");
        let display = format!("{}", err);
        assert!(!display.contains("SELECT"), "DB error display must not leak SQL");
        assert!(!display.contains("users"), "DB error display must not leak table name");
    }

    #[test]
    fn test_is_safe_error_response() {
        let safe = format!("{}", SecureError::new(ErrorCategory::InternalError, "internal"));
        assert!(is_safe_error_response(&safe, "SELECT * FROM users"));
    }

    #[test]
    fn test_is_safe_error_response_detects_leak() {
        let unsafe_response = "Error: SELECT * FROM users failed";
        assert!(!is_safe_error_response(unsafe_response, "SELECT * FROM users"));
    }
}
