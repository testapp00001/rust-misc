//! # Lesson 06: Authorization Bypass Attacks (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub role: Role,
    pub owned_documents: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    User,
    Admin,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub content: String,
    pub is_public: bool,
}

pub struct AuthorizationService {
    users: HashMap<String, User>,
    documents: HashMap<String, Document>,
    allowed_base_dir: PathBuf,
}

impl AuthorizationService {
    pub fn new(base_dir: &str) -> Self {
        Self {
            users: HashMap::new(),
            documents: HashMap::new(),
            allowed_base_dir: PathBuf::from(base_dir),
        }
    }

    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id.clone(), user);
    }

    pub fn add_document(&mut self, doc: Document) {
        self.documents.insert(doc.id.clone(), doc);
    }

    /// IDOR defense: verify ownership OR admin OR public.
    pub fn access_document(&self, user_id: &str, doc_id: &str) -> Result<&Document, String> {
        let user = self
            .users
            .get(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        let doc = self
            .documents
            .get(doc_id)
            .ok_or_else(|| "Document not found".to_string())?;

        // Admin can access anything
        if user.role == Role::Admin {
            return Ok(doc);
        }

        // Public documents are accessible by anyone
        if doc.is_public {
            return Ok(doc);
        }

        // Owner can access their own documents
        if doc.owner_id == user_id {
            return Ok(doc);
        }

        Err("Access denied: you do not own this document".to_string())
    }

    /// Update profile, ignoring client-supplied role.
    pub fn update_profile(
        &mut self,
        user_id: &str,
        new_name: Option<&str>,
        _attempted_role: Option<Role>,
    ) -> Result<&User, String> {
        let user = self
            .users
            .get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        // Only update allowed fields -- role is NEVER set from client input
        if let Some(name) = new_name {
            user.name = name.to_string();
        }

        // attempted_role is intentionally ignored -- server controls roles

        Ok(self.users.get(user_id).unwrap())
    }

    /// Canonicalize and validate path stays within base directory.
    pub fn safe_file_path(&self, requested_path: &str) -> Result<PathBuf, String> {
        // Build the full path by joining with base directory
        let full_path = self.allowed_base_dir.join(requested_path);

        // Canonicalize to resolve `..` and symlinks
        // Since the path may not exist on disk, we do manual canonicalization
        let canonical = Self::canonicalize_manual(&full_path)?;

        // Verify the canonical path starts with the base directory
        if canonical.starts_with(&self.allowed_base_dir) {
            Ok(canonical)
        } else {
            Err(format!(
                "Path traversal detected: '{}' escapes base directory",
                requested_path
            ))
        }
    }

    /// Manual path canonicalization that works even if the file doesn't exist.
    fn canonicalize_manual(path: &Path) -> Result<PathBuf, String> {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    if components.is_empty() {
                        return Err("Path traversal above root".to_string());
                    }
                    components.pop();
                }
                std::path::Component::CurDir => {
                    // Skip `.` components
                }
                other => components.push(other),
            }
        }
        Ok(components.iter().collect())
    }

    pub fn list_user_documents(&self, user_id: &str) -> Vec<&Document> {
        self.documents
            .values()
            .filter(|doc| doc.owner_id == user_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> AuthorizationService {
        let mut svc = AuthorizationService::new("/var/data");

        svc.add_user(User {
            id: "alice".to_string(),
            name: "Alice".to_string(),
            role: Role::User,
            owned_documents: vec!["doc-1".to_string(), "doc-2".to_string()]
                .into_iter()
                .collect(),
        });

        svc.add_user(User {
            id: "bob".to_string(),
            name: "Bob".to_string(),
            role: Role::User,
            owned_documents: vec!["doc-3".to_string()].into_iter().collect(),
        });

        svc.add_user(User {
            id: "admin".to_string(),
            name: "Admin".to_string(),
            role: Role::Admin,
            owned_documents: HashSet::new(),
        });

        svc.add_document(Document {
            id: "doc-1".to_string(),
            owner_id: "alice".to_string(),
            title: "Alice's Report".to_string(),
            content: "Secret data".to_string(),
            is_public: false,
        });

        svc.add_document(Document {
            id: "doc-2".to_string(),
            owner_id: "alice".to_string(),
            title: "Alice's Notes".to_string(),
            content: "Private notes".to_string(),
            is_public: false,
        });

        svc.add_document(Document {
            id: "doc-3".to_string(),
            owner_id: "bob".to_string(),
            title: "Bob's Doc".to_string(),
            content: "Bob's data".to_string(),
            is_public: false,
        });

        svc.add_document(Document {
            id: "doc-public".to_string(),
            owner_id: "admin".to_string(),
            title: "Public Announcement".to_string(),
            content: "Hello everyone".to_string(),
            is_public: true,
        });

        svc
    }

    #[test]
    fn test_owner_can_access_own_document() {
        let svc = setup();
        assert!(svc.access_document("alice", "doc-1").is_ok());
    }

    #[test]
    fn test_idor_blocked_other_users_document() {
        let svc = setup();
        assert!(svc.access_document("alice", "doc-3").is_err());
    }

    #[test]
    fn test_admin_can_access_any_document() {
        let svc = setup();
        assert!(svc.access_document("admin", "doc-1").is_ok());
        assert!(svc.access_document("admin", "doc-3").is_ok());
    }

    #[test]
    fn test_public_document_accessible_by_anyone() {
        let svc = setup();
        assert!(svc.access_document("alice", "doc-public").is_ok());
        assert!(svc.access_document("bob", "doc-public").is_ok());
    }

    #[test]
    fn test_nonexistent_document_fails() {
        let svc = setup();
        assert!(svc.access_document("alice", "nonexistent").is_err());
    }

    #[test]
    fn test_privilege_escalation_blocked() {
        let mut svc = setup();
        let result = svc.update_profile("alice", Some("Alice Hacked"), Some(Role::Admin));
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Alice Hacked");
        assert_eq!(user.role, Role::User);
    }

    #[test]
    fn test_path_traversal_blocked() {
        let svc = setup();
        assert!(svc.safe_file_path("../../../etc/passwd").is_err());
        assert!(svc.safe_file_path("reports/../../../etc/shadow").is_err());
    }

    #[test]
    fn test_safe_path_allowed() {
        let svc = setup();
        let result = svc.safe_file_path("reports/annual.pdf");
        assert!(result.is_ok());
    }

    #[test]
    fn test_nonexistent_user_profile() {
        let mut svc = setup();
        assert!(svc.update_profile("nobody", Some("Ghost"), None).is_err());
    }
}
