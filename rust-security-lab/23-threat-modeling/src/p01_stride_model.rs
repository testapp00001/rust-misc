//! # Lesson 01: STRIDE Threat Model
//!
//! ## The Problem
//!
//! When designing a system, it's easy to focus on the threats you know (like SQL injection)
//! and miss entire categories (like repudiation or tampering). Without a systematic framework,
//! your threat coverage is ad-hoc and incomplete.
//!
//! ## The Solution: STRIDE
//!
//! STRIDE is a mnemonic for six threat categories, originally from Microsoft:
//!
//! ```text
//! S — Spoofing              Impersonating another user or system
//! T — Tampering              Unauthorized modification of data
//! R — Repudiation            Denying an action without proof
//! I — Information Disclosure Exposing data to unauthorized parties
//! D — Denial of Service      Degrading or blocking system availability
//! E — Elevation of Privilege Gaining unauthorized access levels
//! ```
//!
//! Each component in a system should be analyzed against all six categories.
//!
//! ## Why STRIDE Works
//!
//! - **Comprehensive**: Forces you to consider ALL threat types, not just the obvious ones
//! - **Systematic**: Each component gets the same treatment
//! - **Actionable**: Each category suggests specific mitigations
//! - **Collaborative**: Non-security engineers can learn and apply it
//!
//! ## Attack Example: Missing STRIDE Category
//!
//! A team builds an API that validates authentication (anti-spoofing) and encrypts data
//! (anti-information-disclosure), but forgets audit logging (anti-repudiation). When a user
//! performs malicious actions, there's no way to prove who did what.
//!
//! Defense: Apply STRIDE systematically to every component.

use serde::{Deserialize, Serialize};

/// Represents the six STRIDE threat categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrideCategory {
    /// Impersonating another user or system
    Spoofing,
    /// Unauthorized modification of data
    Tampering,
    /// Denying an action without proof
    Repudiation,
    /// Exposing data to unauthorized parties
    InformationDisclosure,
    /// Degrading or blocking system availability
    DenialOfService,
    /// Gaining unauthorized access levels
    ElevationOfPrivilege,
}

/// A single threat identified through STRIDE analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    pub id: String,
    pub category: StrideCategory,
    pub description: String,
    pub component: String,
    pub mitigation: String,
}

/// A STRIDE analysis for a system component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrideAnalysis {
    pub component: String,
    pub threats: Vec<Threat>,
}

impl StrideAnalysis {
    /// Create a new empty STRIDE analysis for a component.
    pub fn new(component: &str) -> Self {
        todo!("Create a new StrideAnalysis with empty threats")
    }

    /// Add a threat to this analysis.
    pub fn add_threat(&mut self, threat: Threat) {
        todo!("Add a threat to the analysis")
    }

    /// Return all threats matching a given STRIDE category.
    pub fn threats_by_category(&self, category: StrideCategory) -> Vec<&Threat> {
        todo!("Filter threats by STRIDE category")
    }

    /// Check if all six STRIDE categories are covered.
    /// Returns true if at least one threat exists for each category.
    pub fn is_complete(&self) -> bool {
        todo!("Check if all 6 STRIDE categories have at least one threat")
    }

    /// Return the set of STRIDE categories that are NOT covered by any threat.
    pub fn missing_categories(&self) -> Vec<StrideCategory> {
        todo!("Find which STRIDE categories have no threats")
    }

    /// Return total threat count.
    pub fn threat_count(&self) -> usize {
        todo!("Return the number of threats")
    }
}

/// Build a STRIDE analysis for a web authentication component.
/// Pre-populate one threat per STRIDE category.
pub fn build_auth_stride_analysis() -> StrideAnalysis {
    todo!("Build a complete STRIDE analysis for web authentication")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stride_analysis_new() {
        let analysis = StrideAnalysis::new("web-api");
        assert_eq!(analysis.component, "web-api");
        assert_eq!(analysis.threat_count(), 0);
    }

    #[test]
    fn test_add_threat() {
        let mut analysis = StrideAnalysis::new("web-api");
        analysis.add_threat(Threat {
            id: "T001".into(),
            category: StrideCategory::Spoofing,
            description: "Attacker forges JWT tokens".into(),
            component: "web-api".into(),
            mitigation: "Validate token signatures with strong keys".into(),
        });
        assert_eq!(analysis.threat_count(), 1);
    }

    #[test]
    fn test_threats_by_category() {
        let mut analysis = StrideAnalysis::new("web-api");
        analysis.add_threat(Threat {
            id: "T001".into(),
            category: StrideCategory::Spoofing,
            description: "Forged tokens".into(),
            component: "web-api".into(),
            mitigation: "Validate signatures".into(),
        });
        analysis.add_threat(Threat {
            id: "T002".into(),
            category: StrideCategory::Tampering,
            description: "Modified payloads".into(),
            component: "web-api".into(),
            mitigation: "Use HMAC".into(),
        });

        let spoofing = analysis.threats_by_category(StrideCategory::Spoofing);
        assert_eq!(spoofing.len(), 1);
        assert_eq!(spoofing[0].id, "T001");

        let tampering = analysis.threats_by_category(StrideCategory::Tampering);
        assert_eq!(tampering.len(), 1);

        let dos = analysis.threats_by_category(StrideCategory::DenialOfService);
        assert_eq!(dos.len(), 0);
    }

    #[test]
    fn test_is_complete_incomplete() {
        let mut analysis = StrideAnalysis::new("web-api");
        analysis.add_threat(Threat {
            id: "T001".into(),
            category: StrideCategory::Spoofing,
            description: "test".into(),
            component: "web-api".into(),
            mitigation: "test".into(),
        });
        assert!(!analysis.is_complete());
    }

    #[test]
    fn test_is_complete_full() {
        let mut analysis = StrideAnalysis::new("web-api");
        let categories = [
            StrideCategory::Spoofing,
            StrideCategory::Tampering,
            StrideCategory::Repudiation,
            StrideCategory::InformationDisclosure,
            StrideCategory::DenialOfService,
            StrideCategory::ElevationOfPrivilege,
        ];
        for (i, cat) in categories.iter().enumerate() {
            analysis.add_threat(Threat {
                id: format!("T{:03}", i + 1),
                category: *cat,
                description: format!("Threat for {:?}", cat),
                component: "web-api".into(),
                mitigation: "Mitigation".into(),
            });
        }
        assert!(analysis.is_complete());
    }

    #[test]
    fn test_missing_categories() {
        let mut analysis = StrideAnalysis::new("web-api");
        analysis.add_threat(Threat {
            id: "T001".into(),
            category: StrideCategory::Spoofing,
            description: "test".into(),
            component: "web-api".into(),
            mitigation: "test".into(),
        });
        let missing = analysis.missing_categories();
        assert_eq!(missing.len(), 5);
        assert!(!missing.contains(&StrideCategory::Spoofing));
        assert!(missing.contains(&StrideCategory::Tampering));
    }

    #[test]
    fn test_build_auth_stride_analysis() {
        let analysis = build_auth_stride_analysis();
        assert_eq!(analysis.component, "web-auth");
        assert!(analysis.is_complete(), "Auth STRIDE analysis should cover all 6 categories");
        assert_eq!(analysis.threat_count(), 6);
    }
}
