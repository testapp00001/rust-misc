//! # Lesson 10: Logging Errors Securely
//!
//! ## The Problem
//!
//! Error logging has two conflicting goals:
//! - **Developers** need full context to debug: what failed, why, where, with what data
//! - **Security** requires that sensitive data never appears in logs
//!
//! The solution is a dual-track approach: log full details internally (server-side),
//! return generic messages to users (client-side). This lesson covers both sides.
//!
//! ## What to Log Internally (Server-Side)
//!
//! | Field | Always Log? | Example |
//! |-------|-------------|---------|
//! | Timestamp | Yes | `2024-01-15T10:30:00Z` |
//! | Error category | Yes | `AUTH_FAILED`, `VALIDATION_ERROR` |
//! | Request ID | Yes | `req_abc123` |
//! | User ID (if known) | Yes | `user_42` |
//! | Error details | Yes | `invalid password for user alice` |
//! | IP address | Yes | `192.168.1.100` |
//! | User agent | Yes | `Mozilla/5.0...` |
//! | Passwords | NEVER | `[REDACTED]` |
//! | Tokens | NEVER | `[REDACTED]` |
//! | Full request body | NEVER | Only sanitized fields |
//!
//! ## What to Return to Users (Client-Side)
//!
//! Only:
//! - Generic error message ("Authentication failed")
//! - Error code ("AUTH_FAILED")
//! - Request ID (for support correlation)
//! - HTTP status code

use std::collections::HashMap;

/// A structured log entry for error events.
#[derive(Debug, Clone)]
pub struct ErrorLogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub category: String,
    pub message: String,
    pub request_id: Option<String>,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub details: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
}

/// Exercise 1: Create a secure error log entry.
///
/// Given error details, create an `ErrorLogEntry` that:
/// - Sets the level based on the category:
///   - "AUTH" categories → Warn (auth failures are security events)
///   - "VALIDATION" categories → Info (user input errors)
///   - Everything else → Error
/// - Includes the request_id and user_id if provided
/// - Sanitizes the message: replace any value in `secrets` with "[REDACTED]"
///
/// The `secrets` parameter is a list of known secret values to redact from the message.
pub fn create_log_entry(
    timestamp: &str,
    category: &str,
    message: &str,
    request_id: Option<&str>,
    user_id: Option<&str>,
    ip_address: Option<&str>,
    secrets: &[&str],
) -> ErrorLogEntry {
    todo!("Create a secure error log entry with redacted secrets")
}

/// Exercise 2: Sanitize a log message by redacting secrets.
///
/// Given a message and a list of known secret values, replace each occurrence
/// of a secret value in the message with "[REDACTED]".
///
/// The replacement should be case-sensitive.
/// If a secret appears multiple times, replace all occurrences.
///
/// Hints:
/// - Use `str::replace()` for each secret
/// - Process secrets in order (in case of overlapping values)
pub fn sanitize_log_message(message: &str, secrets: &[&str]) -> String {
    todo!("Replace secret values with [REDACTED] in log message")
}

/// Exercise 3: Format a log entry as a structured string.
///
/// Format: `[{timestamp}] {level} {category}: {message} | request={request_id} user={user_id} ip={ip}`
///
/// Omit fields that are None.
/// Level should be: "ERROR", "WARN", or "INFO"
pub fn format_log_entry(entry: &ErrorLogEntry) -> String {
    todo!("Format log entry as structured string")
}

/// Exercise 4: Create the user-facing error response from a log entry.
///
/// Given an ErrorLogEntry, produce a JSON-like response for the user.
/// The response must contain ONLY:
/// - The category's user-facing message (use a mapping)
/// - The error code (same as category)
/// - The request_id (if present)
///
/// It must NOT contain:
/// - The internal message
/// - The user_id
/// - The IP address
/// - Any details
///
/// Mapping:
/// - "AUTH_FAILED" → "Authentication failed"
/// - "VALIDATION_ERROR" → "Invalid request"
/// - "PERMISSION_DENIED" → "Access denied"
/// - "RATE_LIMITED" → "Too many requests"
/// - Anything else → "An internal error occurred"
pub fn user_response_from_log(entry: &ErrorLogEntry) -> String {
    todo!("Create user-facing response from log entry (no internal details)")
}

/// Exercise 5: Validate that a log entry is safe to store.
///
/// Check that the log entry does not contain obvious sensitive data:
/// - The message must not contain "password", "secret", "token", or "key" followed by a value
/// - The details values must not contain these words
///
/// Return Ok(()) if safe, Err(description) if sensitive data detected.
///
/// Hints:
/// - Check the message and all detail values
/// - Look for patterns like "password=" or "token: "
pub fn validate_log_entry_safety(entry: &ErrorLogEntry) -> Result<(), String> {
    todo!("Validate that log entry doesn't contain sensitive data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_category_is_warn() {
        let entry = create_log_entry(
            "2024-01-15T10:30:00Z", "AUTH_FAILED", "login failed",
            Some("req_1"), Some("user_1"), Some("10.0.0.1"), &[],
        );
        assert_eq!(entry.level, LogLevel::Warn);
    }

    #[test]
    fn test_validation_category_is_info() {
        let entry = create_log_entry(
            "2024-01-15T10:30:00Z", "VALIDATION_ERROR", "bad input",
            Some("req_1"), None, None, &[],
        );
        assert_eq!(entry.level, LogLevel::Info);
    }

    #[test]
    fn test_other_category_is_error() {
        let entry = create_log_entry(
            "2024-01-15T10:30:00Z", "INTERNAL_ERROR", "db crashed",
            None, None, None, &[],
        );
        assert_eq!(entry.level, LogLevel::Error);
    }

    #[test]
    fn test_log_entry_redacts_secrets() {
        let entry = create_log_entry(
            "2024-01-15T10:30:00Z", "AUTH_FAILED",
            "user alice login with password hunter2",
            Some("req_1"), Some("user_1"), None,
            &["hunter2"],
        );
        assert!(!entry.message.contains("hunter2"), "Log must not contain secret");
        assert!(entry.message.contains("REDACTED"), "Log should contain REDACTED");
    }

    #[test]
    fn test_sanitize_message_redacts() {
        let msg = "User alice auth with key sk_live_abc123";
        let sanitized = sanitize_log_message(msg, &["sk_live_abc123"]);
        assert!(!sanitized.contains("sk_live_abc123"));
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_multiple_secrets() {
        let msg = "token=tok_abc password=pw123";
        let sanitized = sanitize_log_message(msg, &["tok_abc", "pw123"]);
        assert!(!sanitized.contains("tok_abc"));
        assert!(!sanitized.contains("pw123"));
    }

    #[test]
    fn test_format_log_entry() {
        let entry = ErrorLogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: LogLevel::Warn,
            category: "AUTH_FAILED".to_string(),
            message: "login failed".to_string(),
            request_id: Some("req_1".to_string()),
            user_id: Some("user_1".to_string()),
            ip_address: None,
            details: HashMap::new(),
        };
        let formatted = format_log_entry(&entry);
        assert!(formatted.contains("WARN"), "Should contain log level");
        assert!(formatted.contains("AUTH_FAILED"), "Should contain category");
        assert!(formatted.contains("req_1"), "Should contain request ID");
        assert!(!formatted.contains("ip="), "Should not contain None fields");
    }

    #[test]
    fn test_user_response_no_internal_details() {
        let mut details = HashMap::new();
        details.insert("db_query".to_string(), "SELECT * FROM users".to_string());

        let entry = ErrorLogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: LogLevel::Warn,
            category: "AUTH_FAILED".to_string(),
            message: "user alice wrong password".to_string(),
            request_id: Some("req_1".to_string()),
            user_id: Some("user_1".to_string()),
            ip_address: Some("10.0.0.1".to_string()),
            details,
        };
        let response = user_response_from_log(&entry);
        assert!(!response.contains("alice"), "Must not contain username");
        assert!(!response.contains("SELECT"), "Must not contain SQL");
        assert!(!response.contains("10.0.0.1"), "Must not contain IP");
        assert!(response.contains("req_1"), "Should contain request ID");
    }

    #[test]
    fn test_validate_log_safe() {
        let entry = ErrorLogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: LogLevel::Warn,
            category: "AUTH_FAILED".to_string(),
            message: "login failed for user alice".to_string(),
            request_id: None,
            user_id: None,
            ip_address: None,
            details: HashMap::new(),
        };
        assert!(validate_log_entry_safety(&entry).is_ok());
    }

    #[test]
    fn test_validate_log_detects_secret() {
        let entry = ErrorLogEntry {
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            level: LogLevel::Warn,
            category: "AUTH_FAILED".to_string(),
            message: "login failed password=hunter2".to_string(),
            request_id: None,
            user_id: None,
            ip_address: None,
            details: HashMap::new(),
        };
        assert!(validate_log_entry_safety(&entry).is_err());
    }
}
