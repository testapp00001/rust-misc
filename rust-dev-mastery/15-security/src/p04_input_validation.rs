//! # Input Validation
//!
//! All external input is untrusted. This lesson covers comprehensive input
//! validation, sanitization, and injection prevention for secure applications.
//!
//! ## Key Concepts
//! - SQL injection prevention
//! - XSS prevention through output encoding
//! - Path traversal prevention
//! - URL validation
//! - File upload validation
//! - Input length and format validation

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// 1. Sanitizer
// ---------------------------------------------------------------------------

/// Sanitizes user input to prevent common attacks.
pub struct InputSanitizer;

impl InputSanitizer {
    /// Remove or escape potentially dangerous HTML characters.
    pub fn escape_html(input: &str) -> String {
        input
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
    }

    /// Strip all HTML tags from input.
    pub fn strip_html(input: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;
        for c in input.chars() {
            match c {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => result.push(c),
                _ => {}
            }
        }
        result
    }

    /// Sanitize a string for use in a SQL LIKE pattern.
    pub fn escape_like_pattern(input: &str) -> String {
        input
            .replace('%', "\\%")
            .replace('_', "\\_")
    }

    /// Remove control characters (except newline, tab, carriage return).
    pub fn remove_control_chars(input: &str) -> String {
        input
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
            .collect()
    }

    /// Normalize whitespace (collapse multiple spaces, trim).
    pub fn normalize_whitespace(input: &str) -> String {
        let mut result = String::new();
        let mut last_was_space = false;
        for c in input.trim().chars() {
            if c.is_whitespace() {
                if !last_was_space {
                    result.push(' ');
                    last_was_space = true;
                }
            } else {
                result.push(c);
                last_was_space = false;
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// 2. Path Safety
// ---------------------------------------------------------------------------

/// Validates file paths to prevent path traversal attacks.
pub struct PathValidator {
    allowed_roots: Vec<PathBuf>,
    blocked_extensions: Vec<String>,
    max_depth: usize,
}

impl PathValidator {
    pub fn new() -> Self {
        Self {
            allowed_roots: Vec::new(),
            blocked_extensions: vec![
                "exe".into(), "sh".into(), "bat".into(), "cmd".into(),
                "ps1".into(), "vbs".into(),
            ],
            max_depth: 10,
        }
    }

    pub fn allow_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.allowed_roots.push(root.into());
        self
    }

    pub fn block_extension(mut self, ext: &str) -> Self {
        self.blocked_extensions.push(ext.into());
        self
    }

    /// Validate a file path.
    pub fn validate(&self, path: &str) -> Result<PathBuf, ValidationError> {
        let path = Path::new(path);

        // Check for path traversal
        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(ValidationError::PathTraversal);
        }

        // Check depth
        let depth = path.components().count();
        if depth > self.max_depth {
            return Err(ValidationError::PathTooDeep {
                depth,
                max: self.max_depth,
            });
        }

        // Check extension
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if self.blocked_extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                return Err(ValidationError::BlockedExtension(ext.into()));
            }
        }

        // Check allowed roots
        if !self.allowed_roots.is_empty() {
            let canonical = path;
            let allowed = self
                .allowed_roots
                .iter()
                .any(|root| canonical.starts_with(root));
            if !allowed {
                return Err(ValidationError::PathNotAllowed);
            }
        }

        Ok(path.to_path_buf())
    }
}

// ---------------------------------------------------------------------------
// 3. URL Validation
// ---------------------------------------------------------------------------

/// Validates URLs for safety and format.
pub struct UrlValidator {
    allowed_schemes: Vec<String>,
    blocked_hosts: Vec<String>,
    max_length: usize,
}

impl UrlValidator {
    pub fn new() -> Self {
        Self {
            allowed_schemes: vec!["http".into(), "https".into()],
            blocked_hosts: vec!["localhost".into(), "127.0.0.1".into(), "0.0.0.0".into()],
            max_length: 2048,
        }
    }

    pub fn allow_scheme(mut self, scheme: &str) -> Self {
        self.allowed_schemes.push(scheme.into());
        self
    }

    pub fn block_host(mut self, host: &str) -> Self {
        self.blocked_hosts.push(host.into());
        self
    }

    /// Validate a URL.
    pub fn validate(&self, url: &str) -> Result<(), ValidationError> {
        if url.len() > self.max_length {
            return Err(ValidationError::TooLong {
                length: url.len(),
                max: self.max_length,
            });
        }

        // Check scheme
        let scheme = url
            .split("://")
            .next()
            .ok_or(ValidationError::InvalidFormat)?;
        if !self
            .allowed_schemes
            .iter()
            .any(|s| s.eq_ignore_ascii_case(scheme))
        {
            return Err(ValidationError::DisallowedScheme(scheme.into()));
        }

        // Extract host
        let after_scheme = url.split("://").nth(1).unwrap_or("");
        let host = after_scheme
            .split('/')
            .next()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("")
            .split('@')
            .next_back()
            .unwrap_or("");

        // Check blocked hosts
        if self.blocked_hosts.iter().any(|h| h.eq_ignore_ascii_case(host)) {
            return Err(ValidationError::BlockedHost(host.into()));
        }

        // Check for javascript: and data: schemes (XSS)
        let lower = url.to_lowercase();
        if lower.starts_with("javascript:") || lower.starts_with("data:") {
            return Err(ValidationError::DangerousScheme);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 4. Email Validation
// ---------------------------------------------------------------------------

/// Validates email addresses.
pub struct EmailValidator {
    max_length: usize,
}

impl EmailValidator {
    pub fn new() -> Self {
        Self { max_length: 254 }
    }

    pub fn validate(&self, email: &str) -> Result<(), ValidationError> {
        if email.len() > self.max_length {
            return Err(ValidationError::TooLong {
                length: email.len(),
                max: self.max_length,
            });
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(ValidationError::InvalidFormat);
        }

        let (local, domain) = (parts[0], parts[1]);

        if local.is_empty() || local.len() > 64 {
            return Err(ValidationError::InvalidFormat);
        }

        if domain.is_empty() || !domain.contains('.') || domain.ends_with('.') {
            return Err(ValidationError::InvalidFormat);
        }

        // Check for dangerous characters
        if email.contains('<')
            || email.contains('>')
            || email.contains('"')
            || email.contains(';')
        {
            return Err(ValidationError::DangerousCharacters);
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// 5. Numeric Validation
// ---------------------------------------------------------------------------

/// Validates numeric inputs within ranges.
pub struct NumericValidator {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub integer_only: bool,
}

impl NumericValidator {
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
            integer_only: false,
        }
    }

    pub fn min(mut self, min: f64) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: f64) -> Self {
        self.max = Some(max);
        self
    }

    pub fn integer_only(mut self) -> Self {
        self.integer_only = true;
        self
    }

    pub fn validate(&self, input: &str) -> Result<f64, ValidationError> {
        let value: f64 = input
            .trim()
            .parse()
            .map_err(|_| ValidationError::InvalidFormat)?;

        if self.integer_only && value.fract() != 0.0 {
            return Err(ValidationError::InvalidFormat);
        }

        if let Some(min) = self.min {
            if value < min {
                return Err(ValidationError::BelowMinimum { value, min });
            }
        }

        if let Some(max) = self.max {
            if value > max {
                return Err(ValidationError::AboveMaximum { value, max });
            }
        }

        Ok(value)
    }
}

// ---------------------------------------------------------------------------
// 6. Validation Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("path traversal detected")]
    PathTraversal,
    #[error("path depth {depth} exceeds maximum {max}")]
    PathTooDeep { depth: usize, max: usize },
    #[error("blocked file extension: {0}")]
    BlockedExtension(String),
    #[error("path not in allowed roots")]
    PathNotAllowed,
    #[error("input too long: {length} chars (max {max})")]
    TooLong { length: usize, max: usize },
    #[error("invalid format")]
    InvalidFormat,
    #[error("disallowed URL scheme: {0}")]
    DisallowedScheme(String),
    #[error("blocked host: {0}")]
    BlockedHost(String),
    #[error("dangerous scheme (javascript: or data:)")]
    DangerousScheme,
    #[error("dangerous characters detected")]
    DangerousCharacters,
    #[error("value {value} below minimum {min}")]
    BelowMinimum { value: f64, min: f64 },
    #[error("value {value} above maximum {max}")]
    AboveMaximum { value: f64, max: f64 },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(
            InputSanitizer::escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
        assert_eq!(InputSanitizer::escape_html("a & b"), "a &amp; b");
    }

    #[test]
    fn test_strip_html() {
        assert_eq!(InputSanitizer::strip_html("<b>bold</b>"), "bold");
        assert_eq!(
            InputSanitizer::strip_html("<p>Hello <br/> World</p>"),
            "Hello  World"
        );
    }

    #[test]
    fn test_escape_like_pattern() {
        assert_eq!(InputSanitizer::escape_like_pattern("100%"), "100\\%");
        assert_eq!(InputSanitizer::escape_like_pattern("file_name"), "file\\_name");
    }

    #[test]
    fn test_remove_control_chars() {
        assert_eq!(
            InputSanitizer::remove_control_chars("hello\nworld\ttab"),
            "hello\nworld\ttab"
        );
        assert_eq!(
            InputSanitizer::remove_control_chars("a\x00b\x01c"),
            "abc"
        );
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(
            InputSanitizer::normalize_whitespace("  hello   world  "),
            "hello world"
        );
        assert_eq!(
            InputSanitizer::normalize_whitespace("single"),
            "single"
        );
    }

    #[test]
    fn test_path_validator_traversal() {
        let validator = PathValidator::new();
        assert!(validator.validate("../../../etc/passwd").is_err());
        assert!(validator.validate("foo/../../bar").is_err());
    }

    #[test]
    fn test_path_validator_blocked_extension() {
        let validator = PathValidator::new();
        assert!(validator.validate("malware.exe").is_err());
        assert!(validator.validate("script.sh").is_err());
        assert!(validator.validate("safe.txt").is_ok());
    }

    #[test]
    fn test_path_validator_depth() {
        let validator = PathValidator::new();
        let deep_path = "a/b/c/d/e/f/g/h/i/j/k/l/m";
        assert!(validator.validate(deep_path).is_err());
    }

    #[test]
    fn test_url_validator_basic() {
        let validator = UrlValidator::new();
        assert!(validator.validate("https://example.com").is_ok());
        assert!(validator.validate("http://example.com/path").is_ok());
    }

    #[test]
    fn test_url_validator_blocked_scheme() {
        let validator = UrlValidator::new();
        assert!(validator.validate("ftp://example.com").is_err());
    }

    #[test]
    fn test_url_validator_blocked_host() {
        let validator = UrlValidator::new();
        assert!(validator.validate("http://localhost/admin").is_err());
        assert!(validator.validate("http://127.0.0.1:8080").is_err());
    }

    #[test]
    fn test_url_validator_javascript() {
        let validator = UrlValidator::new();
        assert!(validator.validate("javascript:alert(1)").is_err());
        assert!(validator.validate("data:text/html,<h1>hi</h1>").is_err());
    }

    #[test]
    fn test_url_validator_too_long() {
        let validator = UrlValidator::new();
        let long_url = format!("https://example.com/{}", "a".repeat(3000));
        assert!(validator.validate(&long_url).is_err());
    }

    #[test]
    fn test_email_validator_valid() {
        let validator = EmailValidator::new();
        assert!(validator.validate("user@example.com").is_ok());
        assert!(validator.validate("a.b+c@domain.co").is_ok());
    }

    #[test]
    fn test_email_validator_invalid() {
        let validator = EmailValidator::new();
        assert!(validator.validate("not-an-email").is_err());
        assert!(validator.validate("@domain.com").is_err());
        assert!(validator.validate("user@").is_err());
        assert!(validator.validate("user@domain.").is_err());
    }

    #[test]
    fn test_email_validator_dangerous_chars() {
        let validator = EmailValidator::new();
        assert!(validator.validate("user<script>@example.com").is_err());
        assert!(validator.validate("user;drop@example.com").is_err());
    }

    #[test]
    fn test_numeric_validator() {
        let validator = NumericValidator::new().min(0.0).max(100.0);
        assert!(validator.validate("50").is_ok());
        assert!(validator.validate("-1").is_err());
        assert!(validator.validate("101").is_err());
        assert!(validator.validate("abc").is_err());
    }

    #[test]
    fn test_numeric_validator_integer_only() {
        let validator = NumericValidator::new().integer_only();
        assert!(validator.validate("42").is_ok());
        assert!(validator.validate("3.14").is_err());
    }

    #[test]
    fn test_path_validator_allowed_root() {
        let validator = PathValidator::new().allow_root("/var/data");
        assert!(validator.validate("file.txt").is_err()); // not under allowed root
    }

    #[test]
    fn test_url_validator_custom_block() {
        let validator = UrlValidator::new().block_host("evil.com");
        assert!(validator.validate("https://evil.com/attack").is_err());
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::PathTraversal;
        assert!(err.to_string().contains("traversal"));

        let err = ValidationError::TooLong {
            length: 5000,
            max: 2048,
        };
        assert!(err.to_string().contains("5000"));
    }
}
