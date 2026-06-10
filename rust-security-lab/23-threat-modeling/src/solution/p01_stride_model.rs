//! # Lesson 01: STRIDE Threat Model (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

/// Represents the six STRIDE threat categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrideCategory {
    Spoofing,
    Tampering,
    Repudiation,
    InformationDisclosure,
    DenialOfService,
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
    pub fn new(component: &str) -> Self {
        Self {
            component: component.to_string(),
            threats: Vec::new(),
        }
    }

    pub fn add_threat(&mut self, threat: Threat) {
        self.threats.push(threat);
    }

    pub fn threats_by_category(&self, category: StrideCategory) -> Vec<&Threat> {
        self.threats
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        let all_categories = [
            StrideCategory::Spoofing,
            StrideCategory::Tampering,
            StrideCategory::Repudiation,
            StrideCategory::InformationDisclosure,
            StrideCategory::DenialOfService,
            StrideCategory::ElevationOfPrivilege,
        ];
        all_categories
            .iter()
            .all(|cat| self.threats.iter().any(|t| t.category == *cat))
    }

    pub fn missing_categories(&self) -> Vec<StrideCategory> {
        let all_categories = [
            StrideCategory::Spoofing,
            StrideCategory::Tampering,
            StrideCategory::Repudiation,
            StrideCategory::InformationDisclosure,
            StrideCategory::DenialOfService,
            StrideCategory::ElevationOfPrivilege,
        ];
        all_categories
            .iter()
            .filter(|cat| !self.threats.iter().any(|t| t.category == **cat))
            .copied()
            .collect()
    }

    pub fn threat_count(&self) -> usize {
        self.threats.len()
    }
}

pub fn build_auth_stride_analysis() -> StrideAnalysis {
    let mut analysis = StrideAnalysis::new("web-auth");

    analysis.add_threat(Threat {
        id: "AUTH-S001".into(),
        category: StrideCategory::Spoofing,
        description: "Attacker forges or steals authentication tokens to impersonate a user"
            .into(),
        component: "web-auth".into(),
        mitigation: "Use short-lived JWTs with strong signing keys and token rotation".into(),
    });

    analysis.add_threat(Threat {
        id: "AUTH-T001".into(),
        category: StrideCategory::Tampering,
        description: "Attacker modifies authentication request payload to bypass validation"
            .into(),
        component: "web-auth".into(),
        mitigation: "Sign all authentication payloads with HMAC; validate integrity on server"
            .into(),
    });

    analysis.add_threat(Threat {
        id: "AUTH-R001".into(),
        category: StrideCategory::Repudiation,
        description: "User denies performing an authenticated action; no audit trail exists".into(),
        component: "web-auth".into(),
        mitigation: "Log all authentication events with timestamps, IP, and user agent".into(),
    });

    analysis.add_threat(Threat {
        id: "AUTH-I001".into(),
        category: StrideCategory::InformationDisclosure,
        description: "Error messages reveal whether a username exists in the system".into(),
        component: "web-auth".into(),
        mitigation: "Return generic error messages; use constant-time comparison".into(),
    });

    analysis.add_threat(Threat {
        id: "AUTH-D001".into(),
        category: StrideCategory::DenialOfService,
        description: "Attacker floods login endpoint with failed attempts, locking out real users"
            .into(),
        component: "web-auth".into(),
        mitigation: "Rate-limit by IP with exponential backoff; use CAPTCHA after threshold"
            .into(),
    });

    analysis.add_threat(Threat {
        id: "AUTH-E001".into(),
        category: StrideCategory::ElevationOfPrivilege,
        description: "Attacker manipulates role claim in JWT to gain admin access".into(),
        component: "web-auth".into(),
        mitigation: "Validate role claims server-side against database; never trust client claims"
            .into(),
    });

    analysis
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
