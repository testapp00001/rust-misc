//! # Lesson 03: Log Injection Attack (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Detect CRLF injection characters.
pub fn contains_crlf(input: &str) -> bool {
    input.contains('\r') || input.contains('\n')
}

/// Sanitize by removing CRLF characters.
pub fn sanitize_crlf(input: &str) -> String {
    input.replace('\r', "").replace('\n', " | ")
}

/// Safe logging function that sanitizes user input.
pub fn safe_log_user_input(input: &str, max_length: usize) -> String {
    let sanitized: String = input
        .chars()
        .filter(|c| !c.is_control() || *c == '\t')
        .take(max_length)
        .collect();
    format!("[USER_INPUT] {}", sanitized)
}

/// Detect ANSI escape sequences.
pub fn contains_ansi_escape(input: &str) -> bool {
    input.contains('\x1b')
}

/// Detect potential log flooding.
pub fn is_log_flooding(input: &str, threshold: usize) -> bool {
    input.len() > threshold
}

/// Comprehensive log sanitizer.
pub fn comprehensive_sanitize(input: &str, max_length: usize) -> String {
    let mut modified = false;
    let mut result = String::with_capacity(input.len());

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip ANSI escape sequence: ESC [ ... final_char
            modified = true;
            if chars.peek() == Some(&'[') {
                chars.next(); // consume '['
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        if c == '\r' || c == '\n' {
            modified = true;
            continue;
        }
        if c.is_control() && c != '\t' && c != ' ' {
            modified = true;
            continue;
        }
        result.push(c);
    }

    if result.len() > max_length {
        result.truncate(max_length);
        modified = true;
    }

    if modified {
        result.push_str(" [REDACTED]");
    }

    result
}

/// Validate that a log message is safe to write.
pub fn validate_log_input(input: &str) -> Result<String, String> {
    if input.contains('\r') || input.contains('\n') {
        return Err("Input contains CRLF characters".to_string());
    }
    if input.contains('\x1b') {
        return Err("Input contains ANSI escape sequences".to_string());
    }
    if input.len() > 10000 {
        return Err(format!("Input too long: {} bytes (max 10000)", input.len()));
    }
    if input.contains('\0') {
        return Err("Input contains null bytes".to_string());
    }
    Ok(input.to_string())
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
