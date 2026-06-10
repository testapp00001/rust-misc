//! # Lesson 08: Credential Stuffing Defense — Rate Limiting, Breach Detection
//!
//! ## What is Credential Stuffing?
//!
//! Attackers use leaked username/password pairs from breach A to attack service B.
//! - 15+ billion credentials circulate in underground markets
//! - Automated tools try thousands of combinations per minute
//! - Success rate: 0.1-2% (enough for mass compromise)
//!
//! ## Defense Layers
//!
//! 1. **Rate limiting**: Slow down automated attempts
//! 2. **Breach detection**: Check passwords against known breaches (HaveIBeenPwned)
//! 3. **Account lockout**: Lock after N failed attempts
//! 4. **Anomaly detection**: Flag unusual login patterns
//! 5. **MFA**: Even if password is correct, attacker needs second factor
//!
//! ## Rate Limiting Strategies
//!
//! | Strategy | Description |
//! |----------|-------------|
//! | Fixed window | N attempts per minute (burst at window boundary) |
//! | Sliding window | N attempts in rolling time period |
//! | Token bucket | Accumulate tokens over time, spend on attempts |
//! | Per-IP | Limit by source IP address |
//! | Per-account | Limit by target username |

use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// A rate limiter using the sliding window approach.
///
/// Tracks attempt timestamps per key (IP address or username)
/// and rejects requests when the limit is exceeded.
pub struct RateLimiter {
    /// Key → list of attempt timestamps
    attempts: HashMap<String, Vec<Instant>>,
    /// Maximum attempts allowed in the window
    max_attempts: usize,
    /// Window duration
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_attempts: usize, window: Duration) -> Self {
        todo!("Create a new RateLimiter")
    }

    /// Check if an attempt is allowed for the given key.
    ///
    /// Returns `Ok(remaining)` with remaining attempts, or `Err(retry_after)`.
    pub fn check(&mut self, key: &str) -> Result<usize, Duration> {
        todo!("Check rate limit and record attempt")
    }

    /// Reset the attempt counter for a key (e.g., after successful login).
    pub fn reset(&mut self, key: &str) {
        todo!("Reset attempt counter for a key")
    }

    /// Get the number of recent attempts for a key.
    pub fn attempt_count(&self, key: &str) -> usize {
        todo!("Get number of recent attempts")
    }
}

/// Compute the SHA-1 hash prefix for HaveIBeenPwned API lookup.
///
/// The k-anonymity model:
/// 1. Compute SHA-1 of the password
/// 2. Send only the first 5 hex characters (prefix)
/// 3. Server returns all hashes with that prefix
/// 4. Client checks if the full hash is in the response
///
/// This way the server never learns the full password hash.
pub fn sha1_prefix(password: &str) -> (String, String) {
    todo!("Compute SHA-1 hash and split into prefix/suffix")
}

/// Simulate a breach database (password hash suffixes for a given prefix).
///
/// In reality, this would be a query to the HaveIBeenPwned API.
pub fn check_breach_db(password: &str, breached_hashes: &[String]) -> bool {
    todo!("Check if password hash appears in breach database")
}

/// Password strength scoring (simplified).
///
/// Returns a score from 0-100 based on:
/// - Length (longer = better)
/// - Character diversity (mixed case, digits, symbols)
/// - Not in common passwords list
pub fn password_strength(password: &str) -> u32 {
    todo!("Score password strength from 0-100")
}

/// Account lockout tracking.
pub struct AccountLockout {
    /// Username → (failed_count, locked_until)
    accounts: HashMap<String, (usize, Option<Instant>)>,
    max_failures: usize,
    lockout_duration: Duration,
}

impl AccountLockout {
    pub fn new(max_failures: usize, lockout_duration: Duration) -> Self {
        todo!("Create a new AccountLockout tracker")
    }

    /// Record a failed login attempt. Returns true if the account is now locked.
    pub fn record_failure(&mut self, username: &str) -> bool {
        todo!("Record a failed login and check lockout")
    }

    /// Reset after successful login.
    pub fn record_success(&mut self, username: &str) {
        todo!("Reset failure counter after successful login")
    }

    /// Check if an account is currently locked.
    pub fn is_locked(&self, username: &str) -> bool {
        todo!("Check if account is currently locked")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let mut limiter = RateLimiter::new(3, Duration::from_secs(60));
        assert!(limiter.check("user1").is_ok());
        assert!(limiter.check("user1").is_ok());
        assert!(limiter.check("user1").is_ok());
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let mut limiter = RateLimiter::new(2, Duration::from_secs(60));
        assert!(limiter.check("user1").is_ok());
        assert!(limiter.check("user1").is_ok());
        assert!(limiter.check("user1").is_err());
    }

    #[test]
    fn test_rate_limiter_per_key() {
        let mut limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.check("user1").is_ok());
        assert!(limiter.check("user2").is_ok()); // Different key
        assert!(limiter.check("user1").is_err());
    }

    #[test]
    fn test_rate_limiter_reset() {
        let mut limiter = RateLimiter::new(1, Duration::from_secs(60));
        assert!(limiter.check("user1").is_ok());
        assert!(limiter.check("user1").is_err());
        limiter.reset("user1");
        assert!(limiter.check("user1").is_ok());
    }

    #[test]
    fn test_rate_limiter_window_expiry() {
        let mut limiter = RateLimiter::new(1, Duration::from_millis(50));
        assert!(limiter.check("user1").is_ok());
        assert!(limiter.check("user1").is_err());
        thread::sleep(Duration::from_millis(60));
        assert!(limiter.check("user1").is_ok());
    }

    #[test]
    fn test_sha1_prefix_format() {
        let (prefix, suffix) = sha1_prefix("password");
        assert_eq!(prefix.len(), 5, "Prefix should be 5 hex chars");
        assert!(prefix.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(suffix.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_breach_detection() {
        // "password" has been breached billions of times
        let (_prefix, suffix) = sha1_prefix("password");
        let breached = vec![suffix];
        assert!(check_breach_db("password", &breached));
    }

    #[test]
    fn test_breach_not_found() {
        let unique = "xK9!mZ2@qR7#nL5^bT8&jW3*eY6%";
        let breached = vec!["000000000000000000000000000000000000000".to_string()];
        assert!(!check_breach_db(unique, &breached));
    }

    #[test]
    fn test_password_strength_strong() {
        let score = password_strength("xK9!mZ2@qR7#nL5^bT8&");
        assert!(score >= 80, "Strong password should score >= 80, got {}", score);
    }

    #[test]
    fn test_password_strength_weak() {
        let score = password_strength("password");
        assert!(score <= 30, "Weak password should score <= 30, got {}", score);
    }

    #[test]
    fn test_account_lockout() {
        let mut lockout = AccountLockout::new(3, Duration::from_secs(300));
        assert!(!lockout.record_failure("user1"));
        assert!(!lockout.record_failure("user1"));
        assert!(lockout.record_failure("user1")); // Locked after 3rd
        assert!(lockout.is_locked("user1"));
    }

    #[test]
    fn test_account_lockout_success_resets() {
        let mut lockout = AccountLockout::new(3, Duration::from_secs(300));
        lockout.record_failure("user1");
        lockout.record_failure("user1");
        lockout.record_success("user1");
        assert!(!lockout.is_locked("user1"));
    }
}
