//! # Lesson 03: Panic Safety
//!
//! ## The Problem
//!
//! In Rust, panics unwind the stack and print a message to stderr. If that message
//! contains secrets (passwords, API keys, tokens), they end up in logs, crash reports,
//! or terminal output. Even worse, `catch_unwind` can be used to catch panics, but
//! if the panic message contains secrets, those secrets are now in the catch handler.
//!
//! ## Attack Scenarios
//!
//! 1. **Panic with secret in message**: `panic!("Invalid API key: {}", api_key)` leaks the key
//! 2. **Debug format of secret types**: `{:?}` on a struct with password fields prints them
//! 3. **unwrap() on Result with secret context**: Error messages from crypto operations
//! 4. **assert_eq! with secrets**: Test failures print both values
//!
//! ## Defense: Zeroize-on-Drop
//!
//! The `zeroize` crate overwrites memory when values are dropped, preventing secrets
//! from surviving in memory dumps or core files.
//!
//! ## Defense: Redacted Debug
//!
//! Custom `Debug` implementations that print `[REDACTED]` instead of secret values.
//!
//! ## Defense: catch_unwind
//!
//! Use `std::panic::catch_unwind` to catch panics in security-critical code paths,
//! preventing crash information from reaching users.

use std::collections::HashMap;

/// A credential that should never appear in panic messages or debug output.
#[derive(Clone)]
pub struct Credential {
    pub username: String,
    pub secret: String,  // password, API key, token, etc.
}

/// Exercise 1: Implement Debug for Credential that redacts the secret.
///
/// The Debug output must:
/// - Show the username normally
/// - Replace the secret with "[REDACTED]"
/// - Format: `Credential { username: "alice", secret: [REDACTED] }`
///
/// Hints:
/// - Implement `fmt::Formatter::debug_struct`
/// - Use `.field("secret", &"[REDACTED]")` instead of the actual value
impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("Implement Debug with redacted secret")
    }
}

/// Exercise 2: Validate credentials without panicking.
///
/// Given a username and password, validate them against a credential store.
/// Return `Ok(true)` if valid, `Ok(false)` if invalid, `Err` on internal error.
///
/// CRITICAL: This function must NEVER panic, even with:
/// - Empty username
/// - Empty password
/// - Username not in the store
///
/// The function must also NEVER include the password in any error message.
///
/// Hints:
/// - Use `.get()` on the HashMap (returns Option, doesn't panic)
/// - Return Ok(false) for missing users (don't reveal user existence)
/// - Compare passwords using constant-time comparison
pub fn validate_credentials(
    store: &HashMap<String, String>,
    username: &str,
    password: &str,
) -> Result<bool, String> {
    todo!("Validate credentials without panicking or leaking secrets")
}

/// Exercise 3: Wrap a potentially panicking operation with catch_unwind.
///
/// Given a closure that might panic, catch the panic and:
/// - Return Ok(result) if the closure succeeds
/// - Return Err(safe_message) if it panics
///
/// The error message must be a generic "Operation failed" -- never include
/// the panic payload (it might contain secrets).
///
/// Hints:
/// - Use `std::panic::catch_unwind`
/// - The closure needs `AssertUnwindSafe` wrapper or `FnOnce` + `UnwindSafe`
/// - Use `std::panic::AssertUnwindSafe(closure)` to wrap it
pub fn safe_catch_unwind<F, T>(f: F) -> Result<T, String>
where
    F: FnOnce() -> T + std::panic::UnwindSafe,
{
    todo!("Catch panics and return generic error message")
}

/// Exercise 4: Create a panic-safe assertion for secrets.
///
/// Like `assert_eq!` but for secret values. If the values don't match:
/// - Return Err with a generic message "Assertion failed"
/// - Do NOT include either value in the error message
/// - Do NOT panic
///
/// Hints:
/// - Use `==` for comparison (it's fine here since we're not doing crypto)
/// - Return Ok(()) if equal, Err if not
pub fn assert_secret_eq(a: &str, b: &str) -> Result<(), String> {
    todo!("Assert equality without revealing values on failure")
}

/// Exercise 5: Safely format an error context that might contain secrets.
///
/// Given an error message and a context HashMap that might contain secrets,
/// format a log-safe message. Keys containing "secret", "password", "token",
/// or "key" should have their values replaced with "[REDACTED]".
///
/// Format: "Error: {message} | context: {key1}={val1}, {key2}=[REDACTED], ..."
///
/// Hints:
/// - Iterate over the HashMap
/// - Check if each key (lowercase) contains any sensitive word
/// - If sensitive, use "[REDACTED]" instead of the value
pub fn safe_error_context(message: &str, context: &HashMap<String, String>) -> String {
    todo!("Format error context with redacted secrets")
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
            Ok(_) => {} // fine
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
