//! # Lesson 07: Security-Aware Log Levels (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SecLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

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

pub fn classify_event(event: &SecEvent) -> SecLevel {
    match event {
        SecEvent::AuthenticationSuccess => SecLevel::Info,
        SecEvent::AuthenticationFailure => SecLevel::Warn,
        SecEvent::MultipleAuthFailures => SecLevel::Error,
        SecEvent::AuthorizationDenied => SecLevel::Warn,
        SecEvent::DataBreachDetected => SecLevel::Critical,
        SecEvent::ConfigChange => SecLevel::Info,
        SecEvent::AdminAction => SecLevel::Info,
        SecEvent::SqlInjectionDetected => SecLevel::Critical,
        SecEvent::RateLimitExceeded => SecLevel::Warn,
        SecEvent::InvalidInput => SecLevel::Warn,
        SecEvent::SessionCreated => SecLevel::Info,
        SecEvent::SessionExpired => SecLevel::Debug,
        SecEvent::PasswordChanged => SecLevel::Info,
        SecEvent::AccountLocked => SecLevel::Warn,
    }
}

pub fn validate_log_level(event: &SecEvent, level: &SecLevel) -> Result<(), (SecLevel, SecLevel)> {
    let expected = classify_event(event);
    if *level == expected {
        Ok(())
    } else {
        Err((expected, *level))
    }
}

pub struct SecurityEventTracker {
    pub event_counts: HashMap<SecEvent, usize>,
    pub thresholds: HashMap<SecEvent, (usize, SecLevel)>,
}

impl SecurityEventTracker {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        thresholds.insert(SecEvent::AuthenticationFailure, (5, SecLevel::Error));
        thresholds.insert(SecEvent::RateLimitExceeded, (10, SecLevel::Error));
        thresholds.insert(SecEvent::InvalidInput, (20, SecLevel::Error));
        thresholds.insert(SecEvent::AuthorizationDenied, (3, SecLevel::Error));

        SecurityEventTracker {
            event_counts: HashMap::new(),
            thresholds,
        }
    }

    pub fn record_event(&mut self, event: SecEvent) -> (SecLevel, usize) {
        let natural_level = classify_event(&event);
        let count = self.event_counts.entry(event.clone()).or_insert(0);
        *count += 1;
        let current_count = *count;

        // Check if threshold is exceeded
        if let Some(&(threshold, escalated_level)) = self.thresholds.get(&event) {
            if current_count > threshold {
                return (escalated_level, current_count);
            }
        }

        (natural_level, current_count)
    }

    pub fn reset(&mut self) {
        self.event_counts.clear();
    }
}

pub fn analyze_log_config(config: &[(SecEvent, SecLevel)]) -> Vec<String> {
    config
        .iter()
        .filter_map(|(event, configured_level)| {
            let recommended = classify_event(event);
            if *configured_level != recommended {
                Some(format!(
                    "{:?}: configured={:?}, recommended={:?}",
                    event, configured_level, recommended
                ))
            } else {
                None
            }
        })
        .collect()
}

pub fn event_summary(events: &[(SecEvent, SecLevel)]) -> String {
    let mut level_counts: HashMap<SecLevel, usize> = HashMap::new();
    let mut event_type_counts: HashMap<String, usize> = HashMap::new();
    let mut critical_events = Vec::new();
    let mut error_events = Vec::new();

    for (event, level) in events {
        *level_counts.entry(*level).or_insert(0) += 1;
        let event_name = format!("{:?}", event);
        *event_type_counts.entry(event_name.clone()).or_insert(0) += 1;

        if *level == SecLevel::Critical {
            critical_events.push(event_name.clone());
        } else if *level == SecLevel::Error {
            error_events.push(event_name.clone());
        }
    }

    let mut summary = format!("Total events: {}\n", events.len());

    summary.push_str("\nBy level:\n");
    let mut levels: Vec<_> = level_counts.iter().collect();
    levels.sort_by_key(|(l, _)| **l);
    for (level, count) in levels {
        summary.push_str(&format!("  {:?}: {}\n", level, count));
    }

    summary.push_str("\nBy event type:\n");
    let mut types: Vec<_> = event_type_counts.iter().collect();
    types.sort_by_key(|(_, c)| **c);
    for (event, count) in types.iter().rev() {
        summary.push_str(&format!("  {}: {}\n", event, count));
    }

    if !critical_events.is_empty() {
        summary.push_str(&format!(
            "\n!! CRITICAL events: {:?}\n",
            critical_events
        ));
    }
    if !error_events.is_empty() {
        summary.push_str(&format!("!! ERROR events: {:?}\n", error_events));
    }

    summary
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
        for _ in 0..5 {
            let (level, _) = tracker.record_event(SecEvent::AuthenticationFailure);
            assert_eq!(level, SecLevel::Warn);
        }
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
