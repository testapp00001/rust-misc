//! # Lesson 03: Log Injection Attack
//!
//! ## The Attack
//!
//! Log injection occurs when untrusted user input flows directly into log messages
//! without sanitization. The most common variant is **CRLF injection**, where an
//! attacker inserts `\r\n` (carriage return + line feed) to create fake log entries.
//!
//! ## Attack Example
//!
//! An attacker registers with this username:
//! ```
//! admin\r\n2024-01-15 10:30:00 INFO User admin logged in successfully
//! ```
//!
//! The application logs:
//! ```
//! 2024-01-15 10:30:00 INFO Registration attempt for user: admin
//! 2024-01-15 10:30:00 INFO User admin logged in successfully
//! ```
//!
//! The second line is FORGED. An auditor reviewing logs would believe `admin`
//! logged in. This can be used to:
//! - Forge audit trail entries
//! - Cover tracks after an attack
//! - Inject false evidence
//! - Exploit log viewers (XSS in Kibana/Splunk)
//!
//! ## Variants
//!
//! - **CRLF injection**: `\r\n` to create new log lines
//! - **ANSI escape injection**: `\x1b[...` to manipulate terminal-based log viewers
//! - **XML/HTML injection**: `<script>` in logs viewed through web interfaces
//! - **Log flooding**: Massive input to fill disk and rotate out real evidence
//!
//! ## Defense
//!
//! Sanitize ALL user input before logging:
//! 1. Strip or escape `\r` and `\n`
//! 2. Strip or escape ANSI escape sequences
//! 3. Strip or escape HTML/XML special characters
//! 4. Truncate excessively long input
//!
//! ## What You'll Implement
//!
//! 1. Detect CRLF injection attempts in strings
//! 2. Sanitize strings by removing control characters
//! 3. A safe logging function that auto-sanitizes
//! 4. Detect ANSI escape injection
//! 5. Detect log flooding (excessively long input)
//! 6. A comprehensive log sanitizer that handles all variants

/// Exercise 1: Detect if a string contains CRLF injection characters.
///
/// Return true if the input contains `\r` (carriage return) or `\n` (line feed).
///
/// Hints:
/// - Use `input.contains('\r') || input.contains('\n')`
pub fn contains_crlf(input: &str) -> bool {
    todo!("Implement CRLF detection")
}

/// Exercise 2: Sanitize a string by removing CRLF characters.
///
/// Replace `\r` and `\n` with a safe alternative:
/// - `\r` -> remove entirely
/// - `\n` -> replace with ` | ` (pipe separator to show where lines were)
///
/// Hints:
/// - Use `input.replace('\r', "").replace('\n', " | ")`
pub fn sanitize_crlf(input: &str) -> String {
    todo!("Implement CRLF sanitization")
}

/// Exercise 3: A safe logging function that sanitizes user input.
///
/// Format: `[USER_INPUT] <sanitized_input>`
///
/// The function must:
/// - Remove all CRLF characters from the input
/// - Remove all control characters (ASCII 0x00-0x1F except space)
/// - Truncate to max_length characters (default 1000)
/// - Return the sanitized log line
///
/// Hints:
/// - Filter: `input.chars().filter(|c| !c.is_control() || *c == ' ')`
/// - Truncate: `chars.take(max_length).collect()`
pub fn safe_log_user_input(input: &str, max_length: usize) -> String {
    todo!("Implement safe logging function")
}

/// Exercise 4: Detect ANSI escape sequences.
///
/// ANSI escapes start with `\x1b[` (ESC + `[`) and can:
/// - Change terminal colors (hide malicious output)
/// - Move cursor (overwrite previous log lines)
/// - Execute terminal commands in some viewers
///
/// Return true if the input contains ANSI escape sequences.
///
/// Hints:
/// - Check for `\x1b` (ESC character, 0x1B)
/// - Or check for the literal string `\x1b[` in various forms
pub fn contains_ansi_escape(input: &str) -> bool {
    todo!("Implement ANSI escape detection")
}

/// Exercise 5: Detect potential log flooding.
///
/// Log flooding sends massive input to:
/// - Fill disk space, causing log rotation to discard real evidence
/// - Overwhelm log parsing systems
/// - Hide attack traces in noise
///
/// Return true if the input exceeds the threshold length.
///
/// Hints:
/// - Simple length check: `input.len() > threshold`
pub fn is_log_flooding(input: &str, threshold: usize) -> bool {
    todo!("Implement log flooding detection")
}

/// Exercise 6: Comprehensive log sanitizer.
///
/// Apply all sanitization steps:
/// 1. Remove CRLF characters
/// 2. Remove all control characters except space and tab
/// 3. Remove ANSI escape sequences
/// 4. Truncate to max_length (default 2000)
/// 5. If input was modified, append `[REDACTED]` indicator
///
/// Return the sanitized string.
///
/// Hints:
/// - Filter out `\x1b` and control chars
/// - Track whether any changes were made
/// - Use `String::with_capacity` for efficiency
pub fn comprehensive_sanitize(input: &str, max_length: usize) -> String {
    todo!("Implement comprehensive log sanitizer")
}

/// Exercise 7: Validate that a log message is safe to write.
///
/// Return Ok(sanitized_message) if safe, or Err(reason) if the input
/// is potentially malicious.
///
/// Checks:
/// - No CRLF characters
/// - No ANSI escape sequences
/// - Length under 10000 characters
/// - No null bytes
///
/// Hints:
/// - Combine the detection functions
/// - Return specific error messages for each violation
pub fn validate_log_input(input: &str) -> Result<String, String> {
    todo!("Implement log input validation")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_crlf_newline() {
        assert!(contains_crlf("hello\nworld"));
    }

    #[test]
    fn test_detect_crlf_carriage_return() {
        assert!(contains_crlf("hello\rworld"));
    }

    #[test]
    fn test_detect_crlf_clean() {
        assert!(!contains_crlf("hello world"));
    }

    #[test]
    fn test_sanitize_crlf() {
        let result = sanitize_crlf("line1\nline2\rline3");
        assert!(!result.contains('\r'));
        assert!(!result.contains('\n'));
        assert!(result.contains('|'));
    }

    #[test]
    fn test_safe_log_user_input_strips_control_chars() {
        let result = safe_log_user_input("hello\x00world\x07", 1000);
        assert!(!result.contains('\x00'));
        assert!(!result.contains('\x07'));
    }

    #[test]
    fn test_safe_log_truncates() {
        let long_input = "a".repeat(5000);
        let result = safe_log_user_input(&long_input, 100);
        assert!(result.len() <= 100 + "[USER_INPUT] ".len());
    }

    #[test]
    fn test_detect_ansi_escape() {
        assert!(contains_ansi_escape("hello\x1b[31mworld"));
    }

    #[test]
    fn test_detect_no_ansi() {
        assert!(!contains_ansi_escape("hello world"));
    }

    #[test]
    fn test_log_flooding_detection() {
        assert!(is_log_flooding(&"x".repeat(10001), 10000));
        assert!(!is_log_flooding("short message", 10000));
    }

    #[test]
    fn test_comprehensive_sanitize_crlf() {
        let result = comprehensive_sanitize("admin\r\nfake log line", 2000);
        assert!(!result.contains('\r'));
        assert!(!result.contains('\n'));
    }

    #[test]
    fn test_comprehensive_sanitize_ansi() {
        let result = comprehensive_sanitize("hello\x1b[31mworld", 2000);
        assert!(!result.contains('\x1b'));
    }

    #[test]
    fn test_validate_clean_input() {
        assert!(validate_log_input("Normal log message").is_ok());
    }

    #[test]
    fn test_validate_rejects_crlf() {
        assert!(validate_log_input("line1\nline2").is_err());
    }

    #[test]
    fn test_validate_rejects_null_byte() {
        assert!(validate_log_input("hello\x00world").is_err());
    }
}
