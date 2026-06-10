//! # Lesson 02: ABAC Basics -- Attribute-Based Access Control (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

// HashMap available if needed for attribute lookups

#[derive(Debug, Clone)]
pub struct SubjectAttributes {
    pub username: String,
    pub department: String,
    pub clearance_level: u32,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct ResourceAttributes {
    pub resource_id: String,
    pub owner: String,
    pub department: String,
    pub classification_level: u32,
    pub resource_type: String,
}

#[derive(Debug, Clone)]
pub struct EnvironmentAttributes {
    pub hour_of_day: u32,
    pub ip_address: String,
    pub is_internal_network: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Read,
    Write,
    Delete,
}

#[derive(Debug, Clone)]
pub struct AccessRequest {
    pub subject: SubjectAttributes,
    pub resource: ResourceAttributes,
    pub environment: EnvironmentAttributes,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Allow,
    Deny,
    NotApplicable,
}

pub trait Policy: Send + Sync {
    fn evaluate(&self, request: &AccessRequest) -> Decision;
    fn name(&self) -> &str;
}

pub struct AbacEngine {
    policies: Vec<Box<dyn Policy>>,
}

impl AbacEngine {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
        }
    }

    pub fn add_policy(&mut self, policy: Box<dyn Policy>) {
        self.policies.push(policy);
    }

    /// Deny-override combining algorithm:
    /// - Any Deny → final Deny
    /// - No Deny + at least one Allow → Allow
    /// - All NotApplicable → Deny (default deny)
    pub fn evaluate(&self, request: &AccessRequest) -> Decision {
        let mut has_allow = false;

        for policy in &self.policies {
            match policy.evaluate(request) {
                Decision::Deny => return Decision::Deny,
                Decision::Allow => has_allow = true,
                Decision::NotApplicable => {}
            }
        }

        if has_allow {
            Decision::Allow
        } else {
            Decision::Deny
        }
    }
}

pub struct DepartmentMatchPolicy;

impl Policy for DepartmentMatchPolicy {
    fn evaluate(&self, request: &AccessRequest) -> Decision {
        if request.subject.department == request.resource.department {
            Decision::Allow
        } else {
            Decision::Deny
        }
    }

    fn name(&self) -> &str {
        "DepartmentMatch"
    }
}

pub struct ClearancePolicy;

impl Policy for ClearancePolicy {
    fn evaluate(&self, request: &AccessRequest) -> Decision {
        if request.subject.clearance_level >= request.resource.classification_level {
            Decision::Allow
        } else {
            Decision::Deny
        }
    }

    fn name(&self) -> &str {
        "ClearanceLevel"
    }
}

pub struct BusinessHoursPolicy;

impl Policy for BusinessHoursPolicy {
    fn evaluate(&self, request: &AccessRequest) -> Decision {
        match request.action {
            Action::Write | Action::Delete => {
                if request.environment.hour_of_day >= 9 && request.environment.hour_of_day < 17 {
                    Decision::Allow
                } else {
                    Decision::Deny
                }
            }
            Action::Read => Decision::NotApplicable,
        }
    }

    fn name(&self) -> &str {
        "BusinessHours"
    }
}

pub struct ActiveUserPolicy;

impl Policy for ActiveUserPolicy {
    fn evaluate(&self, request: &AccessRequest) -> Decision {
        if request.subject.is_active {
            Decision::NotApplicable
        } else {
            Decision::Deny
        }
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
