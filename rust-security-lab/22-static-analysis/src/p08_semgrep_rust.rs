//! # Lesson 08: Semgrep/Rust-Analyzer Patterns
//!
//! ## The Problem
//!
//! Custom Clippy lints are powerful but hard to write. Semgrep provides a
//! lighter-weight alternative: pattern-based code scanning with simple rules.
//!
//! ```yaml
//! # semgrep rule: ban unsafe blocks
//! rules:
//!   - id: ban-unsafe
//!     pattern: unsafe { ... }
//!     message: "Unsafe code requires security review"
//!     severity: WARNING
//!     languages: [rust]
//! ```
//!
//! ## Semgrep Pattern Types
//!
//! | Pattern | Example | Matches |
//! |---------|---------|---------|
//! | Literal | `unwrap()` | Exact match |
//! | Ellipsis | `f(...)` | Any arguments |
//! | Metavariable | `$X.unwrap()` | Captures `$X` |
//! | Regex | `regex: password\s*=` | Regex match |
//!
//! ## Security Patterns to Detect
//!
//! - `unwrap()` and `expect()` in non-test code
//! - `unsafe` blocks without safety comments
//! - Hardcoded credentials
//! - Weak crypto algorithms
//! - Logging of sensitive data
//!
//! ## Exercise
//!
//! Implement pattern-matching functions that simulate Semgrep-style analysis.
//! These functions detect security anti-patterns in Rust code snippets.

/// Exercise 1: Detect `unsafe` blocks in code.
///
/// Requirements:
/// - Return `true` if `code` contains the keyword "unsafe" followed by "{"
/// - Allow for whitespace between "unsafe" and "{"
/// - This catches `unsafe { ... }` blocks
pub fn has_unsafe_block(code: &str) -> bool {
    todo!("Detect unsafe blocks in code")
}

/// Exercise 2: Detect usage of weak hash algorithms.
///
/// Requirements:
/// - Return `true` if `code` contains references to: "md5", "sha1", "SHA1", "MD5"
/// - Case-insensitive matching
/// - Return `false` if only strong hashes (sha256, sha384, sha512) are used
pub fn uses_weak_hash(code: &str) -> bool {
    todo!("Detect weak hash algorithm usage")
}

/// Exercise 3: Detect potential timing attack vulnerability.
///
/// Timing attacks exploit the fact that string comparison short-circuits.
///
/// Requirements:
/// - Return `true` if `code` contains `==` comparison of variables named
///   with security-sensitive names (containing "token", "secret", "key", "password")
/// - Case-insensitive matching on variable names
/// - Return `false` if no such comparisons found
pub fn has_timing_vulnerability(code: &str) -> bool {
    todo!("Detect potential timing attack vectors")
}

/// Exercise 4: Detect use of `std::process::Command` with user input.
///
/// Command injection is a critical vulnerability.
///
/// Requirements:
/// - Return `true` if `code` contains "Command::new" AND "format!" or string
///   concatenation in the same code block
/// - This indicates potential command injection
pub fn has_command_injection_risk(code: &str) -> bool {
    todo!("Detect potential command injection patterns")
}

/// Exercise 5: Detect missing error handling (bare `?` operator in main).
///
/// Requirements:
/// - Return `true` if `code` contains `fn main()` AND does NOT contain
///   `-> Result` in the main function signature
/// - A main function that can propagate errors without handling them
pub fn has_unhandled_errors(code: &str) -> bool {
    todo!("Detect missing error handling in main")
}

/// Exercise 6: Detect potential SSRF (Server-Side Request Forgery) patterns.
///
/// Requirements:
/// - Return `true` if `code` contains HTTP request methods ("get(", "post(",
///   "put(", "delete(") AND also contains string formatting ("format!") or
///   user input variables ("user_input", "request", "params")
/// - This indicates a URL built from user input
pub fn has_ssrf_risk(code: &str) -> bool {
    todo!("Detect potential SSRF patterns")
}

/// Exercise 7: Extract all function names from a code snippet.
///
/// Requirements:
/// - Find all occurrences of `fn ` followed by a function name
/// - Return a list of function names found
/// - Function names consist of alphanumeric characters and underscores
pub fn extract_function_names(code: &str) -> Vec<String> {
    todo!("Extract function names from code")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_unsafe_block_present() {
        assert!(has_unsafe_block("unsafe { do_something(); }"));
    }

    #[test]
    fn test_has_unsafe_block_with_newline() {
        let code = "unsafe\n{\n    do_something();\n}";
        assert!(has_unsafe_block(code));
    }

    #[test]
    fn test_has_unsafe_block_absent() {
        assert!(!has_unsafe_block("let x = safe_function();"));
    }

    #[test]
    fn test_uses_weak_hash_md5() {
        assert!(uses_weak_hash("use md5::compute;"));
    }

    #[test]
    fn test_uses_weak_hash_sha1() {
        assert!(uses_weak_hash("let hash = sha1::digest(data);"));
    }

    #[test]
    fn test_uses_weak_hash_strong_only() {
        assert!(!uses_weak_hash("use sha2::Sha256;"));
    }

    #[test]
    fn test_has_timing_vulnerability_password() {
        assert!(has_timing_vulnerability("if password == input {"));
    }

    #[test]
    fn test_has_timing_vulnerability_token() {
        assert!(has_timing_vulnerability("if secret_token == user_token {"));
    }

    #[test]
    fn test_has_timing_vulnerability_safe() {
        assert!(!has_timing_vulnerability("if a == b {"));
    }

    #[test]
    fn test_has_command_injection_risk() {
        let code = r#"
            let cmd = format!("echo {}", user_input);
            Command::new("sh").arg("-c").arg(cmd);
        "#;
        assert!(has_command_injection_risk(code));
    }

    #[test]
    fn test_has_command_injection_risk_safe() {
        let code = r#"Command::new("ls").arg("-la");"#;
        assert!(!has_command_injection_risk(code));
    }

    #[test]
    fn test_has_unhandled_errors() {
        let code = "fn main() {\n    let x = risky_function()?;\n}";
        assert!(has_unhandled_errors(code));
    }

    #[test]
    fn test_has_unhandled_errors_with_result() {
        let code = "fn main() -> Result<(), Error> {\n    let x = risky_function()?;\n    Ok(())\n}";
        assert!(!has_unhandled_errors(code));
    }

    #[test]
    fn test_has_ssrf_risk() {
        let code = r#"let url = format!("https://{}/api", user_input); client.get(&url);"#;
        assert!(has_ssrf_risk(code));
    }

    #[test]
    fn test_has_ssrf_risk_safe() {
        let code = r#"client.get("https://api.example.com/data");"#;
        assert!(!has_ssrf_risk(code));
    }

    #[test]
    fn test_extract_function_names() {
        let code = r#"
            fn process_input(data: &str) -> Result<(), Error> { }
            fn validate(data: &str) -> bool { }
        "#;
        let names = extract_function_names(code);
        assert!(names.contains(&"process_input".to_string()));
        assert!(names.contains(&"validate".to_string()));
    }

    #[test]
    fn test_extract_function_names_none() {
        let names = extract_function_names("let x = 42;");
        assert!(names.is_empty());
    }
}
