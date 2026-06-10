//! # Lesson 02: ABAC Basics -- Attribute-Based Access Control
//!
//! ## What is ABAC?
//!
//! Attribute-Based Access Control makes decisions based on ATTRIBUTES of:
//! - **Subject** (user): department, clearance level, job title
//! - **Resource**: classification, owner, creation date
//! - **Environment**: time of day, IP address, location
//! - **Action**: read, write, delete
//!
//! ## ABAC vs RBAC
//!
//! RBAC says: "Admins can do X." (role → permission)
//! ABAC says: "Users in the engineering department can read resources classified
//!            as 'internal' during business hours." (attributes → decision)
//!
//! ## 🔴 Attack: Attribute Spoofing
//!
//! If attributes are supplied by the client without server-side verification,
//! an attacker can claim any attributes. Always fetch attributes from a trusted source.

use std::collections::HashMap;

/// Attributes of the user making the request.
#[derive(Debug, Clone)]
pub struct SubjectAttributes {
    pub username: String,
    pub department: String,
    pub clearance_level: u32,
    pub is_active: bool,
}

/// Attributes of the resource being accessed.
#[derive(Debug, Clone)]
pub struct ResourceAttributes {
    pub resource_id: String,
    pub owner: String,
    pub department: String,
    pub classification_level: u32,
    pub resource_type: String,
}

/// Attributes of the environment at request time.
#[derive(Debug, Clone)]
pub struct EnvironmentAttributes {
    pub hour_of_day: u32,
    pub ip_address: String,
    pub is_internal_network: bool,
}

/// The action being requested.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Read,
    Write,
    Delete,
}

/// An access request combining all attribute types.
#[derive(Debug, Clone)]
pub struct AccessRequest {
    pub subject: SubjectAttributes,
    pub resource: ResourceAttributes,
    pub environment: EnvironmentAttributes,
    pub action: Action,
}

/// Result of a policy evaluation.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Allow,
    Deny,
    NotApplicable,
}

/// A policy rule that evaluates an access request.
pub trait Policy: Send + Sync {
    /// Returns a Decision for the given request.
    fn evaluate(&self, request: &AccessRequest) -> Decision;

    /// A human-readable name for this policy.
    fn name(&self) -> &str;
}

/// The ABAC engine that evaluates requests against a set of policies.
pub struct AbacEngine {
    policies: Vec<Box<dyn Policy>>,
}

impl AbacEngine {
    /// Create a new empty ABAC engine.
    pub fn new() -> Self {
        todo!("Create an AbacEngine with an empty policy list")
    }

    /// Add a policy to the engine.
    pub fn add_policy(&mut self, policy: Box<dyn Policy>) {
        todo!("Add the policy to the engine's list")
    }

    /// Evaluate an access request against all policies.
    ///
    /// Rules:
    /// - If ANY policy returns Deny, the final decision is Deny (deny-override).
    /// - If no policy returns Deny and at least one returns Allow, the decision is Allow.
    /// - If all policies return NotApplicable, the decision is Deny (default deny).
    pub fn evaluate(&self, request: &AccessRequest) -> Decision {
        todo!("Implement deny-override policy combining algorithm")
    }
}

/// Policy: Department matching -- user and resource must be in the same department.
pub struct DepartmentMatchPolicy;

impl Policy for DepartmentMatchPolicy {
    fn evaluate(&self, request: &AccessRequest) -> Decision {
        todo!("Allow if subject and resource departments match, Deny if they don't, NotApplicable otherwise")
    }

    fn name(&self) -> &str {
        "DepartmentMatch"
    }
}

/// Policy: Clearance level -- user clearance must be >= resource classification.
pub struct ClearancePolicy;

impl Policy for ClearancePolicy {
    fn evaluate(&self, request: &AccessRequest) -> Decision {
        todo!("Allow if subject clearance >= resource classification, Deny if not")
    }

    fn name(&self) -> &str {
        "ClearanceLevel"
    }
}

/// Policy: Business hours -- only allow write/delete during hours 9-17.
pub struct BusinessHoursPolicy;

impl Policy for BusinessHoursPolicy {
    fn evaluate(&self, request: &AccessRequest) -> Decision {
        todo!("For Write/Delete actions, Allow only if hour is 9-17, Deny otherwise. NotApplicable for Read.")
    }

    fn name(&self) -> &str {
        "BusinessHours"
    }
}

/// Policy: Active user -- only active users can access resources.
pub struct ActiveUserPolicy;

impl Policy for ActiveUserPolicy {
    fn evaluate(&self, request: &AccessRequest) -> Decision {
        todo!("Deny if user is not active, NotApplicable otherwise")
    }

    fn name(&self) -> &str {
        "ActiveUser"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(
        dept: &str,
        clearance: u32,
        active: bool,
        resource_dept: &str,
        classification: u32,
        action: Action,
        hour: u32,
    ) -> AccessRequest {
        AccessRequest {
            subject: SubjectAttributes {
                username: "testuser".to_string(),
                department: dept.to_string(),
                clearance_level: clearance,
                is_active: active,
            },
            resource: ResourceAttributes {
                resource_id: "res-1".to_string(),
                owner: "other".to_string(),
                department: resource_dept.to_string(),
                classification_level: classification,
                resource_type: "document".to_string(),
            },
            environment: EnvironmentAttributes {
                hour_of_day: hour,
                ip_address: "10.0.0.1".to_string(),
                is_internal_network: true,
            },
            action,
        }
    }

    fn full_engine() -> AbacEngine {
        let mut engine = AbacEngine::new();
        engine.add_policy(Box::new(ActiveUserPolicy));
        engine.add_policy(Box::new(DepartmentMatchPolicy));
        engine.add_policy(Box::new(ClearancePolicy));
        engine.add_policy(Box::new(BusinessHoursPolicy));
        engine
    }

    #[test]
    fn test_department_match_allows() {
        let engine = {
            let mut e = AbacEngine::new();
            e.add_policy(Box::new(DepartmentMatchPolicy));
            e
        };
        let req = make_request("engineering", 3, true, "engineering", 1, Action::Read, 10);
        assert_eq!(engine.evaluate(&req), Decision::Allow);
    }

    #[test]
    fn test_department_mismatch_denies() {
        let engine = {
            let mut e = AbacEngine::new();
            e.add_policy(Box::new(DepartmentMatchPolicy));
            e
        };
        let req = make_request("engineering", 3, true, "finance", 1, Action::Read, 10);
        assert_eq!(engine.evaluate(&req), Decision::Deny);
    }

    #[test]
    fn test_clearance_insufficient_denies() {
        let engine = {
            let mut e = AbacEngine::new();
            e.add_policy(Box::new(ClearancePolicy));
            e
        };
        let req = make_request("eng", 1, true, "eng", 5, Action::Read, 10);
        assert_eq!(engine.evaluate(&req), Decision::Deny);
    }

    #[test]
    fn test_clearance_sufficient_allows() {
        let engine = {
            let mut e = AbacEngine::new();
            e.add_policy(Box::new(ClearancePolicy));
            e
        };
        let req = make_request("eng", 5, true, "eng", 3, Action::Read, 10);
        assert_eq!(engine.evaluate(&req), Decision::Allow);
    }

    #[test]
    fn test_business_hours_write_allowed() {
        let engine = {
            let mut e = AbacEngine::new();
            e.add_policy(Box::new(BusinessHoursPolicy));
            e
        };
        let req = make_request("eng", 3, true, "eng", 1, Action::Write, 14);
        assert_eq!(engine.evaluate(&req), Decision::Allow);
    }

    #[test]
    fn test_business_hours_write_denied_at_night() {
        let engine = {
            let mut e = AbacEngine::new();
            e.add_policy(Box::new(BusinessHoursPolicy));
            e
        };
        let req = make_request("eng", 3, true, "eng", 1, Action::Write, 22);
        assert_eq!(engine.evaluate(&req), Decision::Deny);
    }

    #[test]
    fn test_inactive_user_denied() {
        let engine = {
            let mut e = AbacEngine::new();
            e.add_policy(Box::new(ActiveUserPolicy));
            e
        };
        let req = make_request("eng", 5, false, "eng", 1, Action::Read, 10);
        assert_eq!(engine.evaluate(&req), Decision::Deny);
    }

    #[test]
    fn test_deny_override_combining() {
        let engine = full_engine();
        // Department matches, clearance is fine, but user is inactive → Deny
        let req = make_request("eng", 5, false, "eng", 1, Action::Read, 10);
        assert_eq!(engine.evaluate(&req), Decision::Deny);
    }

    #[test]
    fn test_all_policies_pass() {
        let engine = full_engine();
        let req = make_request("eng", 5, true, "eng", 1, Action::Read, 10);
        assert_eq!(engine.evaluate(&req), Decision::Allow);
    }

    #[test]
    fn test_default_deny_no_policies() {
        let engine = AbacEngine::new();
        let req = make_request("eng", 5, true, "eng", 1, Action::Read, 10);
        assert_eq!(engine.evaluate(&req), Decision::Deny);
    }
}
