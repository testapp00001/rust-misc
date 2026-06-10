//! # Lesson 07: Mitigation Strategies (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effectiveness {
    Full,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLayer {
    Network,
    Application,
    Data,
    Monitoring,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mitigation {
    pub id: String,
    pub threat_id: String,
    pub name: String,
    pub description: String,
    pub layer: SecurityLayer,
    pub effectiveness: Effectiveness,
    pub residual_risk: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationPlan {
    pub system_name: String,
    pub mitigations: Vec<Mitigation>,
}

impl MitigationPlan {
    pub fn new(system_name: &str) -> Self {
        Self {
            system_name: system_name.to_string(),
            mitigations: Vec::new(),
        }
    }

    pub fn add_mitigation(&mut self, mitigation: Mitigation) {
        self.mitigations.push(mitigation);
    }

    pub fn mitigations_for_threat(&self, threat_id: &str) -> Vec<&Mitigation> {
        self.mitigations.iter().filter(|m| m.threat_id == threat_id).collect()
    }

    pub fn mitigations_by_layer(&self, layer: SecurityLayer) -> Vec<&Mitigation> {
        self.mitigations.iter().filter(|m| m.layer == layer).collect()
    }

    pub fn has_defense_in_depth(&self, threat_id: &str) -> bool {
        let layers: Vec<SecurityLayer> = self
            .mitigations
            .iter()
            .filter(|m| m.threat_id == threat_id)
            .map(|m| m.layer)
            .collect();
        let mut unique = layers.clone();
        unique.sort_by_key(|l| format!("{:?}", l));
        unique.dedup_by_key(|l| format!("{:?}", l));
        unique.len() >= 2
    }

    pub fn covered_layers(&self) -> Vec<SecurityLayer> {
        let mut layers: Vec<SecurityLayer> = self.mitigations.iter().map(|m| m.layer).collect();
        layers.sort_by_key(|l| format!("{:?}", l));
        layers.dedup_by_key(|l| format!("{:?}", l));
        layers
    }

    pub fn count(&self) -> usize {
        self.mitigations.len()
    }

    pub fn full_effectiveness_mitigations(&self) -> Vec<&Mitigation> {
        self.mitigations
            .iter()
            .filter(|m| m.effectiveness == Effectiveness::Full)
            .collect()
    }
}

pub fn build_webapp_mitigations() -> MitigationPlan {
    let mut plan = MitigationPlan::new("web-application");

    // Mitigations for T001: SQL Injection
    plan.add_mitigation(Mitigation {
        id: "M001".into(),
        threat_id: "T001".into(),
        name: "WAF SQL Injection Rules".into(),
        description: "Deploy WAF with OWASP Core Rule Set to block SQL injection patterns".into(),
        layer: SecurityLayer::Network,
        effectiveness: Effectiveness::High,
        residual_risk: "Novel SQL injection variants may bypass WAF signatures".into(),
    });

    plan.add_mitigation(Mitigation {
        id: "M002".into(),
        threat_id: "T001".into(),
        name: "Parameterized Queries".into(),
        description: "Use parameterized queries/prepared statements for all database operations".into(),
        layer: SecurityLayer::Application,
        effectiveness: Effectiveness::Full,
        residual_risk: "ORM misconfigurations could still allow injection".into(),
    });

    plan.add_mitigation(Mitigation {
        id: "M003".into(),
        threat_id: "T001".into(),
        name: "Database Activity Monitoring".into(),
        description: "Monitor and alert on anomalous database query patterns".into(),
        layer: SecurityLayer::Monitoring,
        effectiveness: Effectiveness::Medium,
        residual_risk: "Alert fatigue may cause missed incidents".into(),
    });

    // Mitigations for T002: Authentication Bypass
    plan.add_mitigation(Mitigation {
        id: "M004".into(),
        threat_id: "T002".into(),
        name: "Multi-Factor Authentication".into(),
        description: "Require MFA for all user and admin authentication".into(),
        layer: SecurityLayer::Application,
        effectiveness: Effectiveness::High,
        residual_risk: "SIM-swap attacks can bypass SMS-based MFA".into(),
    });

    plan.add_mitigation(Mitigation {
        id: "M005".into(),
        threat_id: "T002".into(),
        name: "Network Segmentation".into(),
        description: "Isolate authentication service in separate network segment".into(),
        layer: SecurityLayer::Network,
        effectiveness: Effectiveness::Medium,
        residual_risk: "Lateral movement from compromised adjacent segment".into(),
    });

    // Mitigations for T003: Data Exfiltration
    plan.add_mitigation(Mitigation {
        id: "M006".into(),
        threat_id: "T003".into(),
        name: "Encryption at Rest".into(),
        description: "Encrypt all sensitive data using AES-256-GCM with managed keys".into(),
        layer: SecurityLayer::Data,
        effectiveness: Effectiveness::Full,
        residual_risk: "Key compromise would negate encryption".into(),
    });

    plan.add_mitigation(Mitigation {
        id: "M007".into(),
        threat_id: "T003".into(),
        name: "DLP Monitoring".into(),
        description: "Deploy data loss prevention to detect bulk data exfiltration".into(),
        layer: SecurityLayer::Monitoring,
        effectiveness: Effectiveness::High,
        residual_risk: "Slow exfiltration may stay below detection threshold".into(),
    });

    plan.add_mitigation(Mitigation {
        id: "M008".into(),
        threat_id: "T003".into(),
        name: "Incident Response Plan".into(),
        description: "Document and rehearse incident response for data breach scenarios".into(),
        layer: SecurityLayer::Response,
        effectiveness: Effectiveness::Medium,
        residual_risk: "Response time may still allow significant data loss".into(),
    });

    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_plan() -> MitigationPlan {
        let mut plan = MitigationPlan::new("test-system");

        plan.add_mitigation(Mitigation {
            id: "M001".into(),
            threat_id: "T001".into(),
            name: "WAF Rules".into(),
            description: "Web application firewall blocks common injection patterns".into(),
            layer: SecurityLayer::Network,
            effectiveness: Effectiveness::High,
            residual_risk: "Novel attack patterns may bypass WAF".into(),
        });
        plan.add_mitigation(Mitigation {
            id: "M002".into(),
            threat_id: "T001".into(),
            name: "Input Validation".into(),
            description: "Server-side input validation on all parameters".into(),
            layer: SecurityLayer::Application,
            effectiveness: Effectiveness::High,
            residual_risk: "Validation logic may miss edge cases".into(),
        });
        plan.add_mitigation(Mitigation {
            id: "M003".into(),
            threat_id: "T002".into(),
            name: "Rate Limiting".into(),
            description: "Per-IP rate limiting on authentication endpoints".into(),
            layer: SecurityLayer::Network,
            effectiveness: Effectiveness::Medium,
            residual_risk: "Distributed attacks may still succeed".into(),
        });
        plan.add_mitigation(Mitigation {
            id: "M004".into(),
            threat_id: "T001".into(),
            name: "Encryption at Rest".into(),
            description: "Encrypt sensitive data in the database".into(),
            layer: SecurityLayer::Data,
            effectiveness: Effectiveness::Full,
            residual_risk: "Key management becomes critical".into(),
        });

        plan
    }

    #[test]
    fn test_plan_creation() {
        let plan = MitigationPlan::new("my-system");
        assert_eq!(plan.system_name, "my-system");
        assert_eq!(plan.count(), 0);
    }

    #[test]
    fn test_add_mitigation() {
        let plan = make_test_plan();
        assert_eq!(plan.count(), 4);
    }

    #[test]
    fn test_mitigations_for_threat() {
        let plan = make_test_plan();
        let t001 = plan.mitigations_for_threat("T001");
        assert_eq!(t001.len(), 3);
        let t002 = plan.mitigations_for_threat("T002");
        assert_eq!(t002.len(), 1);
    }

    #[test]
    fn test_mitigations_by_layer() {
        let plan = make_test_plan();
        let network = plan.mitigations_by_layer(SecurityLayer::Network);
        assert_eq!(network.len(), 2);
        let app = plan.mitigations_by_layer(SecurityLayer::Application);
        assert_eq!(app.len(), 1);
        let data = plan.mitigations_by_layer(SecurityLayer::Data);
        assert_eq!(data.len(), 1);
    }

    #[test]
    fn test_defense_in_depth() {
        let plan = make_test_plan();
        assert!(plan.has_defense_in_depth("T001"));
        assert!(!plan.has_defense_in_depth("T002"));
    }

    #[test]
    fn test_covered_layers() {
        let plan = make_test_plan();
        let layers = plan.covered_layers();
        assert_eq!(layers.len(), 3);
    }

    #[test]
    fn test_full_effectiveness() {
        let plan = make_test_plan();
        let full = plan.full_effectiveness_mitigations();
        assert_eq!(full.len(), 1);
        assert_eq!(full[0].id, "M004");
    }

    #[test]
    fn test_build_webapp_mitigations() {
        let plan = build_webapp_mitigations();
        assert!(plan.count() >= 6, "Should have at least 6 mitigations");
        assert!(plan.covered_layers().len() >= 3, "Should cover at least 3 layers");
    }
}
