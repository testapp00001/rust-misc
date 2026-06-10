//! # Lesson 06: Untrusted Input Handling (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::Deserialize;

/// Enforce a maximum payload size before parsing.
pub fn enforce_payload_limit(input: &str, max_bytes: usize) -> Result<&str, String> {
    if input.len() > max_bytes {
        Err(format!(
            "Payload size {} exceeds maximum allowed {}",
            input.len(),
            max_bytes
        ))
    } else {
        Ok(input)
    }
}

/// Sanitize a string by removing control characters except common whitespace.
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            !c.is_control()
                || *c == '\n'    // 0x0A
                || *c == '\r'    // 0x0D
                || *c == '\t'    // 0x09
        })
        .collect()
}

/// Validate and canonicalize a hostname.
pub fn canonicalize_hostname(input: &str) -> Result<String, String> {
    if input.is_empty() {
        return Err("hostname must not be empty".to_string());
    }

    let lower = input.to_lowercase();

    if lower.len() > 253 {
        return Err(format!(
            "hostname length {} exceeds DNS limit of 253",
            lower.len()
        ));
    }

    if !lower.contains('.') {
        return Err("hostname must contain at least one dot".to_string());
    }

    if lower.starts_with('.') || lower.ends_with('.') {
        return Err("hostname must not start or end with a dot".to_string());
    }

    if lower.starts_with('-') || lower.ends_with('-') {
        return Err("hostname must not start or end with a hyphen".to_string());
    }

    Ok(lower)
}

/// Parse untrusted JSON with size limit and validation.
#[derive(Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub name: String,
    pub port: u16,
    pub host: String,
}

pub fn parse_app_config(json: &str, max_bytes: usize) -> Result<AppConfig, String> {
    enforce_payload_limit(json, max_bytes)?;

    let config: AppConfig =
        serde_json::from_str(json).map_err(|e| format!("parse error: {}", e))?;

    if config.name.is_empty() {
        return Err("name must not be empty".to_string());
    }
    if config.port == 0 {
        return Err("port must not be 0".to_string());
    }
    if config.host.is_empty() {
        return Err("host must not be empty".to_string());
    }

    Ok(config)
}

/// Detect potential path traversal in a filename.
pub fn has_path_traversal(filename: &str) -> Result<bool, String> {
    // Check for ".." as a path component
    for component in filename.split('/') {
        if component == ".." {
            return Ok(true);
        }
    }

    // Check for absolute path
    if filename.starts_with('/') {
        return Ok(true);
    }

    // Check for backslash separators (Windows-style traversal)
    if filename.contains('\\') {
        return Ok(true);
    }

    Ok(false)
}

/// Validate a URL scheme against an allowlist.
pub fn url_has_allowed_scheme(url: &str, allowed: &[&str]) -> Result<bool, String> {
    let scheme_end = url.find(':').ok_or_else(|| "URL has no scheme".to_string())?;
    let scheme = &url[..scheme_end].to_lowercase();
    Ok(allowed.iter().any(|a| a.to_lowercase() == *scheme))
}

/// Sanitize and validate a list of tags.
pub fn sanitize_tags(tags: &mut Vec<String>) -> Result<(), String> {
    // Trim and lowercase
    for tag in tags.iter_mut() {
        *tag = tag.trim().to_lowercase();
    }

    // Remove empty tags
    tags.retain(|t| !t.is_empty());

    // Check individual tag length
    for tag in tags.iter() {
        if tag.len() > 50 {
            return Err(format!("tag '{}' exceeds 50 characters", tag));
        }
    }

    // Check total count
    if tags.len() > 10 {
        return Err(format!(
            "too many tags: {} exceeds maximum of 10",
            tags.len()
        ));
    }

    Ok(())
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
