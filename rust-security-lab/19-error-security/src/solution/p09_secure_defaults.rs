//! # Lesson 09: Secure Defaults (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Fail-closed authentication: deny on error.
pub fn fail_closed_auth(check_result: Result<bool, String>) -> Result<bool, String> {
    match check_result {
        Ok(access) => Ok(access),
        Err(_) => Ok(false), // Fail closed: deny on error
    }
}

/// Fail-open authentication (INSECURE -- for educational comparison only).
pub fn fail_open_auth(check_result: Result<bool, String>) -> Result<bool, String> {
    match check_result {
        Ok(access) => Ok(access),
        Err(_) => Ok(true), // Fail open: grant on error -- DANGEROUS
    }
}

/// Return config value or secure default.
pub fn secure_default(key: &str, value: Option<&str>) -> String {
    if let Some(v) = value {
        return v.to_string();
    }

    match key {
        "max_login_attempts" => "5".to_string(),
        "session_timeout_secs" => "900".to_string(),
        "require_2fa" => "true".to_string(),
        "log_level" => "warn".to_string(),
        "cors_origins" => "".to_string(),
        _ => "denied".to_string(), // Unknown config: deny
    }
}

/// Deny-by-default permission check.
pub fn check_permission(granted: &[&str], required: &str) -> Result<bool, String> {
    if granted.is_empty() || required.is_empty() {
        return Err("Invalid permission check".to_string());
    }
    Ok(granted.contains(&required))
}

/// Secure error recovery strategy.
pub fn secure_recovery<T: Clone>(
    result: Result<T, String>,
    strategy: &str,
    fallback: T,
    retry_count: u32,
    max_retries: u32,
) -> Result<T, String> {
    match result {
        Ok(val) => Ok(val),
        Err(e) => match strategy {
            "deny" => Err(e),
            "allow" => Ok(fallback), // Fail-open: use fallback as "allow"
            "retry" => {
                if retry_count < max_retries {
                    Ok(fallback) // Signal retry by returning fallback
                } else {
                    Err(e) // Max retries exceeded: fail closed
                }
            }
            "fallback" => Ok(fallback),
            _ => Err(e), // Unknown strategy: fail closed
        },
    }
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
