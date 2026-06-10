//! # Lesson 10: Privacy Impact Assessment (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A data flow in the system being assessed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    pub id: String,
    pub description: String,
    pub data_types: Vec<String>,
    pub source: String,
    pub destination: String,
    pub is_encrypted: bool,
    pub has_consent: bool,
    pub retention_days: u32,
}

/// A privacy risk identified during assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyRisk {
    pub id: String,
    pub description: String,
    pub related_flow: String,
    pub impact: u8,
    pub likelihood: u8,
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

/// Calculate risk score = impact * likelihood.
pub fn risk_score(risk: &PrivacyRisk) -> u8 {
    let impact = risk.impact.clamp(1, 5);
    let likelihood = risk.likelihood.clamp(1, 5);
    impact * likelihood
}

/// Classify risk level from score.
pub fn risk_level(score: u8) -> &'static str {
    match score {
        1..=4 => "low",
        5..=9 => "medium",
        10..=16 => "high",
        _ => "critical",
    }
}

/// Find data flows that should be encrypted but aren't.
pub fn check_encryption_compliance(flows: &[DataFlow]) -> Vec<String> {
    let sensitive_types: Vec<&str> = vec!["health", "financial", "biometric", "location"];
    flows
        .iter()
        .filter(|flow| {
            !flow.is_encrypted
                && flow
                    .data_types
                    .iter()
                    .any(|dt| sensitive_types.contains(&dt.as_str()))
        })
        .map(|flow| flow.id.clone())
        .collect()
}

/// Find data flows missing required consent.
pub fn check_consent_compliance(flows: &[DataFlow]) -> Vec<String> {
    flows
        .iter()
        .filter(|flow| flow.source == "user_input" && !flow.has_consent)
        .map(|flow| flow.id.clone())
        .collect()
}

/// Generate PIA summary report.
pub fn pia_summary(pia: &PrivacyImpactAssessment) -> HashMap<String, String> {
    let mut counts: HashMap<String, u32> = HashMap::new();
    for risk in &pia.risks {
        let level = risk_level(risk_score(risk)).to_string();
        *counts.entry(level).or_insert(0) += 1;
    }

    let high_critical: Vec<String> = pia
        .risks
        .iter()
        .filter(|r| {
            let level = risk_level(risk_score(r));
            level == "high" || level == "critical"
        })
        .map(|r| format!("{}: {}", r.id, r.description))
        .collect();

    let mut summary = HashMap::new();
    summary.insert("total_flows".to_string(), pia.data_flows.len().to_string());
    summary.insert("total_risks".to_string(), pia.risks.len().to_string());
    summary.insert(
        "low_risks".to_string(),
        counts.get("low").copied().unwrap_or(0).to_string(),
    );
    summary.insert(
        "medium_risks".to_string(),
        counts.get("medium").copied().unwrap_or(0).to_string(),
    );
    summary.insert(
        "high_risks".to_string(),
        counts.get("high").copied().unwrap_or(0).to_string(),
    );
    summary.insert(
        "critical_risks".to_string(),
        counts.get("critical").copied().unwrap_or(0).to_string(),
    );
    summary.insert("high_critical_details".to_string(), high_critical.join("; "));
    summary
}

/// Check if PIA meets approval criteria.
pub fn check_approval(pia: &PrivacyImpactAssessment) -> Result<(), Vec<String>> {
    let mut issues = Vec::new();

    // 1. No critical risks
    for risk in &pia.risks {
        if risk_score(risk) >= 17 {
            issues.push(format!(
                "Critical risk {}: {} (score {})",
                risk.id,
                risk.description,
                risk_score(risk)
            ));
        }
    }

    // 2. All high risks must have mitigation
    for risk in &pia.risks {
        let score = risk_score(risk);
        if score >= 10 && score < 17 && risk.mitigation.is_empty() {
            issues.push(format!(
                "High risk {} has no mitigation plan",
                risk.id
            ));
        }
    }

    // 3. Sensitive data flows must be encrypted
    let unencrypted = check_encryption_compliance(&pia.data_flows);
    if !unencrypted.is_empty() {
        issues.push(format!(
            "Unencrypted sensitive data flows: {:?}",
            unencrypted
        ));
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

/// Recommend mitigations for a data flow.
pub fn recommend_mitigations(flow: &DataFlow) -> Vec<String> {
    let mut recommendations = Vec::new();

    if !flow.is_encrypted {
        recommendations.push("Enable encryption for data in transit and at rest".to_string());
    }

    if flow.source == "user_input" && !flow.has_consent {
        recommendations.push("Implement consent collection before processing".to_string());
    }

    if flow.retention_days > 365 {
        recommendations.push("Reduce retention period to <= 365 days".to_string());
    }

    let sensitive_types = ["health", "biometric"];
    if flow
        .data_types
        .iter()
        .any(|dt| sensitive_types.contains(&dt.as_str()))
    {
        recommendations.push("Apply additional access controls for sensitive data".to_string());
    }

    recommendations
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
        assert!(!violations.contains(&"f4".to_string()));
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
                make_risk("r1", "f1", 2, 2, "low risk mitigation"),
                make_risk("r2", "f1", 3, 3, "medium mitigation"),
                make_risk("r3", "f1", 4, 3, "high mitigation"),
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
            risks: vec![make_risk("r1", "f1", 2, 2, "acceptable")],
            approved: false,
        };
        assert!(check_approval(&pia).is_ok());
    }

    #[test]
    fn test_check_approval_critical_blocks() {
        let pia = PrivacyImpactAssessment {
            project_name: "Test".into(),
            data_flows: vec![make_flow("f1", vec!["email"], "user_input", true, true, 30)],
            risks: vec![make_risk("r1", "f1", 5, 5, "can't mitigate this")],
            approved: false,
        };
        assert!(check_approval(&pia).is_err());
    }

    #[test]
    fn test_check_approval_high_no_mitigation_blocks() {
        let pia = PrivacyImpactAssessment {
            project_name: "Test".into(),
            data_flows: vec![make_flow("f1", vec!["email"], "user_input", true, true, 30)],
            risks: vec![make_risk("r1", "f1", 4, 3, "")],
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
        assert!(recs.is_empty());
    }
}
