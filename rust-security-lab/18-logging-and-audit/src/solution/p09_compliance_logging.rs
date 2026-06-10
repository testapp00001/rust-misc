//! # Lesson 09: Compliance Logging (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceFramework {
    Gdpr,
    Soc2,
    Hipaa,
    PciDss,
    Sox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceLogEntry {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub user_id: Option<String>,
    pub data_classification: DataClassification,
    pub resource: String,
    pub action: String,
    pub result: String,
    pub applicable_frameworks: Vec<ComplianceFramework>,
    pub contains_pii: bool,
    pub retention_days: u32,
}

pub fn applicable_frameworks(classification: &DataClassification) -> Vec<ComplianceFramework> {
    match classification {
        DataClassification::Public => vec![],
        DataClassification::Internal => vec![ComplianceFramework::Soc2],
        DataClassification::Confidential => {
            vec![ComplianceFramework::Soc2, ComplianceFramework::Gdpr]
        }
        DataClassification::Restricted => vec![
            ComplianceFramework::Soc2,
            ComplianceFramework::Gdpr,
            ComplianceFramework::Hipaa,
            ComplianceFramework::PciDss,
        ],
    }
}

pub fn required_retention_days(frameworks: &[ComplianceFramework]) -> u32 {
    frameworks
        .iter()
        .map(|f| match f {
            ComplianceFramework::Gdpr => 365,
            ComplianceFramework::Soc2 => 365,
            ComplianceFramework::Hipaa => 2190,  // 6 years
            ComplianceFramework::PciDss => 365,
            ComplianceFramework::Sox => 2555,    // 7 years
        })
        .max()
        .unwrap_or(365)
}

pub fn create_compliance_entry(
    event_type: &str,
    user_id: Option<&str>,
    data_classification: DataClassification,
    resource: &str,
    action: &str,
    result: &str,
) -> ComplianceLogEntry {
    let frameworks = applicable_frameworks(&data_classification);
    let retention = required_retention_days(&frameworks);
    let contains_pii = data_classification == DataClassification::Restricted;

    ComplianceLogEntry {
        timestamp: Utc::now(),
        event_type: event_type.to_string(),
        user_id: user_id.map(|s| s.to_string()),
        data_classification,
        resource: resource.to_string(),
        action: action.to_string(),
        result: result.to_string(),
        applicable_frameworks: frameworks,
        contains_pii,
        retention_days: retention,
    }
}

pub fn is_within_retention(entry: &ComplianceLogEntry) -> bool {
    let expiry = entry.timestamp + Duration::days(entry.retention_days as i64);
    Utc::now() < expiry
}

pub fn gdpr_erase_user(
    entries: &mut [ComplianceLogEntry],
    user_id: &str,
) {
    for entry in entries.iter_mut() {
        if entry.user_id.as_deref() == Some(user_id) {
            entry.user_id = Some("[REDACTED]".to_string());
            entry.contains_pii = false;
        }
    }
}

pub fn compliance_report(
    entries: &[ComplianceLogEntry],
    framework: &ComplianceFramework,
) -> String {
    let applicable: Vec<&ComplianceLogEntry> = entries
        .iter()
        .filter(|e| e.applicable_frameworks.contains(framework))
        .collect();

    let within_retention = applicable.iter().filter(|e| is_within_retention(e)).count();
    let expired = applicable.len() - within_retention;
    let pii_entries = applicable.iter().filter(|e| e.contains_pii).count();

    let framework_name = match framework {
        ComplianceFramework::Gdpr => "GDPR",
        ComplianceFramework::Soc2 => "SOC2",
        ComplianceFramework::Hipaa => "HIPAA",
        ComplianceFramework::PciDss => "PCI-DSS",
        ComplianceFramework::Sox => "SOX",
    };

    let violations = detect_violations(entries);

    format!(
        "{} Compliance Report\n\
         ====================\n\
         Total applicable entries: {}\n\
         Within retention: {}\n\
         Expired: {}\n\
         PII-containing entries: {}\n\
         Violations detected: {}",
        framework_name,
        applicable.len(),
        within_retention,
        expired,
        pii_entries,
        violations.len()
    )
}

#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    pub framework: ComplianceFramework,
    pub description: String,
    pub severity: String,
}

pub fn detect_violations(entries: &[ComplianceLogEntry]) -> Vec<ComplianceViolation> {
    let mut violations = Vec::new();

    for entry in entries {
        // Check for expired entries
        if !is_within_retention(entry) {
            for framework in &entry.applicable_frameworks {
                violations.push(ComplianceViolation {
                    framework: framework.clone(),
                    description: format!(
                        "Entry '{}' at {} has exceeded retention period of {} days",
                        entry.event_type, entry.timestamp, entry.retention_days
                    ),
                    severity: "HIGH".to_string(),
                });
            }
        }

        // Check for PII without user_id (can't support right to erasure)
        if entry.contains_pii && entry.user_id.is_none() {
            for framework in &entry.applicable_frameworks {
                violations.push(ComplianceViolation {
                    framework: framework.clone(),
                    description: format!(
                        "PII entry '{}' has no user_id -- cannot support right to erasure",
                        entry.event_type
                    ),
                    severity: "CRITICAL".to_string(),
                });
            }
        }

        // Check restricted data has all required frameworks
        if entry.data_classification == DataClassification::Restricted {
            let required = applicable_frameworks(&DataClassification::Restricted);
            for framework in &required {
                if !entry.applicable_frameworks.contains(framework) {
                    violations.push(ComplianceViolation {
                        framework: framework.clone(),
                        description: format!(
                            "Restricted data entry '{}' missing {:?} framework tag",
                            entry.event_type, framework
                        ),
                        severity: "HIGH".to_string(),
                    });
                }
            }
        }
    }

    violations
}

pub fn requires_audit(action: &str, classification: &DataClassification) -> bool {
    match (action, classification) {
        ("read", DataClassification::Confidential) => true,
        ("read", DataClassification::Restricted) => true,
        ("write", DataClassification::Internal) => true,
        ("write", DataClassification::Confidential) => true,
        ("write", DataClassification::Restricted) => true,
        ("delete", _) => true,
        ("export", _) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_compliance_entry_restricted() {
        let entry = create_compliance_entry(
            "data_access",
            Some("user-1"),
            DataClassification::Restricted,
            "/api/phi",
            "read",
            "success",
        );
        assert!(entry.applicable_frameworks.contains(&ComplianceFramework::Gdpr));
        assert!(entry.applicable_frameworks.contains(&ComplianceFramework::Hipaa));
        assert!(entry.contains_pii);
    }

    #[test]
    fn test_create_compliance_entry_public() {
        let entry = create_compliance_entry(
            "page_view",
            None,
            DataClassification::Public,
            "/about",
            "read",
            "success",
        );
        assert!(entry.applicable_frameworks.is_empty());
        assert!(!entry.contains_pii);
    }

    #[test]
    fn test_retention_check() {
        let entry = ComplianceLogEntry {
            timestamp: Utc::now() - Duration::days(100),
            event_type: "test".to_string(),
            user_id: None,
            data_classification: DataClassification::Internal,
            resource: "/api/test".to_string(),
            action: "read".to_string(),
            result: "success".to_string(),
            applicable_frameworks: vec![ComplianceFramework::Soc2],
            contains_pii: false,
            retention_days: 365,
        };
        assert!(is_within_retention(&entry));
    }

    #[test]
    fn test_retention_expired() {
        let entry = ComplianceLogEntry {
            timestamp: Utc::now() - Duration::days(400),
            event_type: "test".to_string(),
            user_id: None,
            data_classification: DataClassification::Internal,
            resource: "/api/test".to_string(),
            action: "read".to_string(),
            result: "success".to_string(),
            applicable_frameworks: vec![ComplianceFramework::Soc2],
            contains_pii: false,
            retention_days: 365,
        };
        assert!(!is_within_retention(&entry));
    }

    #[test]
    fn test_gdpr_erase() {
        let mut entries = vec![
            create_compliance_entry(
                "data_access", Some("user-1"), DataClassification::Restricted,
                "/api/phi", "read", "success",
            ),
            create_compliance_entry(
                "data_access", Some("user-2"), DataClassification::Restricted,
                "/api/phi", "read", "success",
            ),
        ];
        gdpr_erase_user(&mut entries, "user-1");
        assert_eq!(entries[0].user_id, Some("[REDACTED]".to_string()));
        assert_eq!(entries[1].user_id, Some("user-2".to_string()));
    }

    #[test]
    fn test_compliance_report() {
        let entries = vec![
            create_compliance_entry(
                "data_access", Some("u1"), DataClassification::Restricted,
                "/api/phi", "read", "success",
            ),
        ];
        let report = compliance_report(&entries, &ComplianceFramework::Hipaa);
        assert!(report.contains("HIPAA"));
        assert!(report.contains("1"));
    }

    #[test]
    fn test_requires_audit() {
        assert!(requires_audit("read", &DataClassification::Confidential));
        assert!(requires_audit("write", &DataClassification::Internal));
        assert!(requires_audit("delete", &DataClassification::Public));
        assert!(!requires_audit("read", &DataClassification::Public));
    }

    #[test]
    fn test_retention_calculation() {
        let frameworks = vec![ComplianceFramework::Soc2, ComplianceFramework::Hipaa];
        let days = required_retention_days(&frameworks);
        assert_eq!(days, 2190);
    }

    #[test]
    fn test_detect_violations_pii_without_user() {
        let entry = ComplianceLogEntry {
            timestamp: Utc::now(),
            event_type: "data_access".to_string(),
            user_id: None,
            data_classification: DataClassification::Restricted,
            resource: "/api/phi".to_string(),
            action: "read".to_string(),
            result: "success".to_string(),
            applicable_frameworks: vec![ComplianceFramework::Gdpr],
            contains_pii: true,
            retention_days: 365,
        };
        let violations = detect_violations(&[entry]);
        assert!(!violations.is_empty());
    }
}
