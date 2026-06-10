//! # Lesson 07: Mitigation Strategies
//!
//! ## The Problem
//!
//! Identifying threats is only half the battle. You need to design defenses that are
//! layered, redundant, and practical. A single control that fails means total compromise.
//!
//! ## The Solution: Defense in Depth
//!
//! Defense in depth applies multiple layers of security, so that if one layer fails,
//! others still protect the system:
//!
//! ```text
//! Layer 1: Network         Firewalls, WAF, DDoS protection
//! Layer 2: Application     Input validation, authentication, authorization
//! Layer 3: Data            Encryption at rest, encryption in transit
//! Layer 4: Monitoring      Logging, alerting, intrusion detection
//! Layer 5: Response        Incident response, forensics, recovery
//! ```
//!
//! Each mitigation maps to one or more threats and has:
//! - **Effectiveness**: How well does it reduce the threat?
//! - **Cost**: Implementation and maintenance effort
//! - **Residual risk**: What remains after the mitigation?
//!
//! ## Attack Example: Single Point of Failure
//!
//! A system relies solely on a firewall for security. An attacker gains access through
//! a compromised VPN credential. Once inside, there's no application-level auth, no
//! encryption, no monitoring. Total compromise.
//!
//! Defense: Layer controls so no single failure causes total compromise.

use serde::{Deserialize, Serialize};

/// How effective a mitigation is at reducing a threat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effectiveness {
    /// Blocks the attack completely
    Full,
    /// Significantly reduces risk but doesn't eliminate it
    High,
    /// Moderately reduces risk
    Medium,
    /// Slightly reduces risk (better than nothing)
    Low,
}

/// The security layer where a mitigation operates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SecurityLayer {
    /// Firewalls, WAF, network segmentation
    Network,
    /// Input validation, auth, authz
    Application,
    /// Encryption at rest and in transit
    Data,
    /// Logging, alerting, IDS
    Monitoring,
    /// Incident response, backup, recovery
    Response,
}

/// A mitigation strategy for a specific threat.
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

/// A collection of mitigations forming a defense-in-depth strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationPlan {
    pub system_name: String,
    pub mitigations: Vec<Mitigation>,
}

impl MitigationPlan {
    /// Create a new empty mitigation plan.
    pub fn new(system_name: &str) -> Self {
        todo!("Create empty plan")
    }

    /// Add a mitigation to the plan.
    pub fn add_mitigation(&mut self, mitigation: Mitigation) {
        todo!("Add mitigation")
    }

    /// Return mitigations for a specific threat ID.
    pub fn mitigations_for_threat(&self, threat_id: &str) -> Vec<&Mitigation> {
        todo!("Filter by threat_id")
    }

    /// Return mitigations for a specific security layer.
    pub fn mitigations_by_layer(&self, layer: SecurityLayer) -> Vec<&Mitigation> {
        todo!("Filter by security layer")
    }

    /// Check if a given threat has mitigations at multiple layers (defense in depth).
    /// Returns true if the threat has mitigations in 2+ distinct layers.
    pub fn has_defense_in_depth(&self, threat_id: &str) -> bool {
        todo!("Check if threat has mitigations in multiple layers")
    }

    /// Return all security layers that have at least one mitigation.
    pub fn covered_layers(&self) -> Vec<SecurityLayer> {
        todo!("Collect unique layers")
    }

    /// Count total mitigations.
    pub fn count(&self) -> usize {
        todo!("Return count")
    }

    /// Return mitigations with Full effectiveness.
    pub fn full_effectiveness_mitigations(&self) -> Vec<&Mitigation> {
        todo!("Filter by Full effectiveness")
    }
}

/// Build a mitigation plan for a web application threat model.
pub fn build_webapp_mitigations() -> MitigationPlan {
    todo!("Build plan with at least 6 mitigations covering multiple layers and threats")
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
        // T001 has Network + Application + Data = 3 layers
        assert!(plan.has_defense_in_depth("T001"));
        // T002 only has Network = 1 layer
        assert!(!plan.has_defense_in_depth("T002"));
    }

    #[test]
    fn test_covered_layers() {
        let plan = make_test_plan();
        let layers = plan.covered_layers();
        assert_eq!(layers.len(), 3); // Network, Application, Data
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
