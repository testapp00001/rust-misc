//! # Lesson 09: Secure Password Reset Flow (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;
use rand::RngCore;

/// Generate a cryptographically secure reset token.
pub fn generate_reset_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let raw_token = hex::encode(bytes);
    let hash = ring::digest::digest(&ring::digest::SHA256, &bytes);
    let token_hash = hex::encode(hash.as_ref());
    (raw_token, token_hash)
}

/// Reset token store.
pub struct ResetTokenStore {
    tokens: HashMap<String, TokenEntry>,
}

#[derive(Debug, Clone)]
pub struct TokenEntry {
    pub user_id: String,
    pub expiry: u64,
    pub used: bool,
}

impl ResetTokenStore {
    pub fn new() -> Self {
        ResetTokenStore {
            tokens: HashMap::new(),
        }
    }

    /// Store a new reset token.
    pub fn store_token(
        &mut self,
        token_hash: &str,
        user_id: &str,
        validity_seconds: u64,
        current_time: u64,
    ) {
        self.tokens.insert(token_hash.to_string(), TokenEntry {
            user_id: user_id.to_string(),
            expiry: current_time + validity_seconds,
            used: false,
        });
    }

    /// Validate and consume a token.
    pub fn validate_and_consume(
        &mut self,
        token_hash: &str,
        current_time: u64,
    ) -> Option<String> {
        if let Some(entry) = self.tokens.get_mut(token_hash) {
            if entry.used || current_time > entry.expiry {
                return None;
            }
            entry.used = true;
            Some(entry.user_id.clone())
        } else {
            None
        }
    }

    /// Invalidate all tokens for a given user.
    pub fn invalidate_all_for_user(&mut self, user_id: &str) {
        for entry in self.tokens.values_mut() {
            if entry.user_id == user_id {
                entry.used = true;
            }
        }
    }
}

/// Rate limiter for password reset requests.
pub struct ResetRateLimiter {
    last_request: HashMap<String, u64>,
}

impl ResetRateLimiter {
    pub fn new() -> Self {
        ResetRateLimiter {
            last_request: HashMap::new(),
        }
    }

    /// Check if a reset request is allowed for this email.
    pub fn check_request(&mut self, email: &str, current_time: u64) -> Result<(), u64> {
        let min_interval = 60u64;
        if let Some(&last) = self.last_request.get(email) {
            if current_time < last + min_interval {
                return Err(last + min_interval - current_time);
            }
        }
        self.last_request.insert(email.to_string(), current_time);
        Ok(())
    }
}

/// Implement the full reset request flow.
pub fn request_password_reset(
    email: &str,
    user_exists: bool,
    store: &mut ResetTokenStore,
    rate_limiter: &mut ResetRateLimiter,
    current_time: u64,
) -> Result<Option<String>, String> {
    // Always check rate limit (prevents enumeration via timing)
    if let Err(remaining) = rate_limiter.check_request(email, current_time) {
        return Err(format!("Rate limited. Try again in {} seconds", remaining));
    }

    if !user_exists {
        // Return None but don't error -- prevents user enumeration
        return Ok(None);
    }

    let (raw_token, token_hash) = generate_reset_token();
    store.store_token(&token_hash, email, 3600, current_time);
    Ok(Some(raw_token))
}

/// Implement the reset completion flow.
pub fn complete_password_reset(
    token_hash: &str,
    store: &mut ResetTokenStore,
    current_time: u64,
) -> Result<String, String> {
    let user_id = store.validate_and_consume(token_hash, current_time)
        .ok_or("Invalid or expired token")?;
    store.invalidate_all_for_user(&user_id);
    Ok(user_id)
}

/// Generate a user-friendly reset link.
pub fn generate_reset_link(domain: &str, raw_token: &str) -> String {
    format!("https://{}/reset?token={}", domain, raw_token)
}

/// Constant-time comparison for token hashes.
pub fn secure_token_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_format() {
        let (raw, hash) = generate_reset_token();
        assert_eq!(raw.len(), 64, "Raw token should be 64 hex chars (32 bytes)");
        assert_eq!(hash.len(), 64, "Hash should be 64 hex chars (32 bytes)");
        assert_ne!(raw, hash, "Raw and hash should differ");
    }

    #[test]
    fn test_generate_token_unique() {
        let (raw1, _) = generate_reset_token();
        let (raw2, _) = generate_reset_token();
        assert_ne!(raw1, raw2, "Each token should be unique");
    }

    #[test]
    fn test_token_store_valid_flow() {
        let mut store = ResetTokenStore::new();
        let (_, token_hash) = generate_reset_token();

        store.store_token(&token_hash, "user1", 3600, 1000);
        let result = store.validate_and_consume(&token_hash, 2000);
        assert_eq!(result, Some("user1".to_string()));
    }

    #[test]
    fn test_token_store_expired() {
        let mut store = ResetTokenStore::new();
        let (_, token_hash) = generate_reset_token();

        store.store_token(&token_hash, "user1", 60, 1000);
        // Try at time 2000 (way past expiry)
        let result = store.validate_and_consume(&token_hash, 2000);
        assert!(result.is_none(), "Expired token should not validate");
    }

    #[test]
    fn test_token_store_single_use() {
        let mut store = ResetTokenStore::new();
        let (_, token_hash) = generate_reset_token();

        store.store_token(&token_hash, "user1", 3600, 1000);
        assert!(store.validate_and_consume(&token_hash, 1500).is_some());
        assert!(store.validate_and_consume(&token_hash, 1500).is_none(), "Used token should not validate again");
    }

    #[test]
    fn test_invalidate_all_for_user() {
        let mut store = ResetTokenStore::new();
        let (_, hash1) = generate_reset_token();
        let (_, hash2) = generate_reset_token();

        store.store_token(&hash1, "user1", 3600, 1000);
        store.store_token(&hash2, "user1", 3600, 1000);
        store.invalidate_all_for_user("user1");

        assert!(store.validate_and_consume(&hash1, 1500).is_none());
        assert!(store.validate_and_consume(&hash2, 1500).is_none());
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = ResetRateLimiter::new();
        assert!(limiter.check_request("user@example.com", 1000).is_ok());
        assert!(limiter.check_request("user@example.com", 1030).is_err());
        assert!(limiter.check_request("user@example.com", 1061).is_ok());
    }

    #[test]
    fn test_reset_link_format() {
        let link = generate_reset_link("example.com", "abc123token");
        assert_eq!(link, "https://example.com/reset?token=abc123token");
    }

    #[test]
    fn test_secure_token_compare() {
        assert!(secure_token_compare("abc123", "abc123"));
        assert!(!secure_token_compare("abc123", "xyz789"));
    }

    #[test]
    fn test_full_reset_flow() {
        let mut store = ResetTokenStore::new();
        let mut limiter = ResetRateLimiter::new();

        // Request reset
        let token = request_password_reset(
            "user@example.com", true, &mut store, &mut limiter, 1000,
        ).unwrap();

        assert!(token.is_some(), "Should return token for existing user");

        // Complete reset
        let (_, token_hash) = generate_reset_token();
        store.store_token(&token_hash, "user1", 3600, 1000);
        let user_id = complete_password_reset(&token_hash, &mut store, 1500).unwrap();
        assert_eq!(user_id, "user1");
    }
}
