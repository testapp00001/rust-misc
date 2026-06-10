//! # Lesson 09: Auditing Secret Access
//!
//! ## Why Audit?
//!
//! Every access to a secret should be logged. Without audit logs:
//! - You can't detect unauthorized access
//! - You can't prove compliance (SOC2, HIPAA, PCI-DSS)
//! - You can't investigate incidents
//! - You can't identify over-privileged accounts
//!
//! ## What to Log
//!
//! For every secret access:
//! - **Who**: Which identity (user, service, application) accessed the secret
//! - **What**: Which secret was accessed (by name/path, never log the value!)
//! - **When**: Timestamp of the access
//! - **Where**: Source IP, service, or environment
//! - **Why**: The operation (read, write, rotate, delete)
//! - **Result**: Success or failure
//!
//! ## What NOT to Log
//!
//! NEVER log:
//! - The actual secret value
//! - The decrypted secret
//! - The secret in error messages
//! - The secret in stack traces
//!
//! ## Attack: Insufficient Logging
//!
//! Without proper audit logs:
//! - Stolen credentials go undetected for months (average: 200+ days)
//! - Insider threats are invisible
//! - Compliance audits fail
//! - Incident response is impossible
//!
//! ## Defense
//!
//! 1. Log every secret access to an append-only audit log
//! 2. Alert on unusual access patterns
//! 3. Retain logs for compliance-required periods
//! 4. Protect audit logs from tampering
//! 5. Review logs regularly (not just after incidents)

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

/// The type of operation performed on a secret.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SecretOperation {
    Read,
    Write,
    Rotate,
    Delete,
    List,
    Deny,
}

/// An audit log entry for a secret access event.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// When the access occurred.
    pub timestamp: Instant,
    /// Who accessed the secret (identity).
    pub identity: String,
    /// Which secret was accessed (path/name).
    pub secret_path: String,
    /// What operation was performed.
    pub operation: SecretOperation,
    /// Whether the access was successful.
    pub success: bool,
    /// Source IP or service identifier.
    pub source: String,
    /// Optional error message (must NOT contain secret values).
    pub error: Option<String>,
}

/// An audit logger that tracks all secret access events.
#[derive(Debug)]
pub struct AuditLogger {
    entries: Vec<AuditEntry>,
    /// Alert threshold: max accesses per identity per time window.
    max_accesses_per_window: usize,
    /// Time window for rate-based alerts.
    alert_window: Duration,
}

impl AuditLogger {
    pub fn new(max_accesses_per_window: usize, alert_window: Duration) -> Self {
        Self {
            entries: Vec::new(),
            max_accesses_per_window,
            alert_window,
        }
    }
}

/// Exercise 1: Log a secret access event.
///
/// Create an AuditEntry and add it to the logger's entries.
///
/// Parameters:
/// - `logger`: the audit logger
/// - `identity`: who is accessing
/// - `secret_path`: which secret
/// - `operation`: what operation
/// - `success`: whether it succeeded
/// - `source`: where the access came from
///
/// Hints:
/// - Create an AuditEntry with `Instant::now()` as the timestamp
/// - Push it onto `logger.entries`
pub fn log_access(
    logger: &mut AuditLogger,
    identity: &str,
    secret_path: &str,
    operation: SecretOperation,
    success: bool,
    source: &str,
) {
    todo!("Log a secret access event")
}

/// Exercise 2: Get all access entries for a specific secret.
///
/// Return a Vec of references to all AuditEntry items that match the given
/// secret_path, in chronological order.
///
/// Hints:
/// - Filter entries by `secret_path`
/// - Collect references
pub fn get_access_history<'a>(logger: &'a AuditLogger, secret_path: &str) -> Vec<&'a AuditEntry> {
    todo!("Get all access entries for a specific secret")
}

/// Exercise 3: Get all access entries for a specific identity.
///
/// Return a Vec of references to all AuditEntry items for the given identity.
///
/// Hints:
/// - Filter by `identity`
/// - Collect references
pub fn get_identity_history<'a>(logger: &'a AuditLogger, identity: &str) -> Vec<&'a AuditEntry> {
    todo!("Get all access entries for a specific identity")
}

/// Exercise 4: Detect anomalous access patterns.
///
/// Return a Vec of identity strings that have exceeded the
/// `max_accesses_per_window` threshold within the alert window.
///
/// The alert window is measured backward from the most recent entry.
///
/// Hints:
/// - Find the most recent timestamp in the entries
/// - Compute the window start: most_recent - alert_window
/// - Count accesses per identity within the window
/// - Return identities that exceed the threshold
pub fn detect_anomalies(logger: &AuditLogger) -> Vec<String> {
    todo!("Detect identities with excessive secret access")
}

/// Exercise 5: Generate an audit report.
///
/// Return a human-readable string summarizing the audit log.
///
/// Format:
/// ```
/// Audit Report
/// ============
/// Total events: N
/// Successful: N
/// Failed: N
///
/// Access by operation:
///   Read: N
///   Write: N
///   ...
///
/// Top accessed secrets:
///   /path/to/secret: N accesses
///   ...
/// ```
///
/// Hints:
/// - Count total, successful, and failed events
/// - Group by operation type and count
/// - Group by secret_path and count, sort by count descending
/// - Take top 5 for the "Top accessed secrets" section
pub fn generate_audit_report(logger: &AuditLogger) -> String {
    todo!("Generate a human-readable audit report")
}

/// Exercise 6: Get failed access attempts.
///
/// Return a Vec of references to all entries where `success == false`.
///
/// Hints:
/// - Filter by `success == false`
/// - Collect references
pub fn get_failed_accesses<'a>(logger: &'a AuditLogger) -> Vec<&'a AuditEntry> {
    todo!("Get all failed access attempts")
}

/// Exercise 7: Validate that an error message does not contain secret values.
///
/// Given an error message and a list of known secret values, return:
/// - `Ok(())` if the error message is safe (doesn't contain any secrets)
/// - `Err(secret_snippet)` if the error message contains a secret value,
///   returning the first 8 characters of the detected secret
///
/// Hints:
/// - For each known secret, check if the error message contains it
/// - If a match is found, return Err with the first 8 chars
/// - If no matches, return Ok(())
pub fn validate_error_message(error_msg: &str, known_secrets: &[&str]) -> Result<(), String> {
    todo!("Check that error messages don't leak secrets")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_logger() -> AuditLogger {
        AuditLogger::new(5, Duration::from_secs(60))
    }

    #[test]
    fn test_log_access_and_retrieve() {
        let mut logger = create_test_logger();
        log_access(&mut logger, "alice", "db/password", SecretOperation::Read, true, "10.0.0.1");

        let history = get_access_history(&logger, "db/password");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].identity, "alice");
    }

    #[test]
    fn test_get_access_history_multiple() {
        let mut logger = create_test_logger();
        log_access(&mut logger, "alice", "db/password", SecretOperation::Read, true, "10.0.0.1");
        log_access(&mut logger, "bob", "db/password", SecretOperation::Read, true, "10.0.0.2");
        log_access(&mut logger, "alice", "api/key", SecretOperation::Read, true, "10.0.0.1");

        let history = get_access_history(&logger, "db/password");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_get_identity_history() {
        let mut logger = create_test_logger();
        log_access(&mut logger, "alice", "db/password", SecretOperation::Read, true, "10.0.0.1");
        log_access(&mut logger, "alice", "api/key", SecretOperation::Read, true, "10.0.0.1");
        log_access(&mut logger, "bob", "db/password", SecretOperation::Read, true, "10.0.0.2");

        let history = get_identity_history(&logger, "alice");
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_detect_anomalies() {
        let mut logger = create_test_logger();
        // Alice accesses 6 times (threshold is 5)
        for _ in 0..6 {
            log_access(&mut logger, "alice", "secret1", SecretOperation::Read, true, "10.0.0.1");
        }
        // Bob accesses 3 times (under threshold)
        for _ in 0..3 {
            log_access(&mut logger, "bob", "secret2", SecretOperation::Read, true, "10.0.0.2");
        }

        let anomalies = detect_anomalies(&logger);
        assert!(anomalies.contains(&"alice".to_string()));
        assert!(!anomalies.contains(&"bob".to_string()));
    }

    #[test]
    fn test_generate_audit_report() {
        let mut logger = create_test_logger();
        log_access(&mut logger, "alice", "db/password", SecretOperation::Read, true, "10.0.0.1");
        log_access(&mut logger, "bob", "db/password", SecretOperation::Read, false, "10.0.0.2");
        log_access(&mut logger, "alice", "api/key", SecretOperation::Rotate, true, "10.0.0.1");

        let report = generate_audit_report(&logger);
        assert!(report.contains("Total events: 3"));
        assert!(report.contains("Successful: 2"));
        assert!(report.contains("Failed: 1"));
    }

    #[test]
    fn test_get_failed_accesses() {
        let mut logger = create_test_logger();
        log_access(&mut logger, "alice", "db/password", SecretOperation::Read, true, "10.0.0.1");
        log_access(&mut logger, "bob", "db/password", SecretOperation::Read, false, "10.0.0.2");
        log_access(&mut logger, "eve", "api/key", SecretOperation::Read, false, "10.0.0.3");

        let failed = get_failed_accesses(&logger);
        assert_eq!(failed.len(), 2);
    }

    #[test]
    fn test_validate_error_message_safe() {
        let result = validate_error_message("Connection refused", &["password123"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_error_message_leaks_secret() {
        let result = validate_error_message("Auth failed: password123 invalid", &["password123"]);
        assert!(result.is_err());
    }
}
