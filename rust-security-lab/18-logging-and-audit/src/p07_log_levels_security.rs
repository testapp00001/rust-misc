//! # Lesson 07: Security-Aware Log Levels
//!
//! ## The Problem
//!
//! Developers often log security events at the wrong level:
//! - Authentication failures at DEBUG (invisible in production)
//! - Successful logins at ERROR (alert fatigue)
//! - Data breaches at INFO (gets lost in noise)
//!
//! Security events have specific severity that must map to appropriate log levels.
//! Using the wrong level means attacks go undetected or alerts are ignored.
//!
//! ## Security Log Level Mapping
//!
//! | Event | Correct Level | Why |
//! |-------|--------------|-----|
//! | Auth success | INFO | Normal operation, useful for audit |
//! | Auth failure | WARN | Potential attack, but could be user error |
//! | Multiple auth failures | ERROR | Likely brute force attack |
//! | Authorization denied | WARN | Potential privilege escalation attempt |
//! | Data breach detected | CRITICAL | Immediate human response required |
//! | Config change | INFO | Audit trail, not alarming |
//! | Admin action | INFO | Audit trail |
//! | SQL injection detected | CRITICAL | Active attack |
//! | Rate limit exceeded | WARN | Potential DoS |
//! | Invalid input (XSS attempt) | WARN | Attack attempt |
//!
//! ## What You'll Implement
//!
//! 1. A security event classifier that maps events to log levels
//! 2. A rate-limited logger that escalates repeated events
//! 3. A log level analyzer that checks for misconfigured levels
//! 4. Alert threshold configuration
//! 5. A security event aggregator that detects patterns
//! 6. Log level recommendations for common events

use std::collections::HashMap;

/// Security-aware log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

/// Security event types that need classification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SecEvent {
    AuthenticationSuccess,
    AuthenticationFailure,
    MultipleAuthFailures,
    AuthorizationDenied,
    DataBreachDetected,
    ConfigChange,
    AdminAction,
    SqlInjectionDetected,
    RateLimitExceeded,
    InvalidInput,
    SessionCreated,
    SessionExpired,
    PasswordChanged,
    AccountLocked,
}

/// Exercise 1: Map a security event to the correct log level.
///
/// Use the mapping from the lesson documentation above.
///
/// Hints:
/// - Match on SecEvent variant
/// - Return the appropriate SecLevel
pub fn classify_event(event: &SecEvent) -> SecLevel {
    todo!("Implement event classification")
}

/// Exercise 2: Check if a log level is appropriate for a security event.
///
/// Return Ok(()) if the level matches the expected level for the event.
/// Return Err(expected, actual) if mismatched.
///
/// This helps detect misconfigured logging where security events
/// are logged at too low a level.
pub fn validate_log_level(event: &SecEvent, level: &SecLevel) -> Result<(), (SecLevel, SecLevel)> {
    todo!("Implement log level validation")
}

/// A rate-aware security event tracker.
///
/// Tracks how many times each event type occurs and escalates the
/// effective log level when thresholds are exceeded.
pub struct SecurityEventTracker {
    /// Count of each event type in the current window
    pub event_counts: HashMap<SecEvent, usize>,
    /// Threshold for escalation: event -> (threshold, escalated_level)
    pub thresholds: HashMap<SecEvent, (usize, SecLevel)>,
}

impl SecurityEventTracker {
    /// Exercise 3: Create a new tracker with default thresholds.
    ///
    /// Default thresholds:
    /// - AuthenticationFailure: 5 -> Error
    /// - RateLimitExceeded: 10 -> Error
    /// - InvalidInput: 20 -> Error
    /// - AuthorizationDenied: 3 -> Error
    pub fn new() -> Self {
        todo!("Implement tracker creation with defaults")
    }

    /// Exercise 4: Record an event and return the effective log level.
    ///
    /// The effective level is:
    /// - The event's natural level (from classify_event)
    /// - OR the escalated level if the threshold is exceeded
    ///
    /// Return (effective_level, count_of_this_event).
    pub fn record_event(&mut self, event: SecEvent) -> (SecLevel, usize) {
        todo!("Implement event recording with escalation")
    }

    /// Exercise 5: Reset all counters (e.g., at the start of a new time window).
    pub fn reset(&mut self) {
        todo!("Implement counter reset")
    }
}

/// Exercise 6: Analyze a log configuration and suggest improvements.
///
/// Takes a list of (event_type, configured_level) pairs.
/// Returns a list of recommendations where the configured level
/// doesn't match the recommended level.
///
/// Each recommendation is a String like:
/// "AuthenticationFailure: configured=Debug, recommended=Warn"
pub fn analyze_log_config(config: &[(SecEvent, SecLevel)]) -> Vec<String> {
    todo!("Implement log config analysis")
}

/// Exercise 7: Generate a security event summary.
///
/// Given a list of events with their levels, produce a summary:
/// - Count by level
/// - Count by event type
/// - Highlight any CRITICAL or ERROR events
pub fn event_summary(events: &[(SecEvent, SecLevel)]) -> String {
    todo!("Implement event summary generation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_auth_failure() {
        assert_eq!(classify_event(&SecEvent::AuthenticationFailure), SecLevel::Warn);
    }

    #[test]
    fn test_classify_data_breach() {
        assert_eq!(classify_event(&SecEvent::DataBreachDetected), SecLevel::Critical);
    }

    #[test]
    fn test_classify_sql_injection() {
        assert_eq!(classify_event(&SecEvent::SqlInjectionDetected), SecLevel::Critical);
    }

    #[test]
    fn test_validate_correct_level() {
        assert!(validate_log_level(&SecEvent::AuthenticationFailure, &SecLevel::Warn).is_ok());
    }

    #[test]
    fn test_validate_wrong_level() {
        let result = validate_log_level(&SecEvent::AuthenticationFailure, &SecLevel::Debug);
        assert!(result.is_err());
        let (expected, actual) = result.unwrap_err();
        assert_eq!(expected, SecLevel::Warn);
        assert_eq!(actual, SecLevel::Debug);
    }

    #[test]
    fn test_tracker_escalation() {
        let mut tracker = SecurityEventTracker::new();
        // First few failures should be Warn level
        for _ in 0..5 {
            let (level, _) = tracker.record_event(SecEvent::AuthenticationFailure);
            assert_eq!(level, SecLevel::Warn);
        }
        // After threshold, should escalate to Error
        let (level, count) = tracker.record_event(SecEvent::AuthenticationFailure);
        assert_eq!(level, SecLevel::Error);
        assert_eq!(count, 6);
    }

    #[test]
    fn test_tracker_reset() {
        let mut tracker = SecurityEventTracker::new();
        for _ in 0..10 {
            tracker.record_event(SecEvent::AuthenticationFailure);
        }
        tracker.reset();
        let (level, count) = tracker.record_event(SecEvent::AuthenticationFailure);
        assert_eq!(level, SecLevel::Warn);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_analyze_config() {
        let config = vec![
            (SecEvent::AuthenticationFailure, SecLevel::Debug),
            (SecEvent::DataBreachDetected, SecLevel::Critical),
        ];
        let recommendations = analyze_log_config(&config);
        assert_eq!(recommendations.len(), 1);
        assert!(recommendations[0].contains("AuthenticationFailure"));
    }

    #[test]
    fn test_event_summary() {
        let events = vec![
            (SecEvent::AuthenticationFailure, SecLevel::Warn),
            (SecEvent::AuthenticationFailure, SecLevel::Warn),
            (SecEvent::DataBreachDetected, SecLevel::Critical),
        ];
        let summary = event_summary(&events);
        assert!(summary.contains("Warn: 2"));
        assert!(summary.contains("Critical: 1"));
    }
}
