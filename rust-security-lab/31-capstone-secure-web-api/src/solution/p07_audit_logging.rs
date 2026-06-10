//! # Lesson 07: Audit Logging — Solution
//!
//! Security event logging, structured context.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Security,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityEvent {
    Authentication {
        success: bool,
        user_id: Option<String>,
    },
    Authorization {
        allowed: bool,
        user_id: String,
        permission: String,
    },
    RateLimitExceeded {
        key: String,
    },
    ValidationFailed {
        field: String,
        reason: String,
    },
    SignatureVerification {
        success: bool,
    },
    AdminAction {
        user_id: String,
        action: String,
    },
    Custom {
        description: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: u64,
    pub severity: Severity,
    pub event: SecurityEvent,
    pub client_ip: String,
    pub method: Option<String>,
    pub path: Option<String>,
    pub status_code: Option<u16>,
    pub user_agent: Option<String>,
    pub correlation_id: String,
}

#[derive(Debug)]
pub struct AuditLogger {
    pub entries: Vec<AuditEntry>,
    next_id: u64,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }
}

pub fn log_authentication(
    logger: &mut AuditLogger,
    timestamp: u64,
    client_ip: &str,
    user_id: Option<String>,
    success: bool,
    correlation_id: &str,
) -> AuditEntry {
    let id = format!("audit-{}", logger.next_id);
    logger.next_id += 1;

    let severity = if success {
        Severity::Info
    } else {
        Severity::Security
    };

    let entry = AuditEntry {
        id,
        timestamp,
        severity,
        event: SecurityEvent::Authentication {
            success,
            user_id: user_id.clone(),
        },
        client_ip: client_ip.to_string(),
        method: None,
        path: None,
        status_code: None,
        user_agent: None,
        correlation_id: correlation_id.to_string(),
    };

    logger.entries.push(entry.clone());
    entry
}

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
    let id = format!("audit-{}", logger.next_id);
    logger.next_id += 1;

    let severity = if allowed {
        Severity::Info
    } else {
        Severity::Security
    };

    let entry = AuditEntry {
        id,
        timestamp,
        severity,
        event: SecurityEvent::Authorization {
            allowed,
            user_id: user_id.to_string(),
            permission: permission.to_string(),
        },
        client_ip: client_ip.to_string(),
        method: Some(method.to_string()),
        path: Some(path.to_string()),
        status_code: None,
        user_agent: None,
        correlation_id: correlation_id.to_string(),
    };

    logger.entries.push(entry.clone());
    entry
}

pub fn sanitize_for_log(input: &str, max_len: usize) -> String {
    let mut result: String = input
        .chars()
        .map(|c| match c {
            '\n' | '\r' => ' ',
            '\t' => ' ',
            '\0' => '\0', // will be filtered next
            c => c,
        })
        .collect();

    // Remove null bytes
    result.retain(|c| c != '\0');

    // Truncate
    if result.len() > max_len {
        result.truncate(max_len);
    }

    result
}

pub fn redact_entry(entry: &AuditEntry) -> AuditEntry {
    let mut redacted = entry.clone();

    // Redact IP: replace last octet with "xxx"
    if let Some(last_dot) = redacted.client_ip.rfind('.') {
        redacted.client_ip = format!("{}{}.xxx", &redacted.client_ip[..last_dot], "");
    }

    // Redact user agent
    if redacted.user_agent.is_some() {
        redacted.user_agent = Some("***".to_string());
    }

    redacted
}

pub fn count_by_severity(entries: &[AuditEntry]) -> (usize, usize, usize, usize) {
    let mut info = 0;
    let mut warning = 0;
    let mut security = 0;
    let mut critical = 0;

    for entry in entries {
        match entry.severity {
            Severity::Info => info += 1,
            Severity::Warning => warning += 1,
            Severity::Security => security += 1,
            Severity::Critical => critical += 1,
        }
    }

    (info, warning, security, critical)
}

pub fn filter_by_user<'a>(entries: &'a [AuditEntry], user_id: &str) -> Vec<&'a AuditEntry> {
    entries
        .iter()
        .filter(|entry| match &entry.event {
            SecurityEvent::Authentication {
                user_id: Some(uid), ..
            } => uid == user_id,
            SecurityEvent::Authorization { user_id: uid, .. } => uid == user_id,
            SecurityEvent::AdminAction {
                user_id: uid, ..
            } => uid == user_id,
            _ => false,
        })
        .collect()
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
            &mut logger,
            1700000000,
            "10.0.0.1",
            Some("alice".to_string()),
            true,
            "req-001",
        );
        assert_eq!(entry.severity, Severity::Info);
        assert_eq!(
            entry.event,
            SecurityEvent::Authentication {
                success: true,
                user_id: Some("alice".to_string())
            }
        );
        assert_eq!(entry.id, "audit-1");
    }

    #[test]
    fn test_log_authentication_failure() {
        let mut logger = make_logger();
        let entry = log_authentication(
            &mut logger,
            1700000000,
            "10.0.0.1",
            None,
            false,
            "req-002",
        );
        assert_eq!(entry.severity, Severity::Security);
        assert_eq!(
            entry.event,
            SecurityEvent::Authentication {
                success: false,
                user_id: None
            }
        );
    }

    #[test]
    fn test_log_authorization() {
        let mut logger = make_logger();
        let entry = log_authorization(
            &mut logger,
            1700000000,
            "10.0.0.1",
            "alice",
            "users:read",
            true,
            "GET",
            "/api/users",
            "req-003",
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
            &mut logger,
            1700000000,
            "192.168.1.100",
            Some("alice".to_string()),
            true,
            "req-001",
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
            event: SecurityEvent::Custom {
                description: "test".to_string(),
            },
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
                id: "1".to_string(),
                timestamp: 1,
                severity: Severity::Info,
                event: SecurityEvent::Custom {
                    description: "a".to_string(),
                },
                client_ip: "ip".to_string(),
                method: None,
                path: None,
                status_code: None,
                user_agent: None,
                correlation_id: "c".to_string(),
            },
            AuditEntry {
                id: "2".to_string(),
                timestamp: 2,
                severity: Severity::Security,
                event: SecurityEvent::Custom {
                    description: "b".to_string(),
                },
                client_ip: "ip".to_string(),
                method: None,
                path: None,
                status_code: None,
                user_agent: None,
                correlation_id: "c".to_string(),
            },
            AuditEntry {
                id: "3".to_string(),
                timestamp: 3,
                severity: Severity::Info,
                event: SecurityEvent::Custom {
                    description: "c".to_string(),
                },
                client_ip: "ip".to_string(),
                method: None,
                path: None,
                status_code: None,
                user_agent: None,
                correlation_id: "c".to_string(),
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
                id: "1".to_string(),
                timestamp: 1,
                severity: Severity::Info,
                event: SecurityEvent::Authentication {
                    success: true,
                    user_id: Some("alice".to_string()),
                },
                client_ip: "ip".to_string(),
                method: None,
                path: None,
                status_code: None,
                user_agent: None,
                correlation_id: "c".to_string(),
            },
            AuditEntry {
                id: "2".to_string(),
                timestamp: 2,
                severity: Severity::Info,
                event: SecurityEvent::Authentication {
                    success: true,
                    user_id: Some("bob".to_string()),
                },
                client_ip: "ip".to_string(),
                method: None,
                path: None,
                status_code: None,
                user_agent: None,
                correlation_id: "c".to_string(),
            },
        ];
        let alice_entries = filter_by_user(&entries, "alice");
        assert_eq!(alice_entries.len(), 1);
        assert_eq!(alice_entries[0].id, "1");
    }
}
