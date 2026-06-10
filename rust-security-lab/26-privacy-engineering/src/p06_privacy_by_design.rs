//! # Lesson 06: Privacy by Design -- Embedding Privacy into Architecture
//!
//! ## What is Privacy by Design?
//!
//! Privacy by Design (PbD) is an approach to systems engineering that takes privacy
//! into account throughout the whole engineering process -- from initial design to
//! deployment, operation, and retirement.
//!
//! ## The 7 Foundational Principles (Ann Cavoukian, 1995)
//!
//! 1. **Proactive not Reactive**: Anticipate and prevent privacy-invasive events
//! 2. **Privacy as Default**: Maximum privacy without user action required
//! 3. **Privacy Embedded into Design**: Core functionality, not a bolt-on
//! 4. **Full Functionality**: Positive-sum, not zero-sum (privacy AND security)
//! 5. **End-to-End Security**: Full lifecycle protection
//! 6. **Visibility and Transparency**: Keep it open and verifiable
//! 7. **Respect for User Privacy**: Keep it user-centric
//!
//! ## GDPR Article 25: Data Protection by Design and by Default
//!
//! This is a legal requirement in the EU. Organizations must implement:
//! - Data minimization at design time
//! - Pseudonymization as a default
//! - Privacy-friendly default settings
//! - Necessary safeguards integrated into processing
//!
//! ## Attack: Privacy Theater
//!
//! Many systems claim "privacy by design" but implement it as an afterthought:
//! - Privacy settings default to "share everything"
//! - PII stored in plaintext with encryption only at rest
//! - No audit trail for data access
//! - Consent bundled into ToS with no granularity
//!
//! This lesson builds a privacy-aware system configuration validator.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Configuration for a system's privacy settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Whether data collection requires explicit user consent
    pub requires_consent: bool,
    /// Whether privacy-friendly defaults are enabled
    pub privacy_defaults_enabled: bool,
    /// Fields that are collected by default (without consent)
    pub default_collected_fields: Vec<String>,
    /// Fields that are strictly necessary for the service
    pub required_fields: Vec<String>,
    /// Whether data is encrypted at rest
    pub encryption_at_rest: bool,
    /// Whether data is encrypted in transit
    pub encryption_in_transit: bool,
    /// Whether an audit log exists for data access
    pub audit_log_enabled: bool,
    /// Retention period in days (0 = indefinite)
    pub retention_days: u32,
    /// Whether pseudonymization is applied to identifiers
    pub pseudonymization_enabled: bool,
}

/// A privacy violation found during audit.
#[derive(Debug, Clone, PartialEq)]
pub struct PrivacyViolation {
    pub principle: String,
    pub description: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Exercise 1: Check if the system uses privacy-friendly defaults.
///
/// Privacy as Default (Principle 2) requires:
/// - `privacy_defaults_enabled` is true
/// - `default_collected_fields` is a subset of `required_fields`
///   (don't collect extra fields by default)
///
/// Returns Ok(()) or Err describing the violation.
pub fn check_defaults(config: &SystemConfig) -> Result<(), String> {
    todo!("Check privacy-friendly defaults")
}

/// Exercise 2: Check if consent is properly required.
///
/// Proactive (Principle 1) requires:
/// - `requires_consent` is true
///
/// Returns Ok(()) or Err.
pub fn check_consent(config: &SystemConfig) -> Result<(), String> {
    todo!("Check consent requirements")
}

/// Exercise 3: Check encryption coverage.
///
/// End-to-End Security (Principle 5) requires:
/// - `encryption_at_rest` is true
/// - `encryption_in_transit` is true
///
/// Returns Ok(()) or Err listing what's missing.
pub fn check_encryption(config: &SystemConfig) -> Result<(), String> {
    todo!("Check encryption coverage")
}

/// Exercise 4: Check audit trail.
///
/// Visibility and Transparency (Principle 6) requires:
/// - `audit_log_enabled` is true
///
/// Returns Ok(()) or Err.
pub fn check_audit_trail(config: &SystemConfig) -> Result<(), String> {
    todo!("Check audit trail presence")
}

/// Exercise 5: Check data retention.
///
/// Data Minimization (GDPR Art. 5) requires:
/// - `retention_days` > 0 (not indefinite)
/// - `retention_days` <= 365 * 3 (no more than 3 years)
///
/// Returns Ok(()) or Err.
pub fn check_retention(config: &SystemConfig) -> Result<(), String> {
    todo!("Check data retention policy")
}

/// Exercise 6: Run a full privacy audit on a system configuration.
///
/// Run all checks and collect violations. Return a list of PrivacyViolation.
///
/// Severity mapping:
/// - Consent missing -> Critical
/// - Encryption missing -> High
/// - Defaults not privacy-friendly -> High
/// - No audit trail -> Medium
/// - Retention issues -> Medium
///
/// Hints:
/// - Call each check function
/// - If Err, create a PrivacyViolation with appropriate severity
pub fn full_privacy_audit(config: &SystemConfig) -> Vec<PrivacyViolation> {
    todo!("Run full privacy audit collecting all violations")
}

/// Exercise 7: Score the privacy configuration (0-100).
///
/// Start at 100, subtract points for each violation:
/// - Critical: -25
/// - High: -15
/// - Medium: -10
/// - Low: -5
///
/// Clamp to minimum 0.
pub fn privacy_score(config: &SystemConfig) -> u32 {
    todo!("Compute privacy compliance score")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good_config() -> SystemConfig {
        SystemConfig {
            requires_consent: true,
            privacy_defaults_enabled: true,
            default_collected_fields: vec!["email".into()],
            required_fields: vec!["email".into()],
            encryption_at_rest: true,
            encryption_in_transit: true,
            audit_log_enabled: true,
            retention_days: 365,
            pseudonymization_enabled: true,
        }
    }

    #[test]
    fn test_check_defaults_ok() {
        assert!(check_defaults(&good_config()).is_ok());
    }

    #[test]
    fn test_check_defaults_excess_collection() {
        let mut config = good_config();
        config.default_collected_fields.push("phone".into());
        assert!(check_defaults(&config).is_err());
    }

    #[test]
    fn test_check_defaults_disabled() {
        let mut config = good_config();
        config.privacy_defaults_enabled = false;
        assert!(check_defaults(&config).is_err());
    }

    #[test]
    fn test_check_consent_ok() {
        assert!(check_consent(&good_config()).is_ok());
    }

    #[test]
    fn test_check_consent_missing() {
        let mut config = good_config();
        config.requires_consent = false;
        assert!(check_consent(&config).is_err());
    }

    #[test]
    fn test_check_encryption_ok() {
        assert!(check_encryption(&good_config()).is_ok());
    }

    #[test]
    fn test_check_encryption_missing_at_rest() {
        let mut config = good_config();
        config.encryption_at_rest = false;
        assert!(check_encryption(&config).is_err());
    }

    #[test]
    fn test_check_retention_ok() {
        assert!(check_retention(&good_config()).is_ok());
    }

    #[test]
    fn test_check_retention_indefinite() {
        let mut config = good_config();
        config.retention_days = 0;
        assert!(check_retention(&config).is_err());
    }

    #[test]
    fn test_check_retention_too_long() {
        let mut config = good_config();
        config.retention_days = 365 * 5; // 5 years
        assert!(check_retention(&config).is_err());
    }

    #[test]
    fn test_full_audit_no_violations() {
        let violations = full_privacy_audit(&good_config());
        assert!(violations.is_empty());
    }

    #[test]
    fn test_full_audit_multiple_violations() {
        let mut config = good_config();
        config.requires_consent = false;
        config.encryption_at_rest = false;
        config.audit_log_enabled = false;
        let violations = full_privacy_audit(&config);
        assert!(violations.len() >= 3);
    }

    #[test]
    fn test_privacy_score_perfect() {
        assert_eq!(privacy_score(&good_config()), 100);
    }

    #[test]
    fn test_privacy_score_low() {
        let mut config = good_config();
        config.requires_consent = false;       // -25 (critical)
        config.encryption_at_rest = false;      // -15 (high)
        config.encryption_in_transit = false;   // -15 (high)
        config.privacy_defaults_enabled = false; // -15 (high)
        config.audit_log_enabled = false;       // -10 (medium)
        config.retention_days = 0;              // -10 (medium)
        let score = privacy_score(&config);
        assert!(score <= 30, "Score should be very low, got {}", score);
    }
}
