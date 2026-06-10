//! # Lesson 07: Stack Trace Exposure in Production
//!
//! ## The Problem
//!
//! Stack traces are invaluable for debugging but devastating in production. They reveal:
//! - **File paths**: `/home/app/src/auth/handler.rs` shows directory structure
//! - **Function names**: `process_payment`, `verify_admin_token` reveal business logic
//! - **Line numbers**: Exact code location for targeted exploitation
//! - **Framework versions**: `actix-web-4.3.1` enables CVE lookup
//! - **Dependencies**: `sqlx-0.7.3` reveals database driver and version
//!
//! ## Rust-Specific Risks
//!
//! - `RUST_BACKTRACE=1` enables full backtraces on panic
//! - `RUST_BACKTRACE=full` includes source code snippets
//! - `unwrap()` and `expect()` panics include file:line in the message
//! - `anyhow::Error` can include full chain of causes with backtraces
//!
//! ## Defense
//!
//! 1. **Never set RUST_BACKTRACE in production**
//! 2. **Use `Result` instead of `unwrap()`/`expect()`**
//! 3. **Sanitize `anyhow::Error` before returning to users**
//! 4. **Use `#[cfg(debug_assertions)]` for debug-only output**
//! 5. **Set up panic hooks to log but not expose**

/// Exercise 1: Check if a string looks like a stack trace.
///
/// A stack trace typically contains:
/// - Lines starting with "at " followed by file paths
/// - Lines containing ".rs:" (Rust source files with line numbers)
/// - The word "panicked" or "thread" with thread name
/// - Lines starting with "   " (indented frames)
/// - The word "backtrace" or "Backtrace"
///
/// Return true if the string appears to contain a stack trace.
///
/// Hints:
/// - Check for multiple indicators, not just one
/// - Convert to lowercase for some checks
/// - A single "/" might be a URL, but ".rs:" is strongly indicative
pub fn looks_like_stack_trace(s: &str) -> bool {
    todo!("Detect if a string contains stack trace patterns")
}

/// Exercise 2: Strip stack trace information from an error message.
///
/// Given an error message that might contain a stack trace, remove all
/// stack trace lines and return only the primary error message.
///
/// Rules:
/// - Lines starting with "at " → remove
/// - Lines containing ".rs:" → remove
/// - Lines starting with "   " (3+ spaces, indicating frames) → remove
/// - Lines containing "panicked at" → keep the message part, remove location
/// - Everything else → keep
///
/// Return the cleaned message.
pub fn strip_stack_trace(message: &str) -> String {
    todo!("Remove stack trace lines from error message")
}

/// Exercise 3: Sanitize an anyhow error for user-facing output.
///
/// Given an error chain (as a string from anyhow's Debug or Display),
/// produce a safe user-facing message that:
/// - Contains only "An error occurred" or similar generic text
/// - Does NOT contain any file paths
/// - Does NOT contain function names
/// - Does NOT contain line numbers
/// - Does NOT contain backtrace information
pub fn sanitize_anyhow_error(error_debug: &str) -> String {
    todo!("Sanitize anyhow error for user-facing output")
}

/// Exercise 4: Create a safe panic hook.
///
/// Given a panic message and location (file, line), return a string that:
/// - Logs the full details for server-side (file, line, message)
/// - Returns a tuple: (log_message, user_message)
/// - The user_message must be generic: "An unexpected error occurred"
/// - The log_message must contain file, line, and message
///
/// Format for log_message: "PANIC at {file}:{line}: {message}"
pub fn safe_panic_handler(file: &str, line: u32, message: &str) -> (String, String) {
    todo!("Create safe panic handler with separate log and user messages")
}

/// Exercise 5: Validate that an error response is production-safe.
///
/// Given an HTTP response body (string), check that it does NOT contain:
/// - Stack trace patterns (use `looks_like_stack_trace`)
/// - File paths (containing "/" followed by ".rs")
/// - The word "panicked"
/// - Debug output markers (lines starting with "thread '")
/// - The string "RUST_BACKTRACE"
///
/// Return Ok(()) if safe, Err(reason) if unsafe.
pub fn validate_production_safe(response: &str) -> Result<(), String> {
    todo!("Validate that response is safe for production")
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
