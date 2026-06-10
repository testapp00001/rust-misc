//! # Lesson 01: Environment Variable Security
//!
//! ## Why Environment Variables?
//!
//! Environment variables are the most common way to inject secrets into applications.
//! They keep secrets out of source code and allow different values per environment
//! (dev, staging, production).
//!
//! ## The Problem with `.env` Files
//!
//! `.env` files are convenient for local development but dangerous in practice:
//!
//! 1. **Git accidents**: One missing `.gitignore` entry and your secrets are public forever
//! 2. **No rotation**: The same `.env` file lives for months or years
//! 3. **No audit trail**: Nobody knows who accessed or modified the file
//! 4. **Sharing via chat**: Developers copy-paste `.env` contents in Slack/email
//! 5. **Production leftovers**: `.env` files linger on production servers
//!
//! ## Better Alternatives
//!
//! - **Container orchestrator secrets**: Kubernetes Secrets, Docker secrets
//! - **Secrets manager**: HashiCorp Vault, AWS Secrets Manager, Azure Key Vault
//! - **CI/CD secret injection**: GitHub Actions secrets, GitLab CI variables
//!
//! ## Attack Demo: Environment Variable Exposure
//!
//! On Linux, environment variables can leak through:
//! - `/proc/<pid>/environ` — readable by the same user
//! - Debug endpoints that expose `process.env`
//! - Error messages that include environment context
//! - `docker inspect` on running containers
//!
//! ## Defense
//!
//! 1. Never hardcode secrets in source code
//! 2. Always use `.gitignore` for `.env` files
//! 3. Use a secrets manager for production
//! 4. Limit who can read environment variables in production
//! 5. Audit environment variable access

use std::collections::HashMap;

/// Exercise 1: Read a secret from an environment variable.
///
/// Return `Ok(value)` if the env var exists, `Err(name)` if it doesn't.
///
/// Hints:
/// - Use `std::env::var(name)` which returns `Result<String, VarError>`
/// - `VarError::NotPresent` means the variable doesn't exist
/// - `VarError::NotUnicode` means the value contains invalid Unicode
pub fn read_secret(name: &str) -> Result<String, String> {
    todo!("Read a secret from an environment variable")
}

/// Exercise 2: Read a secret with a default value.
///
/// If the environment variable is not set, return the default value.
/// If it IS set, return the environment variable value (ignoring the default).
///
/// Hints:
/// - Use `std::env::var(name)` and match on the result
/// - `Ok(val)` → return val
/// - `Err(_)` → return default_value
pub fn read_secret_or_default(name: &str, default_value: &str) -> String {
    todo!("Read env var with fallback default")
}

/// Exercise 3: Read multiple secrets and validate they're all present.
///
/// Given a list of required secret names, read each from the environment.
/// Return `Ok(map)` if ALL are present, or `Err(missing_names)` listing
/// which ones are missing.
///
/// Hints:
/// - Iterate over `required_names`
/// - For each, try `std::env::var(name)`
/// - Collect missing ones into a Vec
/// - If the missing Vec is empty, return Ok with the HashMap
pub fn read_required_secrets(required_names: &[&str]) -> Result<HashMap<String, String>, Vec<String>> {
    todo!("Read multiple required secrets, reporting missing ones")
}

/// Exercise 4: Mask a secret for safe logging.
///
/// Given a secret value, return a masked version that shows only the last 4 characters.
/// If the secret is 8 characters or fewer, mask everything.
///
/// Examples:
/// - "sk-1234567890abcdef" → "**************cdef"
/// - "short" → "*****"
/// - "1234" → "****"
///
/// Hints:
/// - Check the length of the secret
/// - If <= 8, return "*".repeat(secret.len())
/// - Otherwise, compute the visible suffix length (4) and masked prefix length
/// - Build the string: "*".repeat(masked_len) + &secret[masked_len..]
pub fn mask_secret(secret: &str) -> String {
    todo!("Mask a secret for safe logging")
}

/// Exercise 5: Simulate the danger of `.env` file exposure.
///
/// Given the contents of a `.env` file as a string, parse it and return
/// all key-value pairs. This shows how easy it is to extract secrets
/// from a `.env` file.
///
/// The format is: `KEY=VALUE` per line, with optional comments (lines starting with #)
/// and optional blank lines. Values may be quoted with single or double quotes.
///
/// Hints:
/// - Split by newlines
/// - Skip lines starting with '#' or empty lines
/// - Split on the first '=' character
/// - Trim quotes from values if present
pub fn parse_env_file(contents: &str) -> HashMap<String, String> {
    todo!("Parse a .env file and extract all key-value pairs")
}

/// Exercise 6: Check if a string looks like it might contain a secret.
///
/// This is a simple heuristic: return true if the value looks like it could be
/// a secret (long, contains mixed case, digits, or special characters).
///
/// Return true if ANY of these conditions are met:
/// - Length >= 16 and contains both letters and digits
/// - Starts with common secret prefixes: "sk-", "pk-", "key-", "token-", "bearer "
/// - Contains "password" or "secret" (case-insensitive) in the key name
///
/// Parameters:
/// - `key`: the variable name (e.g., "API_KEY", "DATABASE_PASSWORD")
/// - `value`: the variable value
pub fn looks_like_secret(key: &str, value: &str) -> bool {
    todo!("Heuristic: does this look like a secret?")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_secret_exists() {
        // Set an env var for testing
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
