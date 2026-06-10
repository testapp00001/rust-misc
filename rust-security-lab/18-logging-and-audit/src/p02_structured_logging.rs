//! # Lesson 02: Structured Security Logging
//!
//! ## The Problem
//!
//! Free-text log messages like `"User admin failed to login"` are easy for humans
//! to read but impossible for machines to parse reliably. When you need to query
//! "show me all failed login attempts in the last hour" across millions of log
//! lines, regex-based parsing is fragile and slow.
//!
//! ## Defense: Structured Logging
//!
//! Structured logs use a consistent schema (JSON is the most common) with
//! mandatory fields:
//!
//! ```json
//! {
//!   "timestamp": "2024-01-15T10:30:00Z",
//!   "level": "WARN",
//!   "event": "authentication_failure",
//!   "user_id": "user-123",
//!   "ip_address": "192.168.1.100",
//!   "reason": "invalid_password",
//!   "correlation_id": "req-abc-123"
//! }
//! ```
//!
//! ## Mandatory Security Fields
//!
//! Every security-relevant log entry MUST include:
//! - **timestamp**: ISO 8601 with timezone (UTC)
//! - **level**: DEBUG, INFO, WARN, ERROR, CRITICAL
//! - **event**: machine-readable event type (e.g., `authentication_failure`)
//! - **correlation_id**: unique ID to trace a request across services
//!
//! ## What You'll Implement
//!
//! 1. A `SecurityEvent` struct with mandatory fields
//! 2. Builder pattern for constructing events
//! 3. JSON serialization with consistent field ordering
//! 4. Event type enumeration for common security events
//! 5. A log sink that collects events for testing
//! 6. Validation that mandatory fields are present

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Security log levels, mapped to severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

/// Common security event types.
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

/// A structured security log event.
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
    pub metadata: std::collections::HashMap<String, String>,
}

/// Exercise 1: Create a new SecurityEvent with mandatory fields.
///
/// - Set timestamp to current UTC time
/// - Set the provided level, event_type, and message
/// - Generate a correlation_id (use a simple format: `"evt-"` + random hex)
/// - Initialize metadata as empty
///
/// Hints:
/// - Use `Utc::now()` for timestamp
/// - Use `format!("evt-{:016x}", rand_value)` for correlation_id
///   (for this exercise, you can use the timestamp nanos as the random value)
pub fn new_security_event(
    level: SecurityLevel,
    event_type: SecurityEventType,
    message: &str,
) -> SecurityEvent {
    todo!("Implement SecurityEvent constructor")
}

/// Exercise 2: Builder pattern -- set optional fields.
///
/// Implement methods on `SecurityEvent` to chain optional field setters:
/// - `with_user_id(mut self, user_id: &str) -> Self`
/// - `with_ip_address(mut self, ip: &str) -> Self`
/// - `with_resource(mut self, resource: &str) -> Self`
/// - `with_metadata(mut self, key: &str, value: &str) -> Self`
///
/// These should be implemented as methods. For this exercise, implement
/// the standalone functions that take and return an owned SecurityEvent.
pub fn with_user_id(mut event: SecurityEvent, user_id: &str) -> SecurityEvent {
    todo!("Set user_id on SecurityEvent")
}

pub fn with_ip_address(mut event: SecurityEvent, ip: &str) -> SecurityEvent {
    todo!("Set ip_address on SecurityEvent")
}

pub fn with_resource(mut event: SecurityEvent, resource: &str) -> SecurityEvent {
    todo!("Set resource on SecurityEvent")
}

pub fn with_metadata(mut event: SecurityEvent, key: &str, value: &str) -> SecurityEvent {
    todo!("Add metadata to SecurityEvent")
}

/// Exercise 3: Serialize a SecurityEvent to JSON string.
///
/// The JSON must:
/// - Use ISO 8601 format for timestamp
/// - Include all non-None fields
/// - Use consistent field ordering (serde handles this)
/// - Use 2-space indentation for readability
///
/// Hints:
/// - Use `serde_json::to_string_pretty(&event)?`
/// - Return `Result<String, String>` wrapping serde errors
pub fn event_to_json(event: &SecurityEvent) -> Result<String, String> {
    todo!("Implement JSON serialization")
}

/// Exercise 4: Deserialize a JSON string back to SecurityEvent.
///
/// Hints:
/// - Use `serde_json::from_str::<SecurityEvent>(&json)?`
/// - Return `Result<SecurityEvent, String>`
pub fn json_to_event(json: &str) -> Result<SecurityEvent, String> {
    todo!("Implement JSON deserialization")
}

/// Exercise 5: Validate that a SecurityEvent has all mandatory fields.
///
/// Rules:
/// - message must not be empty
/// - correlation_id must not be empty
/// - timestamp must be within the last 24 hours (reasonable check)
///
/// Return Ok(()) if valid, Err(description) if not.
pub fn validate_event(event: &SecurityEvent) -> Result<(), String> {
    todo!("Implement event validation")
}

/// Exercise 6: Create a convenience function for authentication failure logging.
///
/// This should create a properly structured WARN-level authentication failure
/// event with the user_id, ip_address, and reason as metadata.
pub fn log_auth_failure(user_id: &str, ip_address: &str, reason: &str) -> SecurityEvent {
    todo!("Implement auth failure logging helper")
}

/// A simple log sink that collects SecurityEvents for testing.
pub struct LogSink {
    pub events: Vec<SecurityEvent>,
}

impl LogSink {
    pub fn new() -> Self {
        LogSink { events: Vec::new() }
    }

    /// Exercise 7: Add an event to the sink.
    pub fn record(&mut self, event: SecurityEvent) {
        todo!("Implement event recording")
    }

    /// Exercise 8: Query events by level.
    pub fn events_with_level(&self, level: SecurityLevel) -> Vec<&SecurityEvent> {
        todo!("Implement level-based filtering")
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
        let mut event = new_security_event(
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
