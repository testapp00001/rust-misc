//! # Lesson 08: Consent Management -- Granular Consent and Audit Trails
//!
//! ## What is Consent Management?
//!
//! GDPR requires that consent be freely given, specific, informed, and unambiguous
//! (Article 4(11)). Consent management systems track what users have consented to,
//! allow withdrawal, and maintain audit trails.
//!
//! ## Requirements for Valid Consent (GDPR Art. 7)
//!
//! 1. **Freely given**: Not bundled with service access
//! 2. **Specific**: Separate consent per purpose
//! 3. **Informed**: User knows what they're consenting to
//! 4. **Unambiguous**: Clear affirmative action (not pre-ticked boxes)
//! 5. **Withdrawable**: As easy to withdraw as to give
//!
//! ## Granular Consent
//!
//! Users should be able to consent to each processing purpose independently:
//! - Marketing emails: YES / NO
//! - Analytics tracking: YES / NO
//! - Third-party sharing: YES / NO
//! - Personalization: YES / NO
//!
//! ## Attack: Consent Bundling
//!
//! Services that force "accept all or use nothing" violate GDPR. Users must be
//! able to use core functionality without consenting to non-essential processing.
//!
//! ## Audit Trail
//!
//! Every consent change must be logged with:
//! - User ID
//! - Purpose
//! - Consent granted/withdrawn
//! - Timestamp
//! - Method (UI click, API call, etc.)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A user's consent state for various purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    pub user_id: String,
    /// Map of purpose -> consented (true/false)
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
    pub method: String,  // "ui_click", "api_call", "admin_override"
}

/// The consent manager tracks all user consents and audit events.
#[derive(Debug, Clone)]
pub struct ConsentManager {
    pub consents: HashMap<String, ConsentRecord>,
    pub audit_log: Vec<ConsentEvent>,
    /// Purposes that are essential (cannot be refused)
    pub essential_purposes: Vec<String>,
}

/// Exercise 1: Create a new ConsentManager.
///
/// Initialize with empty consents, empty audit log, and the given essential purposes.
pub fn new_consent_manager(essential_purposes: Vec<String>) -> ConsentManager {
    todo!("Create a new ConsentManager")
}

/// Exercise 2: Record a consent decision.
///
/// Update the user's consent record and add an event to the audit log.
/// If the user has no existing record, create one.
///
/// Hints:
/// - Get or create ConsentRecord for the user
/// - Update the consents map with the new decision
/// - Set last_updated to the event timestamp
/// - Push the event to audit_log
pub fn record_consent_decision(
    manager: &mut ConsentManager,
    user_id: &str,
    purpose: &str,
    granted: bool,
    timestamp: u64,
    method: &str,
) {
    todo!("Record a consent decision and audit event")
}

/// Exercise 3: Check if a user has consented to a specific purpose.
///
/// Returns:
/// - Some(true) if consented
/// - Some(false) if explicitly refused
/// - None if no decision recorded
///
/// Essential purposes always return Some(true).
pub fn has_consent(
    manager: &ConsentManager,
    user_id: &str,
    purpose: &str,
) -> Option<bool> {
    todo!("Check if user has consented to a purpose")
}

/// Exercise 4: Withdraw all consent for a user.
///
/// Set all purposes to false and record individual audit events for each.
/// Essential purposes are NOT withdrawn.
///
/// Hints:
/// - Get the user's consent record
/// - For each purpose that is not essential and is currently true, set to false
/// - Record an audit event for each withdrawal
pub fn withdraw_all_consent(
    manager: &mut ConsentManager,
    user_id: &str,
    timestamp: u64,
) {
    todo!("Withdraw all non-essential consent for a user")
}

/// Exercise 5: Get the audit trail for a specific user.
///
/// Return all ConsentEvents for the given user_id, in chronological order.
///
/// Hints:
/// - Filter audit_log by user_id
/// - Sort by timestamp
pub fn get_audit_trail<'a>(manager: &'a ConsentManager, user_id: &str) -> Vec<&'a ConsentEvent> {
    todo!("Get consent audit trail for a user")
}

/// Exercise 6: Check if all required consents are in place for a purpose.
///
/// A purpose is "allowed" if the user has given consent (Some(true)).
/// Essential purposes are always allowed.
///
/// Returns true if processing is permitted for this user+purpose combination.
pub fn is_processing_allowed(
    manager: &ConsentManager,
    user_id: &str,
    purpose: &str,
) -> bool {
    todo!("Check if processing is allowed for user+purpose")
}

/// Exercise 7: Generate a consent summary report.
///
/// Return a map of purpose -> (granted_count, refused_count, not_decided_count)
/// across all users.
pub fn consent_summary(manager: &ConsentManager) -> HashMap<String, (usize, usize, usize)> {
    todo!("Generate consent summary across all users")
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
        assert_eq!(has_consent(&manager, "u1", "essential_service"), Some(true), "Essential should not be withdrawn");
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
        assert!(!is_processing_allowed(&manager, "u1", "analytics")); // no decision
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
        assert_eq!(*not_decided, 1); // u3 hasn't decided on marketing
    }
}
