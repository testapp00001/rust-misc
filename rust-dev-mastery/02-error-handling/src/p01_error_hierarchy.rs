//! # Lesson 1: Error Hierarchy Design
//!
//! A well-designed error hierarchy makes errors easy to handle and debug.
//! This lesson covers designing error enums, grouping by domain, nesting
//! errors, and following Rust error handling conventions.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Domain-level error hierarchy
// ---------------------------------------------------------------------------

/// Top-level application error that wraps all domain errors.
/// In a real app, each domain (auth, db, api) would have its own error type.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("internal error: {0}")]
    Internal(String),
}

/// Authentication and authorization errors.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("token expired at {expired_at}")]
    TokenExpired { expired_at: String },

    #[error("insufficient permissions: need {required}, have {actual}")]
    InsufficientPermissions { required: String, actual: String },

    #[error("account locked: {reason}")]
    AccountLocked { reason: String },

    #[error("too many attempts, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
}

/// Database operation errors.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("query failed: {0}")]
    QueryFailed(String),

    #[error("not found: {entity} with {field}={value}")]
    NotFound {
        entity: String,
        field: String,
        value: String,
    },

    #[error("duplicate key: {key}")]
    DuplicateKey { key: String },

    #[error("transaction failed: {0}")]
    TransactionFailed(String),

    #[error("connection pool exhausted (max={max_size})")]
    PoolExhausted { max_size: usize },
}

/// Validation errors with field-level detail.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ValidationError {
    #[error("field '{field}' is required")]
    Required { field: String },

    #[error("field '{field}' has invalid value: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("field '{field}' must be between {min} and {max}")]
    OutOfRange {
        field: String,
        min: String,
        max: String,
    },

    #[error("too many items: got {actual}, max is {max}")]
    TooManyItems { actual: usize, max: usize },

    #[error("multiple validation errors: {0:?}")]
    Multiple(Vec<ValidationError>),
}

/// Configuration errors.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required config key: {key}")]
    MissingKey { key: String },

    #[error("invalid config value for '{key}': {reason}")]
    InvalidValue { key: String, reason: String },

    #[error("config file not found: {path}")]
    FileNotFound { path: String },

    #[error("config parse error: {0}")]
    ParseError(String),
}

// ---------------------------------------------------------------------------
// Error classification for retry logic
// ---------------------------------------------------------------------------

/// Whether an error is transient (retryable) or permanent.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Transient error that may succeed if retried.
    Transient,
    /// Permanent error that will not succeed if retried.
    Permanent,
    /// Error due to client input; never retry.
    Client,
}

impl AuthError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            AuthError::InvalidCredentials => ErrorKind::Client,
            AuthError::TokenExpired { .. } => ErrorKind::Client,
            AuthError::InsufficientPermissions { .. } => ErrorKind::Permanent,
            AuthError::AccountLocked { .. } => ErrorKind::Permanent,
            AuthError::RateLimited { .. } => ErrorKind::Transient,
        }
    }
}

impl DatabaseError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            DatabaseError::ConnectionFailed(_) => ErrorKind::Transient,
            DatabaseError::QueryFailed(_) => ErrorKind::Transient,
            DatabaseError::NotFound { .. } => ErrorKind::Permanent,
            DatabaseError::DuplicateKey { .. } => ErrorKind::Permanent,
            DatabaseError::TransactionFailed(_) => ErrorKind::Transient,
            DatabaseError::PoolExhausted { .. } => ErrorKind::Transient,
        }
    }
}

impl ValidationError {
    pub fn kind(&self) -> ErrorKind {
        ErrorKind::Client
    }
}

impl ConfigError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            ConfigError::MissingKey { .. } => ErrorKind::Permanent,
            ConfigError::InvalidValue { .. } => ErrorKind::Permanent,
            ConfigError::FileNotFound { .. } => ErrorKind::Permanent,
            ConfigError::ParseError(_) => ErrorKind::Permanent,
        }
    }
}

impl AppError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            AppError::Auth(e) => e.kind(),
            AppError::Database(e) => e.kind(),
            AppError::Validation(e) => e.kind(),
            AppError::Config(e) => e.kind(),
            AppError::Internal(_) => ErrorKind::Transient,
        }
    }

    pub fn is_retryable(&self) -> bool {
        self.kind() == ErrorKind::Transient
    }
}

/// Collect multiple validation errors into a single error.
pub fn collect_validation_errors(errors: Vec<ValidationError>) -> ValidationError {
    if errors.len() == 1 {
        errors.into_iter().next().unwrap()
    } else {
        ValidationError::Multiple(errors)
    }
}

/// Builder for validation errors that accumulates multiple issues.
#[derive(Debug, Default)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn required(mut self, field: &str) -> Self {
        self.errors.push(ValidationError::Required {
            field: field.to_string(),
        });
        self
    }

    pub fn invalid(mut self, field: &str, reason: &str) -> Self {
        self.errors.push(ValidationError::InvalidValue {
            field: field.to_string(),
            reason: reason.to_string(),
        });
        self
    }

    pub fn out_of_range(mut self, field: &str, min: &str, max: &str) -> Self {
        self.errors.push(ValidationError::OutOfRange {
            field: field.to_string(),
            min: min.to_string(),
            max: max.to_string(),
        });
        self
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn count(&self) -> usize {
        self.errors.len()
    }

    pub fn build(self) -> Option<ValidationError> {
        if self.errors.is_empty() {
            None
        } else {
            Some(collect_validation_errors(self.errors))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_error_display() {
        let err = AuthError::InvalidCredentials;
        assert_eq!(err.to_string(), "invalid credentials");

        let err = AuthError::TokenExpired {
            expired_at: "2024-01-01".to_string(),
        };
        assert!(err.to_string().contains("2024-01-01"));
    }

    #[test]
    fn test_database_error_display() {
        let err = DatabaseError::NotFound {
            entity: "User".to_string(),
            field: "id".to_string(),
            value: "42".to_string(),
        };
        assert!(err.to_string().contains("User"));
        assert!(err.to_string().contains("42"));
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::Required {
            field: "email".to_string(),
        };
        assert!(err.to_string().contains("email"));
    }

    #[test]
    fn test_app_error_from_auth() {
        let auth_err = AuthError::InvalidCredentials;
        let app_err: AppError = auth_err.into();
        assert!(matches!(app_err, AppError::Auth(_)));
    }

    #[test]
    fn test_app_error_from_database() {
        let db_err = DatabaseError::ConnectionFailed("timeout".to_string());
        let app_err: AppError = db_err.into();
        assert!(matches!(app_err, AppError::Database(_)));
    }

    #[test]
    fn test_app_error_from_validation() {
        let val_err = ValidationError::Required {
            field: "name".to_string(),
        };
        let app_err: AppError = val_err.into();
        assert!(matches!(app_err, AppError::Validation(_)));
    }

    #[test]
    fn test_error_kinds() {
        assert_eq!(AuthError::InvalidCredentials.kind(), ErrorKind::Client);
        assert_eq!(
            AuthError::RateLimited {
                retry_after_secs: 60
            }
            .kind(),
            ErrorKind::Transient
        );
        assert_eq!(
            DatabaseError::ConnectionFailed("x".to_string()).kind(),
            ErrorKind::Transient
        );
        assert_eq!(
            DatabaseError::NotFound {
                entity: "x".into(),
                field: "y".into(),
                value: "z".into()
            }
            .kind(),
            ErrorKind::Permanent
        );
    }

    #[test]
    fn test_is_retryable() {
        let transient: AppError = DatabaseError::ConnectionFailed("x".into()).into();
        assert!(transient.is_retryable());

        let permanent: AppError = AuthError::InvalidCredentials.into();
        assert!(!permanent.is_retryable());
    }

    #[test]
    fn test_validation_errors_builder() {
        let ve = ValidationErrors::new()
            .required("email")
            .invalid("age", "must be positive")
            .out_of_range("score", "0", "100");

        assert!(ve.has_errors());
        assert_eq!(ve.count(), 3);

        let err = ve.build().unwrap();
        assert!(matches!(err, ValidationError::Multiple(_)));
    }

    #[test]
    fn test_validation_errors_empty() {
        let ve = ValidationErrors::new();
        assert!(!ve.has_errors());
        assert!(ve.build().is_none());
    }

    #[test]
    fn test_collect_single_error() {
        let err = collect_validation_errors(vec![ValidationError::Required {
            field: "name".into(),
        }]);
        assert!(matches!(err, ValidationError::Required { .. }));
    }

    #[test]
    fn test_config_error_from() {
        let err: AppError = ConfigError::MissingKey {
            key: "database.url".into(),
        }
        .into();
        assert!(matches!(err, AppError::Config(_)));
    }

    #[test]
    fn test_internal_error() {
        let err = AppError::Internal("something went wrong".to_string());
        assert!(err.is_retryable());
        assert_eq!(err.kind(), ErrorKind::Transient);
    }
}
