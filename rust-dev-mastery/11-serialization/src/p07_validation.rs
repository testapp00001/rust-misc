//! # Validation with Serde
//!
//! Validating data during deserialization ensures that your types are always
//! in a valid state. This lesson covers patterns for validation in serde.
//!
//! Key concepts:
//! - Custom deserialization with validation
//! - Using `serde(with)` for field-level validation
//! - Pre-deserialization checks
//! - Collecting multiple validation errors

use serde::de::{self, Deserialize, Deserializer};
use serde::{Deserialize as DeserializeTrait, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Demonstrates a validated newtype with a custom Deserialize impl.
#[derive(Debug, Clone, PartialEq)]
pub struct Email(pub String);

impl Email {
    pub fn new(s: String) -> Result<Self, ValidationError> {
        if !s.contains('@') || !s.contains('.') {
            return Err(ValidationError::new("invalid email format"));
        }
        if s.len() > 254 {
            return Err(ValidationError::new("email too long"));
        }
        Ok(Email(s))
    }
}

impl Serialize for Email {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Email {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Email::new(s).map_err(de::Error::custom)
    }
}

/// Demonstrates a validated numeric range type.
#[derive(Debug, Clone, PartialEq)]
pub struct Percentage(pub f64);

impl Percentage {
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if !(0.0..=100.0).contains(&value) {
            return Err(ValidationError::new("percentage must be between 0 and 100"));
        }
        Ok(Percentage(value))
    }
}

impl Serialize for Percentage {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let f = f64::deserialize(deserializer)?;
        Percentage::new(f).map_err(de::Error::custom)
    }
}

/// Demonstrates a validated string with length constraints.
#[derive(Debug, Clone, PartialEq)]
pub struct NonEmptyString(pub String);

impl NonEmptyString {
    pub fn new(s: String) -> Result<Self, ValidationError> {
        if s.is_empty() {
            return Err(ValidationError::new("string must not be empty"));
        }
        if s.len() > 1000 {
            return Err(ValidationError::new("string too long (max 1000)"));
        }
        Ok(NonEmptyString(s))
    }
}

impl Serialize for NonEmptyString {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for NonEmptyString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        NonEmptyString::new(s).map_err(de::Error::custom)
    }
}

/// Custom validation error type.
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
}

impl ValidationError {
    pub fn new(message: &str) -> Self {
        ValidationError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "validation error: {}", self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Demonstrates a validated form with multiple fields.
#[derive(Debug, Clone, Serialize, DeserializeTrait)]
pub struct RegistrationForm {
    pub username: NonEmptyString,
    pub email: Email,
    pub age: u32,
    pub score: Percentage,
    pub bio: Option<String>,
}

/// Demonstrates collecting multiple validation errors.
#[derive(Debug, Clone)]
pub struct ValidationErrors {
    pub errors: Vec<(String, String)>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        ValidationErrors { errors: Vec::new() }
    }

    pub fn add(&mut self, field: &str, message: &str) {
        self.errors
            .push((field.to_string(), message.to_string()));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (field, msg) in &self.errors {
            writeln!(f, "  {field}: {msg}")?;
        }
        Ok(())
    }
}

/// Demonstrates post-deserialization validation for complex rules.
#[derive(Debug, Clone, Serialize, DeserializeTrait)]
pub struct Config {
    pub min_connections: u32,
    pub max_connections: u32,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.min_connections > self.max_connections {
            errors.add(
                "min_connections",
                "must be <= max_connections",
            );
        }

        if self.host.is_empty() {
            errors.add("host", "must not be empty");
        }

        if self.port == 0 {
            errors.add("port", "must be non-zero");
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

/// Demonstrates a validator trait for composable validation.
pub trait Validate {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// A validated URL type.
#[derive(Debug, Clone, PartialEq)]
pub struct Url(pub String);

impl Url {
    pub fn new(s: String) -> Result<Self, ValidationError> {
        if !s.starts_with("http://") && !s.starts_with("https://") {
            return Err(ValidationError::new("URL must start with http:// or https://"));
        }
        Ok(Url(s))
    }
}

impl Serialize for Url {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Url {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Url::new(s).map_err(de::Error::custom)
    }
}

/// Demonstrates a validated struct with the Validate trait.
#[derive(Debug, Clone, Serialize, DeserializeTrait)]
pub struct WebhookConfig {
    pub url: Url,
    pub timeout_secs: u32,
    pub retry_count: u32,
    pub headers: HashMap<String, String>,
}

impl Validate for WebhookConfig {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.timeout_secs == 0 {
            errors.add("timeout_secs", "must be non-zero");
        }
        if self.timeout_secs > 300 {
            errors.add("timeout_secs", "must be <= 300");
        }
        if self.retry_count > 10 {
            errors.add("retry_count", "must be <= 10");
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

/// Demonstrates conditional validation.
#[derive(Debug, Clone, Serialize, DeserializeTrait)]
pub struct PaymentRequest {
    pub amount: f64,
    pub currency: String,
    #[serde(default)]
    pub card_number: Option<String>,
    #[serde(default)]
    pub bank_account: Option<String>,
}

impl PaymentRequest {
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if self.amount <= 0.0 {
            errors.add("amount", "must be positive");
        }

        if self.currency.len() != 3 {
            errors.add("currency", "must be a 3-letter currency code");
        }

        if self.card_number.is_none() && self.bank_account.is_none() {
            errors.add("_", "either card_number or bank_account must be provided");
        }

        if let Some(ref card) = self.card_number {
            if card.len() < 13 || card.len() > 19 {
                errors.add("card_number", "must be 13-19 digits");
            }
        }

        if errors.has_errors() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

/// Demonstrates a helper function for deserialize-and-validate.
pub fn deserialize_and_validate<'de, T>(json: &'de str) -> Result<T, Box<dyn std::error::Error>>
where
    T: Deserialize<'de> + Validate,
{
    let value: T = serde_json::from_str(json)?;
    value.validate().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(value)
}

impl std::error::Error for ValidationErrors {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_valid() {
        let email: Email = serde_json::from_str("\"user@example.com\"").unwrap();
        assert_eq!(email.0, "user@example.com");
    }

    #[test]
    fn test_email_invalid() {
        assert!(serde_json::from_str::<Email>("\"invalid\"").is_err());
    }

    #[test]
    fn test_percentage_valid() {
        let p: Percentage = serde_json::from_str("50.5").unwrap();
        assert_eq!(p.0, 50.5);
    }

    #[test]
    fn test_percentage_out_of_range() {
        assert!(serde_json::from_str::<Percentage>("101.0").is_err());
        assert!(serde_json::from_str::<Percentage>("-1.0").is_err());
    }

    #[test]
    fn test_non_empty_string() {
        let s: NonEmptyString = serde_json::from_str("\"hello\"").unwrap();
        assert_eq!(s.0, "hello");
    }

    #[test]
    fn test_non_empty_string_empty() {
        assert!(serde_json::from_str::<NonEmptyString>("\"\"").is_err());
    }

    #[test]
    fn test_registration_form() {
        let json = r#"{
            "username": "alice",
            "email": "alice@example.com",
            "age": 25,
            "score": 85.5
        }"#;
        let form: RegistrationForm = serde_json::from_str(json).unwrap();
        assert_eq!(form.username.0, "alice");
        assert_eq!(form.email.0, "alice@example.com");
    }

    #[test]
    fn test_config_validation() {
        let config = Config {
            min_connections: 10,
            max_connections: 5,
            host: String::new(),
            port: 0,
        };
        let errors = config.validate().unwrap_err();
        assert!(errors.errors.len() >= 3);
    }

    #[test]
    fn test_config_valid() {
        let config = Config {
            min_connections: 1,
            max_connections: 100,
            host: "localhost".to_string(),
            port: 8080,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_webhook_validation() {
        let config = WebhookConfig {
            url: Url::new("https://example.com".to_string()).unwrap(),
            timeout_secs: 0,
            retry_count: 20,
            headers: HashMap::new(),
        };
        let errors = config.validate().unwrap_err();
        assert!(errors.errors.len() >= 2);
    }

    #[test]
    fn test_payment_validation() {
        let payment = PaymentRequest {
            amount: 100.0,
            currency: "USD".to_string(),
            card_number: None,
            bank_account: None,
        };
        let errors = payment.validate().unwrap_err();
        assert!(errors.errors.iter().any(|(f, _)| f == "_"));
    }

    #[test]
    fn test_url_valid() {
        let url: Url = serde_json::from_str("\"https://example.com\"").unwrap();
        assert_eq!(url.0, "https://example.com");
    }

    #[test]
    fn test_url_invalid() {
        assert!(serde_json::from_str::<Url>("\"ftp://example.com\"").is_err());
    }

    #[test]
    fn test_deserialize_and_validate() {
        let json = r#"{
            "url": "https://example.com",
            "timeout_secs": 30,
            "retry_count": 3,
            "headers": {}
        }"#;
        let config: WebhookConfig = deserialize_and_validate(json).unwrap();
        assert_eq!(config.timeout_secs, 30);
    }

    #[test]
    fn test_deserialize_and_validate_fails() {
        let json = r#"{
            "url": "https://example.com",
            "timeout_secs": 0,
            "retry_count": 0,
            "headers": {}
        }"#;
        let result = deserialize_and_validate::<WebhookConfig>(json);
        assert!(result.is_err());
    }
}
