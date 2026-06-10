//! # Lesson 09: Secure Defaults
//!
//! ## The Problem
//!
//! When code encounters an unexpected error, what should it do? Two philosophies:
//!
//! **Fail-open**: "If something goes wrong, allow access / use defaults / continue"
//! This is dangerous because errors become bypass mechanisms.
//!
//! **Fail-closed**: "If something goes wrong, deny access / reject request / stop"
//! This is safe because errors become denial mechanisms.
//!
//! ## Real-World Fail-Open Disasters
//!
//! | Scenario | Fail-Open Behavior | Impact |
//! |----------|-------------------|--------|
//! | Auth middleware error | Returns Ok(()) | Unauthenticated access |
//! | Rate limiter Redis down | Allows all requests | DDoS amplification |
//! | TLS cert check error | Accepts any cert | MITM attacks |
//! | Permission DB timeout | Grants access | Privilege escalation |
//! | Input validation error | Accepts raw input | Injection attacks |
//!
//! ## Defense: Always Fail Closed
//!
//! Every error path must deny access, reject input, or halt processing.
//! Default to the most restrictive behavior.

/// Exercise 1: Implement a fail-closed authentication check.
///
/// Given a Result from an auth check, return:
/// - Ok(true) if the check succeeded and access is granted
/// - Ok(false) if the check succeeded but access is denied
/// - Err("Access denied") if the check itself failed (fail closed!)
///
/// NEVER return Ok(true) on error -- that's fail-open.
///
/// Hints:
/// - Match on the Result
/// - For Err, return Ok(false) or Err -- never Ok(true)
pub fn fail_closed_auth(check_result: Result<bool, String>) -> Result<bool, String> {
    todo!("Implement fail-closed authentication")
}

/// Exercise 2: Implement a fail-open authentication check (INSECURE).
///
/// This demonstrates the dangerous pattern. Given a Result from an auth check:
/// - Ok(true) → Ok(true) -- access granted
/// - Ok(false) → Ok(false) -- access denied
/// - Err → Ok(true) -- ERROR BECOMES ACCESS (fail-open!)
///
/// DO NOT USE THIS IN PRODUCTION. This is for educational comparison only.
pub fn fail_open_auth(check_result: Result<bool, String>) -> Result<bool, String> {
    todo!("Implement fail-open authentication (INSECURE)")
}

/// Exercise 3: Implement a secure default configuration.
///
/// Given an optional configuration value, return the value if present,
/// or a secure default if absent. The secure defaults are:
///
/// | Config | Secure Default | Why |
/// |--------|---------------|-----|
/// | `max_login_attempts` | 5 | Prevent brute force |
/// | `session_timeout_secs` | 900 (15 min) | Limit session hijack window |
/// | `require_2fa` | true | Defense in depth |
/// | `log_level` | "warn" | Don't log secrets at debug level |
/// | `cors_origins` | empty | Deny all cross-origin by default |
///
/// Given a config key and optional value, return the value or the secure default.
/// If the key is unknown, return "denied" (fail closed).
pub fn secure_default(key: &str, value: Option<&str>) -> String {
    todo!("Return config value or secure default")
}

/// Exercise 4: Implement a permission check with deny-by-default.
///
/// Given a list of granted permissions and a required permission, check access.
/// Rules:
/// - If the required permission is in the granted list → Ok(true)
/// - If the required permission is NOT in the list → Ok(false)
/// - If the granted list is empty or the required permission is empty → Err("Invalid permission check")
///
/// The key principle: if anything is uncertain, DENY.
pub fn check_permission(granted: &[&str], required: &str) -> Result<bool, String> {
    todo!("Implement deny-by-default permission check")
}

/// Exercise 5: Implement a secure error recovery strategy.
///
/// Given an operation result and a recovery strategy, determine the outcome.
///
/// Recovery strategies:
/// - "deny": Always return Err on failure (fail closed)
/// - "allow": Return Ok on failure (fail open -- INSECURE)
/// - "retry": Return Ok(retry_count + 1) on failure, capped at max_retries
/// - "fallback": Return Ok(fallback_value) on failure
///
/// On success, always return the original Ok value.
///
/// If the strategy is unknown, default to "deny" (fail closed).
pub fn secure_recovery<T: Clone>(
    result: Result<T, String>,
    strategy: &str,
    fallback: T,
    retry_count: u32,
    max_retries: u32,
) -> Result<T, String> {
    todo!("Implement secure error recovery strategy")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_closed_success_granted() {
        let result = fail_closed_auth(Ok(true));
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_fail_closed_success_denied() {
        let result = fail_closed_auth(Ok(false));
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn test_fail_closed_error_denies() {
        let result = fail_closed_auth(Err("database timeout".to_string()));
        // Must NOT be Ok(true)
        assert_ne!(result, Ok(true), "Fail-closed must not grant access on error");
    }

    #[test]
    fn test_fail_open_grants_on_error() {
        let result = fail_open_auth(Err("database timeout".to_string()));
        assert_eq!(result, Ok(true), "Fail-open dangerously grants access on error");
    }

    #[test]
    fn test_secure_default_known_key() {
        assert_eq!(secure_default("max_login_attempts", Some("10")), "10");
    }

    #[test]
    fn test_secure_default_missing_key() {
        assert_eq!(secure_default("max_login_attempts", None), "5");
    }

    #[test]
    fn test_secure_default_unknown_key() {
        assert_eq!(secure_default("unknown_setting", None), "denied");
    }

    #[test]
    fn test_permission_granted() {
        let result = check_permission(&["read", "write"], "read");
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_permission_denied() {
        let result = check_permission(&["read"], "admin");
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn test_permission_empty_granted_list() {
        let result = check_permission(&[], "read");
        assert!(result.is_err(), "Empty granted list should error");
    }

    #[test]
    fn test_recovery_deny_on_failure() {
        let result: Result<i32, String> = Err("fail".to_string());
        let recovered = secure_recovery(result, "deny", 0, 0, 3);
        assert!(recovered.is_err(), "Deny strategy should propagate error");
    }

    #[test]
    fn test_recovery_fallback_on_failure() {
        let result: Result<i32, String> = Err("fail".to_string());
        let recovered = secure_recovery(result, "fallback", 42, 0, 3);
        assert_eq!(recovered, Ok(42));
    }

    #[test]
    fn test_recovery_unknown_strategy_denies() {
        let result: Result<i32, String> = Err("fail".to_string());
        let recovered = secure_recovery(result, "unknown_strategy", 42, 0, 3);
        assert!(recovered.is_err(), "Unknown strategy should default to deny");
    }

    #[test]
    fn test_recovery_success_unchanged() {
        let result: Result<i32, String> = Ok(100);
        let recovered = secure_recovery(result, "deny", 0, 0, 3);
        assert_eq!(recovered, Ok(100));
    }
}
