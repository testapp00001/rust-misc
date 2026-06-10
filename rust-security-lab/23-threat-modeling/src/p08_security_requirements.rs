//! # Lesson 08: Security Requirements from Threat Model
//!
//! ## The Problem
//!
//! Threat models identify what can go wrong, but they don't tell developers what to build.
//! Without explicit security requirements, mitigations remain abstract ideas that never
//! get implemented or tested.
//!
//! ## The Solution: Derive Testable Security Requirements
//!
//! Each mitigation maps to one or more security requirements that developers can implement
//! and testers can verify:
//!
//! ```text
//! Threat: SQL Injection
//! Mitigation: Parameterized queries
//! Requirement: "All database queries MUST use parameterized statements"
//! Test: "Verify no raw string concatenation in SQL queries (static analysis)"
//!
//! Threat: Authentication Bypass
//! Mitigation: MFA
//! Requirement: "All admin logins MUST require a second factor"
//! Test: "Attempt admin login with only password; verify rejection"
//! ```
//!
//! Good security requirements follow the SMART principle:
//! - **Specific**: Exactly what must be done
//! - **Measurable**: Clear pass/fail criteria
//! - **Achievable**: Technically feasible
//! - **Relevant**: Tied to an identified threat
//! - **Time-bound**: Implemented in a specific release
//!
//! ## Attack Example: Unverifiable Requirement
//!
//! "The system should be secure" is not a testable requirement. "All API endpoints must
//! validate JWT signatures with RSA-256 and reject tokens with expired 'exp' claims" is.
//!
//! Defense: Every security requirement must have a verification method.

use serde::{Deserialize, Serialize};

/// The type of verification for a security requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// Automated test (unit, integration, e2e)
    AutomatedTest,
    /// Manual penetration test
    PenTest,
    /// Code review
    CodeReview,
    /// Static analysis tool
    StaticAnalysis,
    /// Architecture review
    ArchitectureReview,
    /// Compliance audit
    ComplianceAudit,
}

/// The priority of a security requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequirementPriority {
    /// Must be implemented before release
    MustHave,
    /// Should be implemented; release acceptable without it
    ShouldHave,
    /// Nice to have; can be deferred
    CouldHave,
    /// Future consideration
    WontHave,
}

/// A single security requirement derived from a threat.
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

/// A collection of security requirements for a system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirementSet {
    pub system_name: String,
    pub requirements: Vec<SecurityRequirement>,
}

impl SecurityRequirementSet {
    /// Create a new empty requirement set.
    pub fn new(system_name: &str) -> Self {
        todo!("Create empty requirement set")
    }

    /// Add a requirement.
    pub fn add_requirement(&mut self, req: SecurityRequirement) {
        todo!("Add requirement")
    }

    /// Return requirements for a specific threat.
    pub fn for_threat(&self, threat_id: &str) -> Vec<&SecurityRequirement> {
        todo!("Filter by threat_id")
    }

    /// Return only MustHave requirements.
    pub fn must_have(&self) -> Vec<&SecurityRequirement> {
        todo!("Filter by MustHave priority")
    }

    /// Return requirements that are not yet implemented.
    pub fn unimplemented(&self) -> Vec<&SecurityRequirement> {
        todo!("Filter where implemented is false")
    }

    /// Return requirements by verification method.
    pub fn by_verification(&self, method: VerificationMethod) -> Vec<&SecurityRequirement> {
        todo!("Filter by verification method")
    }

    /// Mark a requirement as implemented by ID. Returns true if found.
    pub fn mark_implemented(&mut self, id: &str) -> bool {
        todo!("Find requirement by ID and set implemented = true")
    }

    /// Count total requirements.
    pub fn count(&self) -> usize {
        todo!("Return count")
    }

    /// Check if all MustHave requirements are implemented.
    pub fn all_must_have_implemented(&self) -> bool {
        todo!("Check if all MustHave requirements have implemented = true")
    }
}

/// Build security requirements for a web application threat model.
pub fn build_webapp_requirements() -> SecurityRequirementSet {
    todo!("Build requirements covering at least 3 threats with various priorities and verification methods")
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
        assert_eq!(unimpl.len(), 2); // SR001 and SR003
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
        assert!(rs2.all_must_have_implemented()); // SR002 already implemented
    }

    #[test]
    fn test_build_webapp_requirements() {
        let rs = build_webapp_requirements();
        assert!(rs.count() >= 5, "Should have at least 5 requirements");
        assert!(rs.must_have().len() >= 2, "Should have at least 2 MustHave");
    }
}
