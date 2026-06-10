//! # Lesson 05: Principle of Least Privilege
//!
//! ## What is Least Privilege?
//!
//! Every user, service, and process should operate with the MINIMUM set of privileges
//! needed to perform its function. Start with NOTHING, then grant only what is explicitly
//! required.
//!
//! ## Why Least Privilege?
//!
//! 1. **Limits blast radius**: If an account is compromised, the attacker gets minimal access
//! 2. **Reduces accidents**: A developer can't accidentally delete production data if they lack delete permission
//! 3. **Audit clarity**: You can see exactly what each account needs
//!
//! ## 🔴 Attack: Over-Privileged Service Accounts
//!
//! Service accounts are often granted admin "for convenience." If the service is compromised,
//! the attacker inherits full admin. Always scope service accounts to their specific needs.
//!
//! ## Permission Grant Patterns
//!
//! - **Whitelist (default deny)**: Start with no permissions, explicitly grant needed ones
//! - **Time-bound grants**: Permissions expire after a duration
//! - **Approval workflow**: Sensitive permissions require multi-party approval

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// A permission that can be granted.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    ManageUsers,
    ViewAuditLogs,
    ConfigureSystem,
}

/// A permission grant with optional expiry.
#[derive(Debug, Clone)]
pub struct PermissionGrant {
    pub permission: Permission,
    /// If Some, the grant expires at this Unix timestamp.
    /// If None, the grant is permanent.
    pub expires_at: Option<u64>,
}

/// A user identity with requested permissions.
#[derive(Debug, Clone)]
pub struct AccessRequest {
    pub user: String,
    pub requested_permission: Permission,
    pub justification: String,
}

/// Result of a permission request.
#[derive(Debug, Clone, PartialEq)]
pub enum GrantResult {
    /// Permission was granted.
    Granted,
    /// Permission was denied -- already had it or request is invalid.
    Denied(String),
    /// Permission needs approval from another party.
    NeedsApproval,
}

/// The least-privilege access manager.
pub struct LeastPrivilegeManager {
    /// Maps user to their active permission grants.
    grants: HashMap<String, Vec<PermissionGrant>>,
    /// The default TTL (in seconds) for time-bound grants.
    default_ttl: u64,
    /// Permissions that require approval before being granted.
    requires_approval: HashSet<Permission>,
}

impl LeastPrivilegeManager {
    /// Create a new manager with the given default TTL and approval requirements.
    pub fn new(default_ttl: u64, requires_approval: HashSet<Permission>) -> Self {
        todo!("Initialize the manager")
    }

    /// Grant a permission to a user with the default TTL.
    ///
    /// - If the user already has this permission (and it hasn't expired), return Denied.
    /// - If the permission requires approval, return NeedsApproval.
    /// - Otherwise, grant the permission and return Granted.
    pub fn request_permission(
        &mut self,
        request: &AccessRequest,
    ) -> GrantResult {
        todo!("Check existing grants, approval requirements, then grant if allowed")
    }

    /// Grant a permission directly (bypassing approval requirements).
    ///
    /// Used by approvers to grant permissions that need approval.
    /// Uses the default TTL.
    pub fn approve_permission(&mut self, user: &str, permission: Permission) {
        todo!("Grant the permission with default TTL")
    }

    /// Grant a permanent (no expiry) permission to a user.
    pub fn grant_permanent(&mut self, user: &str, permission: Permission) {
        todo!("Grant a permanent permission (expires_at = None)")
    }

    /// Revoke a specific permission from a user.
    pub fn revoke_permission(&mut self, user: &str, permission: &Permission) {
        todo!("Remove the grant for this permission")
    }

    /// Check if a user currently has a specific permission (considering expiry).
    ///
    /// A permission is active if:
    /// 1. It has been granted, AND
    /// 2. It has not expired (or has no expiry).
    pub fn has_permission(&self, user: &str, permission: &Permission) -> bool {
        todo!("Check grants and expiry")
    }

    /// Get the current Unix timestamp.
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// List all active (non-expired) permissions for a user.
    pub fn active_permissions(&self, user: &str) -> HashSet<Permission> {
        todo!("Collect all non-expired grants for the user")
    }

    /// Clean up expired grants for all users.
    ///
    /// This should be called periodically to prevent memory leaks from expired grants.
    pub fn cleanup_expired(&mut self) {
        todo!("Remove all expired grants from all users")
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
        // Manually create an expired grant
        mgr.grants.insert(
            "bob".to_string(),
            vec![PermissionGrant {
                permission: Permission::Read,
                expires_at: Some(1), // Already expired
            }],
        );
        assert!(!mgr.has_permission("bob", &Permission::Read)); // Expired
        mgr.cleanup_expired();
        assert!(mgr.grants.get("bob").unwrap().is_empty());
    }
}
