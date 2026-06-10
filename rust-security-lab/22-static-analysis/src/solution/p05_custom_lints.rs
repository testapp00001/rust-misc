//! # Lesson 05: Custom Lints (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Find forbidden function calls in code.
pub fn find_forbidden_calls<'a>(code: &'a str, forbidden: &[&'a str]) -> Vec<&'a str> {
    forbidden
        .iter()
        .filter(|&&f| code.contains(f))
        .copied()
        .collect()
}

/// Check for panic-causing macros.
pub fn contains_panic_macros(code: &str) -> bool {
    code.contains("panic!") || code.contains("todo!") || code.contains("unimplemented!") || code.contains("unreachable!")
}

/// Check if function signature uses Result type.
pub fn uses_result_type(signature: &str) -> bool {
    signature.contains("-> Result")
}

/// Detect hardcoded secret patterns in code.
pub fn has_hardcoded_secrets(code: &str) -> bool {
    let lower = code.to_lowercase();
    let patterns = ["password=", "api_key=", "secret=", "token="];
    patterns.iter().any(|p| lower.contains(p))
}

/// Find sensitive variables in log statements.
pub fn find_sensitive_log_vars<'a>(log_statement: &'a str, sensitive_vars: &[&'a str]) -> Vec<&'a str> {
    sensitive_vars
        .iter()
        .filter(|&&var| {
            // Match as whole word: check that the variable is not part of a larger identifier
            // by checking the character before and after the match (include _ as word char)
            let is_word_char = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
            let mut found = false;
            let mut search_start = 0;
            while let Some(pos) = log_statement[search_start..].find(var) {
                let abs_pos = search_start + pos;
                let before_ok = abs_pos == 0
                    || !is_word_char(log_statement.as_bytes()[abs_pos - 1]);
                let after_pos = abs_pos + var.len();
                let after_ok = after_pos >= log_statement.len()
                    || !is_word_char(log_statement.as_bytes()[after_pos]);
                if before_ok && after_ok {
                    found = true;
                    break;
                }
                search_start = abs_pos + 1;
            }
            found
        })
        .copied()
        .collect()
}

/// Check for safety comments on unsafe blocks.
pub fn has_safety_comment(code_block: &str) -> bool {
    if !code_block.contains("unsafe") {
        return true; // No unsafe = no violation
    }
    code_block.contains("SAFETY:") || code_block.contains("Safety:")
}

/// Validate crypto algorithm choice.
pub fn validate_crypto_algorithm(algorithm: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&algorithm) {
        Ok(())
    } else {
        Err(format!("Algorithm {} is not in the allowed list", algorithm))
    }
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
