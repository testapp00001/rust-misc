//! # Lesson 01: Information Leakage in Error Messages
//!
//! ## The Problem
//!
//! Error messages are written for developers, but they often reach users. When an error
//! response contains a stack trace, SQL query, file path, or framework version, it hands
//! attackers a detailed map of your internal infrastructure.
//!
//! ## Real-World Examples
//!
//! | Leaked Info | What Attacker Learns | Impact |
//! |-------------|---------------------|--------|
//! | `Error at /home/app/src/db.rs:42` | File structure, language (Rust) | Targeted attacks |
//! | `SQL: SELECT * FROM users WHERE id='` | DB schema, table names | SQL injection targeting |
//! | `thread 'main' panicked at...` | Framework, version, panic location | CVE exploitation |
//! | `Connection refused to 10.0.1.5:5432` | Internal network topology | Lateral movement |
//! | `Caused by: rusqlite error: ...` | Database driver, version | Supply chain attacks |
//!
//! ## Attack Demo
//!
//! ```text
//! # User sends malformed input to an API:
//! POST /api/users {"id": "'; DROP TABLE users;--"}
//!
//! # VULNERABLE response (leaks DB schema):
//! {
//!   "error": "Database error: rusqlite error: no such column: ''",
//!   "query": "SELECT * FROM users WHERE id = ''",
//!   "file": "/app/src/db.rs",
//!   "line": 42
//! }
//!
//! # SECURE response (generic):
//! {
//!   "error": "Invalid request",
//!   "request_id": "req_abc123"
//! }
//! ```
//!
//! ## Defense Strategy
//!
//! 1. **Sanitize all error messages** before returning to users
//! 2. **Log full errors server-side** for debugging
//! 3. **Use structured error types** that separate internal vs external representations
//! 4. **Return request IDs** so users can reference errors without exposing details

use std::fmt;

/// A raw error that might contain sensitive internal details.
/// This simulates what happens when an error propagates from a database,
/// file system, or network layer without sanitization.
#[derive(Debug)]
pub struct RawError {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub query: Option<String>,
    pub internal_detail: Option<String>,
}

impl fmt::Display for RawError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(ref file) = self.file {
            write!(f, " at {}:{}", file, self.line.unwrap_or(0))?;
        }
        if let Some(ref query) = self.query {
            write!(f, " [query: {}]", query)?;
        }
        if let Some(ref detail) = self.internal_detail {
            write!(f, " ({})", detail)?;
        }
        Ok(())
    }
}

impl std::error::Error for RawError {}

/// Exercise 1: Sanitize an error message for user-facing responses.
///
/// Given a raw error, produce a safe string that:
/// - Contains ONLY a generic, user-friendly message
/// - Does NOT contain file paths, line numbers, SQL queries, or internal details
/// - Returns "An internal error occurred" for any error
///
/// Hints:
/// - Don't use `Display` on `RawError` -- it includes all the sensitive fields
/// - Return a fixed, generic string regardless of the error content
pub fn sanitize_error_for_user(_error: &RawError) -> String {
    todo!("Return a generic error message without any internal details")
}

/// Exercise 2: Extract safe fields for logging (server-side).
///
/// Given a raw error, return a string with ALL details for server-side logging.
/// This is the opposite of `sanitize_error_for_user` -- include everything.
///
/// Format: "ERROR: {message} | file={file}:{line} | query={query} | detail={internal}"
/// Omit fields that are `None`.
///
/// Hints:
/// - Build the string piece by piece
/// - Start with "ERROR: {message}"
/// - Append each field only if it's `Some`
pub fn format_error_for_log(error: &RawError) -> String {
    todo!("Format error with all details for server-side logging")
}

/// Exercise 3: Check if an error response contains sensitive information.
///
/// Given a response string that might be sent to a user, check if it contains
/// any of these sensitive patterns:
/// - File paths (containing "/" or "\")
/// - SQL keywords (SELECT, INSERT, UPDATE, DELETE, DROP, CREATE, ALTER)
/// - Line numbers (patterns like ":42" or "line 42")
/// - IP addresses (patterns like "10.0." or "192.168.")
/// - The word "password", "secret", "token", or "key"
///
/// Return true if ANY sensitive pattern is found (the response is UNSAFE).
///
/// Hints:
/// - Convert to lowercase for case-insensitive matching
/// - Check for substrings, not exact matches
/// - The check for line numbers can look for ":digit" patterns
pub fn contains_sensitive_info(response: &str) -> bool {
    todo!("Check if response contains sensitive patterns")
}

/// Exercise 4: Create a safe API error response.
///
/// Given a raw error and a request ID, produce a JSON-like string response
/// that is safe for users. Format:
///
/// `{"error": "An internal error occurred", "request_id": "{request_id}"}`
///
/// The error field must always be the same generic message.
/// The request_id field should use the provided ID.
///
/// Hints:
/// - Use `format!()` to build the string
/// - Never include any content from the RawError
pub fn safe_api_error_response(_error: &RawError, request_id: &str) -> String {
    todo!("Create safe API error response with request_id")
}

/// Exercise 5: Demonstrate the "double representation" pattern.
///
/// A secure error type needs two representations:
/// - Internal: full details for logging
/// - External: sanitized for users
///
/// Given a tuple of (internal_message, external_message), return whichever
/// representation is requested based on the `for_user` flag.
///
/// If `for_user` is true, return the external message.
/// If `for_user` is false, return the internal message.
pub fn error_representation(internal: &str, external: &str, for_user: bool) -> String {
    todo!("Return internal or external representation based on for_user flag")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_raw_error() -> RawError {
        RawError {
            message: "Failed to execute query".to_string(),
            file: Some("/home/app/src/db.rs".to_string()),
            line: Some(42),
            query: Some("SELECT * FROM users WHERE id = 1".to_string()),
            internal_detail: Some("rusqlite error: no such column".to_string()),
        }
    }

    #[test]
    fn test_sanitize_no_file_path() {
        let error = sample_raw_error();
        let sanitized = sanitize_error_for_user(&error);
        assert!(!sanitized.contains("/home"), "Sanitized error must not contain file paths");
        assert!(!sanitized.contains("db.rs"), "Sanitized error must not contain file names");
    }

    #[test]
    fn test_sanitize_no_sql() {
        let error = sample_raw_error();
        let sanitized = sanitize_error_for_user(&error);
        assert!(!sanitized.contains("SELECT"), "Sanitized error must not contain SQL");
        assert!(!sanitized.contains("users"), "Sanitized error must not contain table names");
    }

    #[test]
    fn test_sanitize_no_line_number() {
        let error = sample_raw_error();
        let sanitized = sanitize_error_for_user(&error);
        assert!(!sanitized.contains("42"), "Sanitized error must not contain line numbers");
    }

    #[test]
    fn test_sanitize_is_generic() {
        let error = sample_raw_error();
        let sanitized = sanitize_error_for_user(&error);
        assert!(sanitized.contains("error") || sanitized.contains("Error"),
                "Sanitized error should be a generic message");
    }

    #[test]
    fn test_log_contains_all_details() {
        let error = sample_raw_error();
        let log_msg = format_error_for_log(&error);
        assert!(log_msg.contains("/home/app/src/db.rs"), "Log should contain file path");
        assert!(log_msg.contains("42"), "Log should contain line number");
        assert!(log_msg.contains("SELECT"), "Log should contain SQL query");
        assert!(log_msg.contains("rusqlite"), "Log should contain internal detail");
    }

    #[test]
    fn test_detects_file_path() {
        assert!(contains_sensitive_info("Error at /home/app/src/main.rs:42"));
    }

    #[test]
    fn test_detects_sql_in_response() {
        assert!(contains_sensitive_info("Query failed: SELECT * FROM users"));
    }

    #[test]
    fn test_detects_ip_address() {
        assert!(contains_sensitive_info("Connection to 10.0.1.5 refused"));
    }

    #[test]
    fn test_safe_response_has_no_details() {
        let error = sample_raw_error();
        let response = safe_api_error_response(&error, "req_abc123");
        assert!(!response.contains("/home"), "API response must not contain file paths");
        assert!(!response.contains("SELECT"), "API response must not contain SQL");
        assert!(response.contains("req_abc123"), "API response should contain request ID");
    }

    #[test]
    fn test_error_representation_user() {
        let result = error_representation("rusqlite error at db.rs:42", "An error occurred", true);
        assert_eq!(result, "An error occurred");
        assert!(!result.contains("rusqlite"), "User-facing error must not contain internals");
    }

    #[test]
    fn test_error_representation_internal() {
        let result = error_representation("rusqlite error at db.rs:42", "An error occurred", false);
        assert_eq!(result, "rusqlite error at db.rs:42");
    }
}
