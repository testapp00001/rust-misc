//! # Lesson 06: Threat Enumeration
//!
//! ## The Problem
//!
//! Identifying a few obvious threats is easy. Identifying ALL possible threats requires
//! systematic enumeration. Without it, you defend against the attacks you know and get
//! surprised by the ones you don't.
//!
//! ## The Solution: Systematic Threat Enumeration
//!
//! Threat enumeration combines multiple approaches:
//!
//! 1. **STRIDE per component**: Apply all 6 STRIDE categories to each component
//! 2. **Kill chain analysis**: Map threats to stages of an attack (reconnaissance, weaponization,
//!    delivery, exploitation, installation, actions on objectives)
//! 3. **Known vulnerability databases**: Check CVEs, CWEs for each component
//! 4. **Threat intelligence**: Check for active exploitation of similar systems
//!
//! ```text
//! Component     STRIDE Threats              Known CVEs    Kill Chain Stage
//! ─────────     ──────────────              ──────────    ────────────────
//! Web Server    Spoofing, Tampering, ...    CVE-2023-xxx  Delivery
//! Database      Info Disclosure, Tampering  CVE-2023-yyy  Exploitation
//! Auth Service  Spoofing, Elevation         CVE-2023-zzz  Installation
//! ```
//!
//! ## Attack Example: Incomplete Enumeration
//!
//! A team enumerates threats for their web server but forgets the CI/CD pipeline.
//! An attacker compromises the build system and injects malware into the deployment artifact.
//! The web server's security was irrelevant because the code itself was compromised.
//!
//! Defense: Enumerate threats for EVERY component, including build, deploy, and monitoring.

use serde::{Deserialize, Serialize};

/// The STRIDE category (reused for enumeration).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatCategory {
    Spoofing,
    Tampering,
    Repudiation,
    InformationDisclosure,
    DenialOfService,
    ElevationOfPrivilege,
}

/// The stage of the attack kill chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KillChainStage {
    Reconnaissance,
    Weaponization,
    Delivery,
    Exploitation,
    Installation,
    CommandAndControl,
    ActionsOnObjectives,
}

/// A single enumerated threat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumeratedThreat {
    pub id: String,
    pub component: String,
    pub category: ThreatCategory,
    pub kill_chain_stage: KillChainStage,
    pub description: String,
    pub cve_ids: Vec<String>,
}

/// A threat enumeration report for a system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEnumeration {
    pub system_name: String,
    pub threats: Vec<EnumeratedThreat>,
}

impl ThreatEnumeration {
    /// Create a new empty enumeration.
    pub fn new(system_name: &str) -> Self {
        todo!("Create empty enumeration")
    }

    /// Add a threat to the enumeration.
    pub fn add_threat(&mut self, threat: EnumeratedThreat) {
        todo!("Add threat")
    }

    /// Return threats for a specific component.
    pub fn threats_for_component(&self, component: &str) -> Vec<&EnumeratedThreat> {
        todo!("Filter threats by component name")
    }

    /// Return threats for a specific STRIDE category.
    pub fn threats_by_category(&self, category: ThreatCategory) -> Vec<&EnumeratedThreat> {
        todo!("Filter by STRIDE category")
    }

    /// Return threats for a specific kill chain stage.
    pub fn threats_by_kill_chain(&self, stage: KillChainStage) -> Vec<&EnumeratedThreat> {
        todo!("Filter by kill chain stage")
    }

    /// Return all unique components mentioned in the enumeration.
    pub fn components(&self) -> Vec<String> {
        todo!("Collect unique component names")
    }

    /// Return all threats that have known CVEs.
    pub fn threats_with_cves(&self) -> Vec<&EnumeratedThreat> {
        todo!("Filter threats where cve_ids is not empty")
    }

    /// Check if every component has at least one threat.
    pub fn all_components_covered(&self, components: &[&str]) -> bool {
        todo!("Check if all given components appear in at least one threat")
    }

    /// Count total threats.
    pub fn count(&self) -> usize {
        todo!("Return count")
    }
}

/// Build a threat enumeration for a microservices system with auth, API, and data services.
pub fn build_microservices_enumeration() -> ThreatEnumeration {
    todo!("Enumerate threats for auth-service, api-gateway, data-service with at least 8 threats")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_enumeration() -> ThreatEnumeration {
        let mut te = ThreatEnumeration::new("test-system");
        te.add_threat(EnumeratedThreat {
            id: "T001".into(),
            component: "auth-service".into(),
            category: ThreatCategory::Spoofing,
            kill_chain_stage: KillChainStage::Exploitation,
            description: "Token forgery".into(),
            cve_ids: vec!["CVE-2023-0001".into()],
        });
        te.add_threat(EnumeratedThreat {
            id: "T002".into(),
            component: "api-gateway".into(),
            category: ThreatCategory::DenialOfService,
            kill_chain_stage: KillChainStage::Delivery,
            description: "Rate limit bypass".into(),
            cve_ids: vec![],
        });
        te.add_threat(EnumeratedThreat {
            id: "T003".into(),
            component: "auth-service".into(),
            category: ThreatCategory::ElevationOfPrivilege,
            kill_chain_stage: KillChainStage::ActionsOnObjectives,
            description: "Role manipulation".into(),
            cve_ids: vec!["CVE-2023-0002".into()],
        });
        te
    }

    #[test]
    fn test_enumeration_new() {
        let te = ThreatEnumeration::new("my-system");
        assert_eq!(te.system_name, "my-system");
        assert_eq!(te.count(), 0);
    }

    #[test]
    fn test_add_threat() {
        let te = make_test_enumeration();
        assert_eq!(te.count(), 3);
    }

    #[test]
    fn test_threats_for_component() {
        let te = make_test_enumeration();
        let auth_threats = te.threats_for_component("auth-service");
        assert_eq!(auth_threats.len(), 2);
        let api_threats = te.threats_for_component("api-gateway");
        assert_eq!(api_threats.len(), 1);
    }

    #[test]
    fn test_threats_by_category() {
        let te = make_test_enumeration();
        let spoofing = te.threats_by_category(ThreatCategory::Spoofing);
        assert_eq!(spoofing.len(), 1);
        let dos = te.threats_by_category(ThreatCategory::DenialOfService);
        assert_eq!(dos.len(), 1);
    }

    #[test]
    fn test_threats_by_kill_chain() {
        let te = make_test_enumeration();
        let exploit = te.threats_by_kill_chain(KillChainStage::Exploitation);
        assert_eq!(exploit.len(), 1);
    }

    #[test]
    fn test_components() {
        let te = make_test_enumeration();
        let components = te.components();
        assert_eq!(components.len(), 2);
        assert!(components.contains(&"auth-service".to_string()));
        assert!(components.contains(&"api-gateway".to_string()));
    }

    #[test]
    fn test_threats_with_cves() {
        let te = make_test_enumeration();
        let with_cves = te.threats_with_cves();
        assert_eq!(with_cves.len(), 2);
    }

    #[test]
    fn test_all_components_covered() {
        let te = make_test_enumeration();
        assert!(te.all_components_covered(&["auth-service", "api-gateway"]));
        assert!(!te.all_components_covered(&["auth-service", "api-gateway", "missing-service"]));
    }

    #[test]
    fn test_build_microservices_enumeration() {
        let te = build_microservices_enumeration();
        assert!(te.count() >= 8, "Should have at least 8 threats");
        assert!(te.components().len() >= 3, "Should cover at least 3 components");
    }
}
