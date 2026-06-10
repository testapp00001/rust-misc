//! # Lesson 08: Consent Management (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A user's consent state for various purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub user_id: String,
    pub consents: HashMap<String, bool>,
    pub last_updated: u64,
}

/// A single consent change event for the audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentEvent {
    pub user_id: String,
    pub purpose: String,
    pub granted: bool,
    pub timestamp: u64,
    pub method: String,
}

/// The consent manager tracks all user consents and audit events.
#[derive(Debug, Clone)]
pub struct ConsentManager {
    pub consents: HashMap<String, ConsentRecord>,
    pub audit_log: Vec<ConsentEvent>,
    pub essential_purposes: Vec<String>,
}

/// Create a new ConsentManager.
pub fn new_consent_manager(essential_purposes: Vec<String>) -> ConsentManager {
    ConsentManager {
        consents: HashMap::new(),
        audit_log: Vec::new(),
        essential_purposes,
    }
}

/// Record a consent decision and audit event.
pub fn record_consent_decision(
    manager: &mut ConsentManager,
    user_id: &str,
    purpose: &str,
    granted: bool,
    timestamp: u64,
    method: &str,
) {
    let record = manager
        .consents
        .entry(user_id.to_string())
        .or_insert_with(|| ConsentRecord {
            user_id: user_id.to_string(),
            consents: HashMap::new(),
            last_updated: timestamp,
        });

    record.consents.insert(purpose.to_string(), granted);
    record.last_updated = timestamp;

    manager.audit_log.push(ConsentEvent {
        user_id: user_id.to_string(),
        purpose: purpose.to_string(),
        granted,
        timestamp,
        method: method.to_string(),
    });
}

/// Check if a user has consented to a specific purpose.
pub fn has_consent(
    manager: &ConsentManager,
    user_id: &str,
    purpose: &str,
) -> Option<bool> {
    if manager.essential_purposes.contains(&purpose.to_string()) {
        return Some(true);
    }
    manager
        .consents
        .get(user_id)
        .and_then(|record| record.consents.get(purpose).copied())
}

/// Withdraw all non-essential consent for a user.
pub fn withdraw_all_consent(
    manager: &mut ConsentManager,
    user_id: &str,
    timestamp: u64,
) {
    if let Some(record) = manager.consents.get_mut(user_id) {
        let purposes: Vec<String> = record.consents.keys().cloned().collect();
        for purpose in purposes {
            if manager.essential_purposes.contains(&purpose) {
                continue;
            }
            if record.consents.get(&purpose) == Some(&true) {
                record.consents.insert(purpose.clone(), false);
                manager.audit_log.push(ConsentEvent {
                    user_id: user_id.to_string(),
                    purpose,
                    granted: false,
                    timestamp,
                    method: "bulk_withdrawal".to_string(),
                });
            }
        }
        record.last_updated = timestamp;
    }
}

/// Get consent audit trail for a user.
pub fn get_audit_trail<'a>(manager: &'a ConsentManager, user_id: &str) -> Vec<&'a ConsentEvent> {
    let mut trail: Vec<&ConsentEvent> = manager
        .audit_log
        .iter()
        .filter(|e| e.user_id == user_id)
        .collect();
    trail.sort_by_key(|e| e.timestamp);
    trail
}

/// Check if processing is allowed for user+purpose.
pub fn is_processing_allowed(
    manager: &ConsentManager,
    user_id: &str,
    purpose: &str,
) -> bool {
    has_consent(manager, user_id, purpose).unwrap_or(false)
}

/// Generate consent summary across all users.
pub fn consent_summary(manager: &ConsentManager) -> HashMap<String, (usize, usize, usize)> {
    let mut all_purposes: Vec<String> = manager.essential_purposes.clone();
    for record in manager.consents.values() {
        for purpose in record.consents.keys() {
            if !all_purposes.contains(purpose) {
                all_purposes.push(purpose.clone());
            }
        }
    }

    let total_users = manager.consents.len();
    let mut summary = HashMap::new();

    for purpose in &all_purposes {
        let mut granted = 0;
        let mut refused = 0;
        let mut not_decided = 0;

        if manager.essential_purposes.contains(purpose) {
            // Essential purposes are always "granted" for all users
            summary.insert(purpose.clone(), (total_users.max(1), 0, 0));
            continue;
        }

        for record in manager.consents.values() {
            match record.consents.get(purpose) {
                Some(true) => granted += 1,
                Some(false) => refused += 1,
                None => not_decided += 1,
            }
        }

        summary.insert(purpose.clone(), (granted, refused, not_decided));
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> ConsentManager {
        new_consent_manager(vec!["essential_service".into()])
    }

    #[test]
    fn test_new_consent_manager() {
        let manager = setup();
        assert!(manager.consents.is_empty());
        assert!(manager.audit_log.is_empty());
        assert_eq!(manager.essential_purposes, vec!["essential_service"]);
    }

    #[test]
    fn test_record_consent() {
        let mut manager = setup();
        record_consent_decision(&mut manager, "u1", "marketing", true, 1000, "ui_click");
        assert_eq!(has_consent(&manager, "u1", "marketing"), Some(true));
        assert_eq!(manager.audit_log.len(), 1);
    }

    #[test]
    fn test_has_consent_unknown() {
        let manager = setup();
        assert_eq!(has_consent(&manager, "u1", "marketing"), None);
    }

    #[test]
    fn test_has_consent_refused() {
        let mut manager = setup();
        record_consent_decision(&mut manager, "u1", "analytics", false, 1000, "ui_click");
        assert_eq!(has_consent(&manager, "u1", "analytics"), Some(false));
    }

    #[test]
    fn test_essential_always_allowed() {
        let manager = setup();
        assert_eq!(has_consent(&manager, "u1", "essential_service"), Some(true));
    }

    #[test]
    fn test_withdraw_all() {
        let mut manager = setup();
        record_consent_decision(&mut manager, "u1", "marketing", true, 1000, "ui_click");
        record_consent_decision(&mut manager, "u1", "analytics", true, 1000, "ui_click");
        record_consent_decision(&mut manager, "u1", "essential_service", true, 1000, "ui_click");

        withdraw_all_consent(&mut manager, "u1", 2000);

        assert_eq!(has_consent(&manager, "u1", "marketing"), Some(false));
        assert_eq!(has_consent(&manager, "u1", "analytics"), Some(false));
        assert_eq!(has_consent(&manager, "u1", "essential_service"), Some(true));
    }

    #[test]
    fn test_audit_trail() {
        let mut manager = setup();
        record_consent_decision(&mut manager, "u1", "marketing", true, 1000, "ui_click");
        record_consent_decision(&mut manager, "u2", "marketing", true, 1500, "ui_click");
        record_consent_decision(&mut manager, "u1", "analytics", false, 2000, "api_call");

        let trail = get_audit_trail(&manager, "u1");
        assert_eq!(trail.len(), 2);
        assert_eq!(trail[0].timestamp, 1000);
        assert_eq!(trail[1].timestamp, 2000);
    }

    #[test]
    fn test_is_processing_allowed() {
        let mut manager = setup();
        record_consent_decision(&mut manager, "u1", "marketing", true, 1000, "ui_click");

        assert!(is_processing_allowed(&manager, "u1", "marketing"));
        assert!(!is_processing_allowed(&manager, "u1", "analytics"));
        assert!(is_processing_allowed(&manager, "u1", "essential_service"));
    }

    #[test]
    fn test_consent_summary() {
        let mut manager = setup();
        record_consent_decision(&mut manager, "u1", "marketing", true, 1000, "ui_click");
        record_consent_decision(&mut manager, "u2", "marketing", false, 1000, "ui_click");
        record_consent_decision(&mut manager, "u3", "analytics", true, 1000, "ui_click");

        let summary = consent_summary(&manager);
        let (granted, refused, not_decided) = summary.get("marketing").unwrap();
        assert_eq!(*granted, 1);
        assert_eq!(*refused, 1);
        assert_eq!(*not_decided, 1);
    }
}
