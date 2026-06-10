//! # Lesson 09: Audit Logging for Authorization Decisions
//!
//! ## Why Audit Authorization?
//!
//! Every authorization decision should be logged:
//! - **Who** tried to access what (user identity)
//! - **What** resource and action they attempted
//! - **When** the attempt occurred
//! - **Decision**: Allowed or Denied
//! - **Why**: Which rule/policy matched
//!
//! ## Why This Matters
//!
//! 1. **Forensics**: After a breach, you need to know what the attacker accessed
//! 2. **Compliance**: SOX, HIPAA, PCI-DSS all require access audit trails
//! 3. **Detection**: Unusual patterns (e.g., mass denied attempts) signal attacks
//! 4. **Debugging**: Understanding why access was denied helps fix misconfigurations
//!
//! ## 🔴 Attack: Audit Log Tampering
//!
//! If an attacker can modify or delete audit logs, they can cover their tracks.
//! Logs should be:
//! - Written to append-only storage
//! - Forwarded to a separate system (SIEM)
//! - Integrity-protected (hash chain or signed)
//!
//! ## Log Levels
//!
//! - **INFO**: Successful access
//! - **WARN**: Denied access (expected for normal users)
//! - **ALERT**: Multiple consecutive denials (possible attack)

use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// The decision made for an authorization request.
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum AuthDecision {
    Allowed,
    Denied,
}

/// Severity level of the audit entry.
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Info,
    Warn,
    Alert,
}

/// A single authorization audit log entry.
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Unix timestamp of the event.
    pub timestamp: u64,
    /// The user who made the request.
    pub user: String,
    /// The resource being accessed.
    pub resource: String,
    /// The action attempted.
    pub action: String,
    /// The decision made.
    pub decision: AuthDecision,
    /// Which rule/policy matched (if any).
    pub matched_rule: Option<String>,
    /// Severity level.
    pub severity: Severity,
    /// Hash of the previous log entry (for integrity chain).
    pub prev_hash: u64,
}

/// The audit logger that records authorization decisions.
pub struct AuditLogger {
    /// The log entries, in chronological order.
    entries: VecDeque<AuditEntry>,
    /// Maximum number of entries to keep in memory.
    max_entries: usize,
    /// Hash of the most recent entry (for chaining).
    last_hash: u64,
    /// Counter of consecutive denials per user (for alert detection).
    denial_counters: std::collections::HashMap<String, u32>,
    /// Threshold for triggering an alert on consecutive denials.
    alert_threshold: u32,
}

impl AuditLogger {
    /// Create a new audit logger.
    pub fn new(max_entries: usize, alert_threshold: u32) -> Self {
        todo!("Initialize the audit logger")
    }

    /// Log an authorization decision.
    ///
    /// 1. Create an AuditEntry with the current timestamp.
    /// 2. Determine severity:
    ///    - Allowed → Info
    ///    - Denied → Warn
    ///    - Denied AND user has >= alert_threshold consecutive denials → Alert
    /// 3. Compute the hash chain: hash of this entry's data + prev_hash.
    /// 4. Update the denial counter for the user.
    /// 5. If entries exceed max_entries, remove the oldest.
    /// 6. Return the hash of the new entry.
    pub fn log_decision(
        &mut self,
        user: &str,
        resource: &str,
        action: &str,
        decision: AuthDecision,
        matched_rule: Option<&str>,
    ) -> u64 {
        todo!("Create audit entry, compute hash chain, manage counters")
    }

    /// Get all log entries.
    pub fn entries(&self) -> &VecDeque<AuditEntry> {
        &self.entries
    }

    /// Get all entries for a specific user.
    pub fn entries_for_user(&self, user: &str) -> Vec<&AuditEntry> {
        todo!("Filter entries by user")
    }

    /// Get all denied entries.
    pub fn denied_entries(&self) -> Vec<&AuditEntry> {
        todo!("Filter entries where decision is Denied")
    }

    /// Get all alert-level entries.
    pub fn alert_entries(&self) -> Vec<&AuditEntry> {
        todo!("Filter entries where severity is Alert")
    }

    /// Verify the integrity of the hash chain.
    ///
    /// Walk through all entries and verify that each entry's prev_hash
    /// matches the hash of the previous entry. Returns true if the chain
    /// is intact.
    pub fn verify_integrity(&self) -> bool {
        todo!("Walk the chain and verify each prev_hash link")
    }

    /// Get the current Unix timestamp.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Compute a hash for an entry (for the hash chain).
    fn compute_entry_hash(entry: &AuditEntry) -> u64 {
        let mut hasher = DefaultHasher::new();
        entry.timestamp.hash(&mut hasher);
        entry.user.hash(&mut hasher);
        entry.resource.hash(&mut hasher);
        entry.action.hash(&mut hasher);
        entry.decision.hash(&mut hasher);
        entry.prev_hash.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_allowed_decision() {
        let mut logger = AuditLogger::new(1000, 5);
        logger.log_decision(
            "alice",
            "/docs/1",
            "read",
            AuthDecision::Allowed,
            Some("owner-rule"),
        );
        assert_eq!(logger.entries().len(), 1);
        let entry = &logger.entries()[0];
        assert_eq!(entry.decision, AuthDecision::Allowed);
        assert_eq!(entry.severity, Severity::Info);
    }

    #[test]
    fn test_log_denied_decision() {
        let mut logger = AuditLogger::new(1000, 5);
        logger.log_decision("bob", "/admin/config", "write", AuthDecision::Denied, None);
        assert_eq!(logger.entries().len(), 1);
        let entry = &logger.entries()[0];
        assert_eq!(entry.decision, AuthDecision::Denied);
        assert_eq!(entry.severity, Severity::Warn);
    }

    #[test]
    fn test_alert_on_repeated_denials() {
        let mut logger = AuditLogger::new(1000, 3);
        // 3 consecutive denials should trigger alert
        for _ in 0..3 {
            logger.log_decision("eve", "/secret", "read", AuthDecision::Denied, None);
        }
        let alerts = logger.alert_entries();
        assert!(!alerts.is_empty());
        // The third denial should be an Alert
        assert_eq!(alerts[0].severity, Severity::Alert);
    }

    #[test]
    fn test_denial_counter_resets_on_allow() {
        let mut logger = AuditLogger::new(1000, 3);
        logger.log_decision("eve", "/secret", "read", AuthDecision::Denied, None);
        logger.log_decision("eve", "/secret", "read", AuthDecision::Denied, None);
        // Allow resets the counter
        logger.log_decision("eve", "/public/info", "read", AuthDecision::Allowed, None);
        // Next denial should be Warn (counter reset), not Alert
        logger.log_decision("eve", "/secret", "read", AuthDecision::Denied, None);
        let alerts = logger.alert_entries();
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_max_entries_limit() {
        let mut logger = AuditLogger::new(5, 100);
        for i in 0..10 {
            logger.log_decision(
                "user",
                "/res",
                "read",
                AuthDecision::Allowed,
                None,
            );
        }
        assert_eq!(logger.entries().len(), 5);
    }

    #[test]
    fn test_entries_for_user() {
        let mut logger = AuditLogger::new(1000, 5);
        logger.log_decision("alice", "/a", "read", AuthDecision::Allowed, None);
        logger.log_decision("bob", "/b", "read", AuthDecision::Allowed, None);
        logger.log_decision("alice", "/c", "write", AuthDecision::Denied, None);
        let alice_entries = logger.entries_for_user("alice");
        assert_eq!(alice_entries.len(), 2);
    }

    #[test]
    fn test_denied_entries() {
        let mut logger = AuditLogger::new(1000, 5);
        logger.log_decision("alice", "/a", "read", AuthDecision::Allowed, None);
        logger.log_decision("bob", "/b", "read", AuthDecision::Denied, None);
        let denied = logger.denied_entries();
        assert_eq!(denied.len(), 1);
        assert_eq!(denied[0].user, "bob");
    }

    #[test]
    fn test_hash_chain_integrity() {
        let mut logger = AuditLogger::new(1000, 5);
        logger.log_decision("alice", "/a", "read", AuthDecision::Allowed, None);
        logger.log_decision("bob", "/b", "read", AuthDecision::Denied, None);
        logger.log_decision("carol", "/c", "write", AuthDecision::Allowed, None);
        assert!(logger.verify_integrity());
    }
}
