//! # Newtype Pattern
//!
//! The newtype pattern wraps a primitive type in a single-field struct to create
//! a distinct type with its own semantics. This prevents primitive obsession and
//! makes APIs self-documenting.
//!
//! ## Key Concepts
//! - **Type safety**: Different IDs (UserId vs OrderId) can't be mixed
//! - **Domain semantics**: Types carry meaning (EmailAddress, not String)
//! - **From/Into**: Ergonomic conversions to/from the inner type
//! - **Deref**: Transparent access to inner type methods when appropriate

use std::fmt;

/// A strongly-typed user ID that cannot be confused with other numeric IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UserId(pub u64);

/// A strongly-typed order ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OrderId(pub u64);

/// A strongly-typed email address with validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn new(email: impl Into<String>) -> Result<Self, ValidationError> {
        let email = email.into();
        let parts: Vec<&str> = email.split('@').collect();
        let has_valid_at = parts.len() == 2 && !parts[0].is_empty();
        if has_valid_at && email.contains('.') && email.len() >= 5 {
            Ok(EmailAddress(email))
        } else {
            Err(ValidationError::InvalidEmail(email))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }

    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }
}

impl fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A percentage value clamped to 0-100.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Percentage(f64);

impl Percentage {
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if (0.0..=100.0).contains(&value) {
            Ok(Percentage(value))
        } else {
            Err(ValidationError::OutOfRange {
                value,
                min: 0.0,
                max: 100.0,
            })
        }
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn as_fraction(&self) -> f64 {
        self.0 / 100.0
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}%", self.0)
    }
}

/// A positive non-zero amount of money.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Money {
    amount: i64, // Stored in cents to avoid floating point
    currency: Currency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Currency {
    USD,
    EUR,
    GBP,
}

impl Money {
    pub fn new(amount_cents: i64, currency: Currency) -> Result<Self, ValidationError> {
        if amount_cents < 0 {
            return Err(ValidationError::NegativeAmount);
        }
        Ok(Money {
            amount: amount_cents,
            currency,
        })
    }

    pub fn from_dollars(amount: f64) -> Result<Self, ValidationError> {
        Self::new((amount * 100.0).round() as i64, Currency::USD)
    }

    pub fn amount_cents(&self) -> i64 {
        self.amount
    }

    pub fn amount_decimal(&self) -> f64 {
        self.amount as f64 / 100.0
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }
}

impl std::ops::Add for Money {
    type Output = Result<Money, ValidationError>;

    fn add(self, rhs: Money) -> Self::Output {
        if self.currency != rhs.currency {
            return Err(ValidationError::CurrencyMismatch);
        }
        Money::new(self.amount + rhs.amount, self.currency)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let symbol = match self.currency {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::GBP => "£",
        };
        write!(f, "{}{:.2}", symbol, self.amount_decimal())
    }
}

/// A URL with validation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url(String);

impl Url {
    pub fn new(url: impl Into<String>) -> Result<Self, ValidationError> {
        let url = url.into();
        if url.starts_with("http://") || url.starts_with("https://") {
            Ok(Url(url))
        } else {
            Err(ValidationError::InvalidUrl(url))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn scheme(&self) -> &str {
        if self.0.starts_with("https") {
            "https"
        } else {
            "http"
        }
    }

    pub fn host(&self) -> &str {
        let without_scheme = self.0.splitn(3, '/').nth(2).unwrap_or("");
        without_scheme.split('/').next().unwrap_or("")
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A port number validated to be in range 1-65535.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Port(u16);

impl Port {
    pub fn new(port: u16) -> Result<Self, ValidationError> {
        if port == 0 {
            Err(ValidationError::InvalidPort(port))
        } else {
            Ok(Port(port))
        }
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A duration in milliseconds, distinct from std::time::Duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Milliseconds(u64);

impl Milliseconds {
    pub fn new(ms: u64) -> Self {
        Milliseconds(ms)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn to_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.0)
    }
}

impl From<std::time::Duration> for Milliseconds {
    fn from(d: std::time::Duration) -> Self {
        Milliseconds(d.as_millis() as u64)
    }
}

impl From<Milliseconds> for std::time::Duration {
    fn from(ms: Milliseconds) -> Self {
        std::time::Duration::from_millis(ms.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    InvalidEmail(String),
    OutOfRange { value: f64, min: f64, max: f64 },
    NegativeAmount,
    CurrencyMismatch,
    InvalidUrl(String),
    InvalidPort(u16),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidEmail(email) => write!(f, "Invalid email: {email}"),
            ValidationError::OutOfRange { value, min, max } => {
                write!(f, "Value {value} out of range [{min}, {max}]")
            }
            ValidationError::NegativeAmount => write!(f, "Amount cannot be negative"),
            ValidationError::CurrencyMismatch => write!(f, "Cannot add different currencies"),
            ValidationError::InvalidUrl(url) => write!(f, "Invalid URL: {url}"),
            ValidationError::InvalidPort(port) => write!(f, "Invalid port: {port}"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Demonstrates using newtypes in a function signature to prevent argument confusion.
pub fn transfer_funds(
    from: UserId,
    to: UserId,
    amount: Money,
) -> Result<Money, ValidationError> {
    // The type system prevents mixing up from/to with OrderId
    let _ = (from, to);
    Ok(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_not_order_id() {
        let uid = UserId(42);
        let oid = OrderId(42);
        // These are different types — cannot compare
        assert_eq!(uid.0, oid.0); // But can access inner value
        // uid == oid would be a compile error
    }

    #[test]
    fn test_email_address_valid() {
        let email = EmailAddress::new("user@example.com").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
        assert_eq!(email.domain(), "example.com");
        assert_eq!(email.local_part(), "user");
    }

    #[test]
    fn test_email_address_invalid() {
        assert!(EmailAddress::new("invalid").is_err());
        assert!(EmailAddress::new("no@").is_err());
        assert!(EmailAddress::new("@.com").is_err());
    }

    #[test]
    fn test_percentage() {
        let p = Percentage::new(75.5).unwrap();
        assert!((p.value() - 75.5).abs() < 0.001);
        assert!((p.as_fraction() - 0.755).abs() < 0.001);
        assert_eq!(format!("{p}"), "75.5%");
    }

    #[test]
    fn test_percentage_out_of_range() {
        assert!(Percentage::new(-1.0).is_err());
        assert!(Percentage::new(101.0).is_err());
        assert!(Percentage::new(0.0).is_ok());
        assert!(Percentage::new(100.0).is_ok());
    }

    #[test]
    fn test_money_from_dollars() {
        let m = Money::from_dollars(19.99).unwrap();
        assert_eq!(m.amount_cents(), 1999);
        assert!((m.amount_decimal() - 19.99).abs() < 0.01);
    }

    #[test]
    fn test_money_addition() {
        let a = Money::new(1000, Currency::USD).unwrap();
        let b = Money::new(500, Currency::USD).unwrap();
        let sum = (a + b).unwrap();
        assert_eq!(sum.amount_cents(), 1500);
    }

    #[test]
    fn test_money_currency_mismatch() {
        let usd = Money::new(1000, Currency::USD).unwrap();
        let eur = Money::new(1000, Currency::EUR).unwrap();
        assert_eq!((usd + eur), Err(ValidationError::CurrencyMismatch));
    }

    #[test]
    fn test_money_display() {
        let m = Money::new(1999, Currency::USD).unwrap();
        assert_eq!(format!("{m}"), "$19.99");

        let m = Money::new(1500, Currency::EUR).unwrap();
        assert_eq!(format!("{m}"), "€15.00");
    }

    #[test]
    fn test_url() {
        let url = Url::new("https://example.com/path").unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host(), "example.com");
    }

    #[test]
    fn test_url_invalid() {
        assert!(Url::new("ftp://example.com").is_err());
        assert!(Url::new("not a url").is_err());
    }

    #[test]
    fn test_port() {
        let port = Port::new(8080).unwrap();
        assert_eq!(port.value(), 8080);
        assert_eq!(format!("{port}"), "8080");
    }

    #[test]
    fn test_port_zero() {
        assert!(Port::new(0).is_err());
    }

    #[test]
    fn test_milliseconds_conversion() {
        let ms = Milliseconds::new(1500);
        let dur = ms.to_duration();
        assert_eq!(dur, std::time::Duration::from_millis(1500));

        let ms2: Milliseconds = std::time::Duration::from_secs(2).into();
        assert_eq!(ms2.value(), 2000);
    }

    #[test]
    fn test_transfer_funds() {
        let from = UserId(1);
        let to = UserId(2);
        let amount = Money::from_dollars(100.0).unwrap();

        let result = transfer_funds(from, to, amount);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::InvalidEmail("bad".into());
        assert!(err.to_string().contains("bad"));
    }
}
