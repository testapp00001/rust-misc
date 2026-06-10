//! # Lesson 09: Risk Assessment
//!
//! ## The Problem
//!
//! Not all threats deserve the same response. Some have catastrophic impact but are
//! extremely unlikely; others have moderate impact but happen frequently. Without a
//! framework for assessing risk, teams either over-invest in unlikely threats or
//! under-invest in common ones.
//!
//! ## The Solution: Likelihood x Impact
//!
//! Risk is quantified as:
//!
//! ```text
//! Risk = Likelihood x Impact
//!
//! Likelihood (1-5):  How likely is this threat to be exploited?
//!   1 = Very unlikely (theoretical, requires nation-state)
//!   2 = Unlikely (requires significant resources)
//!   3 = Possible (requires moderate skill)
//!   4 = Likely (requires basic skill)
//!   5 = Very likely (automated, publicly known exploit)
//!
//! Impact (1-5):  How severe is the damage if exploited?
//!   1 = Negligible (minor inconvenience)
//!   2 = Low (limited data exposure)
//!   3 = Medium (significant data breach)
//!   4 = High (major financial/regulatory impact)
//!   5 = Critical (business-ending, safety risk)
//!
//! Risk Score = Likelihood x Impact (1-25)
//! ```
//!
//! Risk tolerance determines the response:
//! - **1-4**: Accept the risk (document and monitor)
//! - **5-9**: Mitigate with standard controls
//! - **10-16**: Mitigate with enhanced controls; regular review
//! - **17-25**: Critical; immediate action required; executive visibility
//!
//! ## Attack Example: Misjudged Risk
//!
//! A team rates "server room flooding" as low risk (unlikely). But the server room is in
//! a basement next to a river. The likelihood was underestimated, and when the river floods,
//! all on-premise backups are destroyed. Impact: total data loss.
//!
//! Defense: Use structured risk assessment with multiple reviewers and historical data.

use serde::{Deserialize, Serialize};

/// How the organization should respond to a risk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskResponse {
    /// Accept the risk; document and monitor
    Accept,
    /// Reduce with standard controls
    Mitigate,
    /// Reduce with enhanced controls; regular review
    MitigateUrgent,
    /// Immediate action required; executive visibility
    Critical,
}

/// A single risk assessment entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEntry {
    pub id: String,
    pub threat_id: String,
    pub description: String,
    pub likelihood: u8,  // 1-5
    pub impact: u8,      // 1-5
    pub mitigation_plan: String,
    pub residual_likelihood: u8,  // likelihood after mitigation
    pub residual_impact: u8,      // impact after mitigation
}

impl RiskEntry {
    /// Create a new risk entry. Likelihood and impact must be 1-5.
    pub fn new(
        id: &str,
        threat_id: &str,
        description: &str,
        likelihood: u8,
        impact: u8,
        mitigation_plan: &str,
        residual_likelihood: u8,
        residual_impact: u8,
    ) -> Result<Self, String> {
        todo!("Validate likelihood and impact are 1-5 for both initial and residual")
    }

    /// Compute the initial risk score (likelihood * impact).
    pub fn initial_risk(&self) -> u32 {
        todo!("Multiply likelihood by impact")
    }

    /// Compute the residual risk score after mitigation.
    pub fn residual_risk(&self) -> u32 {
        todo!("Multiply residual_likelihood by residual_impact")
    }

    /// Compute risk reduction as a percentage: (initial - residual) / initial * 100.
    /// Returns 0.0 if initial risk is 0.
    pub fn risk_reduction_percent(&self) -> f64 {
        todo!("Compute percentage reduction")
    }

    /// Determine the risk response for the initial risk score.
    pub fn initial_response(&self) -> RiskResponse {
        todo!("Map initial risk score to RiskResponse")
    }

    /// Determine the risk response for the residual risk score.
    pub fn residual_response(&self) -> RiskResponse {
        todo!("Map residual risk score to RiskResponse")
    }
}

/// A complete risk assessment for a system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub system_name: String,
    pub risks: Vec<RiskEntry>,
}

impl RiskAssessment {
    /// Create a new empty risk assessment.
    pub fn new(system_name: &str) -> Self {
        todo!("Create empty assessment")
    }

    /// Add a risk entry.
    pub fn add_risk(&mut self, risk: RiskEntry) {
        todo!("Add risk")
    }

    /// Return all risks sorted by initial risk score descending.
    pub fn ranked_by_initial_risk(&self) -> Vec<&RiskEntry> {
        todo!("Sort by initial_risk descending")
    }

    /// Return all risks sorted by residual risk score descending.
    pub fn ranked_by_residual_risk(&self) -> Vec<&RiskEntry> {
        todo!("Sort by residual_risk descending")
    }

    /// Return only Critical-response risks (initial score 17-25).
    pub fn critical_risks(&self) -> Vec<&RiskEntry> {
        todo!("Filter by initial_response == Critical")
    }

    /// Return the average initial risk score.
    pub fn average_initial_risk(&self) -> f64 {
        todo!("Compute average initial risk")
    }

    /// Return the average residual risk score.
    pub fn average_residual_risk(&self) -> f64 {
        todo!("Compute average residual risk")
    }

    /// Count total risks.
    pub fn count(&self) -> usize {
        todo!("Return count")
    }
}

/// Build a risk assessment for a web application.
pub fn build_webapp_risk_assessment() -> RiskAssessment {
    todo!("Build assessment with at least 5 risks covering different likelihood/impact combinations")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_risks() -> RiskAssessment {
        let mut ra = RiskAssessment::new("test-system");

        ra.add_risk(RiskEntry::new(
            "R001", "T001", "SQL injection on user search endpoint",
            4, 5, "Parameterized queries + WAF",
            1, 5
        ).unwrap());

        ra.add_risk(RiskEntry::new(
            "R002", "T002", "DDoS on public API",
            5, 3, "Rate limiting + CDN",
            3, 2
        ).unwrap());

        ra.add_risk(RiskEntry::new(
            "R003", "T003", "Theoretical timing attack on crypto",
            1, 4, "Constant-time comparison (already implemented)",
            1, 4
        ).unwrap());

        ra
    }

    #[test]
    fn test_risk_entry_valid() {
        let entry = RiskEntry::new("R1", "T1", "test", 3, 3, "plan", 2, 2);
        assert!(entry.is_ok());
    }

    #[test]
    fn test_risk_entry_invalid_likelihood() {
        let entry = RiskEntry::new("R1", "T1", "test", 0, 3, "plan", 2, 2);
        assert!(entry.is_err());
    }

    #[test]
    fn test_risk_entry_invalid_impact() {
        let entry = RiskEntry::new("R1", "T1", "test", 3, 6, "plan", 2, 2);
        assert!(entry.is_err());
    }

    #[test]
    fn test_initial_risk() {
        let entry = RiskEntry::new("R1", "T1", "test", 4, 5, "plan", 1, 5).unwrap();
        assert_eq!(entry.initial_risk(), 20);
    }

    #[test]
    fn test_residual_risk() {
        let entry = RiskEntry::new("R1", "T1", "test", 4, 5, "plan", 1, 5).unwrap();
        assert_eq!(entry.residual_risk(), 5);
    }

    #[test]
    fn test_risk_reduction() {
        let entry = RiskEntry::new("R1", "T1", "test", 4, 5, "plan", 1, 5).unwrap();
        // (20 - 5) / 20 * 100 = 75%
        assert!((entry.risk_reduction_percent() - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_risk_response_levels() {
        let r1 = RiskEntry::new("R1", "T1", "test", 1, 1, "plan", 1, 1).unwrap();
        assert_eq!(r1.initial_response(), RiskResponse::Accept);

        let r2 = RiskEntry::new("R2", "T2", "test", 2, 3, "plan", 1, 1).unwrap();
        assert_eq!(r2.initial_response(), RiskResponse::Mitigate);

        let r3 = RiskEntry::new("R3", "T3", "test", 4, 3, "plan", 1, 1).unwrap();
        assert_eq!(r3.initial_response(), RiskResponse::MitigateUrgent);

        let r4 = RiskEntry::new("R4", "T4", "test", 5, 5, "plan", 1, 1).unwrap();
        assert_eq!(r4.initial_response(), RiskResponse::Critical);
    }

    #[test]
    fn test_risk_assessment_ranking() {
        let ra = make_test_risks();
        let ranked = ra.ranked_by_initial_risk();
        // R002: 5*3=15, R001: 4*5=20, R003: 1*4=4
        assert_eq!(ranked[0].id, "R001"); // 20
        assert_eq!(ranked[1].id, "R002"); // 15
        assert_eq!(ranked[2].id, "R003"); // 4
    }

    #[test]
    fn test_critical_risks() {
        let ra = make_test_risks();
        let critical = ra.critical_risks();
        // R001: 4*5=20 >= 17 => Critical
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].id, "R001");
    }

    #[test]
    fn test_averages() {
        let ra = make_test_risks();
        // Initial: (20 + 15 + 4) / 3 = 13.0
        assert!((ra.average_initial_risk() - 13.0).abs() < 0.01);
        // Residual: (5 + 6 + 4) / 3 = 5.0
        assert!((ra.average_residual_risk() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_build_webapp_risk_assessment() {
        let ra = build_webapp_risk_assessment();
        assert!(ra.count() >= 5, "Should have at least 5 risks");
    }
}
