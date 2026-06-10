//! # Lesson 09: Risk Assessment (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskResponse {
    Accept,
    Mitigate,
    MitigateUrgent,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEntry {
    pub id: String,
    pub threat_id: String,
    pub description: String,
    pub likelihood: u8,
    pub impact: u8,
    pub mitigation_plan: String,
    pub residual_likelihood: u8,
    pub residual_impact: u8,
}

impl RiskEntry {
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
        for (name, val) in [("likelihood", likelihood), ("impact", impact),
            ("residual_likelihood", residual_likelihood), ("residual_impact", residual_impact)] {
            if val < 1 || val > 5 {
                return Err(format!("{} must be 1-5, got {}", name, val));
            }
        }
        Ok(Self {
            id: id.to_string(),
            threat_id: threat_id.to_string(),
            description: description.to_string(),
            likelihood,
            impact,
            mitigation_plan: mitigation_plan.to_string(),
            residual_likelihood,
            residual_impact,
        })
    }

    pub fn initial_risk(&self) -> u32 {
        self.likelihood as u32 * self.impact as u32
    }

    pub fn residual_risk(&self) -> u32 {
        self.residual_likelihood as u32 * self.residual_impact as u32
    }

    pub fn risk_reduction_percent(&self) -> f64 {
        let initial = self.initial_risk();
        if initial == 0 {
            return 0.0;
        }
        let residual = self.residual_risk();
        ((initial as f64 - residual as f64) / initial as f64) * 100.0
    }

    pub fn initial_response(&self) -> RiskResponse {
        match self.initial_risk() {
            1..=4 => RiskResponse::Accept,
            5..=9 => RiskResponse::Mitigate,
            10..=16 => RiskResponse::MitigateUrgent,
            _ => RiskResponse::Critical,
        }
    }

    pub fn residual_response(&self) -> RiskResponse {
        match self.residual_risk() {
            1..=4 => RiskResponse::Accept,
            5..=9 => RiskResponse::Mitigate,
            10..=16 => RiskResponse::MitigateUrgent,
            _ => RiskResponse::Critical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub system_name: String,
    pub risks: Vec<RiskEntry>,
}

impl RiskAssessment {
    pub fn new(system_name: &str) -> Self {
        Self {
            system_name: system_name.to_string(),
            risks: Vec::new(),
        }
    }

    pub fn add_risk(&mut self, risk: RiskEntry) {
        self.risks.push(risk);
    }

    pub fn ranked_by_initial_risk(&self) -> Vec<&RiskEntry> {
        let mut ranked: Vec<&RiskEntry> = self.risks.iter().collect();
        ranked.sort_by(|a, b| b.initial_risk().cmp(&a.initial_risk()));
        ranked
    }

    pub fn ranked_by_residual_risk(&self) -> Vec<&RiskEntry> {
        let mut ranked: Vec<&RiskEntry> = self.risks.iter().collect();
        ranked.sort_by(|a, b| b.residual_risk().cmp(&a.residual_risk()));
        ranked
    }

    pub fn critical_risks(&self) -> Vec<&RiskEntry> {
        self.risks
            .iter()
            .filter(|r| r.initial_response() == RiskResponse::Critical)
            .collect()
    }

    pub fn average_initial_risk(&self) -> f64 {
        if self.risks.is_empty() {
            return 0.0;
        }
        let sum: u32 = self.risks.iter().map(|r| r.initial_risk()).sum();
        sum as f64 / self.risks.len() as f64
    }

    pub fn average_residual_risk(&self) -> f64 {
        if self.risks.is_empty() {
            return 0.0;
        }
        let sum: u32 = self.risks.iter().map(|r| r.residual_risk()).sum();
        sum as f64 / self.risks.len() as f64
    }

    pub fn count(&self) -> usize {
        self.risks.len()
    }
}

pub fn build_webapp_risk_assessment() -> RiskAssessment {
    let mut ra = RiskAssessment::new("web-application");

    ra.add_risk(RiskEntry::new(
        "R001", "T001", "SQL injection on user search endpoint",
        4, 5, "Parameterized queries + WAF + input validation",
        1, 5
    ).unwrap());

    ra.add_risk(RiskEntry::new(
        "R002", "T002", "Volumetric DDoS on public API gateway",
        5, 4, "CDN with DDoS protection + rate limiting + auto-scaling",
        3, 3
    ).unwrap());

    ra.add_risk(RiskEntry::new(
        "R003", "T003", "Stored XSS in user profile fields",
        4, 3, "HTML escaping + CSP headers + input sanitization",
        2, 3
    ).unwrap());

    ra.add_risk(RiskEntry::new(
        "R004", "T004", "JWT token forgery using weak algorithm",
        3, 5, "Enforce RS256 + token rotation + short expiry",
        1, 5
    ).unwrap());

    ra.add_risk(RiskEntry::new(
        "R005", "T005", "Insider threat: developer with production access",
        2, 5, "Least privilege + audit logging + code review requirements",
        1, 4
    ).unwrap());

    ra.add_risk(RiskEntry::new(
        "R006", "T006", "Third-party dependency with known CVE",
        4, 4, "Automated dependency scanning + rapid patching process",
        2, 3
    ).unwrap());

    ra
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
        assert_eq!(ranked[0].id, "R001"); // 20
        assert_eq!(ranked[1].id, "R002"); // 15
        assert_eq!(ranked[2].id, "R003"); // 4
    }

    #[test]
    fn test_critical_risks() {
        let ra = make_test_risks();
        let critical = ra.critical_risks();
        assert_eq!(critical.len(), 1);
        assert_eq!(critical[0].id, "R001");
    }

    #[test]
    fn test_averages() {
        let ra = make_test_risks();
        assert!((ra.average_initial_risk() - 13.0).abs() < 0.01);
        assert!((ra.average_residual_risk() - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_build_webapp_risk_assessment() {
        let ra = build_webapp_risk_assessment();
        assert!(ra.count() >= 5, "Should have at least 5 risks");
    }
}
