//! # Lesson 02: Secure Error Types (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::fmt;

/// Error category for user-facing messages.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    AuthenticationFailed,
    AccessDenied,
    ValidationFailed,
    NotFound,
    InternalError,
    RateLimited,
}

impl ErrorCategory {
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
#[derive(Debug)]
pub struct SecureError {
    pub category: ErrorCategory,
    pub internal_message: String,
    pub request_id: Option<String>,
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

/// Display shows ONLY user-safe information.
impl fmt::Display for SecureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = self.category.error_code();
        match &self.request_id {
            Some(id) => write!(f, "{} (code: {}, request: {})", self.category.user_message(), code, id),
            None => write!(f, "{} (code: {})", self.category.user_message(), code),
        }
    }
}

/// Internal logging method with full details.
impl SecureError {
    pub fn internal_display(&self) -> String {
        let mut parts = vec![format!("[{}] {}", self.category.error_code(), self.internal_message)];
        if let Some(ref source) = self.source {
            parts.push(format!("caused by: {}", source));
        }
        if let Some(ref id) = self.request_id {
            parts.push(format!("request: {}", id));
        }
        parts.join(" | ")
    }
}

/// Create a secure authentication error.
pub fn auth_error(internal_reason: &str, request_id: &str) -> SecureError {
    SecureError::new(ErrorCategory::AuthenticationFailed, internal_reason)
        .with_request_id(request_id)
}

/// Create a secure database error.
pub fn db_error(db_message: &str, source: &str, request_id: &str) -> SecureError {
    SecureError::new(ErrorCategory::InternalError, db_message)
        .with_source(source)
        .with_request_id(request_id)
}

/// Validate that an error response is safe for users.
pub fn is_safe_error_response(response: &str, internal_message: &str) -> bool {
    // Check if the response contains the internal message
    if response.contains(internal_message) {
        return false;
    }

    let lower = response.to_lowercase();

    // Check for SQL keywords
    let sql_keywords = ["select ", "insert ", "update ", "delete ", "drop ", "create ", "alter "];
    for keyword in &sql_keywords {
        if lower.contains(keyword) {
            return false;
        }
    }

    // Check for file paths
    if response.contains('/') || response.contains('\\') {
        return false;
    }

    // Check for IP addresses
    if lower.contains("10.0.") || lower.contains("192.168.") || lower.contains("172.16.") {
        return false;
    }

    true
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
