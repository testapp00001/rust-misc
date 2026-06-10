//! # Lesson 10: Logging Errors Securely (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

/// Create a secure error log entry with redacted secrets.
pub fn create_log_entry(
    timestamp: &str,
    category: &str,
    message: &str,
    request_id: Option<&str>,
    user_id: Option<&str>,
    ip_address: Option<&str>,
    secrets: &[&str],
) -> ErrorLogEntry {
    // Determine log level based on category
    let level = if category.starts_with("AUTH") {
        LogLevel::Warn
    } else if category.starts_with("VALIDATION") {
        LogLevel::Info
    } else {
        LogLevel::Error
    };

    // Sanitize the message
    let sanitized_message = sanitize_log_message(message, secrets);

    ErrorLogEntry {
        timestamp: timestamp.to_string(),
        level,
        category: category.to_string(),
        message: sanitized_message,
        request_id: request_id.map(|s| s.to_string()),
        user_id: user_id.map(|s| s.to_string()),
        ip_address: ip_address.map(|s| s.to_string()),
        details: HashMap::new(),
    }
}

/// Replace secret values with [REDACTED] in log message.
pub fn sanitize_log_message(message: &str, secrets: &[&str]) -> String {
    let mut result = message.to_string();
    for secret in secrets {
        result = result.replace(secret, "[REDACTED]");
    }
    result
}

/// Format log entry as structured string.
pub fn format_log_entry(entry: &ErrorLogEntry) -> String {
    let level_str = match entry.level {
        LogLevel::Error => "ERROR",
        LogLevel::Warn => "WARN",
        LogLevel::Info => "INFO",
    };

    let mut parts = vec![
        format!("[{}]", entry.timestamp),
        format!("{}", level_str),
        format!("{}:", entry.category),
        entry.message.clone(),
    ];

    let mut meta = Vec::new();
    if let Some(ref req_id) = entry.request_id {
        meta.push(format!("request={}", req_id));
    }
    if let Some(ref uid) = entry.user_id {
        meta.push(format!("user={}", uid));
    }
    if let Some(ref ip) = entry.ip_address {
        meta.push(format!("ip={}", ip));
    }

    if !meta.is_empty() {
        parts.push(format!("| {}", meta.join(" ")));
    }

    parts.join(" ")
}

/// Create user-facing response from log entry (no internal details).
pub fn user_response_from_log(entry: &ErrorLogEntry) -> String {
    let (user_message, code) = match entry.category.as_str() {
        "AUTH_FAILED" => ("Authentication failed", "AUTH_FAILED"),
        "VALIDATION_ERROR" => ("Invalid request", "VALIDATION_ERROR"),
        "PERMISSION_DENIED" => ("Access denied", "PERMISSION_DENIED"),
        "RATE_LIMITED" => ("Too many requests", "RATE_LIMITED"),
        _ => ("An internal error occurred", "INTERNAL_ERROR"),
    };

    match &entry.request_id {
        Some(req_id) => format!(
            r#"{{"error": "{}", "code": "{}", "request_id": "{}"}}"#,
            user_message, code, req_id
        ),
        None => format!(
            r#"{{"error": "{}", "code": "{}"}}"#,
            user_message, code
        ),
    }
}

/// Validate that log entry doesn't contain sensitive data.
pub fn validate_log_entry_safety(entry: &ErrorLogEntry) -> Result<(), String> {
    let sensitive_patterns = ["password", "secret", "token", "key"];

    // Check message
    let lower_msg = entry.message.to_lowercase();
    for pattern in &sensitive_patterns {
        if lower_msg.contains(&format!("{}=", pattern))
            || lower_msg.contains(&format!("{}: ", pattern))
            || lower_msg.contains(&format!("{}: ", pattern))
        {
            return Err(format!(
                "Log message contains sensitive data pattern: '{}'",
                pattern
            ));
        }
    }

    // Check detail values
    for (key, value) in &entry.details {
        let lower_val = value.to_lowercase();
        for pattern in &sensitive_patterns {
            if lower_val.contains(pattern) {
                return Err(format!(
                    "Detail '{}' contains sensitive data pattern: '{}'",
                    key, pattern
                ));
            }
        }
    }

    Ok(())
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
