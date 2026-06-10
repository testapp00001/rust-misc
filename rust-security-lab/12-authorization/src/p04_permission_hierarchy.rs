//! # Lesson 04: Permission Hierarchy
//!
//! ## What is a Permission Hierarchy?
//!
//! Permissions can form a hierarchy where higher-level permissions implicitly include
//! lower-level ones. For example:
//!
//! ```
//! Admin > Editor > Viewer
//!   ↓        ↓       ↓
//! delete   write   read
//! manage   (all of  (base
//! config    viewer)  perm)
//! ```
//!
//! ## Why Hierarchies?
//!
//! 1. **Reduces duplication**: Admin doesn't need to list every single permission
//! 2. **Simplifies management**: Promote/demote users by changing their level
//! 3. **Natural modeling**: Maps to real-world org structures
//!
//! ## 🔴 Attack: Hierarchy Manipulation
//!
//! If a system checks for "is admin?" without properly modeling the hierarchy, a user
//! at level N might access resources requiring level N+1 by exploiting gaps in the
//! hierarchy chain.

use std::collections::{HashMap, HashSet};

/// Permission levels in the hierarchy. Higher values = more access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// Can only read/view content.
    Viewer = 0,
    /// Can read and write content.
    Editor = 1,
    /// Can read, write, delete, and manage users.
    Admin = 2,
    /// Full system access including configuration.
    SuperAdmin = 3,
}

/// Individual fine-grained permissions.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Permission {
    Read,
    Write,
    Delete,
    ManageUsers,
    ViewAuditLogs,
    ConfigureSystem,
    ManageRoles,
}

/// A hierarchical RBAC system.
pub struct HierarchicalRbac {
    /// Maps each level to its direct (non-inherited) permissions.
    level_permissions: HashMap<Level, HashSet<Permission>>,
    /// Maps each user to their assigned level.
    user_levels: HashMap<String, Level>,
}

impl HierarchicalRbac {
    /// Create a new hierarchical RBAC system with default level definitions.
    ///
    /// Default permissions per level:
    /// - Viewer: Read
    /// - Editor: Read, Write
    /// - Admin: Read, Write, Delete, ManageUsers, ViewAuditLogs
    /// - SuperAdmin: all permissions
    pub fn new() -> Self {
        todo!("Create the system with default level-to-permission mappings")
    }

    /// Get the DIRECT (non-inherited) permissions for a level.
    pub fn direct_permissions(&self, level: &Level) -> &HashSet<Permission> {
        todo!("Return the direct permissions for this level")
    }

    /// Get ALL permissions for a level, including those inherited from lower levels.
    ///
    /// A level inherits all permissions from levels below it in the hierarchy.
    /// For example, Admin inherits Editor's and Viewer's permissions.
    pub fn effective_permissions(&self, level: &Level) -> HashSet<Permission> {
        todo!("Collect permissions from this level and all lower levels")
    }

    /// Assign a level to a user.
    pub fn assign_level(&mut self, user: &str, level: Level) {
        todo!("Store the user's level")
    }

    /// Get a user's assigned level.
    pub fn get_user_level(&self, user: &str) -> Option<&Level> {
        todo!("Look up the user's level")
    }

    /// Check if a user has a specific permission.
    ///
    /// Consider the user's effective permissions (including inherited ones).
    pub fn has_permission(&self, user: &str, permission: &Permission) -> bool {
        todo!("Check user's effective permissions")
    }

    /// Check if a user is at or above a required level.
    ///
    /// This is useful for endpoint-level access control:
    /// "You must be at least an Editor to access this endpoint."
    pub fn has_minimum_level(&self, user: &str, minimum: &Level) -> bool {
        todo!("Check if user's level >= minimum level")
    }

    /// Promote a user to the next level.
    ///
    /// Returns the new level, or None if already at the maximum level.
    pub fn promote(&mut self, user: &str) -> Option<Level> {
        todo!("Increase the user's level by one step")
    }

    /// Demote a user to the previous level.
    ///
    /// Returns the new level, or None if already at the minimum level.
    pub fn demote(&mut self, user: &str) -> Option<Level> {
        todo!("Decrease the user's level by one step")
    }

    /// List all users at a given level.
    pub fn users_at_level(&self, level: &Level) -> Vec<&str> {
        todo!("Find all users at the specified level")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> HierarchicalRbac {
        let mut rbac = HierarchicalRbac::new();
        rbac.assign_level("alice", Level::Admin);
        rbac.assign_level("bob", Level::Editor);
        rbac.assign_level("carol", Level::Viewer);
        rbac
    }

    #[test]
    fn test_viewer_permissions() {
        let rbac = setup();
        assert!(rbac.has_permission("carol", &Permission::Read));
        assert!(!rbac.has_permission("carol", &Permission::Write));
        assert!(!rbac.has_permission("carol", &Permission::Delete));
    }

    #[test]
    fn test_editor_inherits_viewer() {
        let rbac = setup();
        assert!(rbac.has_permission("bob", &Permission::Read));  // inherited from Viewer
        assert!(rbac.has_permission("bob", &Permission::Write));  // Editor's own
        assert!(!rbac.has_permission("bob", &Permission::Delete)); // Admin only
    }

    #[test]
    fn test_admin_inherits_all_below() {
        let rbac = setup();
        assert!(rbac.has_permission("alice", &Permission::Read));
        assert!(rbac.has_permission("alice", &Permission::Write));
        assert!(rbac.has_permission("alice", &Permission::Delete));
        assert!(rbac.has_permission("alice", &Permission::ManageUsers));
        assert!(rbac.has_permission("alice", &Permission::ViewAuditLogs));
    }

    #[test]
    fn test_minimum_level_check() {
        let rbac = setup();
        // Viewer can access Viewer-level
        assert!(rbac.has_minimum_level("carol", &Level::Viewer));
        // Viewer cannot access Editor-level
        assert!(!rbac.has_minimum_level("carol", &Level::Editor));
        // Editor can access Viewer-level and Editor-level
        assert!(rbac.has_minimum_level("bob", &Level::Viewer));
        assert!(rbac.has_minimum_level("bob", &Level::Editor));
        assert!(!rbac.has_minimum_level("bob", &Level::Admin));
    }

    #[test]
    fn test_promote() {
        let mut rbac = setup();
        let new_level = rbac.promote("carol");
        assert_eq!(new_level, Some(Level::Editor));
        assert!(rbac.has_permission("carol", &Permission::Write));
    }

    #[test]
    fn test_demote() {
        let mut rbac = setup();
        let new_level = rbac.demote("alice");
        assert_eq!(new_level, Some(Level::Editor));
        assert!(!rbac.has_permission("alice", &Permission::Delete));
    }

    #[test]
    fn test_promote_at_max() {
        let mut rbac = setup();
        let new_level = rbac.promote("carol"); // Viewer → Editor
        assert_eq!(new_level, Some(Level::Editor));
        let new_level = rbac.promote("carol"); // Editor → Admin
        assert_eq!(new_level, Some(Level::Admin));
        let new_level = rbac.promote("carol"); // Admin → SuperAdmin
        assert_eq!(new_level, Some(Level::SuperAdmin));
        let new_level = rbac.promote("carol"); // Already at max
        assert_eq!(new_level, None);
    }

    #[test]
    fn test_demote_at_min() {
        let mut rbac = setup();
        let new_level = rbac.demote("carol"); // Already at Viewer (min)
        assert_eq!(new_level, None);
    }

    #[test]
    fn test_unknown_user() {
        let rbac = setup();
        assert!(!rbac.has_permission("unknown", &Permission::Read));
        assert!(!rbac.has_minimum_level("unknown", &Level::Viewer));
    }
}
