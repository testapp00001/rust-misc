//! # Lesson 06: Untrusted Input Handling
//!
//! ## The Threat
//!
//! Every byte of input from an external source is untrusted. Serialization
//! libraries parse these bytes into rich structures that your code then
//! operates on. If you do not apply boundaries, an attacker controls:
//!
//! 1. **Size** -- A 100 MB JSON string can exhaust memory.
//! 2. **Shape** -- Extra or missing fields change semantics.
//! 3. **Content** -- Malicious strings can exploit downstream consumers
//!    (SQL injection, path traversal, command injection).
//!
//! ## Defense-in-Depth
//!
//! A single validation check is never enough. Layer your defenses:
//!
//! 1. **Size limits** -- Reject payloads over a known maximum before parsing.
//! 2. **Structural validation** -- `deny_unknown_fields`, required fields.
//! 3. **Value validation** -- Ranges, patterns, allowlists.
//! 4. **Canonicalization** -- Normalize before comparing or storing.
//!
//! ## Principle
//!
//! Treat deserialization as a trust boundary. The struct you produce from
//! untrusted input is the attack surface. Every field is a potential vector.

use serde::Deserialize;

/// Exercise 1: Enforce a maximum payload size before parsing.
///
/// Return Ok(payload) if the input is at most `max_bytes` bytes.
/// Return Err with a descriptive message if the input exceeds the limit.
pub fn enforce_payload_limit(input: &str, max_bytes: usize) -> Result<&str, String> {
    todo!("Enforce payload size limit")
}

/// Exercise 2: Sanitize a string field by removing control characters.
///
/// Remove all ASCII control characters (bytes 0x00-0x1F and 0x7F) except
/// for common whitespace: newline (0x0A), carriage return (0x0D), and
/// tab (0x09).
///
/// Return the sanitized string.
pub fn sanitize_string(input: &str) -> String {
    todo!("Remove control characters from string")
}

/// Exercise 3: Validate and canonicalize a hostname.
///
/// Rules:
/// - Must not be empty
/// - Must be at most 253 characters
/// - Must contain at least one dot (e.g., "example.com")
/// - Must not start or end with a dot or hyphen
/// - Must be lowercase (convert to lowercase if not)
///
/// Return Ok(canonical_hostname) or Err(message).
pub fn canonicalize_hostname(input: &str) -> Result<String, String> {
    todo!("Validate and canonicalize hostname")
}

/// Exercise 4: Parse untrusted JSON with size limit and validation.
///
/// Steps:
/// 1. Check that `json.len() <= max_bytes`
/// 2. Parse into the target struct (which uses deny_unknown_fields)
/// 3. Validate all fields
///
/// Return Ok(config) or Err(message).
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub name: String,
    pub port: u16,
    pub host: String,
}

pub fn parse_app_config(json: &str, max_bytes: usize) -> Result<AppConfig, String> {
    todo!("Parse untrusted JSON with limits")
}

/// Exercise 5: Detect potential path traversal in a filename.
///
/// Return Ok(true) if the filename contains path traversal indicators:
/// - ".." as a component
/// - Absolute paths (starts with '/')
/// - Backslash separators
///
/// Return Ok(false) if safe.
pub fn has_path_traversal(filename: &str) -> Result<bool, String> {
    todo!("Detect path traversal")
}

/// Exercise 6: Validate a URL scheme against an allowlist.
///
/// Given a URL string and a list of allowed schemes (e.g., ["https", "wss"]),
/// return Ok(true) if the URL uses an allowed scheme, Ok(false) otherwise.
///
/// A URL with no scheme should return Ok(false).
pub fn url_has_allowed_scheme(url: &str, allowed: &[&str]) -> Result<bool, String> {
    todo!("Validate URL scheme")
}

/// Exercise 7: Sanitize and validate a list of tags.
///
/// Given a mutable slice of tag strings:
/// 1. Remove empty tags
/// 2. Trim whitespace from each tag
/// 3. Convert to lowercase
/// 4. Reject if any tag exceeds 50 characters
/// 5. Reject if more than 10 tags remain
///
/// Return Ok(()) if valid after sanitization, Err(message) otherwise.
/// The slice is modified in place.
pub fn sanitize_tags(tags: &mut Vec<String>) -> Result<(), String> {
    todo!("Sanitize and validate tags")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_limit_within() {
        assert!(enforce_payload_limit("hello", 10).is_ok());
    }

    #[test]
    fn test_payload_limit_exceeded() {
        assert!(enforce_payload_limit("hello world", 5).is_err());
    }

    #[test]
    fn test_sanitize_removes_control_chars() {
        let input = "hello\x00world\x07!";
        assert_eq!(sanitize_string(input), "helloworld!");
    }

    #[test]
    fn test_sanitize_preserves_whitespace() {
        let input = "hello\tworld\n";
        assert_eq!(sanitize_string(input), "hello\tworld\n");
    }

    #[test]
    fn test_hostname_canonical() {
        assert_eq!(canonicalize_hostname("Example.COM").unwrap(), "example.com");
    }

    #[test]
    fn test_hostname_no_dot() {
        assert!(canonicalize_hostname("localhost").is_err());
    }

    #[test]
    fn test_hostname_too_long() {
        let long = "a".repeat(254);
        assert!(canonicalize_hostname(&long).is_err());
    }

    #[test]
    fn test_path_traversal_dotdot() {
        assert!(has_path_traversal("../etc/passwd").unwrap());
    }

    #[test]
    fn test_path_traversal_absolute() {
        assert!(has_path_traversal("/etc/passwd").unwrap());
    }

    #[test]
    fn test_path_traversal_safe() {
        assert!(!has_path_traversal("document.pdf").unwrap());
    }

    #[test]
    fn test_url_scheme_allowed() {
        assert!(url_has_allowed_scheme("https://example.com", &["https"]).unwrap());
    }

    #[test]
    fn test_url_scheme_blocked() {
        assert!(!url_has_allowed_scheme("javascript:alert(1)", &["https"]).unwrap());
    }

    #[test]
    fn test_sanitize_tags_basic() {
        let mut tags = vec!["  Rust  ".to_string(), "SECURITY".to_string(), "".to_string()];
        sanitize_tags(&mut tags).unwrap();
        assert_eq!(tags, vec!["rust", "security"]);
    }

    #[test]
    fn test_sanitize_tags_too_many() {
        let mut tags: Vec<String> = (0..11).map(|i| format!("tag{}", i)).collect();
        assert!(sanitize_tags(&mut tags).is_err());
    }
}
