//! # Lesson 06: Audit Logging for Database Access (Reference Solution)
//!
//! See the exercise file for full documentation.

use serde::{Deserialize, Serialize};
use std::fmt;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    Read { table: String, columns: Vec<String> },
    Write { table: String, operation: String },
    SchemaChange { statement: String },
    Auth { success: bool },
    AccessDenied { resource: String, reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub actor: String,
    pub source_ip: String,
    pub action: AuditAction,
    pub level: AuditLevel,
    pub success: bool,
    pub context: Option<String>,
}

pub struct AuditLogger {
    entries: Vec<AuditEntry>,
    _sensitive_fields: Vec<String>,
    counter: u64,
}

impl AuditLogger {
    pub fn new(sensitive_fields: Vec<String>) -> Self {
        AuditLogger {
            entries: Vec::new(),
            _sensitive_fields: sensitive_fields,
            counter: 0,
        }
    }

    fn next_timestamp(&mut self) -> String {
        self.counter += 1;
        format!("2024-01-01T00:00:00.{:03}Z", self.counter % 1000)
    }

    pub fn log_read(&mut self, actor: &str, source_ip: &str, table: &str, columns: &[&str]) {
        let timestamp = self.next_timestamp();
        self.entries.push(AuditEntry {
            timestamp,
            actor: actor.to_string(),
            source_ip: source_ip.to_string(),
            action: AuditAction::Read {
                table: table.to_string(),
                columns: columns.iter().map(|s| s.to_string()).collect(),
            },
            level: AuditLevel::Info,
            success: true,
            context: None,
        });
    }

    pub fn log_write(&mut self, actor: &str, source_ip: &str, table: &str, operation: &str) {
        let timestamp = self.next_timestamp();
        self.entries.push(AuditEntry {
            timestamp,
            actor: actor.to_string(),
            source_ip: source_ip.to_string(),
            action: AuditAction::Write {
                table: table.to_string(),
                operation: operation.to_string(),
            },
            level: AuditLevel::Warning,
            success: true,
            context: None,
        });
    }

    pub fn log_access_denied(&mut self, actor: &str, source_ip: &str, resource: &str, reason: &str) {
        let timestamp = self.next_timestamp();
        self.entries.push(AuditEntry {
            timestamp,
            actor: actor.to_string(),
            source_ip: source_ip.to_string(),
            action: AuditAction::AccessDenied {
                resource: resource.to_string(),
                reason: reason.to_string(),
            },
            level: AuditLevel::Critical,
            success: false,
            context: None,
        });
    }

    pub fn log_auth(&mut self, actor: &str, source_ip: &str, success: bool) {
        let timestamp = self.next_timestamp();
        let level = if success {
            AuditLevel::Info
        } else {
            AuditLevel::Warning
        };
        self.entries.push(AuditEntry {
            timestamp,
            actor: actor.to_string(),
            source_ip: source_ip.to_string(),
            action: AuditAction::Auth { success },
            level,
            success,
            context: None,
        });
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.entries).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn get_entries_by_actor(&self, actor: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.actor == actor)
            .collect()
    }

    pub fn get_entries_by_level(&self, min_level: AuditLevel) -> Vec<&AuditEntry> {
        let min_value = match min_level {
            AuditLevel::Info => 0,
            AuditLevel::Warning => 1,
            AuditLevel::Critical => 2,
        };
        self.entries
            .iter()
            .filter(|e| {
                let entry_value = match e.level {
                    AuditLevel::Info => 0,
                    AuditLevel::Warning => 1,
                    AuditLevel::Critical => 2,
                };
                entry_value >= min_value
            })
            .collect()
    }

    pub fn count_actions(&self) -> (usize, usize, usize) {
        let mut reads = 0;
        let mut writes = 0;
        let mut denied = 0;
        for entry in &self.entries {
            match &entry.action {
                AuditAction::Read { .. } => reads += 1,
                AuditAction::Write { .. } => writes += 1,
                AuditAction::AccessDenied { .. } => denied += 1,
                _ => {}
            }
        }
        (reads, writes, denied)
    }
}

/// Demonstrate audit logging.
pub fn demonstrate_audit_logging() -> (usize, usize, usize) {
    let mut logger = AuditLogger::new(vec!["ssn".to_string(), "password".to_string()]);

    logger.log_auth("alice", "10.0.0.1", true);
    logger.log_auth("eve", "10.0.0.99", false);
    logger.log_read("alice", "10.0.0.1", "users", &["name", "email"]);
    logger.log_write("bob", "10.0.0.2", "orders", "INSERT");
    logger.log_write("bob", "10.0.0.2", "orders", "UPDATE");
    logger.log_access_denied("eve", "10.0.0.99", "admin_panel", "insufficient privileges");

    let total = logger.entries.len();
    let critical = logger.get_entries_by_level(AuditLevel::Critical).len();
    let warning = logger.get_entries_by_level(AuditLevel::Warning).len();

    (total, critical, warning)
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
