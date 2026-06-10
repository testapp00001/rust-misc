//! # Lesson 05: Timing Oracle in Error Handling (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

/// Simulated password hash storage: username -> hash
pub type HashStore = HashMap<String, String>;

/// A dummy bcrypt hash to use when the user doesn't exist.
const DUMMY_HASH: &str = "$2b$12$LJ3m4ys3Lg4LPLACEHOLDERDUMMYHASHXXXXXXXXXXXXXXXXXXXXXXXX";

/// Simulate bcrypt verification.
pub fn simulate_bcrypt_verify(password: &str, hash: &str) -> bool {
    if hash.starts_with("$2b$") || hash.starts_with("$2y$") {
        let iterations = 10_000u64;
        let mut _sum = 0u64;
        for i in 0..iterations {
            _sum = _sum.wrapping_add(i);
        }
        hash.contains(password)
    } else {
        false
    }
}

/// VULNERABLE authentication (for comparison).
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

/// Constant-time authentication -- always runs bcrypt, even for non-existent users.
pub fn authenticate_constant_time(
    store: &HashStore,
    username: &str,
    password: &str,
) -> Result<String, String> {
    // Always get a hash -- use dummy if user doesn't exist
    let hash = match store.get(username) {
        Some(h) => h.clone(),
        None => DUMMY_HASH.to_string(),
    };

    // Always run bcrypt verification (expensive operation)
    let verified = simulate_bcrypt_verify(password, &hash);

    // Only succeed if user exists AND password matches
    if store.contains_key(username) && verified {
        Ok(format!("Welcome, {}!", username))
    } else {
        Err("Authentication failed".to_string())
    }
}

/// Check if two timings are within tolerance percentage.
pub fn timing_similar(a_ns: u64, b_ns: u64, tolerance_percent: u64) -> bool {
    if a_ns == 0 && b_ns == 0 {
        return true;
    }
    let max_val = a_ns.max(b_ns);
    let diff = if a_ns > b_ns { a_ns - b_ns } else { b_ns - a_ns };
    // diff / max * 100 <= tolerance
    diff * 100 <= max_val * tolerance_percent
}

/// Normalize timing by padding to target duration.
pub fn normalize_timing<T>(result: T, elapsed_ns: u64, target_ns: u64) -> (T, u64) {
    if elapsed_ns >= target_ns {
        return (result, elapsed_ns);
    }
    let padding = target_ns - elapsed_ns;
    // Spin loop for padding (in production, use tokio::time::sleep)
    let iterations = padding / 10; // approximate
    let mut _sum = 0u64;
    for i in 0..iterations {
        _sum = _sum.wrapping_add(i);
    }
    (result, elapsed_ns + padding)
}

/// Audit authentication timing for oracle vulnerabilities.
pub fn audit_timing(attempts: &[(&str, &str, u64)]) -> Result<(), String> {
    if attempts.len() < 2 {
        return Ok(());
    }

    let first = attempts[0].2;
    for (username, _password, duration) in &attempts[1..] {
        if !timing_similar(first, *duration, 50) {
            return Err(format!(
                "Timing oracle detected: first attempt {}ns vs {} for '{}' ({}ns difference)",
                first,
                username,
                duration,
                if *duration > first { duration - first } else { first - duration }
            ));
        }
    }
    Ok(())
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
        let r1 = authenticate_vulnerable(&store, "alice", "wrong");
        let r2 = authenticate_vulnerable(&store, "nonexistent", "wrong");
        assert!(r1.is_err());
        assert!(r2.is_err());
    }

    #[test]
    fn test_constant_time_runs_bcrypt_for_missing_user() {
        let store = build_store();
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
        let attempts = vec![
            ("alice", "pw", 100_000),
            ("bob", "pw", 105_000),
            ("charlie", "pw", 98_000),
        ];
        assert!(audit_timing(&attempts).is_ok());
    }

    #[test]
    fn test_audit_timing_detects_oracle() {
        let attempts = vec![
            ("alice", "pw", 100_000),
            ("nonexistent", "pw", 2_000),
        ];
        assert!(audit_timing(&attempts).is_err());
    }
}
