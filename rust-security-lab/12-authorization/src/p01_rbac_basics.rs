//! # Lesson 01: RBAC Basics -- Role-Based Access Control
//!
//! ## What is RBAC?
//!
//! Role-Based Access Control assigns permissions to ROLES, then assigns roles to USERS.
//! Instead of saying "Alice can read, write, and delete," you say "Alice is an Admin"
//! and Admins can read, write, and delete.
//!
//! ## The RBAC Model
//!
//! ```
//! Users  →  Roles  →  Permissions
//! Alice  →  Admin   →  [Read, Write, Delete]
//! Bob    →  Editor  →  [Read, Write]
//! Carol  →  Viewer  →  [Read]
//! ```
//!
//! ## Why RBAC?
//!
//! 1. **Simplified management**: Change a role's permissions → affects all users with that role
//! 2. **Principle of least privilege**: Users get only what their role allows
//! 3. **Auditability**: Easy to see who has what access via role assignments
//!
//! ## 🔴 Attack: Role Manipulation
//!
//! If role assignments are stored client-side (cookies, JWT claims without signature
//! verification), an attacker can modify their role to "admin." Always validate roles
//! server-side.

use std::collections::{HashMap, HashSet};

/// Represents a permission that can be granted to a role.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    ManageUsers,
    ViewAuditLogs,
    ConfigureSystem,
}

/// Represents a role in the RBAC system.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Role {
    Viewer,
    Editor,
    Admin,
    SuperAdmin,
}

/// The RBAC system that manages users, roles, and permissions.
pub struct RbacSystem {
    /// Maps each role to its set of permissions.
    role_permissions: HashMap<Role, HashSet<Permission>>,
    /// Maps each user (by name) to their assigned role.
    user_roles: HashMap<String, Role>,
}

impl RbacSystem {
    /// Create a new empty RBAC system.
    pub fn new() -> Self {
        todo!("Create a new RbacSystem with empty role_permissions and user_roles")
    }

    /// Define what permissions a role has.
    ///
    /// This should set (replace) the permissions for the given role.
    pub fn define_role(&mut self, role: Role, permissions: HashSet<Permission>) {
        todo!("Store the permissions for the given role")
    }

    /// Assign a role to a user.
    ///
    /// If the user already has a role, replace it.
    pub fn assign_role(&mut self, user: &str, role: Role) {
        todo!("Assign the role to the user")
    }

    /// Get the role assigned to a user.
    ///
    /// Returns None if the user has no assigned role.
    pub fn get_user_role(&self, user: &str) -> Option<&Role> {
        todo!("Look up the user's role")
    }

    /// Check if a user has a specific permission.
    ///
    /// Look up the user's role, then check if that role has the permission.
    /// Returns false if the user has no role or the role lacks the permission.
    pub fn has_permission(&self, user: &str, permission: &Permission) -> bool {
        todo!("Check if user's role includes the given permission")
    }

    /// Get all permissions for a user.
    ///
    /// Returns an empty set if the user has no role.
    pub fn get_user_permissions(&self, user: &str) -> HashSet<Permission> {
        todo!("Return all permissions for the user's role")
    }

    /// List all users that have a specific role.
    pub fn users_with_role(&self, role: &Role) -> Vec<&str> {
        todo!("Find all users assigned to the given role")
    }

    /// Revoke a user's role (remove them from the system).
    pub fn revoke_role(&mut self, user: &str) {
        todo!("Remove the user's role assignment")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_system() -> RbacSystem {
        let mut rbac = RbacSystem::new();

        let mut viewer_perms = HashSet::new();
        viewer_perms.insert(Permission::Read);
        rbac.define_role(Role::Viewer, viewer_perms);

        let mut editor_perms = HashSet::new();
        editor_perms.insert(Permission::Read);
        editor_perms.insert(Permission::Write);
        rbac.define_role(Role::Editor, editor_perms);

        let mut admin_perms = HashSet::new();
        admin_perms.insert(Permission::Read);
        admin_perms.insert(Permission::Write);
        admin_perms.insert(Permission::Delete);
        admin_perms.insert(Permission::ManageUsers);
        admin_perms.insert(Permission::ViewAuditLogs);
        admin_perms.insert(Permission::ConfigureSystem);
        rbac.define_role(Role::Admin, admin_perms);

        rbac.assign_role("alice", Role::Admin);
        rbac.assign_role("bob", Role::Editor);
        rbac.assign_role("carol", Role::Viewer);

        rbac
    }

    #[test]
    fn test_role_assignment() {
        let rbac = setup_system();
        assert_eq!(rbac.get_user_role("alice"), Some(&Role::Admin));
        assert_eq!(rbac.get_user_role("bob"), Some(&Role::Editor));
        assert_eq!(rbac.get_user_role("carol"), Some(&Role::Viewer));
        assert_eq!(rbac.get_user_role("unknown"), None);
    }

    #[test]
    fn test_admin_has_all_permissions() {
        let rbac = setup_system();
        assert!(rbac.has_permission("alice", &Permission::Read));
        assert!(rbac.has_permission("alice", &Permission::Write));
        assert!(rbac.has_permission("alice", &Permission::Delete));
        assert!(rbac.has_permission("alice", &Permission::ManageUsers));
        assert!(rbac.has_permission("alice", &Permission::ViewAuditLogs));
        assert!(rbac.has_permission("alice", &Permission::ConfigureSystem));
    }

    #[test]
    fn test_editor_limited_permissions() {
        let rbac = setup_system();
        assert!(rbac.has_permission("bob", &Permission::Read));
        assert!(rbac.has_permission("bob", &Permission::Write));
        assert!(!rbac.has_permission("bob", &Permission::Delete));
        assert!(!rbac.has_permission("bob", &Permission::ManageUsers));
    }

    #[test]
    fn test_viewer_read_only() {
        let rbac = setup_system();
        assert!(rbac.has_permission("carol", &Permission::Read));
        assert!(!rbac.has_permission("carol", &Permission::Write));
        assert!(!rbac.has_permission("carol", &Permission::Delete));
    }

    #[test]
    fn test_unknown_user_denied() {
        let rbac = setup_system();
        assert!(!rbac.has_permission("eve", &Permission::Read));
    }

    #[test]
    fn test_users_with_role() {
        let rbac = setup_system();
        let admins = rbac.users_with_role(&Role::Admin);
        assert!(admins.contains(&"alice"));
        assert_eq!(admins.len(), 1);
    }

    #[test]
    fn test_role_reassignment() {
        let mut rbac = setup_system();
        rbac.assign_role("bob", Role::Admin);
        assert_eq!(rbac.get_user_role("bob"), Some(&Role::Admin));
        assert!(rbac.has_permission("bob", &Permission::Delete));
    }

    #[test]
    fn test_revoke_role() {
        let mut rbac = setup_system();
        rbac.revoke_role("alice");
        assert_eq!(rbac.get_user_role("alice"), None);
        assert!(!rbac.has_permission("alice", &Permission::Read));
    }
}
