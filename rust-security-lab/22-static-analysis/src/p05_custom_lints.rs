//! # Lesson 05: Custom Clippy Lints
//!
//! ## The Problem
//!
//! Built-in Clippy lints catch generic issues. But every project has
//! project-specific security rules that no standard lint covers:
//!
//! - "Never use `HashMap` for token storage (timing attacks)"
//! - "All error types must redact sensitive fields"
//! - "Log messages must not contain PII"
//! - "Database queries must use parameterized statements"
//!
//! ## Custom Lints with Dylint
//!
//! `cargo-dylint` lets you write project-specific lints as Rust libraries:
//!
//! ```bash
//! cargo install cargo-dylint dylint-link
//! ```
//!
//! A custom lint is a `rustc` driver plugin that inspects the AST/HIR and
//! emits warnings or errors for violations.
//!
//! ## Exercise
//!
//! Implement the *logic* that custom lints would enforce. We cannot run
//! `rustc` plugins in normal tests, but we can implement the analysis functions
//! that would power such lints.

/// Exercise 1: Detect usage of forbidden function names.
///
/// A custom lint might ban `.unwrap()` project-wide. This function checks
/// whether a code snippet contains calls to any forbidden function.
///
/// Requirements:
/// - Check if `code` contains any function name from `forbidden` list
/// - Match as substring: if "unwrap" is forbidden, "result.unwrap()" matches
/// - Return a list of found violations (the forbidden strings found)
/// - Case-sensitive matching
pub fn find_forbidden_calls<'a>(code: &'a str, forbidden: &[&str]) -> Vec<&'a str> {
    todo!("Find forbidden function calls in code")
}

/// Exercise 2: Check that all error handling uses `Result`, not `panic!`.
///
/// Requirements:
/// - Return `true` if `code` contains `panic!`, `todo!`, `unimplemented!`, or `unreachable!`
/// - Return `false` if none of these macros appear
/// - This represents a lint that bans panic-causing macros
pub fn contains_panic_macros(code: &str) -> bool {
    todo!("Check for panic-causing macros")
}

/// Exercise 3: Validate that a function signature uses proper error handling.
///
/// Requirements:
/// - Check if `signature` contains "-> Result" (indicating proper error handling)
/// - Return `true` if the signature uses Result, `false` otherwise
/// - This represents a lint that enforces Result return types
pub fn uses_result_type(signature: &str) -> bool {
    todo!("Check if function signature uses Result type")
}

/// Exercise 4: Check for hardcoded secret patterns.
///
/// Requirements:
/// - Check if `code` contains any pattern that looks like a hardcoded secret
/// - Patterns to detect: "password=", "api_key=", "secret=", "token=" followed by a quoted string
/// - Return `true` if any pattern is found, `false` otherwise
/// - Case-insensitive matching
pub fn has_hardcoded_secrets(code: &str) -> bool {
    todo!("Detect hardcoded secret patterns in code")
}

/// Exercise 5: Enforce that logging calls do not log sensitive variables.
///
/// Requirements:
/// - Check if `log_statement` contains any variable name from `sensitive_vars`
/// - Return a list of sensitive variable names found in the log statement
/// - Case-sensitive matching
/// - Match as whole word boundaries (e.g., "password" matches but "password_hash" does not)
pub fn find_sensitive_log_vars<'a>(log_statement: &'a str, sensitive_vars: &[&str]) -> Vec<&'a str> {
    todo!("Find sensitive variables in log statements")
}

/// Exercise 6: Check that unsafe blocks have a safety comment.
///
/// Requirements:
/// - Check if `code_block` contains both "unsafe" and "SAFETY:" (or "Safety:")
/// - Return `true` if unsafe is present AND has a safety comment
/// - Return `true` if no unsafe is present (no violation)
/// - Return `false` if unsafe is present but has NO safety comment
pub fn has_safety_comment(code_block: &str) -> bool {
    todo!("Check for safety comments on unsafe blocks")
}

/// Exercise 7: Validate that crypto algorithm choices meet minimum requirements.
///
/// Requirements:
/// - Check if `algorithm` is in the `allowed` list
/// - Return `Ok(())` if allowed, `Err(String)` with a message if not
/// - The message should say "Algorithm X is not in the allowed list"
pub fn validate_crypto_algorithm(algorithm: &str, allowed: &[&str]) -> Result<(), String> {
    todo!("Validate crypto algorithm choice")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_forbidden_calls_found() {
        let code = "let x = result.unwrap();";
        let violations = find_forbidden_calls(code, &["unwrap", "expect"]);
        assert_eq!(violations, vec!["unwrap"]);
    }

    #[test]
    fn test_find_forbidden_calls_multiple() {
        let code = "let x = result.unwrap(); let y = foo.expect(\"msg\");";
        let violations = find_forbidden_calls(code, &["unwrap", "expect"]);
        assert_eq!(violations, vec!["unwrap", "expect"]);
    }

    #[test]
    fn test_find_forbidden_calls_clean() {
        let code = "let x = result?;";
        let violations = find_forbidden_calls(code, &["unwrap", "expect"]);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_contains_panic_macros_panic() {
        assert!(contains_panic_macros("panic!(\"oh no\")"));
    }

    #[test]
    fn test_contains_panic_macros_todo() {
        assert!(contains_panic_macros("todo!(\"implement later\")"));
    }

    #[test]
    fn test_contains_panic_macros_clean() {
        assert!(!contains_panic_macros("return Ok(42)"));
    }

    #[test]
    fn test_uses_result_type_yes() {
        assert!(uses_result_type("fn parse(input: &str) -> Result<u32, Error>"));
    }

    #[test]
    fn test_uses_result_type_no() {
        assert!(!uses_result_type("fn parse(input: &str) -> u32"));
    }

    #[test]
    fn test_has_hardcoded_secrets_password() {
        assert!(has_hardcoded_secrets(r#"let config = "password=supersecret";"#));
    }

    #[test]
    fn test_has_hardcoded_secrets_api_key() {
        assert!(has_hardcoded_secrets(r#"env = "api_key=AKIA1234567890ABCDEF""#));
    }

    #[test]
    fn test_has_hardcoded_secrets_clean() {
        assert!(!has_hardcoded_secrets(r#"let config = "host=localhost";"#));
    }

    #[test]
    fn test_find_sensitive_log_vars_found() {
        let log = r#"info!("user login: password={}", password)"#;
        let vars = find_sensitive_log_vars(log, &["password", "ssn", "credit_card"]);
        assert_eq!(vars, vec!["password"]);
    }

    #[test]
    fn test_find_sensitive_log_vars_not_whole_word() {
        let log = r#"info!("hash computed: password_hash={}", hash)"#;
        let vars = find_sensitive_log_vars(log, &["password"]);
        assert!(vars.is_empty(), "password_hash should not match 'password'");
    }

    #[test]
    fn test_find_sensitive_log_vars_clean() {
        let log = r#"info!("user login: user_id={}", id)"#;
        let vars = find_sensitive_log_vars(log, &["password", "ssn"]);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_has_safety_comment_present() {
        let code = "// SAFETY: pointer is valid because we just allocated it\nunsafe { *ptr = 42; }";
        assert!(has_safety_comment(code));
    }

    #[test]
    fn test_has_safety_comment_missing() {
        let code = "unsafe { *ptr = 42; }";
        assert!(!has_safety_comment(code));
    }

    #[test]
    fn test_has_safety_comment_no_unsafe() {
        let code = "let x = 42;";
        assert!(has_safety_comment(code), "No unsafe = no violation");
    }

    #[test]
    fn test_validate_crypto_algorithm_allowed() {
        assert!(validate_crypto_algorithm("AES-256-GCM", &["AES-256-GCM", "ChaCha20-Poly1305"]).is_ok());
    }

    #[test]
    fn test_validate_crypto_algorithm_blocked() {
        let result = validate_crypto_algorithm("DES", &["AES-256-GCM", "ChaCha20-Poly1305"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("DES"));
    }
}
