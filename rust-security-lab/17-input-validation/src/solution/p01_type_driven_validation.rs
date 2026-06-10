//! # Lesson 01: Type-Driven Validation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Validation error on '{}': {}", self.field, self.message)
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn new(raw: &str) -> Result<Self, ValidationError> {
        if raw.chars().any(|c| c.is_whitespace()) {
            return Err(ValidationError {
                field: "email".into(),
                message: "must not contain whitespace".into(),
            });
        }

        let at_count = raw.matches('@').count();
        if at_count != 1 {
            return Err(ValidationError {
                field: "email".into(),
                message: "must contain exactly one '@'".into(),
            });
        }

        let parts: Vec<&str> = raw.splitn(2, '@').collect();
        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() {
            return Err(ValidationError {
                field: "email".into(),
                message: "local part must not be empty".into(),
            });
        }

        if !domain.contains('.') {
            return Err(ValidationError {
                field: "email".into(),
                message: "domain must contain at least one '.'".into(),
            });
        }

        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Username(String);

impl Username {
    pub fn new(raw: &str) -> Result<Self, ValidationError> {
        if raw.len() < 3 || raw.len() > 32 {
            return Err(ValidationError {
                field: "username".into(),
                message: "length must be between 3 and 32".into(),
            });
        }

        let first = raw.chars().next().unwrap();
        if !first.is_ascii_alphabetic() {
            return Err(ValidationError {
                field: "username".into(),
                message: "must start with an alphabetic character".into(),
            });
        }

        if !raw
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ValidationError {
                field: "username".into(),
                message: "must contain only alphanumeric, underscore, or hyphen".into(),
            });
        }

        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

const BLOCKED_PORTS: &[u16] = &[22, 25, 445, 3389];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(u16);

impl Port {
    pub fn new(raw: u16) -> Result<Self, ValidationError> {
        if raw == 0 {
            return Err(ValidationError {
                field: "port".into(),
                message: "port must not be 0".into(),
            });
        }

        if BLOCKED_PORTS.contains(&raw) {
            return Err(ValidationError {
                field: "port".into(),
                message: format!("port {} is blocked for security reasons", raw),
            });
        }

        Ok(Self(raw))
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct UserConfig {
    pub email: Email,
    pub username: Username,
    pub display_name: String,
    pub age: u8,
}

pub struct UserConfigBuilder {
    email: Option<Email>,
    username: Option<Username>,
    display_name: Option<String>,
    age: Option<u8>,
}

impl UserConfigBuilder {
    pub fn new() -> Self {
        Self {
            email: None,
            username: None,
            display_name: None,
            age: None,
        }
    }

    pub fn email(mut self, raw: &str) -> Result<Self, ValidationError> {
        self.email = Some(Email::new(raw)?);
        Ok(self)
    }

    pub fn username(mut self, raw: &str) -> Result<Self, ValidationError> {
        self.username = Some(Username::new(raw)?);
        Ok(self)
    }

    pub fn display_name(mut self, raw: &str) -> Result<Self, ValidationError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.len() > 64 {
            return Err(ValidationError {
                field: "display_name".into(),
                message: "must be 1-64 characters after trimming".into(),
            });
        }
        self.display_name = Some(trimmed.to_string());
        Ok(self)
    }

    pub fn age(mut self, raw: u8) -> Result<Self, ValidationError> {
        self.age = Some(raw);
        Ok(self)
    }

    pub fn build(self) -> Result<UserConfig, ValidationError> {
        let email = self.email.ok_or(ValidationError {
            field: "email".into(),
            message: "email is required".into(),
        })?;
        let username = self.username.ok_or(ValidationError {
            field: "username".into(),
            message: "username is required".into(),
        })?;
        let display_name = self.display_name.ok_or(ValidationError {
            field: "display_name".into(),
            message: "display_name is required".into(),
        })?;
        let age = self.age.ok_or(ValidationError {
            field: "age".into(),
            message: "age is required".into(),
        })?;

        if age < 13 || age > 150 {
            return Err(ValidationError {
                field: "age".into(),
                message: "age must be between 13 and 150 (COPPA compliance)".into(),
            });
        }

        Ok(UserConfig {
            email,
            username,
            display_name,
            age,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedString(String);

impl SanitizedString {
    pub fn new(raw: &str) -> Result<Self, ValidationError> {
        let cleaned: String = raw
            .chars()
            .filter(|c| *c != '\0' && (!c.is_control() || *c == '\n' || *c == '\t'))
            .collect();
        let trimmed = cleaned.trim().to_string();

        if trimmed.len() > 1000 {
            return Err(ValidationError {
                field: "sanitized_string".into(),
                message: "must not exceed 1000 characters".into(),
            });
        }

        Ok(Self(trimmed))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_valid() {
        let email = Email::new("alice@example.com").unwrap();
        assert_eq!(email.as_str(), "alice@example.com");
    }

    #[test]
    fn test_email_no_at() {
        assert!(Email::new("aliceexample.com").is_err());
    }

    #[test]
    fn test_email_no_domain_dot() {
        assert!(Email::new("alice@localhost").is_err());
    }

    #[test]
    fn test_email_empty_local() {
        assert!(Email::new("@example.com").is_err());
    }

    #[test]
    fn test_email_whitespace() {
        assert!(Email::new("alice @example.com").is_err());
    }

    #[test]
    fn test_username_valid() {
        let user = Username::new("alice_99").unwrap();
        assert_eq!(user.as_str(), "alice_99");
    }

    #[test]
    fn test_username_too_short() {
        assert!(Username::new("ab").is_err());
    }

    #[test]
    fn test_username_starts_with_number() {
        assert!(Username::new("1alice").is_err());
    }

    #[test]
    fn test_username_special_chars() {
        assert!(Username::new("alice@home").is_err());
    }

    #[test]
    fn test_port_valid() {
        let port = Port::new(8080).unwrap();
        assert_eq!(port.value(), 8080);
    }

    #[test]
    fn test_port_zero() {
        assert!(Port::new(0).is_err());
    }

    #[test]
    fn test_port_blocked() {
        assert!(Port::new(22).is_err());
        assert!(Port::new(445).is_err());
    }

    #[test]
    fn test_builder_valid() {
        let config = UserConfigBuilder::new()
            .email("alice@example.com").unwrap()
            .username("alice").unwrap()
            .display_name("Alice").unwrap()
            .age(25).unwrap()
            .build().unwrap();
        assert_eq!(config.email.as_str(), "alice@example.com");
        assert_eq!(config.username.as_str(), "alice");
        assert_eq!(config.age, 25);
    }

    #[test]
    fn test_builder_missing_field() {
        let result = UserConfigBuilder::new()
            .email("alice@example.com").unwrap()
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_underage() {
        let result = UserConfigBuilder::new()
            .email("child@example.com").unwrap()
            .username("child").unwrap()
            .display_name("Child").unwrap()
            .age(10).unwrap()
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitized_strips_null_bytes() {
        let s = SanitizedString::new("hello\0world").unwrap();
        assert_eq!(s.as_str(), "helloworld");
    }

    #[test]
    fn test_sanitized_trims_whitespace() {
        let s = SanitizedString::new("  hello  ").unwrap();
        assert_eq!(s.as_str(), "hello");
    }

    #[test]
    fn test_sanitized_too_long() {
        let long = "a".repeat(1001);
        assert!(SanitizedString::new(&long).is_err());
    }
}
