//! # Lesson 08: Brute Force Defense -- Rate Limiting, Lockout, Backoff
//!
//! ## The Problem
//!
//! Even with strong hashing, an attacker can try common passwords against your
//! login endpoint. If your server responds in 100ms, an attacker can try
//! 10 passwords/second = 864,000/day. With a list of the 10,000 most common
//! passwords, they'll find matches in minutes.
//!
//! ## Defense Layers
//!
//! 1. **Rate limiting**: Cap the number of attempts per IP per time window
//! 2. **Account lockout**: Lock an account after N failed attempts
//! 3. **Exponential backoff**: Increase delay after each failed attempt
//! 4. **CAPTCHA**: Require human interaction after failures
//! 5. **Password hashing**: Even if they get in, strong hashes protect stored passwords
//!
//! ## Exponential Backoff
//!
//! After each failed attempt, double the required wait time:
//! - Attempt 1: no delay
//! - Attempt 2: 1 second
//! - Attempt 3: 2 seconds
//! - Attempt 4: 4 seconds
//! - Attempt 5: 8 seconds
//! - Attempt 6: 16 seconds
//!
//! After 20 attempts, the attacker waits 2^19 seconds (~6 days).
//!
//! ## Account Lockout
//!
//! After N consecutive failures, lock the account for a fixed period (e.g., 30 min).
//! The user must wait or use an alternative recovery method.
//!
//! ## Rate Limiting
//!
//! Track attempts per IP address. If an IP exceeds the limit within a time window,
//! reject all requests from that IP until the window resets.
//!
//! | Strategy       | Protects Against         | Downside              |
//! |----------------|--------------------------|-----------------------|
//! | Rate limiting  | Distributed attacks      | Doesn't help with botnets |
//! | Account lockout| Targeted account attacks | DoS via lockout       |
//! | Backoff        | Brute force              | Slight complexity     |
//! | CAPTCHA        | Automated attacks        | Bad UX                |
//!
//! Best practice: combine all three (rate limit + lockout + backoff) plus CAPTCHA
//! after repeated failures.

use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Exercise 1: Implement an attempt counter with exponential backoff.
///
/// After each failed login, the required wait time doubles.
/// Returns the number of seconds the caller must wait before retrying.
///
/// Hints:
/// - Track the number of failed attempts
/// - Wait time = 2^(attempts - 1) seconds, capped at max_wait_seconds
/// - After a successful login, reset the counter
pub struct LoginAttemptTracker {
    // Add fields as needed
}

impl LoginAttemptTracker {
    pub fn new() -> Self {
        todo!("Initialize LoginAttemptTracker")
    }

    /// Record a failed login attempt. Returns the required wait time in seconds.
    pub fn record_failure(&mut self, max_wait_seconds: u64) -> u64 {
        todo!("Implement failure recording with exponential backoff")
    }

    /// Record a successful login. Resets the failure counter.
    pub fn record_success(&mut self) {
        todo!("Implement success recording")
    }

    /// Get the current number of consecutive failures.
    pub fn failure_count(&self) -> u32 {
        todo!("Implement failure count getter")
    }
}

/// Exercise 2: Implement a simple rate limiter (sliding window).
///
/// Allows at most `max_attempts` in any `window_duration` time period.
/// Returns true if the attempt is allowed, false if rate-limited.
///
/// Hints:
/// - Store timestamps of recent attempts
/// - Remove timestamps older than `window_duration`
/// - Count remaining timestamps
pub struct RateLimiter {
    // Add fields as needed
}

impl RateLimiter {
    pub fn new() -> Self {
        todo!("Initialize RateLimiter")
    }

    /// Check if an attempt is allowed. If yes, record it and return true.
    /// If rate-limited, return false.
    pub fn check_and_record(&mut self, max_attempts: usize, window: Duration) -> bool {
        todo!("Implement rate limiter check")
    }

    /// Get the number of attempts in the current window.
    pub fn current_count(&self, window: Duration) -> usize {
        todo!("Implement current count")
    }
}

/// Exercise 3: Implement account lockout after N consecutive failures.
///
/// Returns Some(wait_duration) if the account is locked, None if it's not.
///
/// Hints:
/// - Track failure count and lockout expiry time
/// - After `threshold` failures, set a lockout duration
/// - If current time < lockout expiry, return remaining duration
/// - If lockout expired, reset
pub struct AccountLockout {
    // Add fields as needed
}

impl AccountLockout {
    pub fn new() -> Self {
        todo!("Initialize AccountLockout")
    }

    /// Check if the account is currently locked.
    /// Returns Some(remaining_duration) if locked, None if not.
    pub fn is_locked(&self) -> Option<Duration> {
        todo!("Implement lockout check")
    }

    /// Record a failed attempt. May trigger lockout.
    pub fn record_failure(&mut self, threshold: u32, lockout_duration: Duration) {
        todo!("Implement failure recording with lockout")
    }

    /// Record a successful login. Resets everything.
    pub fn record_success(&mut self) {
        todo!("Implement success reset")
    }
}

/// Exercise 4: Implement a combined defense system.
///
/// Combines rate limiting, exponential backoff, and account lockout.
///
/// Returns Ok(wait_seconds) if the attempt is allowed (after waiting),
/// Err(reason) if the attempt should be blocked.
///
/// Hints:
/// - Check rate limiter first
/// - Check account lockout
/// - Calculate backoff wait time
/// - Return the maximum of backoff and lockout wait
pub struct BruteForceDefense {
    pub rate_limiter: RateLimiter,
    pub lockout: AccountLockout,
    pub tracker: LoginAttemptTracker,
}

impl BruteForceDefense {
    pub fn new() -> Self {
        todo!("Initialize BruteForceDefense")
    }

    /// Attempt a login. Returns Ok(wait_before_retry) on failure,
    /// Err("locked") if account is locked, Err("rate_limited") if rate limited.
    pub fn attempt_login(&mut self, success: bool) -> Result<u64, &'static str> {
        todo!("Implement combined defense logic")
    }
}

/// Exercise 5: Generate a CAPTCHA challenge after repeated failures.
///
/// Returns Some(captcha_token) if a CAPTCHA should be shown, None otherwise.
/// Show CAPTCHA after `threshold` failures.
///
/// Hints:
/// - Track failure count
/// - If count >= threshold, generate a random token
pub fn maybe_require_captcha(failure_count: u32, threshold: u32) -> Option<String> {
    todo!("Implement CAPTCHA trigger")
}

/// Exercise 6: Implement IP-based blocking.
///
/// Given a list of failed attempt timestamps and a threshold,
/// determine if the IP should be blocked and for how long.
///
/// Returns Ok(()) if allowed, Err(block_duration) if blocked.
///
/// Hints:
/// - Count failures in the last hour
/// - If count > threshold, block for `block_duration`
pub fn check_ip_block(
    attempt_timestamps: &[u64],
    current_time: u64,
    threshold: usize,
    window_seconds: u64,
    block_duration: u64,
) -> Result<(), u64> {
    todo!("Implement IP-based blocking")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_login_tracker_exponential_backoff() {
        let mut tracker = LoginAttemptTracker::new();
        assert_eq!(tracker.record_failure(3600), 1); // 2^0 = 1
        assert_eq!(tracker.record_failure(3600), 2); // 2^1 = 2
        assert_eq!(tracker.record_failure(3600), 4); // 2^2 = 4
        assert_eq!(tracker.failure_count(), 3);
    }

    #[test]
    fn test_login_tracker_success_resets() {
        let mut tracker = LoginAttemptTracker::new();
        tracker.record_failure(3600);
        tracker.record_failure(3600);
        tracker.record_success();
        assert_eq!(tracker.failure_count(), 0);
        assert_eq!(tracker.record_failure(3600), 1); // Back to 2^0
    }

    #[test]
    fn test_login_tracker_max_wait() {
        let mut tracker = LoginAttemptTracker::new();
        for _ in 0..10 {
            tracker.record_failure(16);
        }
        // Wait time should be capped at 16
        assert_eq!(tracker.record_failure(16), 16);
    }

    #[test]
    fn test_rate_limiter_allows_within_limit() {
        let mut limiter = RateLimiter::new();
        let window = Duration::from_secs(60);
        assert!(limiter.check_and_record(5, window));
        assert!(limiter.check_and_record(5, window));
        assert!(limiter.check_and_record(5, window));
    }

    #[test]
    fn test_rate_limiter_blocks_over_limit() {
        let mut limiter = RateLimiter::new();
        let window = Duration::from_secs(60);
        for _ in 0..3 {
            assert!(limiter.check_and_record(3, window));
        }
        assert!(!limiter.check_and_record(3, window), "Should block after 3 attempts");
    }

    #[test]
    fn test_account_lockout_triggers() {
        let mut lockout = AccountLockout::new();
        assert!(lockout.is_locked().is_none());

        for _ in 0..5 {
            lockout.record_failure(5, Duration::from_secs(300));
        }
        assert!(lockout.is_locked().is_some(), "Should be locked after 5 failures");
    }

    #[test]
    fn test_account_lockout_resets_on_success() {
        let mut lockout = AccountLockout::new();
        for _ in 0..3 {
            lockout.record_failure(5, Duration::from_secs(300));
        }
        lockout.record_success();
        assert!(lockout.is_locked().is_none(), "Should not be locked after success");
    }

    #[test]
    fn test_brute_force_defense_blocks() {
        let mut defense = BruteForceDefense::new();
        // First few attempts should be allowed
        assert!(defense.attempt_login(false).is_ok());
        assert!(defense.attempt_login(false).is_ok());
    }

    #[test]
    fn test_captcha_trigger() {
        assert!(maybe_require_captcha(1, 3).is_none());
        assert!(maybe_require_captcha(3, 3).is_some());
        assert!(maybe_require_captcha(5, 3).is_some());
    }

    #[test]
    fn test_ip_block() {
        let timestamps: Vec<u64> = (0..100).map(|i| 1000 + i).collect();
        assert!(check_ip_block(&timestamps, 2000, 10, 3600, 3600).is_err());
    }

    #[test]
    fn test_ip_block_allows_normal_traffic() {
        let timestamps: Vec<u64> = vec![1000, 1010, 1020];
        assert!(check_ip_block(&timestamps, 2000, 10, 3600, 3600).is_ok());
    }
}
