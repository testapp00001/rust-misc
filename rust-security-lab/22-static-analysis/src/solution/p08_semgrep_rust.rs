//! # Lesson 08: Semgrep/Rust Patterns (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Detect unsafe blocks in code.
pub fn has_unsafe_block(code: &str) -> bool {
    // Match "unsafe" followed by optional whitespace then "{"
    let chars: Vec<char> = code.chars().collect();
    let len = chars.len();
    let keyword: Vec<char> = "unsafe".chars().collect();
    let klen = keyword.len();

    let mut i = 0;
    while i + klen <= len {
        if chars[i..i + klen] == keyword[..] {
            let mut j = i + klen;
            // Skip whitespace
            while j < len && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\n' || chars[j] == '\r') {
                j += 1;
            }
            if j < len && chars[j] == '{' {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Detect weak hash algorithm usage.
pub fn uses_weak_hash(code: &str) -> bool {
    let lower = code.to_lowercase();
    lower.contains("md5") || lower.contains("sha1")
}

/// Detect potential timing attack vectors.
pub fn has_timing_vulnerability(code: &str) -> bool {
    let lower = code.to_lowercase();
    let sensitive_patterns = ["token", "secret", "key", "password"];
    for pattern in &sensitive_patterns {
        if lower.contains(pattern) && code.contains("==") {
            // Check if the == comparison involves a sensitive variable
            // by looking for the pattern near "=="
            if let Some(eq_pos) = code.find("==") {
                let context_start = eq_pos.saturating_sub(50);
                let context_end = (eq_pos + 50).min(code.len());
                let context = &code[context_start..context_end].to_lowercase();
                if context.contains(pattern) {
                    return true;
                }
            }
        }
    }
    false
}

/// Detect potential command injection patterns.
pub fn has_command_injection_risk(code: &str) -> bool {
    code.contains("Command::new") && (code.contains("format!") || code.contains("format!("))
}

/// Detect missing error handling in main.
pub fn has_unhandled_errors(code: &str) -> bool {
    if !code.contains("fn main()") {
        return false;
    }
    // If main has "fn main()" but not "-> Result" in its signature, it may have unhandled errors
    if let Some(main_pos) = code.find("fn main()") {
        // Look for the opening brace of main
        let after_main = &code[main_pos..];
        if let Some(brace_pos) = after_main.find('{') {
            let signature = &after_main[..brace_pos];
            return !signature.contains("-> Result") && code.contains('?');
        }
    }
    false
}

/// Detect potential SSRF patterns.
pub fn has_ssrf_risk(code: &str) -> bool {
    let has_http_call = code.contains(".get(") || code.contains(".post(")
        || code.contains(".put(") || code.contains(".delete(");
    let has_user_input = code.contains("format!") || code.contains("user_input")
        || code.contains("request") || code.contains("params");
    has_http_call && has_user_input
}

/// Extract function names from code.
pub fn extract_function_names(code: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut remaining = code;

    while let Some(pos) = remaining.find("fn ") {
        let after_fn = &remaining[pos + 3..];
        let name: String = after_fn
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() && !name.chars().next().unwrap().is_ascii_digit() {
            names.push(name);
        }
        remaining = &remaining[pos + 3..];
    }

    names
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
