//! # Lesson 03: RBAC Authorization Middleware — Solution
//!
//! Role-permission mapping, endpoint guards.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permission(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone)]
pub struct EndpointPolicy {
    pub method: String,
    pub path: String,
    pub required_permission: Permission,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthzDecision {
    Allow,
    Deny(String),
}

#[derive(Debug, Clone)]
pub struct RbacPolicy {
    pub roles: Vec<Role>,
    pub endpoints: Vec<EndpointPolicy>,
}

impl RbacPolicy {
    pub fn new() -> Self {
        Self {
            roles: Vec::new(),
            endpoints: Vec::new(),
        }
    }
}

pub fn create_default_policy() -> RbacPolicy {
    RbacPolicy {
        roles: vec![
            Role {
                name: "admin".to_string(),
                permissions: vec![
                    Permission("users:read".to_string()),
                    Permission("users:write".to_string()),
                    Permission("audit:read".to_string()),
                    Permission("settings:write".to_string()),
                ],
            },
            Role {
                name: "editor".to_string(),
                permissions: vec![
                    Permission("content:read".to_string()),
                    Permission("content:write".to_string()),
                ],
            },
            Role {
                name: "viewer".to_string(),
                permissions: vec![
                    Permission("content:read".to_string()),
                ],
            },
        ],
        endpoints: vec![
            EndpointPolicy {
                method: "GET".to_string(),
                path: "/api/users".to_string(),
                required_permission: Permission("users:read".to_string()),
            },
            EndpointPolicy {
                method: "POST".to_string(),
                path: "/api/users".to_string(),
                required_permission: Permission("users:write".to_string()),
            },
            EndpointPolicy {
                method: "GET".to_string(),
                path: "/api/audit".to_string(),
                required_permission: Permission("audit:read".to_string()),
            },
            EndpointPolicy {
                method: "GET".to_string(),
                path: "/api/content".to_string(),
                required_permission: Permission("content:read".to_string()),
            },
            EndpointPolicy {
                method: "PUT".to_string(),
                path: "/api/content".to_string(),
                required_permission: Permission("content:write".to_string()),
            },
        ],
    }
}

pub fn has_permission(
    user_roles: &[String],
    required: &Permission,
    policy: &RbacPolicy,
) -> bool {
    for role_name in user_roles {
        if let Some(role) = policy.roles.iter().find(|r| &r.name == role_name) {
            // Check for wildcard permission
            if role.permissions.iter().any(|p| p.0 == "*") {
                return true;
            }
            // Check for exact permission match
            if role.permissions.contains(required) {
                return true;
            }
        }
    }
    false
}

pub fn find_endpoint_permission(
    method: &str,
    path: &str,
    policy: &RbacPolicy,
) -> Option<Permission> {
    policy
        .endpoints
        .iter()
        .find(|ep| ep.method == method && ep.path == path)
        .map(|ep| ep.required_permission.clone())
}

pub fn authorize(
    user_roles: &[String],
    method: &str,
    path: &str,
    policy: &RbacPolicy,
) -> AuthzDecision {
    // Find the required permission for this endpoint
    let required_permission = match find_endpoint_permission(method, path, policy) {
        Some(perm) => perm,
        None => {
            return AuthzDecision::Deny(format!(
                "No policy defined for {} {}", method, path
            ));
        }
    };

    // Check if the user has the required permission
    if has_permission(user_roles, &required_permission, policy) {
        AuthzDecision::Allow
    } else {
        AuthzDecision::Deny(format!(
            "User roles {:?} lack required permission: {}",
            user_roles, required_permission.0
        ))
    }
}

pub fn is_admin(user_roles: &[String], policy: &RbacPolicy) -> bool {
    for role_name in user_roles {
        if let Some(role) = policy.roles.iter().find(|r| &r.name == role_name) {
            if role.permissions.iter().any(|p| p.0 == "*") {
                return true;
            }
        }
    }
    false
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
        assert!(has_permission(&multi_roles(), &Permission("content:read".to_string()), &policy));
        assert!(has_permission(&multi_roles(), &Permission("content:write".to_string()), &policy));
        assert!(!has_permission(&multi_roles(), &Permission("users:write".to_string()), &policy));
    }
}
