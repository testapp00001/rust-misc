//! # Lesson 07: Audit Logging Middleware
//!
//! ## Why Audit Logging?
//!
//! When a security incident occurs, you need to answer:
//! - **Who** made the request? (user ID, IP address)
//! - **What** did they do? (method, path, action)
//! - **When** did it happen? (timestamp)
//! - **What was the result?** (status code, success/failure)
//! - **Was it authorized?** (authorization decision)
//!
//! Without audit logs, breaches go undetected. With good audit logs, you can
//! reconstruct the full timeline of an attack and identify compromised accounts.
//!
//! ## What to Log (Security Events)
//!
//! - Authentication attempts (success and failure)
//! - Authorization decisions (allowed and denied)
//! - Rate limit violations
//! - Input validation failures
//! - Request signing failures
//! - Administrative actions (user creation, role changes)
//! - Data access (who read what)
//!
//! ## What NOT to Log
//!
//! - Passwords or secrets
//! - Full credit card numbers
//! - Session tokens or JWT secrets
//! - Personal data beyond what is needed for security
//!
//! ## Attack Context
//!
//! - **Log injection**: Attacker includes newlines or control characters in logged values
//!   to forge log entries. Defense: sanitize all logged values.
//! - **Log tampering**: Attacker modifies or deletes logs. Defense: write to append-only
//!   storage, use tamper-evident chains.
//! - **Information leakage**: Logs contain secrets. Defense: redact sensitive fields.

use serde::{Deserialize, Serialize};

/// Severity levels for audit log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    /// Informational event (successful login, normal request)
    Info,
    /// Warning (rate limit approaching, unusual pattern)
    Warning,
    /// Security concern (failed auth, authorization denied)
    Security,
    /// Critical security event (multiple failed logins, potential breach)
    Critical,
}

/// The type of security event being logged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityEvent {
    /// Authentication attempt
    Authentication {
        success: bool,
        user_id: Option<String>,
    },
    /// Authorization decision
    Authorization {
        allowed: bool,
        user_id: String,
        permission: String,
    },
    /// Rate limit event
    RateLimitExceeded {
        key: String,
    },
    /// Input validation failure
    ValidationFailed {
        field: String,
        reason: String,
    },
    /// Request signature verification
    SignatureVerification {
        success: bool,
    },
    /// Administrative action
    AdminAction {
        user_id: String,
        action: String,
    },
    /// Generic security event
    Custom {
        description: String,
    },
}

/// A structured audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique identifier for this log entry
    pub id: String,
    /// When the event occurred (Unix timestamp, milliseconds)
    pub timestamp: u64,
    /// Severity level
    pub severity: Severity,
    /// The security event
    pub event: SecurityEvent,
    /// Client IP address
    pub client_ip: String,
    /// HTTP method (if applicable)
    pub method: Option<String>,
    /// Request path (if applicable)
    pub path: Option<String>,
    /// HTTP response status code (if applicable)
    pub status_code: Option<u16>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Correlation ID for request tracing
    pub correlation_id: String,
}

/// The audit logger that collects log entries.
#[derive(Debug)]
pub struct AuditLogger {
    /// Collected log entries
    pub entries: Vec<AuditEntry>,
    /// Counter for generating unique IDs
    next_id: u64,
}

impl AuditLogger {
    /// Create a new audit logger.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }
}

/// Exercise 1: Create an audit entry for an authentication attempt.
///
/// Build an AuditEntry with:
/// - A unique ID (format: "audit-{number}")
/// - The given timestamp
/// - Severity: Info for success, Security for failure
/// - The Authentication event with success flag and user_id
/// - The given client_ip
/// - The correlation_id
///
/// Return the entry and increment the logger's next_id counter.
pub fn log_authentication(
    logger: &mut AuditLogger,
    timestamp: u64,
    client_ip: &str,
    user_id: Option<String>,
    success: bool,
    correlation_id: &str,
) -> AuditEntry {
    todo!("Create authentication audit log entry")
}

/// Exercise 2: Create an audit entry for an authorization decision.
///
/// Build an AuditEntry with:
/// - Severity: Info for allowed, Security for denied
/// - The Authorization event with allowed flag, user_id, and permission
/// - Include method and path from the request
pub fn log_authorization(
    logger: &mut AuditLogger,
    timestamp: u64,
    client_ip: &str,
    user_id: &str,
    permission: &str,
    allowed: bool,
    method: &str,
    path: &str,
    correlation_id: &str,
) -> AuditEntry {
    todo!("Create authorization audit log entry")
}

/// Exercise 3: Sanitize a string for safe logging.
///
/// Remove or replace characters that could be used for log injection:
/// - Replace newlines (\n, \r) with spaces
/// - Replace tabs with spaces
/// - Replace null bytes (\0) with empty string
/// - Truncate to max_len characters
///
/// Return the sanitized string.
pub fn sanitize_for_log(input: &str, max_len: usize) -> String {
    todo!("Sanitize a string for safe logging")
}

/// Exercise 4: Implement a log entry redactor.
///
/// Given an AuditEntry, return a redacted copy where sensitive fields are masked:
/// - client_ip: replace last octet with "xxx" (e.g., "192.168.1.100" -> "192.168.1.xxx")
/// - user_agent: replace with "***" if present
///
/// This is for compliance: logs should be useful for security but not expose
/// unnecessary personal data.
pub fn redact_entry(entry: &AuditEntry) -> AuditEntry {
    todo!("Redact sensitive fields from audit entry")
}

/// Exercise 5: Count security events by severity.
///
/// Given a slice of AuditEntry, count how many entries exist for each severity level.
/// Return (info_count, warning_count, security_count, critical_count).
pub fn count_by_severity(entries: &[AuditEntry]) -> (usize, usize, usize, usize) {
    todo!("Count audit entries by severity level")
}

/// Exercise 6: Filter entries for a specific user.
///
/// Return all entries where the SecurityEvent contains the given user_id.
/// Check all event variants that include a user_id field.
pub fn filter_by_user(entries: &[AuditEntry], user_id: &str) -> Vec<&AuditEntry> {
    todo!("Filter audit entries by user ID")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_logger() -> AuditLogger {
        AuditLogger::new()
    }

    #[test]
    fn test_log_authentication_success() {
        let mut logger = make_logger();
        let entry = log_authentication(
            &mut logger, 1700000000, "10.0.0.1", Some("alice".to_string()), true, "req-001",
        );
        assert_eq!(entry.severity, Severity::Info);
        assert_eq!(
            entry.event,
            SecurityEvent::Authentication { success: true, user_id: Some("alice".to_string()) }
        );
        assert_eq!(entry.id, "audit-1");
    }

    #[test]
    fn test_log_authentication_failure() {
        let mut logger = make_logger();
        let entry = log_authentication(
            &mut logger, 1700000000, "10.0.0.1", None, false, "req-002",
        );
        assert_eq!(entry.severity, Severity::Security);
        assert_eq!(
            entry.event,
            SecurityEvent::Authentication { success: false, user_id: None }
        );
    }

    #[test]
    fn test_log_authorization() {
        let mut logger = make_logger();
        let entry = log_authorization(
            &mut logger, 1700000000, "10.0.0.1", "alice", "users:read", true, "GET", "/api/users", "req-003",
        );
        assert_eq!(entry.severity, Severity::Info);
        assert_eq!(entry.method, Some("GET".to_string()));
        assert_eq!(entry.path, Some("/api/users".to_string()));
    }

    #[test]
    fn test_logger_id_increments() {
        let mut logger = make_logger();
        let e1 = log_authentication(&mut logger, 1, "ip", None, true, "c1");
        let e2 = log_authentication(&mut logger, 2, "ip", None, true, "c2");
        assert_eq!(e1.id, "audit-1");
        assert_eq!(e2.id, "audit-2");
    }

    #[test]
    fn test_sanitize_for_log_newlines() {
        let result = sanitize_for_log("line1\nline2\rline3", 100);
        assert!(!result.contains('\n'));
        assert!(!result.contains('\r'));
    }

    #[test]
    fn test_sanitize_for_log_null_bytes() {
        let result = sanitize_for_log("hello\0world", 100);
        assert!(!result.contains('\0'));
    }

    #[test]
    fn test_sanitize_for_log_truncation() {
        let result = sanitize_for_log("this is a long string", 10);
        assert!(result.len() <= 10);
    }

    #[test]
    fn test_redact_entry_ip() {
        let mut logger = make_logger();
        let entry = log_authentication(
            &mut logger, 1700000000, "192.168.1.100", Some("alice".to_string()), true, "req-001",
        );
        let redacted = redact_entry(&entry);
        assert_eq!(redacted.client_ip, "192.168.1.xxx");
    }

    #[test]
    fn test_redact_entry_user_agent() {
        let entry = AuditEntry {
            id: "audit-1".to_string(),
            timestamp: 1700000000,
            severity: Severity::Info,
            event: SecurityEvent::Custom { description: "test".to_string() },
            client_ip: "10.0.0.1".to_string(),
            method: None,
            path: None,
            status_code: None,
            user_agent: Some("Mozilla/5.0 ...".to_string()),
            correlation_id: "req-001".to_string(),
        };
        let redacted = redact_entry(&entry);
        assert_eq!(redacted.user_agent, Some("***".to_string()));
    }

    #[test]
    fn test_count_by_severity() {
        let entries = vec![
            AuditEntry {
                id: "1".to_string(), timestamp: 1, severity: Severity::Info,
                event: SecurityEvent::Custom { description: "a".to_string() },
                client_ip: "ip".to_string(), method: None, path: None, status_code: None,
                user_agent: None, correlation_id: "c".to_string(),
            },
            AuditEntry {
                id: "2".to_string(), timestamp: 2, severity: Severity::Security,
                event: SecurityEvent::Custom { description: "b".to_string() },
                client_ip: "ip".to_string(), method: None, path: None, status_code: None,
                user_agent: None, correlation_id: "c".to_string(),
            },
            AuditEntry {
                id: "3".to_string(), timestamp: 3, severity: Severity::Info,
                event: SecurityEvent::Custom { description: "c".to_string() },
                client_ip: "ip".to_string(), method: None, path: None, status_code: None,
                user_agent: None, correlation_id: "c".to_string(),
            },
        ];
        let (info, warning, security, critical) = count_by_severity(&entries);
        assert_eq!(info, 2);
        assert_eq!(warning, 0);
        assert_eq!(security, 1);
        assert_eq!(critical, 0);
    }

    #[test]
    fn test_filter_by_user() {
        let entries = vec![
            AuditEntry {
                id: "1".to_string(), timestamp: 1, severity: Severity::Info,
                event: SecurityEvent::Authentication { success: true, user_id: Some("alice".to_string()) },
                client_ip: "ip".to_string(), method: None, path: None, status_code: None,
                user_agent: None, correlation_id: "c".to_string(),
            },
            AuditEntry {
                id: "2".to_string(), timestamp: 2, severity: Severity::Info,
                event: SecurityEvent::Authentication { success: true, user_id: Some("bob".to_string()) },
                client_ip: "ip".to_string(), method: None, path: None, status_code: None,
                user_agent: None, correlation_id: "c".to_string(),
            },
        ];
        let alice_entries = filter_by_user(&entries, "alice");
        assert_eq!(alice_entries.len(), 1);
        assert_eq!(alice_entries[0].id, "1");
    }
}
