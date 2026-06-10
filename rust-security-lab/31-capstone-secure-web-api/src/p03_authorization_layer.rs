//! # Lesson 03: RBAC Authorization Middleware
//!
//! ## Role-Based Access Control (RBAC)
//!
//! RBAC maps users to roles, and roles to permissions. After authentication identifies
//! the user (Lesson 02), authorization determines whether they are allowed to perform
//! the requested action.
//!
//! ```text
//! User "alice"  -->  Roles: ["admin", "editor"]
//! Role "admin"  -->  Permissions: ["users:read", "users:write", "audit:read", "*"]
//! Role "editor" -->  Permissions: ["content:read", "content:write"]
//!
//! Request: GET /api/users
//! Required permission: "users:read"
//! Alice has "admin" role which grants "users:read"  --> ALLOWED
//! ```
//!
//! ## Principle of Least Privilege
//!
//! Every user should have only the minimum permissions needed for their job.
//! Default deny: if a permission is not explicitly granted, it is denied.
//!
//! ## Attack Context
//!
//! - **Privilege escalation**: Normal user accesses admin endpoints. Defense: RBAC check
//!   on every endpoint.
//! - **IDOR (Insecure Direct Object Reference)**: User A accesses User B's data. Defense:
//!   ownership check combined with RBAC.
//! - **Missing authorization**: Developer forgets to add authz check. Defense: middleware
//!   pattern ensures every request is checked.

use serde::{Deserialize, Serialize};

/// A permission represents a specific action on a resource.
/// Format: "resource:action" (e.g., "users:read", "content:write")
/// Wildcard "*" means all permissions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission(pub String);

/// A role is a named set of permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<Permission>,
}

/// Represents an endpoint's authorization requirements.
#[derive(Debug, Clone)]
pub struct EndpointPolicy {
    /// The HTTP method (GET, POST, PUT, DELETE)
    pub method: String,
    /// The URL path pattern (e.g., "/api/users", "/api/admin/settings")
    pub path: String,
    /// The permission required to access this endpoint
    pub required_permission: Permission,
}

/// Authorization decision.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthzDecision {
    /// Access granted
    Allow,
    /// Access denied with reason
    Deny(String),
}

/// The RBAC policy engine.
#[derive(Debug, Clone)]
pub struct RbacPolicy {
    /// All defined roles and their permissions
    pub roles: Vec<Role>,
    /// Endpoint access policies
    pub endpoints: Vec<EndpointPolicy>,
}

impl RbacPolicy {
    /// Create a new empty RBAC policy.
    pub fn new() -> Self {
        Self {
            roles: Vec::new(),
            endpoints: Vec::new(),
        }
    }
}

/// Exercise 1: Create an RBAC policy with predefined roles.
///
/// Create roles:
/// - "admin" with permissions: ["users:read", "users:write", "audit:read", "settings:write"]
/// - "editor" with permissions: ["content:read", "content:write"]
/// - "viewer" with permissions: ["content:read"]
///
/// Create endpoint policies:
/// - GET /api/users requires "users:read"
/// - POST /api/users requires "users:write"
/// - GET /api/audit requires "audit:read"
/// - GET /api/content requires "content:read"
/// - PUT /api/content requires "content:write"
pub fn create_default_policy() -> RbacPolicy {
    todo!("Create default RBAC policy with roles and endpoint policies")
}

/// Exercise 2: Check if a user's roles grant a specific permission.
///
/// Given a list of role names and the RBAC policy, check if any of the user's
/// roles include the required permission.
///
/// A wildcard permission "*" in a role grants all permissions.
///
/// Return true if the user has the permission, false otherwise.
pub fn has_permission(
    user_roles: &[String],
    required: &Permission,
    policy: &RbacPolicy,
) -> bool {
    todo!("Check if user roles grant the required permission")
}

/// Exercise 3: Find the required permission for an endpoint.
///
/// Match the given HTTP method and path against the endpoint policies.
/// Return the required permission if found, None if no policy matches.
pub fn find_endpoint_permission(
    method: &str,
    path: &str,
    policy: &RbacPolicy,
) -> Option<Permission> {
    todo!("Find the required permission for an endpoint")
}

/// Exercise 4: Authorize a request.
///
/// Given the user's roles, the HTTP method, path, and the RBAC policy:
/// 1. Find the required permission for the endpoint
/// 2. If no policy matches, deny by default (fail-closed)
/// 3. Check if the user's roles grant the required permission
/// 4. Return Allow or Deny with a reason
pub fn authorize(
    user_roles: &[String],
    method: &str,
    path: &str,
    policy: &RbacPolicy,
) -> AuthzDecision {
    todo!("Authorize a request using RBAC")
}

/// Exercise 5: Check if a user has a wildcard (admin) permission.
///
/// Return true if any of the user's roles contains the wildcard permission "*".
pub fn is_admin(user_roles: &[String], policy: &RbacPolicy) -> bool {
    todo!("Check if user has admin (wildcard) permission")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin_roles() -> Vec<String> {
        vec!["admin".to_string()]
    }

    fn editor_roles() -> Vec<String> {
        vec!["editor".to_string()]
    }

    fn viewer_roles() -> Vec<String> {
        vec!["viewer".to_string()]
    }

    fn multi_roles() -> Vec<String> {
        vec!["editor".to_string(), "viewer".to_string()]
    }

    #[test]
    fn test_create_default_policy_roles() {
        let policy = create_default_policy();
        assert_eq!(policy.roles.len(), 3);

        let admin = policy.roles.iter().find(|r| r.name == "admin").unwrap();
        assert!(admin.permissions.contains(&Permission("users:read".to_string())));
        assert!(admin.permissions.contains(&Permission("audit:read".to_string())));
    }

    #[test]
    fn test_create_default_policy_endpoints() {
        let policy = create_default_policy();
        assert_eq!(policy.endpoints.len(), 5);
    }

    #[test]
    fn test_admin_has_all_permissions() {
        let policy = create_default_policy();
        assert!(has_permission(&admin_roles(), &Permission("users:read".to_string()), &policy));
        assert!(has_permission(&admin_roles(), &Permission("users:write".to_string()), &policy));
        assert!(has_permission(&admin_roles(), &Permission("audit:read".to_string()), &policy));
    }

    #[test]
    fn test_editor_has_limited_permissions() {
        let policy = create_default_policy();
        assert!(has_permission(&editor_roles(), &Permission("content:read".to_string()), &policy));
        assert!(has_permission(&editor_roles(), &Permission("content:write".to_string()), &policy));
        assert!(!has_permission(&editor_roles(), &Permission("users:write".to_string()), &policy));
    }

    #[test]
    fn test_viewer_read_only() {
        let policy = create_default_policy();
        assert!(has_permission(&viewer_roles(), &Permission("content:read".to_string()), &policy));
        assert!(!has_permission(&viewer_roles(), &Permission("content:write".to_string()), &policy));
    }

    #[test]
    fn test_find_endpoint_permission() {
        let policy = create_default_policy();
        let perm = find_endpoint_permission("GET", "/api/users", &policy);
        assert_eq!(perm, Some(Permission("users:read".to_string())));

        let perm = find_endpoint_permission("POST", "/api/users", &policy);
        assert_eq!(perm, Some(Permission("users:write".to_string())));
    }

    #[test]
    fn test_find_endpoint_permission_unknown() {
        let policy = create_default_policy();
        let perm = find_endpoint_permission("GET", "/api/unknown", &policy);
        assert_eq!(perm, None);
    }

    #[test]
    fn test_authorize_allowed() {
        let policy = create_default_policy();
        let decision = authorize(&admin_roles(), "GET", "/api/users", &policy);
        assert_eq!(decision, AuthzDecision::Allow);
    }

    #[test]
    fn test_authorize_denied() {
        let policy = create_default_policy();
        let decision = authorize(&viewer_roles(), "POST", "/api/users", &policy);
        assert!(matches!(decision, AuthzDecision::Deny(_)));
    }

    #[test]
    fn test_authorize_unknown_endpoint_denied() {
        let policy = create_default_policy();
        let decision = authorize(&admin_roles(), "GET", "/api/unknown", &policy);
        assert!(matches!(decision, AuthzDecision::Deny(_)));
    }

    #[test]
    fn test_multi_role_permissions() {
        let policy = create_default_policy();
        // editor+viewer should have content:read (from either role)
        assert!(has_permission(&multi_roles(), &Permission("content:read".to_string()), &policy));
        // editor+viewer should have content:write (from editor)
        assert!(has_permission(&multi_roles(), &Permission("content:write".to_string()), &policy));
        // But not users:write
        assert!(!has_permission(&multi_roles(), &Permission("users:write".to_string()), &policy));
    }
}
