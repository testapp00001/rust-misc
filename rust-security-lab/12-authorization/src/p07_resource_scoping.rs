//! # Lesson 07: Resource Scoping
//!
//! ## What is Resource Scoping?
//!
//! Resource scoping ensures users can only access resources within their authorized scope.
//! Think of it as "you can see your own files, but not your neighbor's."
//!
//! ## Scoping Models
//!
//! 1. **User-owned**: Each resource belongs to one user (e.g., personal documents)
//! 2. **Team-owned**: Resources belong to a team; team members share access
//! 3. **Organization-owned**: Resources belong to an org; access controlled by org membership
//!
//! ## 🔴 Attack: Scope Leakage
//!
//! Without proper scoping, a user might:
//! - Access another user's resources by guessing IDs (IDOR)
//! - Access resources from a different team/organization
//! - Access resources they previously owned but transferred
//!
//! ## Defense: Always Filter by Scope
//!
//! Every data access query MUST include the user's scope as a filter.
//! Never return "all records" and filter client-side.

use std::collections::{HashMap, HashSet};

/// A user with team membership.
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub team_ids: HashSet<String>,
}

/// A team that owns resources.
#[derive(Debug, Clone)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub member_ids: HashSet<String>,
}

/// A resource that belongs to either a user or a team.
#[derive(Debug, Clone)]
pub struct Resource {
    pub id: String,
    pub title: String,
    /// If Some, the resource is user-owned.
    pub owner_user_id: Option<String>,
    /// If Some, the resource is team-owned.
    pub owner_team_id: Option<String>,
    /// Content of the resource.
    pub content: String,
}

/// The resource scoping service.
pub struct ResourceScopingService {
    users: HashMap<String, User>,
    teams: HashMap<String, Team>,
    resources: HashMap<String, Resource>,
}

impl ResourceScopingService {
    /// Create a new empty service.
    pub fn new() -> Self {
        todo!("Initialize with empty maps")
    }

    /// Add a user.
    pub fn add_user(&mut self, user: User) {
        todo!("Insert user")
    }

    /// Add a team.
    pub fn add_team(&mut self, team: Team) {
        todo!("Insert team")
    }

    /// Add a resource.
    pub fn add_resource(&mut self, resource: Resource) {
        todo!("Insert resource")
    }

    /// Check if a user can access a specific resource.
    ///
    /// Rules:
    /// - User-owned resources: only the owner can access
    /// - Team-owned resources: any member of the owning team can access
    /// - Resources with no owner: deny (orphaned resources are inaccessible)
    pub fn can_access(&self, user_id: &str, resource_id: &str) -> bool {
        todo!("Check ownership: user-owned or team-owned with membership")
    }

    /// Access a resource, returning it if allowed.
    pub fn access_resource(
        &self,
        user_id: &str,
        resource_id: &str,
    ) -> Result<&Resource, String> {
        todo!("Check can_access, then return the resource or an error")
    }

    /// List all resources accessible by a user.
    ///
    /// This includes:
    /// - Resources directly owned by the user
    /// - Resources owned by any team the user is a member of
    pub fn list_accessible_resources(&self, user_id: &str) -> Vec<&Resource> {
        todo!("Collect all resources the user can access")
    }

    /// Transfer ownership of a user-owned resource to another user.
    ///
    /// Only the current owner can transfer.
    pub fn transfer_to_user(
        &mut self,
        resource_id: &str,
        current_owner: &str,
        new_owner: &str,
    ) -> Result<(), String> {
        todo!("Verify current ownership, then change owner_user_id")
    }

    /// Transfer ownership of a resource to a team.
    ///
    /// Only the current owner (user) can transfer to a team.
    pub fn transfer_to_team(
        &mut self,
        resource_id: &str,
        current_owner: &str,
        team_id: &str,
    ) -> Result<(), String> {
        todo!("Verify current ownership and team exists, then change to team-owned")
    }

    /// Add a user to a team.
    pub fn add_to_team(&mut self, user_id: &str, team_id: &str) -> Result<(), String> {
        todo!("Add user to team's member set and team to user's team set")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> ResourceScopingService {
        let mut svc = ResourceScopingService::new();

        svc.add_user(User {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            team_ids: vec!["team-eng".to_string()].into_iter().collect(),
        });
        svc.add_user(User {
            id: "bob".to_string(),
            name: "Bob".to_string(),
            team_ids: vec!["team-eng".to_string()].into_iter().collect(),
        });
        svc.add_user(User {
            id: "carol".to_string(),
            name: "Carol".to_string(),
            team_ids: vec!["team-sales".to_string()].into_iter().collect(),
        });

        svc.add_team(Team {
            id: "team-eng".to_string(),
            name: "Engineering".to_string(),
            member_ids: vec!["alice".to_string(), "bob".to_string()]
                .into_iter()
                .collect(),
        });
        svc.add_team(Team {
            id: "team-sales".to_string(),
            name: "Sales".to_string(),
            member_ids: vec!["carol".to_string()].into_iter().collect(),
        });

        svc.add_resource(Resource {
            id: "res-1".to_string(),
            title: "Alice's private doc".to_string(),
            owner_user_id: Some("alice".to_string()),
            owner_team_id: None,
            content: "Private".to_string(),
        });
        svc.add_resource(Resource {
            id: "res-2".to_string(),
            title: "Engineering spec".to_string(),
            owner_user_id: None,
            owner_team_id: Some("team-eng".to_string()),
            content: "Spec".to_string(),
        });
        svc.add_resource(Resource {
            id: "res-3".to_string(),
            title: "Sales playbook".to_string(),
            owner_user_id: None,
            owner_team_id: Some("team-sales".to_string()),
            content: "Playbook".to_string(),
        });

        svc
    }

    #[test]
    fn test_user_own_resource() {
        let svc = setup();
        assert!(svc.can_access("alice", "res-1"));
    }

    #[test]
    fn test_user_cannot_access_other_user_resource() {
        let svc = setup();
        assert!(!svc.can_access("bob", "res-1"));
        assert!(!svc.can_access("carol", "res-1"));
    }

    #[test]
    fn test_team_member_can_access_team_resource() {
        let svc = setup();
        assert!(svc.can_access("alice", "res-2"));
        assert!(svc.can_access("bob", "res-2"));
    }

    #[test]
    fn test_non_team_member_cannot_access_team_resource() {
        let svc = setup();
        assert!(!svc.can_access("carol", "res-2")); // Carol is in sales, not eng
        assert!(!svc.can_access("alice", "res-3")); // Alice is in eng, not sales
    }

    #[test]
    fn test_list_accessible_resources() {
        let svc = setup();
        let alice_resources = svc.list_accessible_resources("alice");
        // Alice owns res-1 and is in team-eng (res-2)
        let ids: Vec<&str> = alice_resources.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"res-1"));
        assert!(ids.contains(&"res-2"));
        assert!(!ids.contains(&"res-3"));
    }

    #[test]
    fn test_transfer_ownership() {
        let mut svc = setup();
        assert!(svc.transfer_to_user("res-1", "alice", "bob").is_ok());
        // Alice can no longer access
        assert!(!svc.can_access("alice", "res-1"));
        // Bob can now access
        assert!(svc.can_access("bob", "res-1"));
    }

    #[test]
    fn test_transfer_only_owner_can_transfer() {
        let mut svc = setup();
        assert!(svc.transfer_to_user("res-1", "bob", "carol").is_err());
    }

    #[test]
    fn test_transfer_to_team() {
        let mut svc = setup();
        assert!(svc.transfer_to_team("res-1", "alice", "team-sales").is_ok());
        // Alice no longer directly owns it
        assert!(!svc.can_access("alice", "res-1"));
        // Carol (sales) can now access
        assert!(svc.can_access("carol", "res-1"));
    }
}
