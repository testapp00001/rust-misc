//! # Lesson 05: Principle of Least Privilege (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    ManageUsers,
    ViewAuditLogs,
    ConfigureSystem,
}

#[derive(Debug, Clone)]
pub struct PermissionGrant {
    pub permission: Permission,
    pub expires_at: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct AccessRequest {
    pub user: String,
    pub requested_permission: Permission,
    pub justification: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GrantResult {
    Granted,
    Denied(String),
    NeedsApproval,
}

pub struct LeastPrivilegeManager {
    grants: HashMap<String, Vec<PermissionGrant>>,
    default_ttl: u64,
    requires_approval: HashSet<Permission>,
}

impl LeastPrivilegeManager {
    pub fn new(default_ttl: u64, requires_approval: HashSet<Permission>) -> Self {
        Self {
            grants: HashMap::new(),
            default_ttl,
            requires_approval,
        }
    }

    pub fn request_permission(&mut self, request: &AccessRequest) -> GrantResult {
        // Check if permission requires approval
        if self.requires_approval.contains(&request.requested_permission) {
            return GrantResult::NeedsApproval;
        }

        // Check if user already has this permission (non-expired)
        if self.has_permission(&request.user, &request.requested_permission) {
            return GrantResult::Denied("User already has this permission".to_string());
        }

        // Grant with default TTL
        let grant = PermissionGrant {
            permission: request.requested_permission.clone(),
            expires_at: Some(Self::current_timestamp() + self.default_ttl),
        };
        self.grants
            .entry(request.user.clone())
            .or_default()
            .push(grant);

        GrantResult::Granted
    }

    pub fn approve_permission(&mut self, user: &str, permission: Permission) {
        let grant = PermissionGrant {
            permission,
            expires_at: Some(Self::current_timestamp() + self.default_ttl),
        };
        self.grants
            .entry(user.to_string())
            .or_default()
            .push(grant);
    }

    pub fn grant_permanent(&mut self, user: &str, permission: Permission) {
        let grant = PermissionGrant {
            permission,
            expires_at: None,
        };
        self.grants
            .entry(user.to_string())
            .or_default()
            .push(grant);
    }

    pub fn revoke_permission(&mut self, user: &str, permission: &Permission) {
        if let Some(grants) = self.grants.get_mut(user) {
            grants.retain(|g| &g.permission != permission);
        }
    }

    pub fn has_permission(&self, user: &str, permission: &Permission) -> bool {
        let now = Self::current_timestamp();
        self.grants
            .get(user)
            .map(|grants| {
                grants.iter().any(|g| {
                    g.permission == *permission
                        && g.expires_at.map_or(true, |exp| now <= exp)
                })
            })
            .unwrap_or(false)
    }

    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn active_permissions(&self, user: &str) -> HashSet<Permission> {
        let now = Self::current_timestamp();
        self.grants
            .get(user)
            .map(|grants| {
                grants
                    .iter()
                    .filter(|g| g.expires_at.map_or(true, |exp| now <= exp))
                    .map(|g| g.permission.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn cleanup_expired(&mut self) {
        let now = Self::current_timestamp();
        for grants in self.grants.values_mut() {
            grants.retain(|g| g.expires_at.map_or(true, |exp| now <= exp));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> LeastPrivilegeManager {
        let mut requires_approval = HashSet::new();
        requires_approval.insert(Permission::Delete);
        requires_approval.insert(Permission::ManageUsers);
        requires_approval.insert(Permission::ConfigureSystem);
        LeastPrivilegeManager::new(3600, requires_approval)
    }

    #[test]
    fn test_basic_grant() {
        let mut mgr = setup();
        let result = mgr.request_permission(&AccessRequest {
            user: "alice".to_string(),
            requested_permission: Permission::Read,
            justification: "Need to read project docs".to_string(),
        });
        assert_eq!(result, GrantResult::Granted);
        assert!(mgr.has_permission("alice", &Permission::Read));
    }

    #[test]
    fn test_deny_duplicate_grant() {
        let mut mgr = setup();
        mgr.request_permission(&AccessRequest {
            user: "alice".to_string(),
            requested_permission: Permission::Read,
            justification: "Need read".to_string(),
        });
        let result = mgr.request_permission(&AccessRequest {
            user: "alice".to_string(),
            requested_permission: Permission::Read,
            justification: "Need read again".to_string(),
        });
        assert!(matches!(result, GrantResult::Denied(_)));
    }

    #[test]
    fn test_needs_approval_for_sensitive() {
        let mut mgr = setup();
        let result = mgr.request_permission(&AccessRequest {
            user: "alice".to_string(),
            requested_permission: Permission::Delete,
            justification: "Need to clean up old files".to_string(),
        });
        assert_eq!(result, GrantResult::NeedsApproval);
        assert!(!mgr.has_permission("alice", &Permission::Delete));
    }

    #[test]
    fn test_approve_permission() {
        let mut mgr = setup();
        mgr.approve_permission("alice", Permission::Delete);
        assert!(mgr.has_permission("alice", &Permission::Delete));
    }

    #[test]
    fn test_revoke_permission() {
        let mut mgr = setup();
        mgr.approve_permission("bob", Permission::Write);
        assert!(mgr.has_permission("bob", &Permission::Write));
        mgr.revoke_permission("bob", &Permission::Write);
        assert!(!mgr.has_permission("bob", &Permission::Write));
    }

    #[test]
    fn test_unknown_user_no_permission() {
        let mgr = setup();
        assert!(!mgr.has_permission("unknown", &Permission::Read));
    }

    #[test]
    fn test_active_permissions_list() {
        let mut mgr = setup();
        mgr.approve_permission("alice", Permission::Read);
        mgr.approve_permission("alice", Permission::Write);
        let perms = mgr.active_permissions("alice");
        assert!(perms.contains(&Permission::Read));
        assert!(perms.contains(&Permission::Write));
        assert!(!perms.contains(&Permission::Delete));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut mgr = setup();
        mgr.grants.insert(
            "bob".to_string(),
            vec![PermissionGrant {
                permission: Permission::Read,
                expires_at: Some(1),
            }],
        );
        assert!(!mgr.has_permission("bob", &Permission::Read));
        mgr.cleanup_expired();
        assert!(mgr.grants.get("bob").unwrap().is_empty());
    }
}
