//! # Lesson 06: Audit Logging for Database Access
//!
//! ## Why Audit Logging?
//!
//! Audit logging answers the critical question: **who accessed what data, when, and
//! from where?** Without it, you cannot:
//! - Detect unauthorized access
//! - Investigate breaches
//! - Prove compliance (GDPR, HIPAA, PCI-DSS, SOX)
//! - Identify insider threats
//!
//! ## What to Log
//!
//! ```text
//! MUST LOG:
//!   - All authentication attempts (success and failure)
//!   - All data access to sensitive tables
//!   - All data modifications (INSERT, UPDATE, DELETE)
//!   - All schema changes (CREATE, ALTER, DROP)
//!   - All privilege changes (GRANT, REVOKE)
//!   - All backup/restore operations
//!
//! MUST NOT LOG:
//!   - Actual password values
//!   - Full credit card numbers
//!   - Encryption keys
//!   - Session tokens
//! ```
//!
//! ## Log Entry Structure
//!
//! A well-structured audit log entry contains:
//! - Timestamp (UTC, with millisecond precision)
//! - Actor (user ID, IP address, service account)
//! - Action (SELECT, INSERT, UPDATE, DELETE, etc.)
//! - Resource (table, column, row ID)
//! - Status (success, failure, denied)
//! - Context (query hash, session ID, request ID)

use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity level for audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLevel {
    Info,
    Warning,
    Critical,
}

impl fmt::Display for AuditLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuditLevel::Info => write!(f, "INFO"),
            AuditLevel::Warning => write!(f, "WARN"),
            AuditLevel::Critical => write!(f, "CRIT"),
        }
    }
}

/// Type of database action being audited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    /// Read operation (SELECT)
    Read { table: String, columns: Vec<String> },
    /// Write operation (INSERT, UPDATE, DELETE)
    Write { table: String, operation: String },
    /// Schema change (CREATE, ALTER, DROP)
    SchemaChange { statement: String },
    /// Authentication event
    Auth { success: bool },
    /// Access denied
    AccessDenied { resource: String, reason: String },
}

/// A single audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// ISO 8601 timestamp (UTC)
    pub timestamp: String,
    /// User or service account identifier
    pub actor: String,
    /// IP address of the client
    pub source_ip: String,
    /// The action performed
    pub action: AuditAction,
    /// Severity level
    pub level: AuditLevel,
    /// Whether the action succeeded
    pub success: bool,
    /// Additional context (query hash, session ID, etc.)
    pub context: Option<String>,
}

/// The audit log — a thread-safe, append-only log of database access.
///
/// Exercise: Implement the audit logger.
pub struct AuditLogger {
    entries: Vec<AuditEntry>,
    /// Sensitive field names that should be redacted from logs
    sensitive_fields: Vec<String>,
}

impl AuditLogger {
    /// Create a new audit logger.
    ///
    /// Exercise: Initialize with empty entries and the given sensitive fields.
    pub fn new(sensitive_fields: Vec<String>) -> Self {
        todo!("Create a new AuditLogger")
    }

    /// Log a database read operation.
    ///
    /// Exercise: Create an AuditEntry for a SELECT operation and append it.
    ///
    /// Hints:
    /// - Use `AuditAction::Read` with the table and columns
    /// - Set level to `Info` for normal reads
    /// - Set timestamp to current UTC (use a simple counter or string for now)
    pub fn log_read(&mut self, actor: &str, source_ip: &str, table: &str, columns: &[&str]) {
        todo!("Log a database read operation")
    }

    /// Log a database write operation.
    ///
    /// Exercise: Create an AuditEntry for INSERT/UPDATE/DELETE.
    ///
    /// Hints:
    /// - Use `AuditAction::Write` with the table and operation type
    /// - Set level to `Warning` for writes (more sensitive than reads)
    pub fn log_write(&mut self, actor: &str, source_ip: &str, table: &str, operation: &str) {
        todo!("Log a database write operation")
    }

    /// Log an access denied event.
    ///
    /// Exercise: Create an AuditEntry for a denied request.
    ///
    /// Hints:
    /// - Use `AuditAction::AccessDenied`
    /// - Set level to `Critical`
    /// - Set success to false
    pub fn log_access_denied(&mut self, actor: &str, source_ip: &str, resource: &str, reason: &str) {
        todo!("Log an access denied event")
    }

    /// Log an authentication event.
    ///
    /// Exercise: Create an AuditEntry for login attempt.
    ///
    /// Hints:
    /// - Use `AuditAction::Auth`
    /// - Level depends on success: `Info` for success, `Warning` for failure
    pub fn log_auth(&mut self, actor: &str, source_ip: &str, success: bool) {
        todo!("Log an authentication event")
    }

    /// Get all entries as JSON.
    ///
    /// Exercise: Serialize the entries to a JSON string.
    ///
    /// Hints:
    /// - Use `serde_json::to_string_pretty(&self.entries)`
    pub fn to_json(&self) -> String {
        todo!("Serialize audit entries to JSON")
    }

    /// Get entries filtered by actor.
    ///
    /// Exercise: Return entries where the actor matches.
    pub fn get_entries_by_actor(&self, actor: &str) -> Vec<&AuditEntry> {
        todo!("Filter entries by actor")
    }

    /// Get entries filtered by severity level.
    ///
    /// Exercise: Return entries with the given level or higher severity.
    /// Critical > Warning > Info.
    pub fn get_entries_by_level(&self, min_level: AuditLevel) -> Vec<&AuditEntry> {
        todo!("Filter entries by minimum severity level")
    }

    /// Count entries by action type.
    ///
    /// Exercise: Return counts of reads, writes, and denied actions.
    pub fn count_actions(&self) -> (usize, usize, usize) {
        todo!("Return (reads, writes, denied) counts")
    }
}

/// Demonstrate audit logging.
///
/// Exercise: Create a logger, log various events, and return the counts.
///
/// Hints:
/// - Create a logger with sensitive_fields = ["ssn", "password", "credit_card"]
/// - Log several reads, writes, auth events, and denied access
/// - Return (total_entries, critical_count, warning_count)
pub fn demonstrate_audit_logging() -> (usize, usize, usize) {
    todo!("Demonstrate audit logging and return counts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = AuditLogger::new(vec!["ssn".to_string(), "password".to_string()]);
        assert_eq!(logger.to_json(), "[]");
    }

    #[test]
    fn test_log_read() {
        let mut logger = AuditLogger::new(vec![]);
        logger.log_read("alice", "10.0.0.1", "users", &["name", "email"]);
        let entries = logger.get_entries_by_actor("alice");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_log_write() {
        let mut logger = AuditLogger::new(vec![]);
        logger.log_write("bob", "10.0.0.2", "orders", "INSERT");
        let entries = logger.get_entries_by_actor("bob");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_log_auth_events() {
        let mut logger = AuditLogger::new(vec![]);
        logger.log_auth("alice", "10.0.0.1", true);
        logger.log_auth("eve", "10.0.0.99", false);

        let alice_entries = logger.get_entries_by_actor("alice");
        assert_eq!(alice_entries.len(), 1);
        assert!(alice_entries[0].success);

        let eve_entries = logger.get_entries_by_actor("eve");
        assert_eq!(eve_entries.len(), 1);
        assert!(!eve_entries[0].success);
    }

    #[test]
    fn test_log_access_denied() {
        let mut logger = AuditLogger::new(vec![]);
        logger.log_access_denied("eve", "10.0.0.99", "admin_panel", "insufficient privileges");
        let entries = logger.get_entries_by_actor("eve");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].level, AuditLevel::Critical);
    }

    #[test]
    fn test_filter_by_level() {
        let mut logger = AuditLogger::new(vec![]);
        logger.log_read("alice", "10.0.0.1", "users", &["name"]);
        logger.log_write("bob", "10.0.0.2", "users", "UPDATE");
        logger.log_access_denied("eve", "10.0.0.99", "users", "no access");

        let critical = logger.get_entries_by_level(AuditLevel::Critical);
        assert_eq!(critical.len(), 1, "Should find 1 critical entry");

        let warnings = logger.get_entries_by_level(AuditLevel::Warning);
        assert!(warnings.len() >= 2, "Should find at least 2 warning+ entries");
    }

    #[test]
    fn test_count_actions() {
        let mut logger = AuditLogger::new(vec![]);
        logger.log_read("a", "1.1.1.1", "t", &["c"]);
        logger.log_read("b", "2.2.2.2", "t", &["c"]);
        logger.log_write("c", "3.3.3.3", "t", "INSERT");
        logger.log_access_denied("d", "4.4.4.4", "t", "denied");

        let (reads, writes, denied) = logger.count_actions();
        assert_eq!(reads, 2);
        assert_eq!(writes, 1);
        assert_eq!(denied, 1);
    }

    #[test]
    fn test_to_json_is_valid() {
        let mut logger = AuditLogger::new(vec![]);
        logger.log_auth("alice", "10.0.0.1", true);
        let json = logger.to_json();
        let parsed: Result<Vec<AuditEntry>, _> = serde_json::from_str(&json);
        assert!(parsed.is_ok(), "JSON output must be valid");
        assert_eq!(parsed.unwrap().len(), 1);
    }

    #[test]
    fn test_demonstrate_logging() {
        let (total, critical, warning) = demonstrate_audit_logging();
        assert!(total > 0, "Should have logged some entries");
        assert!(critical > 0, "Should have critical entries");
        assert!(warning > 0, "Should have warning entries");
    }
}
