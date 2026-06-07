//! # Request Validation
//!
//! Input validation is the first line of defense against bad data and attacks.
//! This lesson covers validation strategies including serde-based validation,
//! custom validators, validation middleware, and structured error responses.
//!
//! ## Key Concepts
//! - Serde deserialization with validation
//! - Custom validation functions and types
//! - Field-level and cross-field validation
//! - Structured validation error responses
//! - Sanitization of inputs
//! - Extractor-based validation in web frameworks

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. Validation Result Types
// ---------------------------------------------------------------------------

/// The result of validating a single field.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FieldViolation {
    pub field: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_value: Option<String>,
}

/// Aggregated validation result.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<FieldViolation>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            violations: Vec::new(),
        }
    }

    pub fn error(violations: Vec<FieldViolation>) -> Self {
        Self {
            valid: false,
            violations,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    /// Merge another validation result into this one.
    pub fn merge(mut self, other: ValidationResult) -> Self {
        if !other.valid {
            self.valid = false;
        }
        self.violations.extend(other.violations);
        self
    }
}

// ---------------------------------------------------------------------------
// 2. Validator Traits and Combinators
// ---------------------------------------------------------------------------

/// A trait for types that can validate themselves.
pub trait Validate {
    fn validate(&self) -> ValidationResult;
}

/// Validates that a string is not empty or whitespace.
pub fn validate_not_empty(value: &str, field: &str) -> Option<FieldViolation> {
    if value.trim().is_empty() {
        Some(FieldViolation {
            field: field.into(),
            code: "REQUIRED".into(),
            message: format!("{field} cannot be empty"),
            rejected_value: Some(value.to_string()),
        })
    } else {
        None
    }
}

/// Validates string length is within bounds.
pub fn validate_length(value: &str, field: &str, min: usize, max: usize) -> Option<FieldViolation> {
    let len = value.len();
    if len < min {
        Some(FieldViolation {
            field: field.into(),
            code: "TOO_SHORT".into(),
            message: format!("{field} must be at least {min} characters"),
            rejected_value: None,
        })
    } else if len > max {
        Some(FieldViolation {
            field: field.into(),
            code: "TOO_LONG".into(),
            message: format!("{field} must be at most {max} characters"),
            rejected_value: None,
        })
    } else {
        None
    }
}

/// Validates that a string looks like an email address.
pub fn validate_email(value: &str, field: &str) -> Option<FieldViolation> {
    // Simplified but practical email validation
    let parts: Vec<&str> = value.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || !parts[1].contains('.') || parts[1].ends_with('.') {
        Some(FieldViolation {
            field: field.into(),
            code: "INVALID_FORMAT".into(),
            message: format!("{field} must be a valid email address"),
            rejected_value: Some(value.to_string()),
        })
    } else {
        None
    }
}

/// Validates a numeric value is within range.
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    field: &str,
    min: Option<T>,
    max: Option<T>,
) -> Option<FieldViolation> {
    if let Some(ref m) = min {
        if value < *m {
            return Some(FieldViolation {
                field: field.into(),
                code: "TOO_SMALL".into(),
                message: format!("{field} must be at least {m}"),
                rejected_value: Some(value.to_string()),
            });
        }
    }
    if let Some(ref m) = max {
        if value > *m {
            return Some(FieldViolation {
                field: field.into(),
                code: "TOO_LARGE".into(),
                message: format!("{field} must be at most {m}"),
                rejected_value: Some(value.to_string()),
            });
        }
    }
    None
}

/// Validates a URL string.
pub fn validate_url(value: &str, field: &str) -> Option<FieldViolation> {
    if !value.starts_with("http://") && !value.starts_with("https://") {
        Some(FieldViolation {
            field: field.into(),
            code: "INVALID_URL".into(),
            message: format!("{field} must be a valid HTTP(S) URL"),
            rejected_value: Some(value.to_string()),
        })
    } else if value.len() > 2048 {
        Some(FieldViolation {
            field: field.into(),
            code: "TOO_LONG".into(),
            message: format!("{field} must not exceed 2048 characters"),
            rejected_value: None,
        })
    } else {
        None
    }
}

/// Validates a string matches a regex-like pattern (simplified).
pub fn validate_pattern(value: &str, field: &str, pattern_name: &str, check: fn(&str) -> bool) -> Option<FieldViolation> {
    if !check(value) {
        Some(FieldViolation {
            field: field.into(),
            code: "INVALID_FORMAT".into(),
            message: format!("{field} does not match required pattern: {pattern_name}"),
            rejected_value: Some(value.to_string()),
        })
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// 3. Domain Model Validation
// ---------------------------------------------------------------------------

/// A user registration request that needs thorough validation.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_confirm: String,
    pub age: Option<u32>,
    pub website: Option<String>,
    pub bio: Option<String>,
}

impl Validate for RegisterRequest {
    fn validate(&self) -> ValidationResult {
        let mut violations = Vec::new();

        // Username validation
        if let Some(v) = validate_not_empty(&self.username, "username") {
            violations.push(v);
        }
        if let Some(v) = validate_length(&self.username, "username", 3, 32) {
            violations.push(v);
        }
        if let Some(v) = validate_pattern(&self.username, "username", "alphanumeric with hyphens/underscores", |s| {
            s.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        }) {
            violations.push(v);
        }

        // Email validation
        if let Some(v) = validate_not_empty(&self.email, "email") {
            violations.push(v);
        }
        if let Some(v) = validate_email(&self.email, "email") {
            violations.push(v);
        }

        // Password validation
        if let Some(v) = validate_not_empty(&self.password, "password") {
            violations.push(v);
        }
        if let Some(v) = validate_length(&self.password, "password", 8, 128) {
            violations.push(v);
        }
        // Password strength
        if !self.password.chars().any(|c| c.is_uppercase()) {
            violations.push(FieldViolation {
                field: "password".into(),
                code: "WEAK_PASSWORD".into(),
                message: "password must contain at least one uppercase letter".into(),
                rejected_value: None,
            });
        }
        if !self.password.chars().any(|c| c.is_ascii_digit()) {
            violations.push(FieldViolation {
                field: "password".into(),
                code: "WEAK_PASSWORD".into(),
                message: "password must contain at least one digit".into(),
                rejected_value: None,
            });
        }

        // Cross-field validation: passwords must match
        if self.password != self.password_confirm {
            violations.push(FieldViolation {
                field: "password_confirm".into(),
                code: "MISMATCH".into(),
                message: "password confirmation does not match".into(),
                rejected_value: None,
            });
        }

        // Age validation (optional)
        if let Some(age) = self.age {
            if let Some(v) = validate_range(age, "age", Some(13), Some(150)) {
                violations.push(v);
            }
        }

        // Website URL validation (optional)
        if let Some(ref url) = self.website {
            if let Some(v) = validate_url(url, "website") {
                violations.push(v);
            }
        }

        // Bio length validation (optional)
        if let Some(ref bio) = self.bio {
            if let Some(v) = validate_length(bio, "bio", 0, 500) {
                violations.push(v);
            }
        }

        if violations.is_empty() {
            ValidationResult::ok()
        } else {
            ValidationResult::error(violations)
        }
    }
}

/// An order creation request with business rule validation.
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: String,
    pub items: Vec<OrderItemRequest>,
    pub coupon_code: Option<String>,
    pub shipping_address: AddressRequest,
}

#[derive(Debug, Deserialize)]
pub struct OrderItemRequest {
    pub product_id: String,
    pub quantity: u32,
}

#[derive(Debug, Deserialize)]
pub struct AddressRequest {
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub country: String,
}

impl Validate for CreateOrderRequest {
    fn validate(&self) -> ValidationResult {
        let mut violations = Vec::new();

        if let Some(v) = validate_not_empty(&self.customer_id, "customer_id") {
            violations.push(v);
        }

        // Must have at least one item
        if self.items.is_empty() {
            violations.push(FieldViolation {
                field: "items".into(),
                code: "REQUIRED".into(),
                message: "order must contain at least one item".into(),
                rejected_value: None,
            });
        }

        // Validate each item
        for (i, item) in self.items.iter().enumerate() {
            let field_prefix = format!("items[{i}]");

            if item.product_id.trim().is_empty() {
                violations.push(FieldViolation {
                    field: format!("{field_prefix}.product_id"),
                    code: "REQUIRED".into(),
                    message: "product_id cannot be empty".into(),
                    rejected_value: None,
                });
            }

            if item.quantity == 0 {
                violations.push(FieldViolation {
                    field: format!("{field_prefix}.quantity"),
                    code: "TOO_SMALL".into(),
                    message: "quantity must be at least 1".into(),
                    rejected_value: Some("0".into()),
                });
            }

            if item.quantity > 1000 {
                violations.push(FieldViolation {
                    field: format!("{field_prefix}.quantity"),
                    code: "TOO_LARGE".into(),
                    message: "quantity cannot exceed 1000".into(),
                    rejected_value: Some(item.quantity.to_string()),
                });
            }
        }

        // Validate address
        let addr = &self.shipping_address;
        if let Some(v) = validate_not_empty(&addr.street, "shipping_address.street") {
            violations.push(v);
        }
        if let Some(v) = validate_not_empty(&addr.city, "shipping_address.city") {
            violations.push(v);
        }
        if let Some(v) = validate_not_empty(&addr.zip, "shipping_address.zip") {
            violations.push(v);
        }
        // Validate country is 2-letter ISO code
        if addr.country.len() != 2 || !addr.country.chars().all(|c| c.is_ascii_uppercase()) {
            violations.push(FieldViolation {
                field: "shipping_address.country".into(),
                code: "INVALID_FORMAT".into(),
                message: "country must be a 2-letter uppercase ISO code (e.g. US, GB)".into(),
                rejected_value: Some(addr.country.clone()),
            });
        }

        if violations.is_empty() {
            ValidationResult::ok()
        } else {
            ValidationResult::error(violations)
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Sanitization
// ---------------------------------------------------------------------------

/// Sanitize a string by removing or escaping potentially dangerous characters.
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
        .collect()
}

/// Sanitize HTML by escaping special characters.
pub fn sanitize_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Normalize an email address to lowercase and trim whitespace.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Truncate a string to a maximum length, appending "..." if truncated.
pub fn truncate_string(input: &str, max_len: usize) -> String {
    if input.len() <= max_len {
        input.to_string()
    } else {
        let truncated: String = input.chars().take(max_len.saturating_sub(3)).collect();
        format!("{truncated}...")
    }
}

// ---------------------------------------------------------------------------
// 5. Validation Middleware / Extractor Pattern
// ---------------------------------------------------------------------------

/// A validated wrapper that guarantees the inner value has been validated.
/// This is the extractor pattern: the web framework validates before the handler runs.
#[derive(Debug)]
pub struct Validated<T: Validate>(pub T);

impl<T: Validate> Validated<T> {
    /// Attempt to create a Validated value. Returns error details on failure.
    pub fn try_new(value: T) -> Result<Self, ValidationResult> {
        let result = value.validate();
        if result.is_valid() {
            Ok(Self(value))
        } else {
            Err(result)
        }
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("hello", "name").is_none());
        assert!(validate_not_empty("", "name").is_some());
        assert!(validate_not_empty("   ", "name").is_some());
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("hello", "name", 3, 10).is_none());
        assert!(validate_length("hi", "name", 3, 10).is_some());
        assert!(validate_length("a".repeat(11).as_str(), "name", 3, 10).is_some());
    }

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com", "email").is_none());
        assert!(validate_email("a.b+c@domain.co", "email").is_none());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("not-an-email", "email").is_some());
        assert!(validate_email("@domain.com", "email").is_some());
        assert!(validate_email("user@domain.", "email").is_some());
        assert!(validate_email("", "email").is_some());
    }

    #[test]
    fn test_validate_range() {
        assert!(validate_range(5, "age", Some(1), Some(10)).is_none());
        assert!(validate_range(0, "age", Some(1), Some(10)).is_some());
        assert!(validate_range(11, "age", Some(1), Some(10)).is_some());
        // No bounds
        assert!(validate_range(100, "val", None::<i32>, None::<i32>).is_none());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com", "url").is_none());
        assert!(validate_url("http://localhost:8080", "url").is_none());
        assert!(validate_url("ftp://server", "url").is_some());
        assert!(validate_url("not-a-url", "url").is_some());
    }

    #[test]
    fn test_validate_pattern() {
        let is_alphanumeric = |s: &str| s.chars().all(|c| c.is_alphanumeric());
        assert!(validate_pattern("hello123", "name", "alphanumeric", is_alphanumeric).is_none());
        assert!(validate_pattern("hello world", "name", "alphanumeric", is_alphanumeric).is_some());
    }

    #[test]
    fn test_register_request_valid() {
        let req = RegisterRequest {
            username: "alice_42".into(),
            email: "alice@example.com".into(),
            password: "SecureP4ss".into(),
            password_confirm: "SecureP4ss".into(),
            age: Some(25),
            website: Some("https://alice.dev".into()),
            bio: Some("Hello world".into()),
        };

        let result = req.validate();
        assert!(result.is_valid(), "violations: {:?}", result.violations);
    }

    #[test]
    fn test_register_request_empty_username() {
        let req = RegisterRequest {
            username: "".into(),
            email: "alice@example.com".into(),
            password: "SecureP4ss".into(),
            password_confirm: "SecureP4ss".into(),
            age: None,
            website: None,
            bio: None,
        };

        let result = req.validate();
        assert!(!result.is_valid());
        assert!(result.violations.iter().any(|v| v.field == "username" && v.code == "REQUIRED"));
    }

    #[test]
    fn test_register_request_password_mismatch() {
        let req = RegisterRequest {
            username: "alice".into(),
            email: "alice@example.com".into(),
            password: "SecureP4ss1".into(),
            password_confirm: "SecureP4ss2".into(),
            age: None,
            website: None,
            bio: None,
        };

        let result = req.validate();
        assert!(!result.is_valid());
        assert!(result.violations.iter().any(|v| v.field == "password_confirm"));
    }

    #[test]
    fn test_register_request_weak_password() {
        let req = RegisterRequest {
            username: "alice".into(),
            email: "alice@example.com".into(),
            password: "alllowercase1".into(),
            password_confirm: "alllowercase1".into(),
            age: None,
            website: None,
            bio: None,
        };

        let result = req.validate();
        assert!(!result.is_valid());
        assert!(result.violations.iter().any(|v| v.code == "WEAK_PASSWORD"));
    }

    #[test]
    fn test_register_request_too_young() {
        let req = RegisterRequest {
            username: "alice".into(),
            email: "alice@example.com".into(),
            password: "SecureP4ss".into(),
            password_confirm: "SecureP4ss".into(),
            age: Some(10),
            website: None,
            bio: None,
        };

        let result = req.validate();
        assert!(!result.is_valid());
        assert!(result.violations.iter().any(|v| v.field == "age"));
    }

    #[test]
    fn test_register_request_multiple_errors() {
        let req = RegisterRequest {
            username: "a".into(),          // too short
            email: "bad".into(),           // invalid
            password: "short".into(),      // too short, weak
            password_confirm: "different".into(), // mismatch
            age: Some(200),               // too old
            website: Some("ftp://bad".into()), // invalid URL
            bio: None,
        };

        let result = req.validate();
        assert!(!result.is_valid());
        assert!(result.violations.len() >= 5);
    }

    #[test]
    fn test_create_order_valid() {
        let req = CreateOrderRequest {
            customer_id: "cust-1".into(),
            items: vec![
                OrderItemRequest {
                    product_id: "prod-1".into(),
                    quantity: 2,
                },
            ],
            coupon_code: None,
            shipping_address: AddressRequest {
                street: "123 Main St".into(),
                city: "Springfield".into(),
                state: "IL".into(),
                zip: "62701".into(),
                country: "US".into(),
            },
        };

        let result = req.validate();
        assert!(result.is_valid(), "violations: {:?}", result.violations);
    }

    #[test]
    fn test_create_order_empty_items() {
        let req = CreateOrderRequest {
            customer_id: "cust-1".into(),
            items: vec![],
            coupon_code: None,
            shipping_address: AddressRequest {
                street: "123 Main St".into(),
                city: "Springfield".into(),
                state: "IL".into(),
                zip: "62701".into(),
                country: "US".into(),
            },
        };

        let result = req.validate();
        assert!(!result.is_valid());
        assert!(result.violations.iter().any(|v| v.field == "items"));
    }

    #[test]
    fn test_create_order_zero_quantity() {
        let req = CreateOrderRequest {
            customer_id: "cust-1".into(),
            items: vec![OrderItemRequest {
                product_id: "prod-1".into(),
                quantity: 0,
            }],
            coupon_code: None,
            shipping_address: AddressRequest {
                street: "123 Main St".into(),
                city: "Springfield".into(),
                state: "IL".into(),
                zip: "62701".into(),
                country: "US".into(),
            },
        };

        let result = req.validate();
        assert!(!result.is_valid());
        assert!(result.violations.iter().any(|v| v.field.contains("quantity")));
    }

    #[test]
    fn test_create_order_invalid_country() {
        let req = CreateOrderRequest {
            customer_id: "cust-1".into(),
            items: vec![OrderItemRequest {
                product_id: "p1".into(),
                quantity: 1,
            }],
            coupon_code: None,
            shipping_address: AddressRequest {
                street: "123 Main".into(),
                city: "City".into(),
                state: "ST".into(),
                zip: "12345".into(),
                country: "USA".into(), // should be 2-letter
            },
        };

        let result = req.validate();
        assert!(!result.is_valid());
        assert!(result
            .violations
            .iter()
            .any(|v| v.field == "shipping_address.country"));
    }

    #[test]
    fn test_validation_result_merge() {
        let r1 = ValidationResult::error(vec![FieldViolation {
            field: "a".into(),
            code: "ERR".into(),
            message: "err".into(),
            rejected_value: None,
        }]);
        let r2 = ValidationResult::error(vec![FieldViolation {
            field: "b".into(),
            code: "ERR".into(),
            message: "err".into(),
            rejected_value: None,
        }]);

        let merged = r1.merge(r2);
        assert!(!merged.is_valid());
        assert_eq!(merged.violations.len(), 2);
    }

    #[test]
    fn test_validation_result_merge_ok_into_error() {
        let r1 = ValidationResult::ok();
        let r2 = ValidationResult::error(vec![FieldViolation {
            field: "x".into(),
            code: "ERR".into(),
            message: "err".into(),
            rejected_value: None,
        }]);

        let merged = r1.merge(r2);
        assert!(!merged.is_valid());
    }

    #[test]
    fn test_sanitize_string() {
        assert_eq!(sanitize_string("hello\nworld"), "hello\nworld");
        assert_eq!(sanitize_string("tab\there"), "tab\there");
        // Control chars (except newline, CR, tab) are removed
        assert_eq!(sanitize_string("a\x00b\x01c"), "abc");
    }

    #[test]
    fn test_sanitize_html() {
        assert_eq!(
            sanitize_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
        assert_eq!(sanitize_html("a & b"), "a &amp; b");
        assert_eq!(sanitize_html("\"quoted\""), "&quot;quoted&quot;");
    }

    #[test]
    fn test_normalize_email() {
        assert_eq!(normalize_email("  User@Example.COM  "), "user@example.com");
        assert_eq!(normalize_email("test@test.org"), "test@test.org");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 2), "hi");
        assert_eq!(truncate_string("a".repeat(20).as_str(), 5), "aa...");
    }

    #[test]
    fn test_validated_extractor_success() {
        let req = RegisterRequest {
            username: "alice".into(),
            email: "alice@example.com".into(),
            password: "SecureP4ss".into(),
            password_confirm: "SecureP4ss".into(),
            age: None,
            website: None,
            bio: None,
        };

        let validated = Validated::try_new(req);
        assert!(validated.is_ok());
    }

    #[test]
    fn test_validated_extractor_failure() {
        let req = RegisterRequest {
            username: "".into(),
            email: "bad".into(),
            password: "weak".into(),
            password_confirm: "different".into(),
            age: None,
            website: None,
            bio: None,
        };

        let validated = Validated::try_new(req);
        assert!(validated.is_err());
        let result = validated.unwrap_err();
        assert!(!result.is_valid());
        assert!(!result.violations.is_empty());
    }
}
