//! # Authorization
//!
//! Authorization determines what an authenticated user is allowed to do.
//! This lesson covers RBAC (Role-Based Access Control), ABAC (Attribute-Based
//! Access Control), and middleware-based authorization patterns.
//!
//! ## Key Concepts
//! - RBAC: roles and permissions
//! - ABAC: attribute-based policies
//! - Permission checking middleware
//! - Resource-level authorization
//! - Policy composition

use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// 1. Permission Model
// ---------------------------------------------------------------------------

/// A permission represents a specific action on a resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

impl Permission {
    pub fn new(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
        }
    }

    pub fn matches(&self, resource: &str, action: &str) -> bool {
        (self.resource == "*" || self.resource == resource)
            && (self.action == "*" || self.action == action)
    }
}

// ---------------------------------------------------------------------------
// 2. RBAC (Role-Based Access Control)
// ---------------------------------------------------------------------------

/// Manages roles and their associated permissions.
#[derive(Debug)]
pub struct RbacManager {
    roles: HashMap<String, Role>,
    user_roles: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Clone)]
pub struct Role {
    pub name: String,
    pub permissions: HashSet<Permission>,
    pub parent_roles: Vec<String>,
}

impl Role {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            permissions: HashSet::new(),
            parent_roles: Vec::new(),
        }
    }

    pub fn with_permission(mut self, perm: Permission) -> Self {
        self.permissions.insert(perm);
        self
    }

    pub fn with_parent(mut self, parent: impl Into<String>) -> Self {
        self.parent_roles.push(parent.into());
        self
    }
}

impl RbacManager {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            user_roles: HashMap::new(),
        }
    }

    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    pub fn assign_role(&mut self, user_id: &str, role_name: &str) {
        self.user_roles
            .entry(user_id.into())
            .or_default()
            .insert(role_name.into());
    }

    pub fn revoke_role(&mut self, user_id: &str, role_name: &str) {
        if let Some(roles) = self.user_roles.get_mut(user_id) {
            roles.remove(role_name);
        }
    }

    /// Check if a user has a specific permission.
    pub fn has_permission(&self, user_id: &str, resource: &str, action: &str) -> bool {
        let user_roles = match self.user_roles.get(user_id) {
            Some(roles) => roles,
            None => return false,
        };

        for role_name in user_roles {
            if self.role_has_permission(role_name, resource, action, &mut HashSet::new()) {
                return true;
            }
        }

        false
    }

    fn role_has_permission(
        &self,
        role_name: &str,
        resource: &str,
        action: &str,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visited.contains(role_name) {
            return false; // prevent infinite recursion
        }
        visited.insert(role_name.into());

        let role = match self.roles.get(role_name) {
            Some(r) => r,
            None => return false,
        };

        // Check direct permissions
        if role.permissions.iter().any(|p| p.matches(resource, action)) {
            return true;
        }

        // Check parent roles
        for parent in &role.parent_roles {
            if self.role_has_permission(parent, resource, action, visited) {
                return true;
            }
        }

        false
    }

    pub fn get_user_roles(&self, user_id: &str) -> Vec<&str> {
        self.user_roles
            .get(user_id)
            .map(|roles| roles.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

// ---------------------------------------------------------------------------
// 3. ABAC (Attribute-Based Access Control)
// ---------------------------------------------------------------------------

/// A policy that evaluates based on attributes.
#[derive(Debug)]
pub struct Policy {
    pub name: String,
    pub condition: PolicyCondition,
    pub effect: PolicyEffect,
}

#[derive(Debug, Clone)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone)]
pub enum PolicyCondition {
    /// Always matches.
    Always,
    /// Never matches.
    Never,
    /// Attribute equals a value.
    AttrEquals(String, String),
    /// Attribute is in a set of values.
    AttrIn(String, Vec<String>),
    /// AND of two conditions.
    And(Box<PolicyCondition>, Box<PolicyCondition>),
    /// OR of two conditions.
    Or(Box<PolicyCondition>, Box<PolicyCondition>),
    /// NOT of a condition.
    Not(Box<PolicyCondition>),
}

impl PolicyCondition {
    pub fn evaluate(&self, context: &HashMap<String, String>) -> bool {
        match self {
            Self::Always => true,
            Self::Never => false,
            Self::AttrEquals(key, expected) => {
                context.get(key).map_or(false, |v| v == expected)
            }
            Self::AttrIn(key, values) => {
                context.get(key).map_or(false, |v| values.contains(v))
            }
            Self::And(a, b) => a.evaluate(context) && b.evaluate(context),
            Self::Or(a, b) => a.evaluate(context) || b.evaluate(context),
            Self::Not(inner) => !inner.evaluate(context),
        }
    }
}

/// Evaluates ABAC policies.
pub struct PolicyEngine {
    policies: Vec<Policy>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            policies: Vec::new(),
    }
    }

    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    /// Evaluate all policies. Returns Some(true) for allow, Some(false) for deny.
    pub fn evaluate(&self, context: &HashMap<String, String>) -> Option<bool> {
        // Last matching policy wins (explicit deny overrides allow)
        let mut result = None;
        for policy in &self.policies {
            if policy.condition.evaluate(context) {
                result = Some(matches!(policy.effect, PolicyEffect::Allow));
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// 4. Authorization Middleware
// ---------------------------------------------------------------------------

/// Checks authorization for a request.
pub struct Authorizer {
    rbac: RbacManager,
    engine: PolicyEngine,
}

impl Authorizer {
    pub fn new(rbac: RbacManager, engine: PolicyEngine) -> Self {
        Self { rbac, engine }
    }

    /// Check if a user is authorized for a request.
    pub fn authorize(
        &self,
        user_id: &str,
        resource: &str,
        action: &str,
        context: &HashMap<String, String>,
    ) -> AuthorizationResult {
        // Check RBAC first
        if !self.rbac.has_permission(user_id, resource, action) {
            return AuthorizationResult::Denied("insufficient permissions".into());
        }

        // Check ABAC policies
        match self.engine.evaluate(context) {
            Some(true) => AuthorizationResult::Allowed,
            Some(false) => AuthorizationResult::Denied("policy denied".into()),
            None => AuthorizationResult::Allowed, // no policy applies
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AuthorizationResult {
    Allowed,
    Denied(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_rbac() -> RbacManager {
        let mut rbac = RbacManager::new();

        let viewer = Role::new("viewer")
            .with_permission(Permission::new("posts", "read"))
            .with_permission(Permission::new("comments", "read"));

        let editor = Role::new("editor")
            .with_parent("viewer")
            .with_permission(Permission::new("posts", "write"))
            .with_permission(Permission::new("posts", "delete"));

        let admin = Role::new("admin")
            .with_parent("editor")
            .with_permission(Permission::new("*", "*"));

        rbac.add_role(viewer);
        rbac.add_role(editor);
        rbac.add_role(admin);

        rbac.assign_role("user-1", "viewer");
        rbac.assign_role("user-2", "editor");
        rbac.assign_role("user-3", "admin");

        rbac
    }

    #[test]
    fn test_permission_matching() {
        let perm = Permission::new("posts", "read");
        assert!(perm.matches("posts", "read"));
        assert!(!perm.matches("posts", "write"));
        assert!(!perm.matches("comments", "read"));
    }

    #[test]
    fn test_permission_wildcard() {
        let perm = Permission::new("*", "read");
        assert!(perm.matches("posts", "read"));
        assert!(perm.matches("comments", "read"));
        assert!(!perm.matches("posts", "write"));

        let perm = Permission::new("posts", "*");
        assert!(perm.matches("posts", "read"));
        assert!(perm.matches("posts", "write"));
    }

    #[test]
    fn test_rbac_viewer() {
        let rbac = setup_rbac();
        assert!(rbac.has_permission("user-1", "posts", "read"));
        assert!(!rbac.has_permission("user-1", "posts", "write"));
    }

    #[test]
    fn test_rbac_editor_inherits_viewer() {
        let rbac = setup_rbac();
        assert!(rbac.has_permission("user-2", "posts", "read")); // inherited
        assert!(rbac.has_permission("user-2", "posts", "write")); // direct
        assert!(!rbac.has_permission("user-2", "users", "delete")); // not granted
    }

    #[test]
    fn test_rbac_admin_all() {
        let rbac = setup_rbac();
        assert!(rbac.has_permission("user-3", "posts", "read"));
        assert!(rbac.has_permission("user-3", "anything", "everything"));
    }

    #[test]
    fn test_rbac_unknown_user() {
        let rbac = setup_rbac();
        assert!(!rbac.has_permission("unknown", "posts", "read"));
    }

    #[test]
    fn test_rbac_revoke() {
        let mut rbac = setup_rbac();
        rbac.revoke_role("user-1", "viewer");
        assert!(!rbac.has_permission("user-1", "posts", "read"));
    }

    #[test]
    fn test_rbac_get_user_roles() {
        let rbac = setup_rbac();
        let roles = rbac.get_user_roles("user-2");
        assert!(roles.contains(&"editor"));
    }

    #[test]
    fn test_abac_condition_equals() {
        let cond = PolicyCondition::AttrEquals("role".into(), "admin".into());
        let mut ctx = HashMap::new();
        ctx.insert("role".into(), "admin".into());
        assert!(cond.evaluate(&ctx));

        ctx.insert("role".into(), "user".into());
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_abac_condition_in() {
        let cond = PolicyCondition::AttrIn("country".into(), vec!["US".into(), "CA".into()]);
        let mut ctx = HashMap::new();
        ctx.insert("country".into(), "US".into());
        assert!(cond.evaluate(&ctx));

        ctx.insert("country".into(), "UK".into());
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_abac_condition_and() {
        let cond = PolicyCondition::And(
            Box::new(PolicyCondition::AttrEquals("role".into(), "admin".into())),
            Box::new(PolicyCondition::AttrEquals("active".into(), "true".into())),
        );
        let mut ctx = HashMap::new();
        ctx.insert("role".into(), "admin".into());
        ctx.insert("active".into(), "true".into());
        assert!(cond.evaluate(&ctx));

        ctx.insert("active".into(), "false".into());
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn test_abac_condition_not() {
        let cond = PolicyCondition::Not(Box::new(PolicyCondition::Always));
        assert!(!cond.evaluate(&HashMap::new()));
    }

    #[test]
    fn test_policy_engine() {
        let mut engine = PolicyEngine::new();
        engine.add_policy(Policy {
            name: "allow admin".into(),
            condition: PolicyCondition::AttrEquals("role".into(), "admin".into()),
            effect: PolicyEffect::Allow,
        });
        engine.add_policy(Policy {
            name: "deny all".into(),
            condition: PolicyCondition::Always,
            effect: PolicyEffect::Deny,
        });

        let mut ctx = HashMap::new();
        ctx.insert("role".into(), "admin".into());
        assert_eq!(engine.evaluate(&ctx), Some(false)); // deny wins (last match)

        ctx.insert("role".into(), "user".into());
        assert_eq!(engine.evaluate(&ctx), Some(false));
    }

    #[test]
    fn test_authorizer() {
        let rbac = setup_rbac();
        let engine = PolicyEngine::new();
        let authorizer = Authorizer::new(rbac, engine);

        let ctx = HashMap::new();
        assert_eq!(
            authorizer.authorize("user-1", "posts", "read", &ctx),
            AuthorizationResult::Allowed
        );
        assert!(matches!(
            authorizer.authorize("user-1", "posts", "write", &ctx),
            AuthorizationResult::Denied(_)
        ));
    }
}
