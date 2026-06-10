//! # Lesson 09: Audit Logging for Authorization Decisions (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum AuthDecision {
    Allowed,
    Denied,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Info,
    Warn,
    Alert,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub user: String,
    pub resource: String,
    pub action: String,
    pub decision: AuthDecision,
    pub matched_rule: Option<String>,
    pub severity: Severity,
    pub prev_hash: u64,
}

pub struct AuditLogger {
    entries: VecDeque<AuditEntry>,
    max_entries: usize,
    last_hash: u64,
    denial_counters: std::collections::HashMap<String, u32>,
    alert_threshold: u32,
}

impl AuditLogger {
    pub fn new(max_entries: usize, alert_threshold: u32) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
            last_hash: 0,
            denial_counters: std::collections::HashMap::new(),
            alert_threshold,
        }
    }

    pub fn log_decision(
        &mut self,
        user: &str,
        resource: &str,
        action: &str,
        decision: AuthDecision,
        matched_rule: Option<&str>,
    ) -> u64 {
        // Update denial counter
        let counter = self
            .denial_counters
            .entry(user.to_string())
            .or_insert(0);

        let severity = match decision {
            AuthDecision::Allowed => {
                *counter = 0;
                Severity::Info
            }
            AuthDecision::Denied => {
                *counter += 1;
                if *counter >= self.alert_threshold {
                    Severity::Alert
                } else {
                    Severity::Warn
                }
            }
        };

        let entry = AuditEntry {
            timestamp: Self::current_timestamp(),
            user: user.to_string(),
            resource: resource.to_string(),
            action: action.to_string(),
            decision,
            matched_rule: matched_rule.map(|s| s.to_string()),
            severity,
            prev_hash: self.last_hash,
        };

        let hash = Self::compute_entry_hash(&entry);
        self.last_hash = hash;

        self.entries.push_back(entry);

        // Enforce max entries limit
        while self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }

        hash
    }

    pub fn entries(&self) -> &VecDeque<AuditEntry> {
        &self.entries
    }

    pub fn entries_for_user(&self, user: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.user == user)
            .collect()
    }

    pub fn denied_entries(&self) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.decision == AuthDecision::Denied)
            .collect()
    }

    pub fn alert_entries(&self) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.severity == Severity::Alert)
            .collect()
    }

    /// Verify the integrity of the hash chain.
    ///
    /// Walk through all entries and verify that each entry's prev_hash
    /// matches the hash of the previous entry.
    pub fn verify_integrity(&self) -> bool {
        let mut expected_prev: u64 = 0;

        for entry in &self.entries {
            if entry.prev_hash != expected_prev {
                return false;
            }
            expected_prev = Self::compute_entry_hash(entry);
        }

        true
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

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
        for _ in 0..3 {
            logger.log_decision("eve", "/secret", "read", AuthDecision::Denied, None);
        }
        let alerts = logger.alert_entries();
        assert!(!alerts.is_empty());
        assert_eq!(alerts[0].severity, Severity::Alert);
    }

    #[test]
    fn test_denial_counter_resets_on_allow() {
        let mut logger = AuditLogger::new(1000, 3);
        logger.log_decision("eve", "/secret", "read", AuthDecision::Denied, None);
        logger.log_decision("eve", "/secret", "read", AuthDecision::Denied, None);
        logger.log_decision("eve", "/public/info", "read", AuthDecision::Allowed, None);
        logger.log_decision("eve", "/secret", "read", AuthDecision::Denied, None);
        let alerts = logger.alert_entries();
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_max_entries_limit() {
        let mut logger = AuditLogger::new(5, 100);
        for _ in 0..10 {
            logger.log_decision("user", "/res", "read", AuthDecision::Allowed, None);
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
