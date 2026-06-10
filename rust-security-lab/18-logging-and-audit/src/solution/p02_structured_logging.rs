//! # Lesson 02: Structured Security Logging (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthorizationDenied,
    DataAccess,
    DataModification,
    AdminAction,
    SecurityViolation,
    SystemStartup,
    SystemShutdown,
    ConfigurationChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub level: SecurityLevel,
    pub event_type: SecurityEventType,
    pub message: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub resource: Option<String>,
    pub correlation_id: String,
    pub metadata: HashMap<String, String>,
}

pub fn new_security_event(
    level: SecurityLevel,
    event_type: SecurityEventType,
    message: &str,
) -> SecurityEvent {
    let now = Utc::now();
    let nanos = now.timestamp_nanos_opt().unwrap_or(0) as u64;
    SecurityEvent {
        timestamp: now,
        level,
        event_type,
        message: message.to_string(),
        user_id: None,
        ip_address: None,
        resource: None,
        correlation_id: format!("evt-{:016x}", nanos),
        metadata: HashMap::new(),
    }
}

pub fn with_user_id(mut event: SecurityEvent, user_id: &str) -> SecurityEvent {
    event.user_id = Some(user_id.to_string());
    event
}

pub fn with_ip_address(mut event: SecurityEvent, ip: &str) -> SecurityEvent {
    event.ip_address = Some(ip.to_string());
    event
}

pub fn with_resource(mut event: SecurityEvent, resource: &str) -> SecurityEvent {
    event.resource = Some(resource.to_string());
    event
}

pub fn with_metadata(mut event: SecurityEvent, key: &str, value: &str) -> SecurityEvent {
    event.metadata.insert(key.to_string(), value.to_string());
    event
}

pub fn event_to_json(event: &SecurityEvent) -> Result<String, String> {
    serde_json::to_string_pretty(event).map_err(|e| e.to_string())
}

pub fn json_to_event(json: &str) -> Result<SecurityEvent, String> {
    serde_json::from_str::<SecurityEvent>(json).map_err(|e| e.to_string())
}

pub fn validate_event(event: &SecurityEvent) -> Result<(), String> {
    if event.message.is_empty() {
        return Err("message must not be empty".to_string());
    }
    if event.correlation_id.is_empty() {
        return Err("correlation_id must not be empty".to_string());
    }
    let diff = Utc::now() - event.timestamp;
    if diff.num_hours() > 24 || diff.num_hours() < -1 {
        return Err("timestamp must be within the last 24 hours".to_string());
    }
    Ok(())
}

pub fn log_auth_failure(user_id: &str, ip_address: &str, reason: &str) -> SecurityEvent {
    let event = new_security_event(
        SecurityLevel::Warn,
        SecurityEventType::AuthenticationFailure,
        &format!("Authentication failed for user {}", user_id),
    );
    let event = with_user_id(event, user_id);
    let event = with_ip_address(event, ip_address);
    with_metadata(event, "reason", reason)
}

pub struct LogSink {
    pub events: Vec<SecurityEvent>,
}

impl LogSink {
    pub fn new() -> Self {
        LogSink { events: Vec::new() }
    }

    pub fn record(&mut self, event: SecurityEvent) {
        self.events.push(event);
    }

    pub fn events_with_level(&self, level: SecurityLevel) -> Vec<&SecurityEvent> {
        self.events.iter().filter(|e| e.level == level).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_security_event_has_timestamp() {
        let event = new_security_event(
            SecurityLevel::Info,
            SecurityEventType::SystemStartup,
            "System started",
        );
        let diff = Utc::now() - event.timestamp;
        assert!(diff.num_seconds() < 5, "Timestamp should be recent");
    }

    #[test]
    fn test_new_security_event_has_correlation_id() {
        let event = new_security_event(
            SecurityLevel::Info,
            SecurityEventType::SystemStartup,
            "Test",
        );
        assert!(!event.correlation_id.is_empty());
        assert!(event.correlation_id.starts_with("evt-"));
    }

    #[test]
    fn test_builder_pattern() {
        let event = new_security_event(
            SecurityLevel::Warn,
            SecurityEventType::AuthenticationFailure,
            "Bad password",
        );
        let event = with_user_id(event, "user-123");
        let event = with_ip_address(event, "10.0.0.1");
        let event = with_resource(event, "/api/login");
        let event = with_metadata(event, "attempt", "3");

        assert_eq!(event.user_id, Some("user-123".to_string()));
        assert_eq!(event.ip_address, Some("10.0.0.1".to_string()));
        assert_eq!(event.resource, Some("/api/login".to_string()));
        assert_eq!(event.metadata.get("attempt"), Some(&"3".to_string()));
    }

    #[test]
    fn test_json_roundtrip() {
        let event = new_security_event(
            SecurityLevel::Info,
            SecurityEventType::AuthenticationSuccess,
            "User logged in",
        );
        let event = with_user_id(event, "user-456");

        let json = event_to_json(&event).unwrap();
        let parsed = json_to_event(&json).unwrap();

        assert_eq!(parsed.event_type, SecurityEventType::AuthenticationSuccess);
        assert_eq!(parsed.user_id, Some("user-456".to_string()));
    }

    #[test]
    fn test_json_contains_required_fields() {
        let event = new_security_event(
            SecurityLevel::Error,
            SecurityEventType::SecurityViolation,
            "Intrusion detected",
        );
        let json = event_to_json(&event).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("level"));
        assert!(json.contains("event_type"));
        assert!(json.contains("correlation_id"));
    }

    #[test]
    fn test_validate_valid_event() {
        let event = new_security_event(
            SecurityLevel::Info,
            SecurityEventType::SystemStartup,
            "System started",
        );
        assert!(validate_event(&event).is_ok());
    }

    #[test]
    fn test_validate_empty_message() {
        let event = new_security_event(
            SecurityLevel::Info,
            SecurityEventType::SystemStartup,
            "",
        );
        assert!(validate_event(&event).is_err());
    }

    #[test]
    fn test_log_auth_failure_helper() {
        let event = log_auth_failure("user-789", "192.168.1.1", "invalid_password");
        assert_eq!(event.level, SecurityLevel::Warn);
        assert_eq!(event.event_type, SecurityEventType::AuthenticationFailure);
        assert_eq!(event.user_id, Some("user-789".to_string()));
        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(event.metadata.get("reason"), Some(&"invalid_password".to_string()));
    }

    #[test]
    fn test_log_sink_filter_by_level() {
        let mut sink = LogSink::new();
        sink.record(new_security_event(
            SecurityLevel::Info,
            SecurityEventType::SystemStartup,
            "Started",
        ));
        sink.record(new_security_event(
            SecurityLevel::Warn,
            SecurityEventType::AuthenticationFailure,
            "Failed login",
        ));
        sink.record(new_security_event(
            SecurityLevel::Error,
            SecurityEventType::SecurityViolation,
            "Attack detected",
        ));

        assert_eq!(sink.events_with_level(SecurityLevel::Warn).len(), 1);
        assert_eq!(sink.events_with_level(SecurityLevel::Info).len(), 1);
        assert_eq!(sink.events_with_level(SecurityLevel::Error).len(), 1);
    }
}
