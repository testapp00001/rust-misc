//! # Lesson 10: Privacy Impact Assessment -- Identify and Mitigate Risks
//!
//! ## What is a Privacy Impact Assessment (PIA)?
//!
//! A PIA is a systematic process for evaluating the potential effects of a project
//! on individuals' privacy. GDPR Article 35 requires a Data Protection Impact
//! Assessment (DPIA) when processing is likely to result in high risk.
//!
//! ## When is a DPIA Required?
//!
//! - Systematic and extensive profiling with significant effects
//! - Large-scale processing of sensitive categories
//! - Systematic monitoring of publicly accessible areas
//! - Any processing on the supervisory authority's "mandatory DPIA" list
//!
//! ## PIA Process
//!
//! 1. **Identify data flows**: What personal data is collected, where does it go?
//! 2. **Identify risks**: What could go wrong? (breach, misuse, re-identification)
//! 3. **Assess severity**: How bad would it be? (impact x likelihood)
//! 4. **Identify mitigations**: How do we reduce the risk?
//! 5. **Residual risk**: What risk remains after mitigations?
//! 6. **Approval**: Sign off if residual risk is acceptable
//!
//! ## Attack: Skipping the PIA
//!
//! Organizations that skip PIAs often discover privacy issues after deployment,
//! when they're expensive to fix and may have already caused harm. The PIA is
//! the "threat modeling" equivalent for privacy.
//!
//! ## Risk Scoring
//!
//! Risk = Impact x Likelihood
//! - Impact: 1 (minimal) to 5 (severe)
//! - Likelihood: 1 (rare) to 5 (almost certain)
//! - Risk score: 1-25
//!   - 1-4: Low (acceptable)
//!   - 5-9: Medium (monitor)
//!   - 10-16: High (mitigate before launch)
//!   - 17-25: Critical (do not proceed without major changes)

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A data flow in the system being assessed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    pub id: String,
    pub description: String,
    pub data_types: Vec<String>,       // e.g., ["email", "location", "health"]
    pub source: String,                // e.g., "user_input", "third_party_api"
    pub destination: String,           // e.g., "database", "analytics", "third_party"
    pub is_encrypted: bool,
    pub has_consent: bool,
    pub retention_days: u32,
}

/// A privacy risk identified during assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyRisk {
    pub id: String,
    pub description: String,
    pub related_flow: String,          // DataFlow ID
    pub impact: u8,                    // 1-5
    pub likelihood: u8,                // 1-5
    pub mitigation: String,
}

/// The full privacy impact assessment.
#[derive(Debug, Clone)]
pub struct PrivacyImpactAssessment {
    pub project_name: String,
    pub data_flows: Vec<DataFlow>,
    pub risks: Vec<PrivacyRisk>,
    pub approved: bool,
}

/// Exercise 1: Calculate the risk score for a privacy risk.
///
/// Risk score = impact * likelihood (both 1-5, result 1-25).
///
/// Hints:
/// - Simply multiply impact by likelihood
/// - Clamp inputs to 1-5 range
pub fn risk_score(risk: &PrivacyRisk) -> u8 {
    todo!("Calculate risk score = impact * likelihood")
}

/// Exercise 2: Classify risk level from score.
///
/// - 1-4: "low"
/// - 5-9: "medium"
/// - 10-16: "high"
/// - 17-25: "critical"
pub fn risk_level(score: u8) -> &'static str {
    todo!("Classify risk level from score")
}

/// Exercise 3: Check if data flows require encryption.
///
/// Flows involving sensitive data types ("health", "financial", "biometric", "location")
/// MUST be encrypted. Return IDs of flows that violate this.
///
/// Hints:
/// - Define a set of sensitive data types
/// - Check each flow: if it has sensitive types AND is not encrypted, it's a violation
pub fn check_encryption_compliance(flows: &[DataFlow]) -> Vec<String> {
    todo!("Find data flows that should be encrypted but aren't")
}

/// Exercise 4: Check if data flows have proper consent.
///
/// Flows where the source is "user_input" should have consent.
/// Return IDs of flows that violate this.
///
/// Hints:
/// - Filter flows where source == "user_input" and has_consent == false
pub fn check_consent_compliance(flows: &[DataFlow]) -> Vec<String> {
    todo!("Find data flows missing required consent")
}

/// Exercise 5: Generate a PIA summary report.
///
/// Return a report containing:
/// - Total data flows
/// - Total risks identified
/// - Risk counts by level (low, medium, high, critical)
/// - List of high/critical risks
///
/// Hints:
/// - Use risk_score and risk_level to classify each risk
/// - Group by level
pub fn pia_summary(pia: &PrivacyImpactAssessment) -> HashMap<String, String> {
    todo!("Generate PIA summary report")
}

/// Exercise 6: Determine if the PIA should be approved.
///
/// A PIA can be approved if:
/// 1. There are NO critical risks (score >= 17)
/// 2. All high risks (score >= 10) have non-empty mitigation descriptions
/// 3. All data flows have encryption if they handle sensitive data
///
/// Returns Ok(()) if approvable, or Err with list of blocking issues.
pub fn check_approval(pia: &PrivacyImpactAssessment) -> Result<(), Vec<String>> {
    todo!("Check if PIA meets approval criteria")
}

/// Exercise 7: Recommend mitigations for a data flow.
///
/// Based on the flow's properties, return a list of recommended actions:
/// - If not encrypted: "Enable encryption for data in transit and at rest"
/// - If no consent and source is "user_input": "Implement consent collection"
/// - If retention > 365: "Reduce retention period to <= 365 days"
/// - If data_types contains "health" or "biometric": "Apply additional access controls"
///
/// Hints:
/// - Check each condition and push recommendation strings
pub fn recommend_mitigations(flow: &DataFlow) -> Vec<String> {
    todo!("Recommend mitigations for a data flow")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_flow(id: &str, data_types: Vec<&str>, source: &str, encrypted: bool, consent: bool, retention: u32) -> DataFlow {
        DataFlow {
            id: id.to_string(),
            description: format!("Flow {}", id),
            data_types: data_types.into_iter().map(|s| s.to_string()).collect(),
            source: source.to_string(),
            destination: "database".to_string(),
            is_encrypted: encrypted,
            has_consent: consent,
            retention_days: retention,
        }
    }

    fn make_risk(id: &str, flow_id: &str, impact: u8, likelihood: u8, mitigation: &str) -> PrivacyRisk {
        PrivacyRisk {
            id: id.to_string(),
            description: format!("Risk {}", id),
            related_flow: flow_id.to_string(),
            impact,
            likelihood,
            mitigation: mitigation.to_string(),
        }
    }

    #[test]
    fn test_risk_score() {
        let risk = make_risk("r1", "f1", 3, 4, "");
        assert_eq!(risk_score(&risk), 12);
    }

    #[test]
    fn test_risk_score_clamped() {
        let risk = make_risk("r1", "f1", 0, 10, "");
        // After clamping: impact=1, likelihood=5
        assert_eq!(risk_score(&risk), 5);
    }

    #[test]
    fn test_risk_levels() {
        assert_eq!(risk_level(1), "low");
        assert_eq!(risk_level(4), "low");
        assert_eq!(risk_level(5), "medium");
        assert_eq!(risk_level(9), "medium");
        assert_eq!(risk_level(10), "high");
        assert_eq!(risk_level(16), "high");
        assert_eq!(risk_level(17), "critical");
        assert_eq!(risk_level(25), "critical");
    }

    #[test]
    fn test_check_encryption_compliance() {
        let flows = vec![
            make_flow("f1", vec!["email"], "user_input", true, true, 30),
            make_flow("f2", vec!["health"], "user_input", false, true, 30),
            make_flow("f3", vec!["location"], "sensor", false, true, 30),
            make_flow("f4", vec!["name"], "user_input", false, true, 30),
        ];
        let violations = check_encryption_compliance(&flows);
        assert!(violations.contains(&"f2".to_string()));
        assert!(violations.contains(&"f3".to_string()));
        assert!(!violations.contains(&"f1".to_string()));
        assert!(!violations.contains(&"f4".to_string()), "name is not sensitive");
    }

    #[test]
    fn test_check_consent_compliance() {
        let flows = vec![
            make_flow("f1", vec!["email"], "user_input", true, true, 30),
            make_flow("f2", vec!["email"], "user_input", true, false, 30),
            make_flow("f3", vec!["email"], "system_generated", true, false, 30),
        ];
        let violations = check_consent_compliance(&flows);
        assert_eq!(violations, vec!["f2"]);
    }

    #[test]
    fn test_pia_summary_counts() {
        let pia = PrivacyImpactAssessment {
            project_name: "Test".into(),
            data_flows: vec![make_flow("f1", vec!["email"], "user_input", true, true, 30)],
            risks: vec![
                make_risk("r1", "f1", 2, 2, "low risk mitigation"),  // score 4 = low
                make_risk("r2", "f1", 3, 3, "medium mitigation"),    // score 9 = medium
                make_risk("r3", "f1", 4, 3, "high mitigation"),      // score 12 = high
            ],
            approved: false,
        };
        let summary = pia_summary(&pia);
        assert!(summary.contains_key("total_flows"));
        assert!(summary.contains_key("total_risks"));
        assert!(summary.contains_key("low_risks"));
        assert!(summary.contains_key("medium_risks"));
        assert!(summary.contains_key("high_risks"));
    }

    #[test]
    fn test_check_approval_ok() {
        let pia = PrivacyImpactAssessment {
            project_name: "Test".into(),
            data_flows: vec![make_flow("f1", vec!["email"], "user_input", true, true, 30)],
            risks: vec![
                make_risk("r1", "f1", 2, 2, "acceptable"),
            ],
            approved: false,
        };
        assert!(check_approval(&pia).is_ok());
    }

    #[test]
    fn test_check_approval_critical_blocks() {
        let pia = PrivacyImpactAssessment {
            project_name: "Test".into(),
            data_flows: vec![make_flow("f1", vec!["email"], "user_input", true, true, 30)],
            risks: vec![
                make_risk("r1", "f1", 5, 5, "can't mitigate this"),
            ],
            approved: false,
        };
        assert!(check_approval(&pia).is_err());
    }

    #[test]
    fn test_check_approval_high_no_mitigation_blocks() {
        let pia = PrivacyImpactAssessment {
            project_name: "Test".into(),
            data_flows: vec![make_flow("f1", vec!["email"], "user_input", true, true, 30)],
            risks: vec![
                make_risk("r1", "f1", 4, 3, ""),  // score 12 = high, empty mitigation
            ],
            approved: false,
        };
        assert!(check_approval(&pia).is_err());
    }

    #[test]
    fn test_recommend_mitigations() {
        let flow = make_flow("f1", vec!["health", "email"], "user_input", false, false, 400);
        let recs = recommend_mitigations(&flow);
        assert!(recs.iter().any(|r| r.contains("encryption")));
        assert!(recs.iter().any(|r| r.contains("consent")));
        assert!(recs.iter().any(|r| r.contains("retention")));
        assert!(recs.iter().any(|r| r.contains("access control")));
    }

    #[test]
    fn test_recommend_mitigations_healthy_flow() {
        let flow = make_flow("f1", vec!["email"], "user_input", true, true, 30);
        let recs = recommend_mitigations(&flow);
        assert!(recs.is_empty(), "No recommendations needed for compliant flow");
    }
}
