//! # Lesson 05: Timing Oracle in Error Handling
//!
//! ## The Problem
//!
//! Even when error messages are identical, the *time* to respond can reveal information.
//! If checking a valid user takes 100ms (DB lookup + bcrypt) but an invalid user takes
//! 2ms (early return), an attacker measuring response times can distinguish them.
//!
//! ## Attack Walkthrough
//!
//! ```text
//! # Server code (VULNERABLE):
//! fn authenticate(user, pass):
//!     record = db.find_user(user)      # 50ms if exists, 2ms if not
//!     if record.is_none():
//!         return Err("Auth failed")    # 2ms total
//!     verify_bcrypt(pass, record.hash) # 100ms
//!     return Err("Auth failed")        # 150ms total
//!
//! # Attacker measures response times:
//! Request 1 (alice):    152ms  → alice exists (took bcrypt path)
//! Request 2 (nobody):     4ms  → nobody doesn't exist (early return)
//! ```
//!
//! ## Defense: Constant-Time Authentication
//!
//! Always run the expensive operation (bcrypt) even for non-existent users.
//! Use a dummy hash so the timing is indistinguishable.
//!
//! ```text
//! fn authenticate(user, pass):
//!     record = db.find_user(user)          # 50ms (or 2ms)
//!     hash = record.hash OR DUMMY_HASH     # always use a hash
//!     verify_bcrypt(pass, hash)            # always 100ms
//!     if record.is_none() OR !verified:
//!         return Err("Auth failed")        # always ~152ms
//! ```

use std::collections::HashMap;

/// Simulated password hash storage: username -> hash
pub type HashStore = HashMap<String, String>;

/// A dummy bcrypt hash to use when the user doesn't exist.
/// This ensures the bcrypt verification step always runs.
const DUMMY_HASH: &str = "$2b$12$LJ3m4ys3Lg4LPLACEHOLDERDUMMYHASHXXXXXXXXXXXXXXXXXXXXXXXX";

/// Simulate bcrypt verification.
/// In real code, this uses `bcrypt::verify`. Here we simulate the timing:
/// - If hash starts with "$2b$", it takes `cost_ms` milliseconds
/// - Otherwise, it returns immediately (simulating invalid hash format)
///
/// Returns true if password matches hash.
pub fn simulate_bcrypt_verify(password: &str, hash: &str) -> bool {
    if hash.starts_with("$2b$") || hash.starts_with("$2y$") {
        // Simulate bcrypt work: sleep for a short time
        // In tests we use a spin loop instead of actual sleep
        let iterations = 10_000u64;
        let mut _sum = 0u64;
        for i in 0..iterations {
            _sum = _sum.wrapping_add(i);
        }
        // Simple comparison (real bcrypt does much more)
        hash.contains(password)
    } else {
        false
    }
}

/// Exercise 1: VULNERABLE authentication (for comparison).
///
/// This function has a timing oracle: it returns early when the user doesn't exist,
/// skipping the expensive bcrypt verification.
///
/// DO NOT FIX THIS -- it demonstrates the vulnerability.
pub fn authenticate_vulnerable(
    store: &HashStore,
    username: &str,
    password: &str,
) -> Result<String, String> {
    match store.get(username) {
        None => return Err("Authentication failed".to_string()),
        Some(hash) => {
            if simulate_bcrypt_verify(password, hash) {
                Ok(format!("Welcome, {}!", username))
            } else {
                Err("Authentication failed".to_string())
            }
        }
    }
}

/// Exercise 2: Implement constant-time authentication.
///
/// The function must:
/// - Always run `simulate_bcrypt_verify` even if the user doesn't exist
/// - Use `DUMMY_HASH` when no user is found
/// - Return the same error message for all failures
/// - Return Ok(username) only on success
///
/// This makes the response time the same regardless of whether the user exists.
///
/// Hints:
/// - Get the hash from the store, or use DUMMY_HASH as fallback
/// - Always call simulate_bcrypt_verify
/// - Only succeed if both user exists AND password matches
pub fn authenticate_constant_time(
    store: &HashStore,
    username: &str,
    password: &str,
) -> Result<String, String> {
    todo!("Implement constant-time authentication")
}

/// Exercise 3: Measure if two operations have similar timing.
///
/// Given two durations in nanoseconds, return true if they are within
/// the given tolerance percentage of each other.
///
/// Formula: |a - b| / max(a, b) * 100 <= tolerance_percent
///
/// If both are 0, return true (identical).
pub fn timing_similar(a_ns: u64, b_ns: u64, tolerance_percent: u64) -> bool {
    todo!("Check if two timings are within tolerance")
}

/// Exercise 4: Add artificial delay to normalize response time.
///
/// Given an operation that took `elapsed_ns` nanoseconds, and a target duration
/// `target_ns`, sleep (spin) for the difference if the operation was faster.
///
/// This is an alternative to the "always run bcrypt" approach: run the fast path
/// but pad the time to match the slow path.
///
/// Return the total time (elapsed + padding) as a tuple of (operation_result, total_ns).
/// The total should be >= target_ns.
pub fn normalize_timing<T>(result: T, elapsed_ns: u64, target_ns: u64) -> (T, u64) {
    todo!("Pad timing to normalize response duration")
}

/// Exercise 5: Audit authentication timing for oracle vulnerabilities.
///
/// Given a list of (username, password, duration_ns) from auth attempts,
/// check if there's a significant timing difference between valid and invalid users.
///
/// Return:
/// - Ok(()) if all timings are similar (within 50% tolerance)
/// - Err(description) if timings differ significantly
pub fn audit_timing(attempts: &[(&str, &str, u64)]) -> Result<(), String> {
    todo!("Audit authentication timing for oracle vulnerabilities")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_store() -> HashStore {
        let mut store = HashMap::new();
        store.insert("alice".to_string(), "$2b$12$alicehashplaceholderXXXXXXXXXXXXXXXXXXXXX".to_string());
        store.insert("bob".to_string(), "$2b$12$bobhashplaceholderXXXXXXXXXXXXXXXXXXXXXXX".to_string());
        store
    }

    #[test]
    fn test_vulnerable_early_return() {
        let store = build_store();
        // Both should fail, but the vulnerable version returns early for non-existent users
        let r1 = authenticate_vulnerable(&store, "alice", "wrong");
        let r2 = authenticate_vulnerable(&store, "nonexistent", "wrong");
        assert!(r1.is_err());
        assert!(r2.is_err());
        // The timing difference is the vulnerability (tested conceptually here)
    }

    #[test]
    fn test_constant_time_runs_bcrypt_for_missing_user() {
        let store = build_store();
        // This should not panic or fail -- it should use DUMMY_HASH
        let result = authenticate_constant_time(&store, "nonexistent", "password");
        assert!(result.is_err(), "Non-existent user should fail");
        assert_eq!(result.unwrap_err(), "Authentication failed");
    }

    #[test]
    fn test_constant_time_same_error_messages() {
        let store = build_store();
        let r1 = authenticate_constant_time(&store, "alice", "wrong").unwrap_err();
        let r2 = authenticate_constant_time(&store, "nonexistent", "wrong").unwrap_err();
        assert_eq!(r1, r2, "Error messages must be identical");
    }

    #[test]
    fn test_constant_time_success() {
        let store = build_store();
        let result = authenticate_constant_time(&store, "alice", "alicehashplaceholder");
        assert!(result.is_ok(), "Valid credentials should succeed");
    }

    #[test]
    fn test_timing_similar_identical() {
        assert!(timing_similar(100, 100, 50));
    }

    #[test]
    fn test_timing_similar_within_tolerance() {
        assert!(timing_similar(100, 140, 50)); // 40% difference
    }

    #[test]
    fn test_timing_similar_outside_tolerance() {
        assert!(!timing_similar(10, 100, 50)); // 900% difference
    }

    #[test]
    fn test_timing_similar_both_zero() {
        assert!(timing_similar(0, 0, 50));
    }

    #[test]
    fn test_normalize_timing_pads() {
        let (result, total) = normalize_timing("ok", 10, 100);
        assert_eq!(result, "ok");
        assert!(total >= 100, "Total time should be at least target: got {}", total);
    }

    #[test]
    fn test_normalize_timing_no_pad_needed() {
        let (result, total) = normalize_timing("ok", 150, 100);
        assert_eq!(result, "ok");
        assert!(total >= 150, "Should not reduce time below elapsed");
    }

    #[test]
    fn test_audit_timing_secure() {
        // All similar timings -- secure
        let attempts = vec![
            ("alice", "pw", 100_000),
            ("bob", "pw", 105_000),
            ("charlie", "pw", 98_000),
        ];
        assert!(audit_timing(&attempts).is_ok());
    }

    #[test]
    fn test_audit_timing_detects_oracle() {
        // Very different timings -- vulnerable
        let attempts = vec![
            ("alice", "pw", 100_000),
            ("nonexistent", "pw", 2_000),
        ];
        assert!(audit_timing(&attempts).is_err());
    }
}
