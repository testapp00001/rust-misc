//! # Lesson 07: Resource Scoping (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub team_ids: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub member_ids: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct Resource {
    pub id: String,
    pub title: String,
    pub owner_user_id: Option<String>,
    pub owner_team_id: Option<String>,
    pub content: String,
}

pub struct ResourceScopingService {
    users: HashMap<String, User>,
    teams: HashMap<String, Team>,
    resources: HashMap<String, Resource>,
}

impl ResourceScopingService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            teams: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id.clone(), user);
    }

    pub fn add_team(&mut self, team: Team) {
        self.teams.insert(team.id.clone(), team);
    }

    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.insert(resource.id.clone(), resource);
    }

    pub fn can_access(&self, user_id: &str, resource_id: &str) -> bool {
        let resource = match self.resources.get(resource_id) {
            Some(r) => r,
            None => return false,
        };

        // Check user-owned
        if let Some(ref owner) = resource.owner_user_id {
            return owner == user_id;
        }

        // Check team-owned
        if let Some(ref team_id) = resource.owner_team_id {
            return self
                .teams
                .get(team_id)
                .map(|team| team.member_ids.contains(user_id))
                .unwrap_or(false);
        }

        // No owner -- deny
        false
    }

    pub fn access_resource(
        &self,
        user_id: &str,
        resource_id: &str,
    ) -> Result<&Resource, String> {
        if self.can_access(user_id, resource_id) {
            Ok(self.resources.get(resource_id).unwrap())
        } else {
            Err("Access denied: resource not in your scope".to_string())
        }
    }

    pub fn list_accessible_resources(&self, user_id: &str) -> Vec<&Resource> {
        let user = match self.users.get(user_id) {
            Some(u) => u,
            None => return Vec::new(),
        };

        self.resources
            .values()
            .filter(|r| {
                // User-owned
                if let Some(ref owner) = r.owner_user_id {
                    if owner == user_id {
                        return true;
                    }
                }
                // Team-owned
                if let Some(ref team_id) = r.owner_team_id {
                    if user.team_ids.contains(team_id) {
                        return true;
                    }
                }
                false
            })
            .collect()
    }

    pub fn transfer_to_user(
        &mut self,
        resource_id: &str,
        current_owner: &str,
        new_owner: &str,
    ) -> Result<(), String> {
        let resource = self
            .resources
            .get_mut(resource_id)
            .ok_or("Resource not found")?;

        // Only current owner can transfer
        match &resource.owner_user_id {
            Some(owner) if owner == current_owner => {}
            _ => return Err("Only the current owner can transfer this resource".to_string()),
        }

        resource.owner_user_id = Some(new_owner.to_string());
        resource.owner_team_id = None;
        Ok(())
    }

    pub fn transfer_to_team(
        &mut self,
        resource_id: &str,
        current_owner: &str,
        team_id: &str,
    ) -> Result<(), String> {
        // Verify team exists
        if !self.teams.contains_key(team_id) {
            return Err("Team not found".to_string());
        }

        let resource = self
            .resources
            .get_mut(resource_id)
            .ok_or("Resource not found")?;

        match &resource.owner_user_id {
            Some(owner) if owner == current_owner => {}
            _ => return Err("Only the current owner can transfer this resource".to_string()),
        }

        resource.owner_user_id = None;
        resource.owner_team_id = Some(team_id.to_string());
        Ok(())
    }

    pub fn add_to_team(&mut self, user_id: &str, team_id: &str) -> Result<(), String> {
        let team = self
            .teams
            .get_mut(team_id)
            .ok_or("Team not found")?;
        team.member_ids.insert(user_id.to_string());

        let user = self
            .users
            .get_mut(user_id)
            .ok_or("User not found")?;
        user.team_ids.insert(team_id.to_string());

        Ok(())
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
        assert!(!svc.can_access("carol", "res-2"));
        assert!(!svc.can_access("alice", "res-3"));
    }

    #[test]
    fn test_list_accessible_resources() {
        let svc = setup();
        let alice_resources = svc.list_accessible_resources("alice");
        let ids: Vec<&str> = alice_resources.iter().map(|r| r.id.as_str()).collect();
        assert!(ids.contains(&"res-1"));
        assert!(ids.contains(&"res-2"));
        assert!(!ids.contains(&"res-3"));
    }

    #[test]
    fn test_transfer_ownership() {
        let mut svc = setup();
        assert!(svc.transfer_to_user("res-1", "alice", "bob").is_ok());
        assert!(!svc.can_access("alice", "res-1"));
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
        assert!(!svc.can_access("alice", "res-1"));
        assert!(svc.can_access("carol", "res-1"));
    }
}
