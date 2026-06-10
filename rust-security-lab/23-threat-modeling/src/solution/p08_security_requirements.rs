//! # Lesson 08: Security Requirements from Threat Model (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMethod {
    AutomatedTest,
    PenTest,
    CodeReview,
    StaticAnalysis,
    ArchitectureReview,
    ComplianceAudit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementPriority {
    MustHave,
    ShouldHave,
    CouldHave,
    WontHave,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    pub id: String,
    pub threat_id: String,
    pub description: String,
    pub priority: RequirementPriority,
    pub verification: VerificationMethod,
    pub verification_detail: String,
    pub implemented: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirementSet {
    pub system_name: String,
    pub requirements: Vec<SecurityRequirement>,
}

impl SecurityRequirementSet {
    pub fn new(system_name: &str) -> Self {
        Self {
            system_name: system_name.to_string(),
            requirements: Vec::new(),
        }
    }

    pub fn add_requirement(&mut self, req: SecurityRequirement) {
        self.requirements.push(req);
    }

    pub fn for_threat(&self, threat_id: &str) -> Vec<&SecurityRequirement> {
        self.requirements.iter().filter(|r| r.threat_id == threat_id).collect()
    }

    pub fn must_have(&self) -> Vec<&SecurityRequirement> {
        self.requirements
            .iter()
            .filter(|r| r.priority == RequirementPriority::MustHave)
            .collect()
    }

    pub fn unimplemented(&self) -> Vec<&SecurityRequirement> {
        self.requirements.iter().filter(|r| !r.implemented).collect()
    }

    pub fn by_verification(&self, method: VerificationMethod) -> Vec<&SecurityRequirement> {
        self.requirements.iter().filter(|r| r.verification == method).collect()
    }

    pub fn mark_implemented(&mut self, id: &str) -> bool {
        if let Some(req) = self.requirements.iter_mut().find(|r| r.id == id) {
            req.implemented = true;
            true
        } else {
            false
        }
    }

    pub fn count(&self) -> usize {
        self.requirements.len()
    }

    pub fn all_must_have_implemented(&self) -> bool {
        self.requirements
            .iter()
            .filter(|r| r.priority == RequirementPriority::MustHave)
            .all(|r| r.implemented)
    }
}

pub fn build_webapp_requirements() -> SecurityRequirementSet {
    let mut rs = SecurityRequirementSet::new("web-application");

    // Requirements from T001: SQL Injection
    rs.add_requirement(SecurityRequirement {
        id: "SR001".into(),
        threat_id: "T001".into(),
        description: "All database queries MUST use parameterized statements or ORM".into(),
        priority: RequirementPriority::MustHave,
        verification: VerificationMethod::StaticAnalysis,
        verification_detail: "Run semgrep rule: no raw SQL string concatenation in src/".into(),
        implemented: false,
    });

    rs.add_requirement(SecurityRequirement {
        id: "SR002".into(),
        threat_id: "T001".into(),
        description: "All user input MUST be validated before processing".into(),
        priority: RequirementPriority::MustHave,
        verification: VerificationMethod::AutomatedTest,
        verification_detail: "Integration tests with sqlmap and XSS payloads on all endpoints".into(),
        implemented: false,
    });

    // Requirements from T002: Authentication Bypass
    rs.add_requirement(SecurityRequirement {
        id: "SR003".into(),
        threat_id: "T002".into(),
        description: "Admin accounts MUST require multi-factor authentication".into(),
        priority: RequirementPriority::MustHave,
        verification: VerificationMethod::PenTest,
        verification_detail: "Penetration test: attempt admin login with password only".into(),
        implemented: false,
    });

    rs.add_requirement(SecurityRequirement {
        id: "SR004".into(),
        threat_id: "T002".into(),
        description: "Authentication tokens MUST expire within 15 minutes".into(),
        priority: RequirementPriority::MustHave,
        verification: VerificationMethod::AutomatedTest,
        verification_detail: "Test: use token after 16 minutes; verify rejection".into(),
        implemented: false,
    });

    // Requirements from T003: Data Exfiltration
    rs.add_requirement(SecurityRequirement {
        id: "SR005".into(),
        threat_id: "T003".into(),
        description: "All sensitive data MUST be encrypted at rest using AES-256-GCM".into(),
        priority: RequirementPriority::MustHave,
        verification: VerificationMethod::ComplianceAudit,
        verification_detail: "Audit: verify encryption configuration and key rotation policy".into(),
        implemented: false,
    });

    rs.add_requirement(SecurityRequirement {
        id: "SR006".into(),
        threat_id: "T003".into(),
        description: "Bulk data export MUST be logged and require manager approval".into(),
        priority: RequirementPriority::ShouldHave,
        verification: VerificationMethod::CodeReview,
        verification_detail: "Review export endpoints for approval workflow and logging".into(),
        implemented: false,
    });

    // Requirements from T004: Cross-Site Scripting
    rs.add_requirement(SecurityRequirement {
        id: "SR007".into(),
        threat_id: "T004".into(),
        description: "All HTML output MUST be context-appropriately escaped".into(),
        priority: RequirementPriority::MustHave,
        verification: VerificationMethod::AutomatedTest,
        verification_detail: "Automated XSS scanner on all rendered pages".into(),
        implemented: false,
    });

    rs.add_requirement(SecurityRequirement {
        id: "SR008".into(),
        threat_id: "T004".into(),
        description: "Content-Security-Policy header MUST be set with strict directives".into(),
        priority: RequirementPriority::ShouldHave,
        verification: VerificationMethod::ArchitectureReview,
        verification_detail: "Review HTTP response headers for CSP configuration".into(),
        implemented: false,
    });

    rs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_requirements() -> SecurityRequirementSet {
        let mut rs = SecurityRequirementSet::new("test-system");

        rs.add_requirement(SecurityRequirement {
            id: "SR001".into(),
            threat_id: "T001".into(),
            description: "All SQL queries must use parameterized statements".into(),
            priority: RequirementPriority::MustHave,
            verification: VerificationMethod::StaticAnalysis,
            verification_detail: "Run sqlx::query! macro; reject raw string queries".into(),
            implemented: false,
        });
        rs.add_requirement(SecurityRequirement {
            id: "SR002".into(),
            threat_id: "T001".into(),
            description: "Input validation on all user-facing endpoints".into(),
            priority: RequirementPriority::MustHave,
            verification: VerificationMethod::AutomatedTest,
            verification_detail: "Fuzz test all input fields with sqlmap payloads".into(),
            implemented: true,
        });
        rs.add_requirement(SecurityRequirement {
            id: "SR003".into(),
            threat_id: "T002".into(),
            description: "MFA required for admin accounts".into(),
            priority: RequirementPriority::ShouldHave,
            verification: VerificationMethod::PenTest,
            verification_detail: "Attempt admin login with single factor; verify rejection".into(),
            implemented: false,
        });

        rs
    }

    #[test]
    fn test_requirement_set_creation() {
        let rs = SecurityRequirementSet::new("my-system");
        assert_eq!(rs.system_name, "my-system");
        assert_eq!(rs.count(), 0);
    }

    #[test]
    fn test_add_requirement() {
        let rs = make_test_requirements();
        assert_eq!(rs.count(), 3);
    }

    #[test]
    fn test_for_threat() {
        let rs = make_test_requirements();
        let t001 = rs.for_threat("T001");
        assert_eq!(t001.len(), 2);
        let t002 = rs.for_threat("T002");
        assert_eq!(t002.len(), 1);
    }

    #[test]
    fn test_must_have() {
        let rs = make_test_requirements();
        let must = rs.must_have();
        assert_eq!(must.len(), 2);
    }

    #[test]
    fn test_unimplemented() {
        let rs = make_test_requirements();
        let unimpl = rs.unimplemented();
        assert_eq!(unimpl.len(), 2);
    }

    #[test]
    fn test_by_verification() {
        let rs = make_test_requirements();
        let sa = rs.by_verification(VerificationMethod::StaticAnalysis);
        assert_eq!(sa.len(), 1);
        let at = rs.by_verification(VerificationMethod::AutomatedTest);
        assert_eq!(at.len(), 1);
    }

    #[test]
    fn test_mark_implemented() {
        let mut rs = make_test_requirements();
        assert!(rs.mark_implemented("SR001"));
        assert!(!rs.mark_implemented("NONEXISTENT"));
        assert_eq!(rs.unimplemented().len(), 1);
    }

    #[test]
    fn test_all_must_have_implemented() {
        let rs = make_test_requirements();
        assert!(!rs.all_must_have_implemented());

        let mut rs2 = make_test_requirements();
        rs2.mark_implemented("SR001");
        assert!(rs2.all_must_have_implemented());
    }

    #[test]
    fn test_build_webapp_requirements() {
        let rs = build_webapp_requirements();
        assert!(rs.count() >= 5, "Should have at least 5 requirements");
        assert!(rs.must_have().len() >= 2, "Should have at least 2 MustHave");
    }
}
