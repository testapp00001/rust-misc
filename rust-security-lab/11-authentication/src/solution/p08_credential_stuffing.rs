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
        Self {
            attempts: HashMap::new(),
            max_attempts,
            window,
        }
    }

    /// Check if an attempt is allowed for the given key.
    ///
    /// Returns `Ok(remaining)` with remaining attempts, or `Err(retry_after)`.
    pub fn check(&mut self, key: &str) -> Result<usize, Duration> {
        let now = Instant::now();
        let entry = self.attempts.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove expired entries
        entry.retain(|t| now.duration_since(*t) < self.window);

        if entry.len() >= self.max_attempts {
            // Calculate when the oldest attempt expires
            let oldest = entry.first().unwrap();
            let retry_after = self.window - now.duration_since(*oldest);
            return Err(retry_after);
        }

        // Record this attempt
        entry.push(now);
        Ok(self.max_attempts - entry.len())
    }

    /// Reset the attempt counter for a key (e.g., after successful login).
    pub fn reset(&mut self, key: &str) {
        self.attempts.remove(key);
    }

    /// Get the number of recent attempts for a key.
    pub fn attempt_count(&self, key: &str) -> usize {
        self.attempts.get(key).map_or(0, |v| v.len())
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
    let hash = {
        use ring::digest;
        digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, password.as_bytes())
    };
    let hex = hex::encode(hash.as_ref());
    let prefix = hex[..5].to_string();
    let suffix = hex[5..].to_string();
    (prefix, suffix)
}

/// Simulate a breach database (password hash suffixes for a given prefix).
///
/// In reality, this would be a query to the HaveIBeenPwned API.
pub fn check_breach_db(password: &str, breached_hashes: &[String]) -> bool {
    let (_prefix, suffix) = sha1_prefix(password);
    breached_hashes.iter().any(|h| h.to_uppercase() == suffix.to_uppercase())
}

/// Password strength scoring (simplified).
///
/// Returns a score from 0-100 based on:
/// - Length (longer = better)
/// - Character diversity (mixed case, digits, symbols)
/// - Not in common passwords list
pub fn password_strength(password: &str) -> u32 {
    let mut score: u32 = 0;

    // Length scoring (up to 40 points)
    score += (password.len() as u32 * 4).min(40);

    // Character diversity (up to 40 points)
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

    if has_lower { score += 10; }
    if has_upper { score += 10; }
    if has_digit { score += 10; }
    if has_symbol { score += 10; }

    // Common password penalty
    let common = ["password", "123456", "qwerty", "admin", "letmein", "welcome"];
    if common.iter().any(|&c| password.to_lowercase().contains(c)) {
        score = score.saturating_sub(30);
    }

    score.min(100)
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
        Self {
            accounts: HashMap::new(),
            max_failures,
            lockout_duration,
        }
    }

    /// Record a failed login attempt. Returns true if the account is now locked.
    pub fn record_failure(&mut self, username: &str) -> bool {
        let entry = self
            .accounts
            .entry(username.to_string())
            .or_insert((0, None));

        // Check if currently locked
        if let Some(locked_until) = entry.1 {
            if Instant::now() < locked_until {
                return true; // Still locked
            }
            // Lock expired, reset
            entry.0 = 0;
            entry.1 = None;
        }

        entry.0 += 1;

        if entry.0 >= self.max_failures {
            entry.1 = Some(Instant::now() + self.lockout_duration);
            return true;
        }

        false
    }

    /// Reset after successful login.
    pub fn record_success(&mut self, username: &str) {
        self.accounts.remove(username);
    }

    /// Check if an account is currently locked.
    pub fn is_locked(&self, username: &str) -> bool {
        if let Some((_, Some(locked_until))) = self.accounts.get(username) {
            return Instant::now() < *locked_until;
        }
        false
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
