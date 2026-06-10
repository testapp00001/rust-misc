//! # Lesson 09: Token Refresh — Access + Refresh Token Rotation
//!
//! ## Two-Token Model
//!
//! ```text
//! Access token:   Short-lived (15 min), sent with every API request
//! Refresh token:  Long-lived (7 days), used ONLY to get new access tokens
//! ```
//!
//! Why two tokens?
//! - Access tokens are sent frequently → high exposure risk → keep them short-lived
//! - Refresh tokens are used rarely → lower exposure → can be longer-lived
//! - If an access token leaks, it expires in minutes
//! - If a refresh token leaks, it can be revoked server-side
//!
//! ## Refresh Token Rotation
//!
//! Each time a refresh token is used, it's replaced with a new one.
//! The old refresh token is invalidated.
//!
//! ```text
//! 1. Client: Uses refresh_token_A to get new access + refresh tokens
//! 2. Server: Invalidates refresh_token_A, issues refresh_token_B
//! 3. Client: Uses refresh_token_B next time
//!
//! If attacker steals refresh_token_A and uses it AFTER the legitimate client,
//! the server detects reuse of an already-rotated token → revokes the entire family.
//! ```
//!
//! ## Attack Context
//!
//! Without rotation:
//! - Stolen refresh token works indefinitely until expiry
//! - No way to detect token theft
//!
//! With rotation:
//! - Token reuse detected → entire token family revoked
//! - Attacker and legitimate client compete → one gets locked out
//! - Server logs suspicious activity

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Token pair returned from login or refresh.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: u64,
    pub refresh_expires_at: u64,
}

/// A refresh token record stored server-side.
#[derive(Debug, Clone)]
pub struct RefreshTokenRecord {
    pub token: String,
    pub user_id: String,
    pub family_id: String,
    pub expires_at: u64,
    pub used: bool,
}

/// Token service that manages access and refresh tokens.
pub struct TokenService {
    /// All refresh tokens (for revocation checking)
    refresh_tokens: HashMap<String, RefreshTokenRecord>,
    /// Token family tracking (for rotation violation detection)
    /// family_id → list of refresh tokens in this family
    token_families: HashMap<String, Vec<String>>,
    /// Access token lifetime in seconds
    access_lifetime: u64,
    /// Refresh token lifetime in seconds
    refresh_lifetime: u64,
    /// Counter for generating unique tokens
    counter: u64,
}

impl TokenService {
    pub fn new(access_lifetime: u64, refresh_lifetime: u64) -> Self {
        todo!("Create a new TokenService")
    }

    fn generate_token(&mut self, prefix: &str) -> String {
        self.counter += 1;
        format!("{}_{}", prefix, self.counter)
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Issue a new token pair (e.g., after login).
    pub fn issue_tokens(&mut self, user_id: &str) -> TokenPair {
        todo!("Issue a new access + refresh token pair")
    }

    /// Refresh tokens: exchange a refresh token for a new token pair.
    ///
    /// Implements refresh token rotation with reuse detection:
    /// 1. Validate the refresh token exists and isn't expired
    /// 2. If the token was already used → REUSE DETECTED → revoke entire family
    /// 3. Mark the old refresh token as used
    /// 4. Issue new access + refresh tokens in the same family
    pub fn refresh(&mut self, refresh_token: &str) -> Result<TokenPair, RefreshError> {
        todo!("Refresh tokens with rotation and reuse detection")
    }

    /// Revoke all tokens in a family (used when reuse is detected).
    pub fn revoke_family(&mut self, family_id: &str) {
        todo!("Revoke all tokens in a family")
    }

    /// Revoke a specific user's tokens (logout).
    pub fn revoke_user_tokens(&mut self, user_id: &str) {
        todo!("Revoke all tokens for a user")
    }

    /// Check if a refresh token is valid (exists and not expired).
    pub fn is_valid_refresh_token(&self, token: &str) -> bool {
        todo!("Check if a refresh token is valid")
    }

    /// Get the number of active refresh tokens.
    pub fn active_token_count(&self) -> usize {
        todo!("Count active refresh tokens")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RefreshError {
    InvalidToken,
    ExpiredToken,
    TokenReused,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_issue_tokens() {
        let mut service = TokenService::new(900, 604800);
        let tokens = service.issue_tokens("user123");
        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert!(tokens.access_expires_at > tokens.refresh_expires_at - 604800);
    }

    #[test]
    fn test_refresh_basic() {
        let mut service = TokenService::new(900, 604800);
        let tokens = service.issue_tokens("user123");
        let old_refresh = tokens.refresh_token.clone();

        let new_tokens = service.refresh(&old_refresh).unwrap();
        assert_ne!(new_tokens.access_token, tokens.access_token);
        assert_ne!(new_tokens.refresh_token, tokens.refresh_token);
    }

    #[test]
    fn test_refresh_invalidates_old_token() {
        let mut service = TokenService::new(900, 604800);
        let tokens = service.issue_tokens("user123");

        // First refresh succeeds
        let _new_tokens = service.refresh(&tokens.refresh_token).unwrap();

        // Second refresh with same token fails (reuse detected)
        let result = service.refresh(&tokens.refresh_token);
        assert_eq!(result, Err(RefreshError::TokenReused));
    }

    #[test]
    fn test_refresh_reuse_revokes_family() {
        let mut service = TokenService::new(900, 604800);
        let tokens = service.issue_tokens("user123");

        // Normal refresh
        let new_tokens = service.refresh(&tokens.refresh_token).unwrap();

        // Attacker replays the original token
        let _ = service.refresh(&tokens.refresh_token);

        // The new token should also be invalidated (family revoked)
        let result = service.refresh(&new_tokens.refresh_token);
        assert!(
            result.is_err(),
            "Entire token family should be revoked after reuse detection"
        );
    }

    #[test]
    fn test_refresh_invalid_token() {
        let mut service = TokenService::new(900, 604800);
        let result = service.refresh("nonexistent_token");
        assert_eq!(result, Err(RefreshError::InvalidToken));
    }

    #[test]
    fn test_revoke_user_tokens() {
        let mut service = TokenService::new(900, 604800);
        service.issue_tokens("user1");
        service.issue_tokens("user1");
        assert_eq!(service.active_token_count(), 2);

        service.revoke_user_tokens("user1");
        assert_eq!(service.active_token_count(), 0);
    }

    #[test]
    fn test_token_family_tracking() {
        let mut service = TokenService::new(900, 604800);
        let tokens = service.issue_tokens("user123");

        // Refresh 3 times
        let mut current_refresh = tokens.refresh_token.clone();
        for _ in 0..3 {
            let new_tokens = service.refresh(&current_refresh).unwrap();
            current_refresh = new_tokens.refresh_token;
        }

        // All old tokens should be used, only the latest should be valid
        assert!(service.is_valid_refresh_token(&current_refresh));
    }
}
