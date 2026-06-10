//! # Lesson 01: RBAC Basics -- Role-Based Access Control (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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
    role_permissions: HashMap<Role, HashSet<Permission>>,
    user_roles: HashMap<String, Role>,
}

impl RbacSystem {
    pub fn new() -> Self {
        Self {
            role_permissions: HashMap::new(),
            user_roles: HashMap::new(),
        }
    }

    pub fn define_role(&mut self, role: Role, permissions: HashSet<Permission>) {
        self.role_permissions.insert(role, permissions);
    }

    pub fn assign_role(&mut self, user: &str, role: Role) {
        self.user_roles.insert(user.to_string(), role);
    }

    pub fn get_user_role(&self, user: &str) -> Option<&Role> {
        self.user_roles.get(user)
    }

    pub fn has_permission(&self, user: &str, permission: &Permission) -> bool {
        match self.user_roles.get(user) {
            Some(role) => self
                .role_permissions
                .get(role)
                .map(|perms| perms.contains(permission))
                .unwrap_or(false),
            None => false,
        }
    }

    pub fn get_user_permissions(&self, user: &str) -> HashSet<Permission> {
        match self.user_roles.get(user) {
            Some(role) => self
                .role_permissions
                .get(role)
                .cloned()
                .unwrap_or_default(),
            None => HashSet::new(),
        }
    }

    pub fn users_with_role(&self, role: &Role) -> Vec<&str> {
        self.user_roles
            .iter()
            .filter(|(_, r)| *r == role)
            .map(|(name, _)| name.as_str())
            .collect()
    }

    pub fn revoke_role(&mut self, user: &str) {
        self.user_roles.remove(user);
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
