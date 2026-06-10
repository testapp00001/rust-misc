//! # Lesson 07: Stack Trace Exposure in Production (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Check if a string looks like a stack trace.
pub fn looks_like_stack_trace(s: &str) -> bool {
    let lower = s.to_lowercase();
    let mut indicators = 0;

    // Check for panic indicators
    if lower.contains("panicked") || lower.contains("thread '") {
        indicators += 1;
    }

    // Check for Rust source file references
    if s.contains(".rs:") {
        indicators += 1;
    }

    // Check for "at " followed by a path
    if s.lines().any(|line| line.trim_start().starts_with("at ") && line.contains('/')) {
        indicators += 1;
    }

    // Check for indented stack frames
    if s.lines().any(|line| line.starts_with("   ") && (line.contains(':') || line.contains('('))) {
        indicators += 1;
    }

    // Check for backtrace markers
    if lower.contains("backtrace") {
        indicators += 1;
    }

    // Require at least 2 indicators to avoid false positives
    indicators >= 2
}

/// Strip stack trace lines from error message.
pub fn strip_stack_trace(message: &str) -> String {
    let mut result = Vec::new();

    for line in message.lines() {
        let trimmed = line.trim_start();

        // Remove "at /path/to/file.rs:NN" lines
        if trimmed.starts_with("at ") && (trimmed.contains('/') || trimmed.contains('\\')) {
            continue;
        }

        // Remove lines containing ".rs:" (source file references)
        if trimmed.contains(".rs:") && !trimmed.starts_with("Error") && !trimmed.starts_with("Caused") {
            continue;
        }

        // Remove indented stack frames (3+ spaces at start)
        if line.starts_with("   ") && (line.contains(':') || line.contains('(')) {
            continue;
        }

        // Remove backtrace header lines
        if trimmed.to_lowercase().contains("stack backtrace") {
            continue;
        }

        // Handle "panicked at" lines -- keep the message, remove the location
        if trimmed.contains("panicked at") {
            // Extract just the panic message
            if let Some(msg_start) = trimmed.find("'") {
                if let Some(msg_end) = trimmed[msg_start + 1..].find("'") {
                    result.push(trimmed[msg_start + 1..msg_start + 1 + msg_end].to_string());
                    continue;
                }
            }
            result.push("Panic occurred".to_string());
            continue;
        }

        result.push(line.to_string());
    }

    result.join("\n").trim().to_string()
}

/// Sanitize anyhow error for user-facing output.
pub fn sanitize_anyhow_error(_error_debug: &str) -> String {
    "An error occurred".to_string()
}

/// Create safe panic handler with separate log and user messages.
pub fn safe_panic_handler(file: &str, line: u32, message: &str) -> (String, String) {
    let log_message = format!("PANIC at {}:{}: {}", file, line, message);
    let user_message = "An unexpected error occurred".to_string();
    (log_message, user_message)
}

/// Validate that response is safe for production.
pub fn validate_production_safe(response: &str) -> Result<(), String> {
    if looks_like_stack_trace(response) {
        return Err("Response contains stack trace information".to_string());
    }

    // Check for .rs file paths
    if response.contains(".rs:") {
        return Err("Response contains Rust source file references".to_string());
    }

    // Check for panic messages
    if response.to_lowercase().contains("panicked") {
        return Err("Response contains panic information".to_string());
    }

    // Check for thread debug output
    if response.contains("thread '") {
        return Err("Response contains thread debug output".to_string());
    }

    // Check for RUST_BACKTRACE
    if response.contains("RUST_BACKTRACE") {
        return Err("Response contains backtrace configuration".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_rust_stack_trace() {
        let trace = r#"thread 'main' panicked at 'index out of bounds', src/main.rs:42:5
stack backtrace:
   0: std::panicking::begin_panic
   1: myapp::process at /home/app/src/handler.rs:100"#;
        assert!(looks_like_stack_trace(trace));
    }

    #[test]
    fn test_no_false_positive_on_normal_text() {
        assert!(!looks_like_stack_trace("Invalid request parameter"));
    }

    #[test]
    fn test_no_false_positive_on_url() {
        assert!(!looks_like_stack_trace("Visit https://example.com for more info"));
    }

    #[test]
    fn test_strip_removes_at_lines() {
        let msg = "Error occurred\nat /home/app/src/main.rs:42\nat /home/app/src/lib.rs:10";
        let stripped = strip_stack_trace(msg);
        assert!(!stripped.contains("at /home"), "Should remove 'at' lines");
        assert!(stripped.contains("Error occurred"), "Should keep error message");
    }

    #[test]
    fn test_strip_removes_indented_frames() {
        let msg = "Error\n   0: frame_one\n   1: frame_two\nEnd of error";
        let stripped = strip_stack_trace(msg);
        assert!(!stripped.contains("frame_one"), "Should remove indented frames");
        assert!(stripped.contains("Error"), "Should keep main message");
    }

    #[test]
    fn test_sanitize_anyhow_no_file_paths() {
        let error = r#"Error: failed to read config
Caused by: No such file or directory (os error 2)
   at /home/app/src/config.rs:42
   at /home/app/src/main.rs:10"#;
        let sanitized = sanitize_anyhow_error(error);
        assert!(!sanitized.contains("/home"), "Must not contain file paths");
        assert!(!sanitized.contains("config.rs"), "Must not contain file names");
    }

    #[test]
    fn test_sanitize_anyhow_no_backtrace() {
        let error = "Error: something failed\nstack backtrace:\n   0: frame";
        let sanitized = sanitize_anyhow_error(error);
        assert!(!sanitized.contains("backtrace"), "Must not contain backtrace");
    }

    #[test]
    fn test_safe_panic_handler_log_has_details() {
        let (log_msg, _) = safe_panic_handler("/app/src/main.rs", 42, "index out of bounds");
        assert!(log_msg.contains("/app/src/main.rs"), "Log should contain file");
        assert!(log_msg.contains("42"), "Log should contain line");
        assert!(log_msg.contains("index out of bounds"), "Log should contain message");
    }

    #[test]
    fn test_safe_panic_handler_user_generic() {
        let (_, user_msg) = safe_panic_handler("/app/src/main.rs", 42, "secret details");
        assert!(!user_msg.contains("/app"), "User message must not contain file");
        assert!(!user_msg.contains("42"), "User message must not contain line");
        assert!(!user_msg.contains("secret"), "User message must not contain details");
    }

    #[test]
    fn test_production_safe_clean_response() {
        assert!(validate_production_safe(r#"{"error": "Invalid request"}"#).is_ok());
    }

    #[test]
    fn test_production_safe_detects_stack_trace() {
        let response = r#"{"error": "panic at src/main.rs:42"}"#;
        assert!(validate_production_safe(response).is_err());
    }

    #[test]
    fn test_production_safe_detects_backtrace() {
        let response = "Error\nRUST_BACKTRACE=1\n   0: frame";
        assert!(validate_production_safe(response).is_err());
    }
}
