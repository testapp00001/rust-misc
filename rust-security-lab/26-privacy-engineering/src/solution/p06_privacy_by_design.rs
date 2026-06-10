//! # Lesson 06: Privacy by Design (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// Configuration for a system's privacy settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub requires_consent: bool,
    pub privacy_defaults_enabled: bool,
    pub default_collected_fields: Vec<String>,
    pub required_fields: Vec<String>,
    pub encryption_at_rest: bool,
    pub encryption_in_transit: bool,
    pub audit_log_enabled: bool,
    pub retention_days: u32,
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

/// Check privacy-friendly defaults (Principle 2: Privacy as Default).
pub fn check_defaults(config: &SystemConfig) -> Result<(), String> {
    if !config.privacy_defaults_enabled {
        return Err("Privacy defaults are disabled".to_string());
    }
    let required: HashSet<&str> = config.required_fields.iter().map(|s| s.as_str()).collect();
    let extra: Vec<&str> = config
        .default_collected_fields
        .iter()
        .filter(|f| !required.contains(f.as_str()))
        .map(|s| s.as_str())
        .collect();
    if !extra.is_empty() {
        return Err(format!(
            "Default collection includes non-required fields: {:?}",
            extra
        ));
    }
    Ok(())
}

/// Check consent requirements (Principle 1: Proactive).
pub fn check_consent(config: &SystemConfig) -> Result<(), String> {
    if config.requires_consent {
        Ok(())
    } else {
        Err("System does not require user consent".to_string())
    }
}

/// Check encryption coverage (Principle 5: End-to-End Security).
pub fn check_encryption(config: &SystemConfig) -> Result<(), String> {
    let mut missing = Vec::new();
    if !config.encryption_at_rest {
        missing.push("encryption at rest");
    }
    if !config.encryption_in_transit {
        missing.push("encryption in transit");
    }
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("Missing encryption: {}", missing.join(", ")))
    }
}

/// Check audit trail (Principle 6: Visibility and Transparency).
pub fn check_audit_trail(config: &SystemConfig) -> Result<(), String> {
    if config.audit_log_enabled {
        Ok(())
    } else {
        Err("Audit logging is not enabled".to_string())
    }
}

/// Check data retention (GDPR Art. 5: Minimization).
pub fn check_retention(config: &SystemConfig) -> Result<(), String> {
    if config.retention_days == 0 {
        Err("Retention period is indefinite (0 days)".to_string())
    } else if config.retention_days > 365 * 3 {
        Err(format!(
            "Retention period {} days exceeds 3-year maximum",
            config.retention_days
        ))
    } else {
        Ok(())
    }
}

/// Run full privacy audit collecting all violations.
pub fn full_privacy_audit(config: &SystemConfig) -> Vec<PrivacyViolation> {
    let mut violations = Vec::new();

    if let Err(desc) = check_consent(config) {
        violations.push(PrivacyViolation {
            principle: "Proactive Consent".to_string(),
            description: desc,
            severity: ViolationSeverity::Critical,
        });
    }
    if let Err(desc) = check_encryption(config) {
        violations.push(PrivacyViolation {
            principle: "End-to-End Security".to_string(),
            description: desc,
            severity: ViolationSeverity::High,
        });
    }
    if let Err(desc) = check_defaults(config) {
        violations.push(PrivacyViolation {
            principle: "Privacy as Default".to_string(),
            description: desc,
            severity: ViolationSeverity::High,
        });
    }
    if let Err(desc) = check_audit_trail(config) {
        violations.push(PrivacyViolation {
            principle: "Visibility and Transparency".to_string(),
            description: desc,
            severity: ViolationSeverity::Medium,
        });
    }
    if let Err(desc) = check_retention(config) {
        violations.push(PrivacyViolation {
            principle: "Data Minimization".to_string(),
            description: desc,
            severity: ViolationSeverity::Medium,
        });
    }

    violations
}

/// Compute privacy compliance score (0-100).
pub fn privacy_score(config: &SystemConfig) -> u32 {
    let violations = full_privacy_audit(config);
    let penalty: u32 = violations
        .iter()
        .map(|v| match v.severity {
            ViolationSeverity::Critical => 25,
            ViolationSeverity::High => 15,
            ViolationSeverity::Medium => 10,
            ViolationSeverity::Low => 5,
        })
        .sum();
    100u32.saturating_sub(penalty)
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
        config.retention_days = 365 * 5;
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
        config.requires_consent = false;
        config.encryption_at_rest = false;
        config.encryption_in_transit = false;
        config.privacy_defaults_enabled = false;
        config.audit_log_enabled = false;
        config.retention_days = 0;
        let score = privacy_score(&config);
        assert!(score <= 30, "Score should be very low, got {}", score);
    }
}
