//! # Lesson 06: Secret Injection Patterns (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SecretTemplate {
    pub template: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub content: String,
    pub injected_keys: Vec<String>,
}

pub fn extract_placeholders(template: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut remaining = template;
    while let Some(start) = remaining.find("{{.Secrets.") {
        let after_prefix = &remaining[start + 11..]; // len of "{{.Secrets."
        if let Some(end) = after_prefix.find("}}") {
            let key = &after_prefix[..end];
            if !key.is_empty() {
                keys.push(key.to_string());
            }
            remaining = &after_prefix[end + 2..];
        } else {
            break;
        }
    }
    keys
}

fn extract_placeholders_with_prefix(template: &str, prefix: &str) -> Vec<String> {
    let full_prefix = format!("{{{{{}}}", prefix); // e.g. "{{.Secrets."
    let mut keys = Vec::new();
    let mut remaining = template;
    while let Some(start) = remaining.find(&full_prefix) {
        let after_prefix = &remaining[start + full_prefix.len()..];
        if let Some(end) = after_prefix.find("}}") {
            let key = &after_prefix[..end];
            if !key.is_empty() {
                keys.push(key.to_string());
            }
            remaining = &after_prefix[end + 2..];
        } else {
            break;
        }
    }
    keys
}

pub fn resolve_template(
    template: &SecretTemplate,
    secrets: &HashMap<String, String>,
) -> Result<ResolvedConfig, Vec<String>> {
    let keys = extract_placeholders(&template.template);
    let mut missing = Vec::new();
    for key in &keys {
        if !secrets.contains_key(key) {
            missing.push(key.clone());
        }
    }
    if !missing.is_empty() {
        return Err(missing);
    }

    let mut content = template.template.clone();
    let mut injected_keys = Vec::new();
    for key in &keys {
        let placeholder = format!("{{{{.Secrets.{}}}}}", key);
        let value = secrets.get(key).unwrap();
        content = content.replace(&placeholder, value);
        injected_keys.push(key.clone());
    }

    Ok(ResolvedConfig {
        content,
        injected_keys,
    })
}

pub fn inject_into_config(
    content: &str,
    secrets: &HashMap<String, String>,
) -> Result<(String, Vec<String>), String> {
    let secret_keys = extract_placeholders_with_prefix(content, ".Secrets.");
    let env_keys = extract_placeholders_with_prefix(content, ".Env.");

    let mut result = content.to_string();
    let mut injected = Vec::new();

    for key in &secret_keys {
        let placeholder = format!("{{{{.Secrets.{}}}}}", key);
        match secrets.get(key) {
            Some(value) => {
                result = result.replace(&placeholder, value);
                injected.push(key.clone());
            }
            None => return Err(format!("Missing secret: {}", key)),
        }
    }

    for key in &env_keys {
        let placeholder = format!("{{{{.Env.{}}}}}", key);
        match std::env::var(key) {
            Ok(value) => {
                result = result.replace(&placeholder, &value);
                injected.push(key.clone());
            }
            Err(_) => return Err(format!("Missing env var: {}", key)),
        }
    }

    Ok((result, injected))
}

pub fn sanitize_config(content: &str, secrets: &HashMap<String, String>) -> String {
    let mut result = content.to_string();
    for value in secrets.values() {
        if value.len() >= 4 {
            result = result.replace(value, "[REDACTED]");
        }
    }
    result
}

pub fn format_secret_file(secrets: &HashMap<String, String>) -> String {
    let mut keys: Vec<&String> = secrets.keys().collect();
    keys.sort();
    let mut output = String::new();
    for key in keys {
        output.push_str(&format!("{}={}\n", key, secrets.get(key).unwrap()));
    }
    output
}

pub fn parse_secret_file(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let value = trimmed[eq_pos + 1..].trim().to_string();
            map.insert(key, value);
        }
    }
    map
}

pub fn validate_no_placeholders(content: &str) -> Result<(), String> {
    if let Some(pos) = content.find("{{.") {
        let snippet_end = content[pos..]
            .find("}}")
            .map(|e| pos + e + 2)
            .unwrap_or_else(|| std::cmp::min(pos + 50, content.len()));
        Err(content[pos..snippet_end].to_string())
    } else {
        Ok(())
    }
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
