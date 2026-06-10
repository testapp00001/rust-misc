//! # Lesson 10: Input Sanitization Patterns
//!
//! ## The Problem
//!
//! Input sanitization is the process of cleaning user input to make it safe
//! for a specific use. It is the **last line of defense** -- output encoding
//! (Lesson 04) and parameterized queries (Lesson 02) are preferred.
//!
//! However, sanitization is still useful as defense in depth:
//! - Rejecting obviously malicious input early
//! - Cleaning data before storage (removing control characters)
//! - Normalizing input for consistent processing
//!
//! ## Allowlist vs Denylist
//!
//! | Approach   | Description                    | When to Use                      |
//! |------------|--------------------------------|----------------------------------|
//! | Allowlist  | Only permit known-good values  | Strongly preferred -- always try  |
//! | Denylist   | Only reject known-bad values   | Supplement allowlist, never alone |
//!
//! An allowlist for a username might be: `[a-zA-Z0-9_-]`
//! A denylist might be: no `<script>`, no `DROP TABLE`, etc.
//!
//! The problem with denylists: attackers find new bypasses. The problem
//! with allowlists: they may be too restrictive. Use allowlists where
//! possible; supplement with denylists where needed.
//!
//! ## Defense in Depth Layers
//!
//! ```
//! User Input
//!     |
//!     v
//! [1. Type Validation]  -- Reject wrong types early
//!     |
//!     v
//! [2. Format Validation] -- Check format (email, URL, etc.)
//!     |
//!     v
//! [3. Business Rules]    -- Domain-specific validation
//!     |
//!     v
//! [4. Sanitization]      -- Clean for storage
//!     |
//!     v
//! [5. Output Encoding]   -- Encode for output context
//! ```

use regex::Regex;

/// A general-purpose input sanitizer that applies multiple cleaning rules.
///
/// Sanitization steps (in order):
/// 1. Trim leading/trailing whitespace
/// 2. Remove null bytes
/// 3. Normalize Unicode whitespace (tabs, multiple spaces) to single space
/// 4. Remove control characters (except newline and tab)
/// 5. Enforce maximum length (truncate if needed)
///
/// Returns the sanitized string.
pub fn sanitize_input(input: &str, max_length: usize) -> String {
    todo!("Implement multi-step input sanitization")
}

/// Validate input against an allowlist pattern.
///
/// The `pattern` is a regex that must match the ENTIRE input string.
/// If the pattern matches, the input is allowed. If not, it is rejected.
///
/// Returns Ok(input) if the pattern matches, Err(message) if not.
pub fn validate_allowlist<'a>(input: &'a str, pattern: &str) -> Result<&'a str, String> {
    todo!("Validate input against allowlist regex")
}

/// Validate input against a denylist of patterns.
///
/// The `patterns` are substrings (not regexes) that must NOT appear in
/// the input (case-insensitive comparison).
///
/// Returns Ok(input) if none of the patterns are found,
/// Err(message) listing the first matched pattern.
pub fn validate_denylist<'a>(input: &'a str, patterns: &[&str]) -> Result<&'a str, String> {
    todo!("Validate input against denylist patterns")
}

/// A composable validation pipeline.
///
/// Chains multiple validation steps together. Each step is a function
/// that takes a &str and returns Result<&str, String>. If any step
/// fails, the pipeline stops and returns the error.
///
/// # Example
/// ```ignore
/// let result = ValidationPipeline::new()
///     .add_step(|s| if s.len() >= 3 { Ok(s) } else { Err("too short".into()) })
///     .add_step(|s| if s.is_ascii() { Ok(s) } else { Err("not ascii".into()) })
///     .validate("hello");
/// ```
pub struct ValidationPipeline<'a> {
    steps: Vec<Box<dyn Fn(&str) -> Result<&str, String> + 'a>>,
}

impl<'a> ValidationPipeline<'a> {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Add a validation step to the pipeline.
    pub fn add_step<F>(mut self, step: F) -> Self
    where
        F: Fn(&str) -> Result<&str, String> + 'a,
    {
        self.steps.push(Box::new(step));
        self
    }

    /// Run all validation steps in order.
    ///
    /// Returns Ok(input) if all steps pass, Err(message) from the first
    /// failing step.
    pub fn validate(&self, input: &str) -> Result<&str, String> {
        todo!("Execute all pipeline steps in order")
    }
}

/// A field-level sanitizer for web form data.
///
/// Combines sanitization and validation for common web form fields.
pub struct FieldSanitizer {
    max_length: usize,
    allowlist_pattern: Option<String>,
    strip_html: bool,
}

impl FieldSanitizer {
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            allowlist_pattern: None,
            strip_html: false,
        }
    }

    /// Set an allowlist regex pattern for the field.
    pub fn with_allowlist(mut self, pattern: &str) -> Self {
        self.allowlist_pattern = Some(pattern.to_string());
        self
    }

    /// Enable HTML tag stripping.
    pub fn with_strip_html(mut self) -> Self {
        self.strip_html = true;
        self
    }

    /// Sanitize a field value using the configured rules.
    ///
    /// Steps:
    /// 1. Trim whitespace
    /// 2. Strip HTML tags if enabled (remove `<...>` sequences)
    /// 3. Remove null bytes
    /// 4. Truncate to max_length
    /// 5. Check allowlist if configured
    ///
    /// Returns Ok(sanitized_value) or Err(message).
    pub fn sanitize(&self, input: &str) -> Result<String, String> {
        todo!("Apply field-level sanitization rules")
    }
}

/// Strip HTML tags from a string.
///
/// Removes all `<...>` sequences. This is a naive implementation --
/// for production use, use a proper HTML parser like `ammonia`.
///
/// Also decodes common HTML entities:
/// - `&amp;` -> `&`
/// - `&lt;` -> `<`
/// - `&gt;` -> `>`
/// - `&quot;` -> `"`
/// - `&#x27;` -> `'`
pub fn strip_html_tags(input: &str) -> String {
    todo!("Strip HTML tags and decode entities")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_trims_whitespace() {
        assert_eq!(sanitize_input("  hello  ", 100), "hello");
    }

    #[test]
    fn test_sanitize_removes_null_bytes() {
        assert_eq!(sanitize_input("hello\0world", 100), "helloworld");
    }

    #[test]
    fn test_sanitize_truncates() {
        let result = sanitize_input("hello world", 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_allowlist_valid() {
        let result = validate_allowlist("alice_99", r"^[a-zA-Z0-9_]{3,32}$");
        assert!(result.is_ok());
    }

    #[test]
    fn test_allowlist_invalid() {
        let result = validate_allowlist("alice@home!", r"^[a-zA-Z0-9_]{3,32}$");
        assert!(result.is_err());
    }

    #[test]
    fn test_denylist_clean() {
        let result = validate_denylist("hello world", &["<script>", "DROP TABLE"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_denylist_match() {
        let result = validate_denylist("hello <script>alert(1)</script>", &["<script>"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_all_pass() {
        let pipeline = ValidationPipeline::new()
            .add_step(|s| if !s.is_empty() { Ok(s) } else { Err("empty".into()) })
            .add_step(|s| if s.is_ascii() { Ok(s) } else { Err("not ascii".into()) });
        let result = pipeline.validate("hello");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_first_failure() {
        let pipeline = ValidationPipeline::new()
            .add_step(|s| if s.len() >= 10 { Ok(s) } else { Err("too short".into()) })
            .add_step(|s| if s.is_ascii() { Ok(s) } else { Err("not ascii".into()) });
        let result = pipeline.validate("hi");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("short"));
    }

    #[test]
    fn test_field_sanitizer_strips_html() {
        let sanitizer = FieldSanitizer::new(100).with_strip_html();
        let result = sanitizer.sanitize("<b>hello</b>").unwrap();
        assert!(!result.contains('<'));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_field_sanitizer_enforces_allowlist() {
        let sanitizer = FieldSanitizer::new(100).with_allowlist(r"^[a-z]+$");
        assert!(sanitizer.sanitize("hello").is_ok());
        assert!(sanitizer.sanitize("hello123").is_err());
    }

    #[test]
    fn test_strip_html_entities() {
        let result = strip_html_tags("a &amp; b &lt; c");
        assert!(result.contains("a & b"));
        assert!(result.contains("< c"));
    }
}
