//! # Lesson 09: Auditing Secret Access (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SecretOperation {
    Read,
    Write,
    Rotate,
    Delete,
    List,
    Deny,
}

#[derive(Debug, Clone)]
pub struct AuditEntry {
    pub timestamp: Instant,
    pub identity: String,
    pub secret_path: String,
    pub operation: SecretOperation,
    pub success: bool,
    pub source: String,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct AuditLogger {
    entries: Vec<AuditEntry>,
    max_accesses_per_window: usize,
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

pub fn log_access(
    logger: &mut AuditLogger,
    identity: &str,
    secret_path: &str,
    operation: SecretOperation,
    success: bool,
    source: &str,
) {
    logger.entries.push(AuditEntry {
        timestamp: Instant::now(),
        identity: identity.to_string(),
        secret_path: secret_path.to_string(),
        operation,
        success,
        source: source.to_string(),
        error: None,
    });
}

pub fn get_access_history<'a>(logger: &'a AuditLogger, secret_path: &str) -> Vec<&'a AuditEntry> {
    logger
        .entries
        .iter()
        .filter(|e| e.secret_path == secret_path)
        .collect()
}

pub fn get_identity_history<'a>(logger: &'a AuditLogger, identity: &str) -> Vec<&'a AuditEntry> {
    logger
        .entries
        .iter()
        .filter(|e| e.identity == identity)
        .collect()
}

pub fn detect_anomalies(logger: &AuditLogger) -> Vec<String> {
    if logger.entries.is_empty() {
        return Vec::new();
    }

    // Find the most recent timestamp
    let most_recent = logger
        .entries
        .iter()
        .map(|e| e.timestamp)
        .max()
        .unwrap();

    let window_start = most_recent.checked_sub(logger.alert_window).unwrap_or(Instant::now());

    // Count accesses per identity within the window
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for entry in &logger.entries {
        if entry.timestamp >= window_start {
            *counts.entry(&entry.identity).or_insert(0) += 1;
        }
    }

    counts
        .iter()
        .filter(|(_, &count)| count > logger.max_accesses_per_window)
        .map(|(identity, _)| identity.to_string())
        .collect()
}

pub fn generate_audit_report(logger: &AuditLogger) -> String {
    let total = logger.entries.len();
    let successful = logger.entries.iter().filter(|e| e.success).count();
    let failed = total - successful;

    // Count by operation
    let mut op_counts: HashMap<String, usize> = HashMap::new();
    for entry in &logger.entries {
        let op_name = format!("{:?}", entry.operation);
        *op_counts.entry(op_name).or_insert(0) += 1;
    }

    // Count by secret path
    let mut path_counts: HashMap<&str, usize> = HashMap::new();
    for entry in &logger.entries {
        *path_counts.entry(&entry.secret_path).or_insert(0) += 1;
    }
    let mut top_paths: Vec<(&&str, &usize)> = path_counts.iter().collect();
    top_paths.sort_by(|a, b| b.1.cmp(a.1));

    let mut report = format!(
        "Audit Report\n============\nTotal events: {}\nSuccessful: {}\nFailed: {}\n\nAccess by operation:\n",
        total, successful, failed
    );
    for (op, count) in &op_counts {
        report.push_str(&format!("  {}: {}\n", op, count));
    }

    report.push_str("\nTop accessed secrets:\n");
    for (path, count) in top_paths.iter().take(5) {
        report.push_str(&format!("  {}: {} accesses\n", path, count));
    }

    report
}

pub fn get_failed_accesses<'a>(logger: &'a AuditLogger) -> Vec<&'a AuditEntry> {
    logger.entries.iter().filter(|e| !e.success).collect()
}

pub fn validate_error_message(error_msg: &str, known_secrets: &[&str]) -> Result<(), String> {
    for secret in known_secrets {
        if error_msg.contains(secret) {
            let snippet = if secret.len() > 8 {
                &secret[..8]
            } else {
                secret
            };
            return Err(snippet.to_string());
        }
    }
    Ok(())
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
        log_access(
            &mut logger,
            "alice",
            "db/password",
            SecretOperation::Read,
            true,
            "10.0.0.1",
        );

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
        for _ in 0..6 {
            log_access(&mut logger, "alice", "secret1", SecretOperation::Read, true, "10.0.0.1");
        }
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
