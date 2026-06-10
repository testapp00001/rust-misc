//! # Lesson 10: Secure Logout — Token Revocation, Session Cleanup
//!
//! ## Why Logout is Harder Than You Think
//!
//! Logging out isn't just "delete the cookie." You must:
//!
//! 1. **Server-side revocation**: Invalidate the token/session server-side
//! 2. **Cookie clearing**: Remove all auth cookies with correct attributes
//! 3. **Token blacklisting**: For JWTs (stateless), maintain a revocation list
//! 4. **All sessions**: Optionally revoke all sessions (security action)
//! 5. **Downstream services**: In microservices, propagate revocation
//!
//! ## Stateless vs Stateful Logout
//!
//! | Approach | Logout mechanism | Tradeoff |
//! |----------|-----------------|----------|
//! | Sessions (stateful) | Delete server-side session | Simple, immediate |
//! | JWT (stateless) | Token blacklist OR short expiry | Complex, requires shared state |
//! | JWT + refresh tokens | Revoke refresh token | Access token still valid until expiry |
//!
//! ## Attack Context
//!
//! - **Stolen token**: If logout doesn't revoke server-side, stolen token still works
//! - **Shared computer**: Must clear ALL client-side storage (cookies, localStorage)
//! - **Race condition**: Token used between logout request and revocation propagation
//!
//! **Defense**: Always revoke server-side. Never rely on client-side clearing alone.

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents an active session.
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub created_at: u64,
    pub last_active: u64,
    pub ip_address: String,
    pub user_agent: String,
}

/// Cookie configuration for clearing.
#[derive(Debug, Clone)]
pub struct CookieConfig {
    pub name: String,
    pub path: String,
    pub domain: Option<String>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: String,
}

impl CookieConfig {
    /// Generate a Set-Cookie header value that clears the cookie.
    ///
    /// Setting max-age=0 and an empty value tells the browser to delete the cookie.
    pub fn clear_header(&self) -> String {
        todo!("Generate a Set-Cookie header that clears this cookie")
    }
}

/// A token revocation list (blacklist).
///
/// In production, this would be Redis or a database with TTL.
/// Entries expire automatically when the token would have expired anyway.
pub struct RevocationList {
    /// Set of revoked token IDs/jti claims
    revoked: HashMap<String, u64>, // token_id → expiry timestamp
}

impl RevocationList {
    pub fn new() -> Self {
        todo!("Create a new RevocationList")
    }

    /// Revoke a token with the given ID and expiry time.
    pub fn revoke(&mut self, token_id: &str, expires_at: u64) {
        todo!("Add a token to the revocation list")
    }

    /// Check if a token has been revoked.
    pub fn is_revoked(&self, token_id: &str) -> bool {
        todo!("Check if a token is in the revocation list")
    }

    /// Clean up expired entries (tokens that would have expired anyway).
    pub fn cleanup(&mut self) {
        todo!("Remove expired entries from the revocation list")
    }
}

/// Session manager that handles logout and session management.
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    /// user_id → set of session_ids
    user_sessions: HashMap<String, HashSet<String>>,
    revocation_list: RevocationList,
}

impl SessionManager {
    pub fn new() -> Self {
        todo!("Create a new SessionManager")
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Create a new session.
    pub fn create_session(
        &mut self,
        session_id: &str,
        user_id: &str,
        ip_address: &str,
        user_agent: &str,
    ) {
        todo!("Create a new session and track it")
    }

    /// Logout: destroy a specific session and revoke its token.
    ///
    /// This is a single-session logout (e.g., "log out of this browser").
    pub fn logout(&mut self, session_id: &str, token_id: &str, token_expires_at: u64) -> bool {
        todo!("Logout: revoke token + destroy session")
    }

    /// Logout all sessions for a user.
    ///
    /// This is a "log out everywhere" / security action.
    /// Useful when:
    /// - User suspects account compromise
    /// - Password changed
    /// - Admin force-logout
    pub fn logout_all(
        &mut self,
        user_id: &str,
        token_expiry: u64,
    ) -> Vec<String> {
        todo!("Logout all sessions for a user")
    }

    /// Get all active sessions for a user.
    pub fn get_user_sessions(&self, user_id: &str) -> Vec<&Session> {
        todo!("Get all active sessions for a user")
    }

    /// Check if a token has been revoked.
    pub fn is_token_revoked(&self, token_id: &str) -> bool {
        self.revocation_list.is_revoked(token_id)
    }

    /// Get the number of active sessions.
    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }
}

/// Generate all Set-Cookie headers needed for secure logout.
///
/// In a real application, you'd clear:
/// - Session cookie
/// - CSRF token cookie
/// - Any auth-related cookies
pub fn generate_logout_cookies() -> Vec<(String, String)> {
    todo!("Generate Set-Cookie headers for all auth cookies")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_session() {
        let mut mgr = SessionManager::new();
        mgr.create_session("sess1", "user1", "192.168.1.1", "Mozilla/5.0");
        let sessions = mgr.get_user_sessions("user1");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, "sess1");
    }

    #[test]
    fn test_logout_removes_session() {
        let mut mgr = SessionManager::new();
        mgr.create_session("sess1", "user1", "192.168.1.1", "Mozilla/5.0");
        assert_eq!(mgr.active_session_count(), 1);

        let result = mgr.logout("sess1", "token1", 9999999999);
        assert!(result);
        assert_eq!(mgr.active_session_count(), 0);
    }

    #[test]
    fn test_logout_revokes_token() {
        let mut mgr = SessionManager::new();
        mgr.create_session("sess1", "user1", "192.168.1.1", "Mozilla/5.0");

        mgr.logout("sess1", "token1", 9999999999);
        assert!(mgr.is_token_revoked("token1"));
    }

    #[test]
    fn test_logout_all_sessions() {
        let mut mgr = SessionManager::new();
        mgr.create_session("sess1", "user1", "192.168.1.1", "Chrome");
        mgr.create_session("sess2", "user1", "10.0.0.1", "Firefox");
        mgr.create_session("sess3", "user2", "172.16.0.1", "Safari");

        let removed = mgr.logout_all("user1", 9999999999);
        assert_eq!(removed.len(), 2);
        assert_eq!(mgr.active_session_count(), 1); // Only user2's session remains
    }

    #[test]
    fn test_logout_all_preserves_other_users() {
        let mut mgr = SessionManager::new();
        mgr.create_session("sess1", "user1", "192.168.1.1", "Chrome");
        mgr.create_session("sess2", "user2", "10.0.0.1", "Firefox");

        mgr.logout_all("user1", 9999999999);
        let user2_sessions = mgr.get_user_sessions("user2");
        assert_eq!(user2_sessions.len(), 1);
    }

    #[test]
    fn test_cookie_clear_header() {
        let config = CookieConfig {
            name: "session".to_string(),
            path: "/".to_string(),
            domain: None,
            secure: true,
            http_only: true,
            same_site: "Strict".to_string(),
        };
        let header = config.clear_header();
        assert!(header.contains("session=;"));
        assert!(header.contains("Max-Age=0"));
        assert!(header.contains("Secure"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("SameSite=Strict"));
    }

    #[test]
    fn test_generate_logout_cookies() {
        let cookies = generate_logout_cookies();
        assert_eq!(cookies.len(), 3);
        for (name, header) in &cookies {
            assert!(header.contains("Max-Age=0"), "Cookie '{}' should be cleared", name);
        }
    }

    #[test]
    fn test_revocation_list() {
        let mut list = RevocationList::new();
        assert!(!list.is_revoked("token1"));

        list.revoke("token1", 9999999999);
        assert!(list.is_revoked("token1"));
        assert!(!list.is_revoked("token2"));
    }
}
