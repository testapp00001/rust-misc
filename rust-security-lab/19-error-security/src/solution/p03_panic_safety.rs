//! # Lesson 03: Panic Safety (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

/// A credential that should never appear in panic messages or debug output.
#[derive(Clone)]
pub struct Credential {
    pub username: String,
    pub secret: String,
}

/// Debug for Credential redacts the secret.
impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential")
            .field("username", &self.username)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

/// Validate credentials without panicking or leaking secrets.
pub fn validate_credentials(
    store: &HashMap<String, String>,
    username: &str,
    password: &str,
) -> Result<bool, String> {
    // Use .get() -- returns None for missing users, never panics
    match store.get(username) {
        Some(stored_password) => {
            // Constant-time comparison to prevent timing attacks
            Ok(constant_time_eq(password, stored_password))
        }
        None => {
            // Return Ok(false), not Err -- don't reveal user existence
            Ok(false)
        }
    }
}

/// Constant-time string comparison.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Wrap a potentially panicking operation with catch_unwind.
pub fn safe_catch_unwind<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    match std::panic::catch_unwind(f) {
        Ok(result) => Ok(result),
        Err(_) => Err("Operation failed".to_string()),
    }
}

/// Assert equality without revealing values on failure.
pub fn assert_secret_eq(a: &str, b: &str) -> Result<(), String> {
    if a == b {
        Ok(())
    } else {
        Err("Assertion failed".to_string())
    }
}

/// Format error context with redacted secrets.
pub fn safe_error_context(message: &str, context: &HashMap<String, String>) -> String {
    let sensitive_words = ["secret", "password", "token", "key"];
    let mut pairs: Vec<String> = Vec::new();

    for (k, v) in context {
        let lower_key = k.to_lowercase();
        let is_sensitive = sensitive_words.iter().any(|w| lower_key.contains(w));
        if is_sensitive {
            pairs.push(format!("{}=[REDACTED]", k));
        } else {
            pairs.push(format!("{}={}", k, v));
        }
    }

    pairs.sort(); // Sort for deterministic output in tests
    format!("Error: {} | context: {}", message, pairs.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_redacts_secret() {
        let cred = Credential {
            username: "alice".to_string(),
            secret: "super_secret_password".to_string(),
        };
        let debug = format!("{:?}", cred);
        assert!(!debug.contains("super_secret_password"), "Debug must not contain secret");
        assert!(debug.contains("REDACTED"), "Debug should contain [REDACTED]");
        assert!(debug.contains("alice"), "Debug should contain username");
    }

    #[test]
    fn test_validate_valid_credentials() {
        let mut store = HashMap::new();
        store.insert("alice".to_string(), "correct_password".to_string());
        let result = validate_credentials(&store, "alice", "correct_password");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);
    }

    #[test]
    fn test_validate_invalid_password() {
        let mut store = HashMap::new();
        store.insert("alice".to_string(), "correct_password".to_string());
        let result = validate_credentials(&store, "alice", "wrong_password");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_validate_unknown_user_no_panic() {
        let store = HashMap::new();
        let result = validate_credentials(&store, "unknown_user", "password");
        assert!(result.is_ok(), "Must not panic for unknown user");
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_validate_empty_username_no_panic() {
        let store = HashMap::new();
        let result = validate_credentials(&store, "", "password");
        assert!(result.is_ok(), "Must not panic for empty username");
    }

    #[test]
    fn test_validate_error_no_password_leak() {
        let store = HashMap::new();
        let result = validate_credentials(&store, "alice", "secret123");
        match result {
            Ok(_) => {}
            Err(e) => {
                assert!(!e.contains("secret123"), "Error must not contain password");
            }
        }
    }

    #[test]
    fn test_catch_unwind_success() {
        let result = safe_catch_unwind(|| 42);
        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_catch_unwind_catches_panic() {
        let result: Result<i32, String> = safe_catch_unwind(|| {
            panic!("secret_api_key_12345");
        });
        assert!(result.is_err(), "Should catch the panic");
        let err = result.unwrap_err();
        assert!(!err.contains("secret_api_key_12345"), "Error must not contain panic payload");
        assert!(err.contains("Operation failed") || err.contains("failed"),
                "Error should be generic");
    }

    #[test]
    fn test_assert_secret_eq_match() {
        assert!(assert_secret_eq("same", "same").is_ok());
    }

    #[test]
    fn test_assert_secret_eq_mismatch_no_leak() {
        let result = assert_secret_eq("actual_password", "wrong_password");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.contains("actual_password"), "Error must not reveal first value");
        assert!(!err.contains("wrong_password"), "Error must not reveal second value");
    }

    #[test]
    fn test_safe_error_context_redacts() {
        let mut context = HashMap::new();
        context.insert("user".to_string(), "alice".to_string());
        context.insert("password".to_string(), "secret123".to_string());
        context.insert("api_token".to_string(), "tok_abc123".to_string());

        let msg = safe_error_context("auth failed", &context);
        assert!(!msg.contains("secret123"), "Must not contain password value");
        assert!(!msg.contains("tok_abc123"), "Must not contain token value");
        assert!(msg.contains("alice"), "Should contain non-secret values");
        assert!(msg.contains("REDACTED"), "Should contain REDACTED marker");
    }
}
