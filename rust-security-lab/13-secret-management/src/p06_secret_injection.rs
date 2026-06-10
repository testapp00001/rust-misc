//! # Lesson 06: Secret Injection Patterns
//!
//! ## The Problem
//!
//! Applications need secrets at runtime. How do you get secrets from a secrets
//! manager into your application safely?
//!
//! ## Injection Patterns
//!
//! ### 1. Environment Variable Injection
//! ```bash
//! export DB_PASSWORD=$(vault kv get -field=password secret/db)
//! ./myapp
//! ```
//!
//! ### 2. File-Based Injection (Sidecar / Init Container)
//! ```text
//! Secrets manager writes to a shared volume
//! Application reads from the file
//! File is a tmpfs (RAM-backed) filesystem — never touches disk
//! ```
//!
//! ### 3. Template Rendering
//! ```text
//! Config template:  database_password: {{.Secrets.DB_PASSWORD}}
//! Rendered:         database_password: actual_secret_value
//! ```
//!
//! ### 4. Direct API Injection
//! ```text
//! Application calls secrets manager API at startup
//! Caches secrets in memory
//! Refreshes before TTL expires
//! ```
//!
//! ## Attack: Injection Vulnerabilities
//!
//! If secret injection is not handled carefully:
//! - Template injection: `{{.Secrets}}` in user input could leak secrets
//! - Log injection: secrets appearing in process listings or logs
//! - Environment variable sniffing: `/proc/<pid>/environ`
//! - File permission issues: secret files readable by other users
//!
//! ## Defense
//!
//! 1. Use tmpfs for secret files (never write to disk)
//! 2. Set restrictive file permissions (0600)
//! 3. Clear secret variables from environment after reading
//! 4. Never log secret values
//! 5. Use constant-time comparison for secret validation

use std::collections::HashMap;

/// A secret template with placeholder syntax.
///
/// Placeholders use `{{.Secrets.KEY}}` syntax, similar to Go templates.
#[derive(Debug, Clone)]
pub struct SecretTemplate {
    pub template: String,
}

/// A resolved configuration where all placeholders have been replaced.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub content: String,
    pub injected_keys: Vec<String>,
}

/// Exercise 1: Extract all placeholder keys from a template.
///
/// Given a template string like:
///   "host=db.{{.Secrets.ENV}}.com password={{.Secrets.DB_PASS}}"
///
/// Return a Vec of the key names: ["ENV", "DB_PASS"]
///
/// Hints:
/// - Look for `{{.Secrets.` patterns
/// - Extract the key name up to the closing `}}`
/// - Use string scanning or split-based approach
pub fn extract_placeholders(template: &str) -> Vec<String> {
    todo!("Extract placeholder keys from a template")
}

/// Exercise 2: Resolve a template by replacing placeholders with secret values.
///
/// Replace all `{{.Secrets.KEY}}` occurrences with the corresponding value
/// from the secrets map.
///
/// Return `Ok(ResolvedConfig)` if all placeholders are resolved.
/// Return `Err(missing_keys)` if any placeholder keys are not in the secrets map.
///
/// Hints:
/// - Extract all placeholder keys first
/// - Check that all keys exist in the secrets map
/// - If any are missing, return Err with the list of missing keys
/// - Otherwise, replace each `{{.Secrets.KEY}}` with the value
/// - Track which keys were injected
pub fn resolve_template(
    template: &SecretTemplate,
    secrets: &HashMap<String, String>,
) -> Result<ResolvedConfig, Vec<String>> {
    todo!("Resolve a template by replacing placeholders")
}

/// Exercise 3: Inject secrets into a config file content.
///
/// Similar to resolve_template, but works with raw config file content.
/// Support two formats:
/// - `{{.Secrets.KEY}}` — replace with the secret value
/// - `{{.Env.KEY}}` — replace with the environment variable value
///
/// Return the resolved content and a list of all keys that were injected.
///
/// Hints:
/// - Extract placeholders for both `{{.Secrets.` and `{{.Env.` patterns
/// - For Secrets placeholders, look up in the secrets map
/// - For Env placeholders, look up in std::env::var
/// - Replace all occurrences
pub fn inject_into_config(
    content: &str,
    secrets: &HashMap<String, String>,
) -> Result<(String, Vec<String>), String> {
    todo!("Inject secrets and env vars into config content")
}

/// Exercise 4: Sanitize a configuration by removing secret values.
///
/// Given a config string and a map of secret values, replace any occurrence
/// of a secret value with `[REDACTED]`. This is useful for logging configs
/// without leaking secrets.
///
/// Hints:
/// - Iterate over the secrets values
/// - For each, replace all occurrences in the content with "[REDACTED]"
/// - Be careful: short secrets (< 4 chars) should not be redacted
///   (too many false positives)
pub fn sanitize_config(content: &str, secrets: &HashMap<String, String>) -> String {
    todo!("Remove secret values from config for safe logging")
}

/// Exercise 5: Create a secret file content with proper formatting.
///
/// Given a map of key-value secrets, format them as a properties file:
/// ```text
/// KEY1=value1
/// KEY2=value2
/// ```
///
/// Keys should be sorted alphabetically.
/// Each line should end with `\n`.
///
/// Hints:
/// - Collect keys into a Vec and sort them
/// - Format each as "key=value\n"
/// - Concatenate all lines
pub fn format_secret_file(secrets: &HashMap<String, String>) -> String {
    todo!("Format secrets as a properties file")
}

/// Exercise 6: Parse a secret properties file back into a map.
///
/// Parse a properties file (same format as Exercise 5) into a HashMap.
/// Handle:
/// - Lines with `KEY=VALUE` format
/// - Empty lines (skip)
/// - Lines starting with `#` (skip, these are comments)
/// - Values may contain `=` characters (split only on the first `=`)
///
/// Hints:
/// - Split by newlines
/// - For each line, skip if empty or starts with '#'
/// - Split on the first '=' character
/// - Trim whitespace from key and value
pub fn parse_secret_file(content: &str) -> HashMap<String, String> {
    todo!("Parse a secret properties file")
}

/// Exercise 7: Validate that a resolved config contains no remaining placeholders.
///
/// Return Ok(()) if the content contains no `{{.` patterns.
/// Return Err(first_placeholder) with the first unresolved placeholder found.
///
/// Hints:
/// - Search for "{{." in the content
/// - If found, extract up to the next "}}" or 50 chars
/// - Return Err with that substring
/// - If not found, return Ok(())
pub fn validate_no_placeholders(content: &str) -> Result<(), String> {
    todo!("Check for unresolved placeholders in config")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_secrets() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("DB_HOST".to_string(), "prod-db.example.com".to_string());
        m.insert("DB_PASS".to_string(), "s3cret_p@ss".to_string());
        m.insert("API_KEY".to_string(), "sk-1234567890".to_string());
        m
    }

    #[test]
    fn test_extract_placeholders() {
        let template = "host={{.Secrets.DB_HOST}} pass={{.Secrets.DB_PASS}}";
        let keys = extract_placeholders(template);
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"DB_HOST".to_string()));
        assert!(keys.contains(&"DB_PASS".to_string()));
    }

    #[test]
    fn test_extract_placeholders_none() {
        let template = "no placeholders here";
        let keys = extract_placeholders(template);
        assert!(keys.is_empty());
    }

    #[test]
    fn test_resolve_template_success() {
        let template = SecretTemplate {
            template: "host={{.Secrets.DB_HOST}} pass={{.Secrets.DB_PASS}}".to_string(),
        };
        let result = resolve_template(&template, &test_secrets());
        assert!(result.is_ok());
        let resolved = result.unwrap();
        assert!(resolved.content.contains("prod-db.example.com"));
        assert!(resolved.content.contains("s3cret_p@ss"));
        assert_eq!(resolved.injected_keys.len(), 2);
    }

    #[test]
    fn test_resolve_template_missing_key() {
        let template = SecretTemplate {
            template: "key={{.Secrets.MISSING_KEY}}".to_string(),
        };
        let result = resolve_template(&template, &test_secrets());
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert!(missing.contains(&"MISSING_KEY".to_string()));
    }

    #[test]
    fn test_sanitize_config() {
        let config = "host=prod-db.example.com\npass=s3cret_p@ss";
        let sanitized = sanitize_config(config, &test_secrets());
        assert!(!sanitized.contains("s3cret_p@ss"));
        assert!(sanitized.contains("[REDACTED]"));
        // Short values (< 4 chars) should not be redacted
    }

    #[test]
    fn test_format_and_parse_secret_file() {
        let secrets = test_secrets();
        let formatted = format_secret_file(&secrets);
        let parsed = parse_secret_file(&formatted);
        assert_eq!(parsed, secrets);
    }

    #[test]
    fn test_parse_secret_file_with_comments() {
        let content = "# This is a comment\nKEY1=value1\n\n# Another comment\nKEY2=value2\n";
        let parsed = parse_secret_file(content);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed.get("KEY1").unwrap(), "value1");
        assert_eq!(parsed.get("KEY2").unwrap(), "value2");
    }

    #[test]
    fn test_validate_no_placeholders_clean() {
        assert!(validate_no_placeholders("no placeholders here").is_ok());
    }

    #[test]
    fn test_validate_no_placeholders_found() {
        let result = validate_no_placeholders("host={{.Secrets.DB_HOST}}");
        assert!(result.is_err());
    }
}
