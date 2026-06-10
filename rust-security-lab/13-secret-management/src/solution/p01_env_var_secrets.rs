//! # Lesson 01: Environment Variable Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::HashMap;

pub fn read_secret(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|_| name.to_string())
}

pub fn read_secret_or_default(name: &str, default_value: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default_value.to_string())
}

pub fn read_required_secrets(required_names: &[&str]) -> Result<HashMap<String, String>, Vec<String>> {
    let mut map = HashMap::new();
    let mut missing = Vec::new();

    for &name in required_names {
        match std::env::var(name) {
            Ok(val) => { map.insert(name.to_string(), val); }
            Err(_) => { missing.push(name.to_string()); }
        }
    }

    if missing.is_empty() { Ok(map) } else { Err(missing) }
}

pub fn mask_secret(secret: &str) -> String {
    let len = secret.len();
    if len <= 8 {
        "*".repeat(len)
    } else {
        let suffix = &secret[len - 4..];
        format!("{}{}", "*".repeat(len - suffix.len() - 1), suffix)
    }
}

pub fn parse_env_file(contents: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let raw_value = trimmed[eq_pos + 1..].trim();
            let value = if (raw_value.starts_with('"') && raw_value.ends_with('"'))
                || (raw_value.starts_with('\'') && raw_value.ends_with('\''))
            {
                raw_value[1..raw_value.len() - 1].to_string()
            } else {
                raw_value.to_string()
            };
            map.insert(key, value);
        }
    }
    map
}

pub fn looks_like_secret(key: &str, value: &str) -> bool {
    let key_lower = key.to_lowercase();

    // Check by key name
    if key_lower.contains("password") || key_lower.contains("secret") {
        return true;
    }

    // Check by prefix
    let value_lower = value.to_lowercase();
    if value_lower.starts_with("sk-")
        || value_lower.starts_with("pk-")
        || value_lower.starts_with("key-")
        || value_lower.starts_with("token-")
        || value_lower.starts_with("bearer ")
    {
        return true;
    }

    // Check by length + mixed content
    if value.len() >= 16 {
        let has_letter = value.chars().any(|c| c.is_alphabetic());
        let has_digit = value.chars().any(|c| c.is_ascii_digit());
        if has_letter && has_digit {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_secret_exists() {
        std::env::set_var("TEST_SECRET_P01", "my_secret_value");
        let result = read_secret("TEST_SECRET_P01");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "my_secret_value");
        std::env::remove_var("TEST_SECRET_P01");
    }

    #[test]
    fn test_read_secret_missing() {
        std::env::remove_var("DEFINITELY_NOT_SET_P01");
        let result = read_secret("DEFINITELY_NOT_SET_P01");
        assert!(result.is_err());
    }

    #[test]
    fn test_read_secret_or_default_set() {
        std::env::set_var("TEST_DEFAULT_P01", "from_env");
        let result = read_secret_or_default("TEST_DEFAULT_P01", "fallback");
        assert_eq!(result, "from_env");
        std::env::remove_var("TEST_DEFAULT_P01");
    }

    #[test]
    fn test_read_secret_or_default_missing() {
        std::env::remove_var("DEFINITELY_NOT_SET_DEFAULT_P01");
        let result = read_secret_or_default("DEFINITELY_NOT_SET_DEFAULT_P01", "fallback");
        assert_eq!(result, "fallback");
    }

    #[test]
    fn test_read_required_secrets_all_present() {
        std::env::set_var("REQ_A_P01", "val_a");
        std::env::set_var("REQ_B_P01", "val_b");
        let result = read_required_secrets(&["REQ_A_P01", "REQ_B_P01"]);
        assert!(result.is_ok());
        let map = result.unwrap();
        assert_eq!(map.get("REQ_A_P01").unwrap(), "val_a");
        assert_eq!(map.get("REQ_B_P01").unwrap(), "val_b");
        std::env::remove_var("REQ_A_P01");
        std::env::remove_var("REQ_B_P01");
    }

    #[test]
    fn test_read_required_secrets_some_missing() {
        std::env::set_var("REQ_C_P01", "val_c");
        std::env::remove_var("REQ_D_P01");
        let result = read_required_secrets(&["REQ_C_P01", "REQ_D_P01"]);
        assert!(result.is_err());
        let missing = result.unwrap_err();
        assert!(missing.contains(&"REQ_D_P01".to_string()));
        assert!(!missing.contains(&"REQ_C_P01".to_string()));
        std::env::remove_var("REQ_C_P01");
    }

    #[test]
    fn test_mask_secret_long() {
        let masked = mask_secret("sk-1234567890abcdef");
        assert_eq!(masked, "**************cdef");
        assert!(!masked.contains("1234567890"));
        assert!(masked.ends_with("cdef"));
    }

    #[test]
    fn test_mask_secret_short() {
        let masked = mask_secret("short");
        assert_eq!(masked.len(), 5);
        assert_eq!(masked, "*****");
    }

    #[test]
    fn test_mask_secret_exactly_8() {
        let masked = mask_secret("12345678");
        assert_eq!(masked, "********");
    }

    #[test]
    fn test_parse_env_file() {
        let contents = r#"# This is a comment
DB_HOST=localhost
DB_PASSWORD="super_secret"
API_KEY='sk-12345'

# Another comment
TOKEN=bearer_abc123
"#;
        let map = parse_env_file(contents);
        assert_eq!(map.get("DB_HOST").unwrap(), "localhost");
        assert_eq!(map.get("DB_PASSWORD").unwrap(), "super_secret");
        assert_eq!(map.get("API_KEY").unwrap(), "sk-12345");
        assert_eq!(map.get("TOKEN").unwrap(), "bearer_abc123");
        assert_eq!(map.len(), 4);
    }

    #[test]
    fn test_looks_like_secret_by_prefix() {
        assert!(looks_like_secret("API_KEY", "sk-1234567890"));
        assert!(looks_like_secret("PRIVATE_KEY", "pk-abcdef"));
        assert!(looks_like_secret("AUTH", "bearer xyz"));
        assert!(looks_like_secret("TOKEN", "token-abc123"));
    }

    #[test]
    fn test_looks_like_secret_by_name() {
        assert!(looks_like_secret("DATABASE_PASSWORD", "anything"));
        assert!(looks_like_secret("MY_SECRET_VALUE", "anything"));
        assert!(!looks_like_secret("HOSTNAME", "localhost"));
    }

    #[test]
    fn test_looks_like_secret_by_length() {
        assert!(looks_like_secret("VAR", "abcdefgh12345678"));
        assert!(!looks_like_secret("VAR", "short"));
        assert!(!looks_like_secret("VAR", "alllettersnonumbershere"));
    }
}
