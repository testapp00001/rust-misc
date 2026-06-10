//! # Lesson 06: Threat Enumeration (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreatCategory {
    Spoofing,
    Tampering,
    Repudiation,
    InformationDisclosure,
    DenialOfService,
    ElevationOfPrivilege,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumeratedThreat {
    pub id: String,
    pub component: String,
    pub category: ThreatCategory,
    pub kill_chain_stage: KillChainStage,
    pub description: String,
    pub cve_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEnumeration {
    pub system_name: String,
    pub threats: Vec<EnumeratedThreat>,
}

impl ThreatEnumeration {
    pub fn new(system_name: &str) -> Self {
        Self {
            system_name: system_name.to_string(),
            threats: Vec::new(),
        }
    }

    pub fn add_threat(&mut self, threat: EnumeratedThreat) {
        self.threats.push(threat);
    }

    pub fn threats_for_component(&self, component: &str) -> Vec<&EnumeratedThreat> {
        self.threats.iter().filter(|t| t.component == component).collect()
    }

    pub fn threats_by_category(&self, category: ThreatCategory) -> Vec<&EnumeratedThreat> {
        self.threats.iter().filter(|t| t.category == category).collect()
    }

    pub fn threats_by_kill_chain(&self, stage: KillChainStage) -> Vec<&EnumeratedThreat> {
        self.threats.iter().filter(|t| t.kill_chain_stage == stage).collect()
    }

    pub fn components(&self) -> Vec<String> {
        let mut comps: Vec<String> = self.threats.iter().map(|t| t.component.clone()).collect();
        comps.sort();
        comps.dedup();
        comps
    }

    pub fn threats_with_cves(&self) -> Vec<&EnumeratedThreat> {
        self.threats.iter().filter(|t| !t.cve_ids.is_empty()).collect()
    }

    pub fn all_components_covered(&self, components: &[&str]) -> bool {
        let known = self.components();
        components.iter().all(|c| known.contains(&c.to_string()))
    }

    pub fn count(&self) -> usize {
        self.threats.len()
    }
}

pub fn build_microservices_enumeration() -> ThreatEnumeration {
    let mut te = ThreatEnumeration::new("microservices-platform");

    te.add_threat(EnumeratedThreat {
        id: "T001".into(),
        component: "auth-service".into(),
        category: ThreatCategory::Spoofing,
        kill_chain_stage: KillChainStage::Exploitation,
        description: "Attacker forges JWT tokens using weak signing algorithm (RS256 -> none)".into(),
        cve_ids: vec!["CVE-2022-23529".into()],
    });

    te.add_threat(EnumeratedThreat {
        id: "T002".into(),
        component: "auth-service".into(),
        category: ThreatCategory::ElevationOfPrivilege,
        kill_chain_stage: KillChainStage::ActionsOnObjectives,
        description: "Attacker manipulates role claim to escalate from user to admin".into(),
        cve_ids: vec![],
    });

    te.add_threat(EnumeratedThreat {
        id: "T003".into(),
        component: "auth-service".into(),
        category: ThreatCategory::InformationDisclosure,
        kill_chain_stage: KillChainStage::Exploitation,
        description: "Username enumeration via different error messages on login".into(),
        cve_ids: vec![],
    });

    te.add_threat(EnumeratedThreat {
        id: "T004".into(),
        component: "api-gateway".into(),
        category: ThreatCategory::DenialOfService,
        kill_chain_stage: KillChainStage::Delivery,
        description: "Volumetric DDoS attack overwhelms gateway capacity".into(),
        cve_ids: vec![],
    });

    te.add_threat(EnumeratedThreat {
        id: "T005".into(),
        component: "api-gateway".into(),
        category: ThreatCategory::Tampering,
        kill_chain_stage: KillChainStage::Exploitation,
        description: "Attacker modifies request headers to bypass rate limiting".into(),
        cve_ids: vec!["CVE-2023-44487".into()],
    });

    te.add_threat(EnumeratedThreat {
        id: "T006".into(),
        component: "data-service".into(),
        category: ThreatCategory::InformationDisclosure,
        kill_chain_stage: KillChainStage::Exploitation,
        description: "SQL injection leaks sensitive user data".into(),
        cve_ids: vec![],
    });

    te.add_threat(EnumeratedThreat {
        id: "T007".into(),
        component: "data-service".into(),
        category: ThreatCategory::Tampering,
        kill_chain_stage: KillChainStage::ActionsOnObjectives,
        description: "NoSQL injection modifies data integrity".into(),
        cve_ids: vec![],
    });

    te.add_threat(EnumeratedThreat {
        id: "T008".into(),
        component: "data-service".into(),
        category: ThreatCategory::Repudiation,
        kill_chain_stage: KillChainStage::ActionsOnObjectives,
        description: "Data modifications not logged; attacker can deny changes".into(),
        cve_ids: vec![],
    });

    te.add_threat(EnumeratedThreat {
        id: "T009".into(),
        component: "api-gateway".into(),
        category: ThreatCategory::Spoofing,
        kill_chain_stage: KillChainStage::Reconnaissance,
        description: "API key leakage from client-side code enables impersonation".into(),
        cve_ids: vec![],
    });

    te
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
