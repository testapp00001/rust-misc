//! # Lesson 04: Permission Hierarchy (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Viewer = 0,
    Editor = 1,
    Admin = 2,
    SuperAdmin = 3,
}

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

pub struct HierarchicalRbac {
    level_permissions: HashMap<Level, HashSet<Permission>>,
    user_levels: HashMap<String, Level>,
}

impl HierarchicalRbac {
    pub fn new() -> Self {
        let mut level_permissions = HashMap::new();

        let mut viewer = HashSet::new();
        viewer.insert(Permission::Read);
        level_permissions.insert(Level::Viewer, viewer);

        let mut editor = HashSet::new();
        editor.insert(Permission::Write);
        level_permissions.insert(Level::Editor, editor);

        let mut admin = HashSet::new();
        admin.insert(Permission::Delete);
        admin.insert(Permission::ManageUsers);
        admin.insert(Permission::ViewAuditLogs);
        level_permissions.insert(Level::Admin, admin);

        let mut super_admin = HashSet::new();
        super_admin.insert(Permission::ConfigureSystem);
        super_admin.insert(Permission::ManageRoles);
        level_permissions.insert(Level::SuperAdmin, super_admin);

        Self {
            level_permissions,
            user_levels: HashMap::new(),
        }
    }

    pub fn direct_permissions(&self, level: &Level) -> &HashSet<Permission> {
        self.level_permissions.get(level).unwrap_or_else(|| {
            // Return a reference to an empty set (leak a static for lifetime)
            static EMPTY: std::sync::OnceLock<HashSet<Permission>> = std::sync::OnceLock::new();
            EMPTY.get_or_init(HashSet::new)
        })
    }

    pub fn effective_permissions(&self, level: &Level) -> HashSet<Permission> {
        let mut all = HashSet::new();
        // Collect permissions from this level and all lower levels
        for &lvl in &[Level::Viewer, Level::Editor, Level::Admin, Level::SuperAdmin] {
            if lvl <= *level {
                if let Some(perms) = self.level_permissions.get(&lvl) {
                    all.extend(perms.iter().cloned());
                }
            }
        }
        all
    }

    pub fn assign_level(&mut self, user: &str, level: Level) {
        self.user_levels.insert(user.to_string(), level);
    }

    pub fn get_user_level(&self, user: &str) -> Option<&Level> {
        self.user_levels.get(user)
    }

    pub fn has_permission(&self, user: &str, permission: &Permission) -> bool {
        match self.user_levels.get(user) {
            Some(level) => self.effective_permissions(level).contains(permission),
            None => false,
        }
    }

    pub fn has_minimum_level(&self, user: &str, minimum: &Level) -> bool {
        match self.user_levels.get(user) {
            Some(level) => level >= minimum,
            None => false,
        }
    }

    pub fn promote(&mut self, user: &str) -> Option<Level> {
        let current = self.user_levels.get(user)?;
        match current {
            Level::Viewer => {
                self.user_levels.insert(user.to_string(), Level::Editor);
                Some(Level::Editor)
            }
            Level::Editor => {
                self.user_levels.insert(user.to_string(), Level::Admin);
                Some(Level::Admin)
            }
            Level::Admin => {
                self.user_levels.insert(user.to_string(), Level::SuperAdmin);
                Some(Level::SuperAdmin)
            }
            Level::SuperAdmin => None,
        }
    }

    pub fn demote(&mut self, user: &str) -> Option<Level> {
        let current = self.user_levels.get(user)?;
        match current {
            Level::SuperAdmin => {
                self.user_levels.insert(user.to_string(), Level::Admin);
                Some(Level::Admin)
            }
            Level::Admin => {
                self.user_levels.insert(user.to_string(), Level::Editor);
                Some(Level::Editor)
            }
            Level::Editor => {
                self.user_levels.insert(user.to_string(), Level::Viewer);
                Some(Level::Viewer)
            }
            Level::Viewer => None,
        }
    }

    pub fn users_at_level(&self, level: &Level) -> Vec<&str> {
        self.user_levels
            .iter()
            .filter(|(_, l)| *l == level)
            .map(|(name, _)| name.as_str())
            .collect()
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
        assert!(rbac.has_permission("bob", &Permission::Read));
        assert!(rbac.has_permission("bob", &Permission::Write));
        assert!(!rbac.has_permission("bob", &Permission::Delete));
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
        assert!(rbac.has_minimum_level("carol", &Level::Viewer));
        assert!(!rbac.has_minimum_level("carol", &Level::Editor));
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
        let new_level = rbac.promote("carol");
        assert_eq!(new_level, Some(Level::Editor));
        let new_level = rbac.promote("carol");
        assert_eq!(new_level, Some(Level::Admin));
        let new_level = rbac.promote("carol");
        assert_eq!(new_level, Some(Level::SuperAdmin));
        let new_level = rbac.promote("carol");
        assert_eq!(new_level, None);
    }

    #[test]
    fn test_demote_at_min() {
        let mut rbac = setup();
        let new_level = rbac.demote("carol");
        assert_eq!(new_level, None);
    }

    #[test]
    fn test_unknown_user() {
        let rbac = setup();
        assert!(!rbac.has_permission("unknown", &Permission::Read));
        assert!(!rbac.has_minimum_level("unknown", &Level::Viewer));
    }
}
