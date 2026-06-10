//! # Lesson 06: Authorization Bypass Attacks
//!
//! ## What is Authorization Bypass?
//!
//! Authorization bypass occurs when an attacker circumvents access controls to perform
//! actions or access resources they should not be able to. Common patterns:
//!
//! ## IDOR (Insecure Direct Object Reference)
//!
//! ```
//! GET /api/users/123/documents  ← Normal request
//! GET /api/users/456/documents  ← Attacker changes user ID to access another's data
//!
//! The server checks "is the user authenticated?" but NOT "does user 123 OWN these documents?"
//! ```
//!
//! ## Path Traversal
//!
//! ```
//! GET /files/report.pdf         ← Normal
//! GET /files/../../../etc/passwd ← Attacker escapes the files directory
//!
//! The server must canonicalize paths and verify they stay within allowed boundaries.
//! ```
//!
//! ## Privilege Escalation
//!
//! ```
//! POST /api/profile { "name": "Bob", "role": "admin" }  ← User injects role field
//!
//! The server must ignore client-supplied role assignments and use server-side state.
//! ```
//!
//! ## 🔴 Defense Patterns
//!
//! 1. **Always verify ownership**: Check that the authenticated user OWNS the requested resource
//! 2. **Canonicalize paths**: Resolve `..` and symlinks before checking boundaries
//! 3. **Server-side authority**: Never trust client-supplied role/permission fields
//! 4. **Default deny**: If ownership check fails, deny access

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Represents a user in the system.
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub role: Role,
    /// IDs of documents this user owns.
    pub owned_documents: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    User,
    Admin,
}

/// A document in the system.
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub owner_id: String,
    pub title: String,
    pub content: String,
    pub is_public: bool,
}

/// The authorization service that prevents bypass attacks.
pub struct AuthorizationService {
    users: HashMap<String, User>,
    documents: HashMap<String, Document>,
    /// The base directory for file access (simulated).
    allowed_base_dir: PathBuf,
}

impl AuthorizationService {
    /// Create a new authorization service.
    pub fn new(base_dir: &str) -> Self {
        todo!("Initialize with empty users/documents maps and the base directory")
    }

    /// Add a user to the system.
    pub fn add_user(&mut self, user: User) {
        todo!("Insert the user into the users map")
    }

    /// Add a document to the system.
    pub fn add_document(&mut self, doc: Document) {
        todo!("Insert the document into the documents map")
    }

    /// Check if a user can access a document by ID (IDOR defense).
    ///
    /// Rules:
    /// - The user can access their own documents.
    /// - Admins can access any document.
    /// - Anyone can access public documents.
    /// - Otherwise, deny.
    ///
    /// Returns Ok(&Document) if allowed, Err with a message if denied.
    pub fn access_document(&self, user_id: &str, doc_id: &str) -> Result<&Document, String> {
        todo!("Verify the user is authenticated AND authorized to access this document")
    }

    /// Update a user's profile, but IGNORE any client-supplied role field.
    ///
    /// This prevents privilege escalation: the server determines roles,
    /// not the client.
    ///
    /// Returns the updated User or an error if the user doesn't exist.
    pub fn update_profile(
        &mut self,
        user_id: &str,
        new_name: Option<&str>,
        attempted_role: Option<Role>,
    ) -> Result<&User, String> {
        todo!("Update only allowed fields (name), ignore role changes")
    }

    /// Safely resolve a file path, preventing path traversal.
    ///
    /// The rules:
    /// 1. Canonicalize the requested path relative to the base directory.
    /// 2. Verify the canonical path starts with the base directory.
    /// 3. If the path escapes the base directory, return an error.
    ///
    /// Returns the canonical safe path, or an error if the path is unsafe.
    pub fn safe_file_path(&self, requested_path: &str) -> Result<PathBuf, String> {
        todo!("Canonicalize and validate the path stays within base directory")
    }

    /// List all documents owned by a user.
    pub fn list_user_documents(&self, user_id: &str) -> Vec<&Document> {
        todo!("Return all documents where owner_id matches the user")
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
        // Alice tries to access Bob's document
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
        // Alice tries to make herself an admin
        let result = svc.update_profile("alice", Some("Alice Hacked"), Some(Role::Admin));
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.name, "Alice Hacked"); // Name was updated
        assert_eq!(user.role, Role::User); // Role was NOT changed
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
