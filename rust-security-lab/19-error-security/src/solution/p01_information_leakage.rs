//! # Lesson 01: Information Leakage in Error Messages (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::fmt;

/// A raw error that might contain sensitive internal details.
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

/// Sanitize an error message for user-facing responses.
///
/// Always returns a generic message -- never expose internal details.
pub fn sanitize_error_for_user(_error: &RawError) -> String {
    "An internal error occurred".to_string()
}

/// Extract safe fields for logging (server-side).
///
/// Includes ALL details -- this is for server-side logs only, never shown to users.
pub fn format_error_for_log(error: &RawError) -> String {
    let mut parts = vec![format!("ERROR: {}", error.message)];
    if let (Some(ref file), line) = (&error.file, error.line) {
        parts.push(format!("file={}:{}", file, line.unwrap_or(0)));
    }
    if let Some(ref query) = error.query {
        parts.push(format!("query={}", query));
    }
    if let Some(ref detail) = error.internal_detail {
        parts.push(format!("detail={}", detail));
    }
    parts.join(" | ")
}

/// Check if an error response contains sensitive information.
///
/// Returns true if ANY sensitive pattern is found (the response is UNSAFE).
pub fn contains_sensitive_info(response: &str) -> bool {
    let lower = response.to_lowercase();

    // File paths
    if response.contains('/') || response.contains('\\') {
        return true;
    }

    // SQL keywords
    let sql_keywords = ["select ", "insert ", "update ", "delete ", "drop ", "create ", "alter "];
    for keyword in &sql_keywords {
        if lower.contains(keyword) {
            return true;
        }
    }

    // Line number patterns (":digit" at end of path-like strings)
    if lower.contains("line ") {
        return true;
    }

    // Internal IP addresses
    if lower.contains("10.0.") || lower.contains("192.168.") || lower.contains("172.16.") {
        return true;
    }

    // Sensitive keywords
    let sensitive_words = ["password", "secret", "token", "key"];
    for word in &sensitive_words {
        if lower.contains(word) {
            return true;
        }
    }

    false
}

/// Create a safe API error response with request_id for correlation.
pub fn safe_api_error_response(_error: &RawError, request_id: &str) -> String {
    format!(
        "{{\"error\": \"An internal error occurred\", \"request_id\": \"{}\"}}",
        request_id
    )
}

/// Demonstrate the "double representation" pattern.
///
/// Returns internal or external representation based on `for_user` flag.
pub fn error_representation(internal: &str, external: &str, for_user: bool) -> String {
    if for_user {
        external.to_string()
    } else {
        internal.to_string()
    }
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
