//! # Lesson 4: Custom Error Types (Manual Implementation)
//!
//! Sometimes you need to implement the Error trait manually, without thiserror.
//! This lesson covers the Error trait, Display, Debug, source(), and how to
//! build errors that are interoperable with the std ecosystem.

use std::fmt;

// ---------------------------------------------------------------------------
// Manual Error trait implementation
// ---------------------------------------------------------------------------

/// A custom error type with manual implementation.
/// This is how errors worked before thiserror existed.
#[derive(Debug)]
pub enum ServiceError {
    NotFound { resource: String, id: String },
    Unauthorized { reason: String },
    Conflict { message: String },
    Internal { source: Box<dyn std::error::Error + Send + Sync> },
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::NotFound { resource, id } => {
                write!(f, "{} with id '{}' not found", resource, id)
            }
            ServiceError::Unauthorized { reason } => {
                write!(f, "unauthorized: {}", reason)
            }
            ServiceError::Conflict { message } => {
                write!(f, "conflict: {}", message)
            }
            ServiceError::Internal { source } => {
                write!(f, "internal error: {}", source)
            }
        }
    }
}

impl std::error::Error for ServiceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ServiceError::Internal { source } => Some(source.as_ref()),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Struct-based error (not enum)
// ---------------------------------------------------------------------------

/// A struct-based error that carries rich context.
/// Useful when all errors share common metadata.
#[derive(Debug)]
pub struct ApiError {
    pub code: u16,
    pub message: String,
    pub details: Option<String>,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub request_id: Option<String>,
}

impl ApiError {
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            source: None,
            request_id: None,
        }
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_source(mut self, source: impl std::error::Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.request_id = Some(id.into());
        self
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, message)
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref details) = self.details {
            write!(f, ": {}", details)?;
        }
        if let Some(ref req_id) = self.request_id {
            write!(f, " (request_id: {})", req_id)?;
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|s| s.as_ref() as &(dyn std::error::Error + 'static))
    }
}

// ---------------------------------------------------------------------------
// Error with associated data
// ---------------------------------------------------------------------------

/// An error that carries structured data alongside the error message.
#[derive(Debug)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub value: Option<String>,
    pub rule: ValidationRule,
}

#[derive(Debug, Clone)]
pub enum ValidationRule {
    Required,
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Range { min: f64, max: f64 },
    Custom(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "validation error on '{}': {}", self.field, self.message)?;
        if let Some(ref value) = self.value {
            write!(f, " (value: '{}')", value)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    pub fn required(field: &str) -> Self {
        Self {
            field: field.to_string(),
            message: "this field is required".to_string(),
            value: None,
            rule: ValidationRule::Required,
        }
    }

    pub fn min_length(field: &str, value: &str, min: usize) -> Self {
        Self {
            field: field.to_string(),
            message: format!("minimum length is {} characters", min),
            value: Some(value.to_string()),
            rule: ValidationRule::MinLength(min),
        }
    }

    pub fn pattern(field: &str, value: &str, pattern: &str) -> Self {
        Self {
            field: field.to_string(),
            message: format!("must match pattern '{}'", pattern),
            value: Some(value.to_string()),
            rule: ValidationRule::Pattern(pattern.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Error wrapper that adds context
// ---------------------------------------------------------------------------

/// A wrapper that adds context to any error.
/// Similar to anyhow::Context but as a concrete type.
#[derive(Debug)]
pub struct ContextError<E> {
    pub context: String,
    pub source: E,
}

impl<E: fmt::Display> fmt::Display for ContextError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl<E: std::error::Error + 'static> std::error::Error for ContextError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Extension trait to add context to any Result.
pub trait ResultExt<T, E> {
    fn with_context_msg(self, msg: impl Into<String>) -> Result<T, ContextError<E>>;
}

impl<T, E: std::error::Error> ResultExt<T, E> for Result<T, E> {
    fn with_context_msg(self, msg: impl Into<String>) -> Result<T, ContextError<E>> {
        self.map_err(|e| ContextError {
            context: msg.into(),
            source: e,
        })
    }
}

// ---------------------------------------------------------------------------
// Composite error: multiple errors collected
// ---------------------------------------------------------------------------

/// An error that represents multiple failures.
#[derive(Debug)]
pub struct CompositeError {
    pub errors: Vec<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for CompositeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} errors occurred:", self.errors.len())?;
        for (i, err) in self.errors.iter().enumerate() {
            write!(f, "\n  {}. {}", i + 1, err)?;
        }
        Ok(())
    }
}

impl std::error::Error for CompositeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.errors.first().map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl CompositeError {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    pub fn push(&mut self, err: impl std::error::Error + Send + Sync + 'static) {
        self.errors.push(Box::new(err));
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }
}

impl Default for CompositeError {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_service_error_display() {
        let err = ServiceError::NotFound {
            resource: "User".into(),
            id: "42".into(),
        };
        assert_eq!(err.to_string(), "User with id '42' not found");
    }

    #[test]
    fn test_service_error_source() {
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "db down");
        let err = ServiceError::Internal {
            source: Box::new(inner),
        };
        assert!(err.source().is_some());
        assert!(err.source().unwrap().to_string().contains("db down"));
    }

    #[test]
    fn test_service_error_no_source() {
        let err = ServiceError::Unauthorized {
            reason: "no token".into(),
        };
        assert!(err.source().is_none());
    }

    #[test]
    fn test_api_error_builder() {
        let err = ApiError::bad_request("invalid input")
            .with_details("field 'name' is missing")
            .with_request_id("req-123");

        assert_eq!(err.code, 400);
        assert!(err.to_string().contains("invalid input"));
        assert!(err.to_string().contains("field 'name' is missing"));
        assert!(err.to_string().contains("req-123"));
    }

    #[test]
    fn test_api_error_with_source() {
        let inner = std::io::Error::new(std::io::ErrorKind::Other, "timeout");
        let err = ApiError::internal("service unavailable").with_source(inner);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_api_error_not_found() {
        let err = ApiError::not_found("resource not found");
        assert_eq!(err.code, 404);
    }

    #[test]
    fn test_validation_error_required() {
        let err = ValidationError::required("email");
        assert!(err.to_string().contains("'email'"));
        assert!(err.to_string().contains("required"));
    }

    #[test]
    fn test_validation_error_min_length() {
        let err = ValidationError::min_length("username", "ab", 3);
        assert!(err.to_string().contains("'username'"));
        assert!(err.to_string().contains("minimum length"));
        assert!(err.to_string().contains("'ab'"));
    }

    #[test]
    fn test_validation_error_pattern() {
        let err = ValidationError::pattern("email", "invalid", r"^.+@.+$");
        assert!(err.to_string().contains("pattern"));
    }

    #[test]
    fn test_context_error() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let wrapped = ContextError {
            context: "loading config".into(),
            source: inner,
        };
        assert!(wrapped.to_string().contains("loading config"));
        assert!(wrapped.to_string().contains("missing"));
        assert!(wrapped.source().is_some());
    }

    #[test]
    fn test_result_ext() {
        let result: Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "fail"));
        let wrapped = result.with_context_msg("step 1");
        assert!(wrapped.is_err());
        let err = wrapped.unwrap_err();
        assert!(err.to_string().contains("step 1"));
        assert!(err.to_string().contains("fail"));
    }

    #[test]
    fn test_composite_error() {
        let mut comp = CompositeError::new();
        assert!(comp.is_empty());

        comp.push(std::io::Error::new(std::io::ErrorKind::Other, "error 1"));
        comp.push(std::io::Error::new(std::io::ErrorKind::Other, "error 2"));

        assert_eq!(comp.len(), 2);
        let msg = comp.to_string();
        assert!(msg.contains("2 errors"));
        assert!(msg.contains("error 1"));
        assert!(msg.contains("error 2"));
    }

    #[test]
    fn test_composite_error_default() {
        let comp = CompositeError::default();
        assert!(comp.is_empty());
    }

    #[test]
    fn test_api_error_display_no_extras() {
        let err = ApiError::new(500, "internal error");
        assert_eq!(err.to_string(), "[500] internal error");
    }
}
