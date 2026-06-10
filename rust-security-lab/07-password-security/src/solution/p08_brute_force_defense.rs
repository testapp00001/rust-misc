//! # Lesson 08: Brute Force Defense -- Rate Limiting, Lockout, Backoff (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::time::{Duration, Instant};

/// Attempt counter with exponential backoff.
pub struct LoginAttemptTracker {
    failures: u32,
}

impl LoginAttemptTracker {
    pub fn new() -> Self {
        LoginAttemptTracker { failures: 0 }
    }

    /// Record a failed login attempt. Returns the required wait time in seconds.
    pub fn record_failure(&mut self, max_wait_seconds: u64) -> u64 {
        self.failures += 1;
        let wait = 2u64.saturating_pow(self.failures - 1);
        wait.min(max_wait_seconds)
    }

    /// Record a successful login. Resets the failure counter.
    pub fn record_success(&mut self) {
        self.failures = 0;
    }

    /// Get the current number of consecutive failures.
    pub fn failure_count(&self) -> u32 {
        self.failures
    }
}

/// Simple rate limiter (sliding window).
pub struct RateLimiter {
    attempts: Vec<Instant>,
}

impl RateLimiter {
    pub fn new() -> Self {
        RateLimiter { attempts: Vec::new() }
    }

    /// Check if an attempt is allowed. If yes, record it and return true.
    pub fn check_and_record(&mut self, max_attempts: usize, window: Duration) -> bool {
        let now = Instant::now();
        self.attempts.retain(|t| now.duration_since(*t) < window);
        if self.attempts.len() < max_attempts {
            self.attempts.push(now);
            true
        } else {
            false
        }
    }

    /// Get the number of attempts in the current window.
    pub fn current_count(&self, window: Duration) -> usize {
        let now = Instant::now();
        self.attempts.iter().filter(|t| now.duration_since(**t) < window).count()
    }
}

/// Account lockout after N consecutive failures.
pub struct AccountLockout {
    failures: u32,
    locked_until: Option<Instant>,
}

impl AccountLockout {
    pub fn new() -> Self {
        AccountLockout {
            failures: 0,
            locked_until: None,
        }
    }

    /// Check if the account is currently locked.
    pub fn is_locked(&self) -> Option<Duration> {
        if let Some(until) = self.locked_until {
            let now = Instant::now();
            if now < until {
                return Some(until.duration_since(now));
            }
        }
        None
    }

    /// Record a failed attempt. May trigger lockout.
    pub fn record_failure(&mut self, threshold: u32, lockout_duration: Duration) {
        self.failures += 1;
        if self.failures >= threshold {
            self.locked_until = Some(Instant::now() + lockout_duration);
        }
    }

    /// Record a successful login. Resets everything.
    pub fn record_success(&mut self) {
        self.failures = 0;
        self.locked_until = None;
    }
}

/// Combined defense system.
pub struct BruteForceDefense {
    pub rate_limiter: RateLimiter,
    pub lockout: AccountLockout,
    pub tracker: LoginAttemptTracker,
}

impl BruteForceDefense {
    pub fn new() -> Self {
        BruteForceDefense {
            rate_limiter: RateLimiter::new(),
            lockout: AccountLockout::new(),
            tracker: LoginAttemptTracker::new(),
        }
    }

    /// Attempt a login.
    pub fn attempt_login(&mut self, success: bool) -> Result<u64, &'static str> {
        // Check rate limiter first
        if !self.rate_limiter.check_and_record(10, Duration::from_secs(60)) {
            return Err("rate_limited");
        }

        // Check account lockout
        if self.lockout.is_locked().is_some() {
            return Err("locked");
        }

        if success {
            self.tracker.record_success();
            self.lockout.record_success();
            Ok(0)
        } else {
            let backoff = self.tracker.record_failure(300);
            self.lockout.record_failure(5, Duration::from_secs(300));
            Ok(backoff)
        }
    }
}

/// Generate a CAPTCHA challenge after repeated failures.
pub fn maybe_require_captcha(failure_count: u32, threshold: u32) -> Option<String> {
    if failure_count >= threshold {
        let token: [u8; 16] = rand::random();
        Some(hex::encode(token))
    } else {
        None
    }
}

/// Implement IP-based blocking.
pub fn check_ip_block(
    attempt_timestamps: &[u64],
    current_time: u64,
    threshold: usize,
    window_seconds: u64,
    block_duration: u64,
) -> Result<(), u64> {
    let window_start = current_time.saturating_sub(window_seconds);
    let recent_count = attempt_timestamps.iter()
        .filter(|&&ts| ts >= window_start && ts <= current_time)
        .count();

    if recent_count > threshold {
        Err(block_duration)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
