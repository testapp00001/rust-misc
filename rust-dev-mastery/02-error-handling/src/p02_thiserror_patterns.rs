//! # Lesson 2: thiserror Patterns
//!
//! thiserror is the standard derive macro for creating error types.
//! This lesson covers derive macros, #[from], #[source], #[context],
//! display formatting, and advanced thiserror patterns.

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Basic thiserror patterns
// ---------------------------------------------------------------------------

/// Simple error with string messages.
#[derive(Debug, thiserror::Error)]
pub enum SimpleError {
    #[error("not found")]
    NotFound,

    #[error("permission denied for resource '{0}'")]
    PermissionDenied(String),

    #[error("operation timed out after {0}ms")]
    Timeout(u64),
}

/// Error with #[from] for automatic conversion from inner errors.
#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    #[error("path error: {0}")]
    Path(String),
}

/// Error with #[source] for preserving the error chain without Display.
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("connection failed to {host}:{port}")]
    ConnectionFailed {
        host: String,
        port: u16,
        #[source]
        source: std::io::Error,
    },

    #[error("TLS handshake failed")]
    TlsError {
        #[source]
        source: TlsError,
    },

    #[error("request timeout")]
    Timeout,
}

#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("certificate invalid: {0}")]
    InvalidCertificate(String),

    #[error("certificate expired")]
    CertificateExpired,
}

// ---------------------------------------------------------------------------
// Multi-field errors with rich context
// ---------------------------------------------------------------------------

/// A parse error with detailed context about what was expected vs received.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("expected token '{expected}' at line {line}, column {col}, found '{found}'")]
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        col: usize,
    },

    #[error("unexpected end of input at line {line}, column {col}")]
    UnexpectedEof { line: usize, col: usize },

    #[error("invalid escape sequence '\\{0}' at line {1}")]
    InvalidEscape(char, usize),

    #[error("number overflow: value {value} exceeds maximum {max}")]
    Overflow { value: String, max: String },

    #[error("duplicate key '{key}' at line {line}")]
    DuplicateKey { key: String, line: usize },
}

impl ParseError {
    /// Create a position-aware error.
    pub fn unexpected_token(
        expected: impl Into<String>,
        found: impl Into<String>,
        line: usize,
        col: usize,
    ) -> Self {
        Self::UnexpectedToken {
            expected: expected.into(),
            found: found.into(),
            line,
            col,
        }
    }

    /// Check if this error is at a specific line.
    pub fn at_line(&self, line: usize) -> bool {
        match self {
            ParseError::UnexpectedToken {
                line: l, ..
            }
            | ParseError::UnexpectedEof { line: l, .. }
            | ParseError::InvalidEscape(_, l)
            | ParseError::DuplicateKey { line: l, .. } => *l == line,
            ParseError::Overflow { .. } => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Error with generic inner type
// ---------------------------------------------------------------------------

/// A wrapper error that can contain any error implementing std::error::Error.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config from '{path}'")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse config")]
    ParseError(#[from] serde_json::Error),

    #[error("missing required field '{field}'")]
    MissingField { field: String },

    #[error("invalid value for '{field}': {reason}")]
    InvalidField { field: String, reason: String },
}

// ---------------------------------------------------------------------------
// Error with backtrace support (nightly feature, simulated here)
// ---------------------------------------------------------------------------

/// An error type that demonstrates how backtrace would be captured.
/// On stable Rust, we simulate with a location string.
#[derive(Debug, thiserror::Error)]
pub enum TracedError {
    #[error("error at {location}: {message}")]
    WithLocation {
        message: String,
        location: String,
    },

    #[error("error in {context}: {source}")]
    WithContext {
        context: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl TracedError {
    /// Create an error with caller location (simulated).
    pub fn at(message: impl Into<String>, location: impl Into<String>) -> Self {
        Self::WithLocation {
            message: message.into(),
            location: location.into(),
        }
    }

    /// Wrap an error with additional context.
    pub fn wrap(
        context: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::WithContext {
            context: context.into(),
            source: Box::new(source),
        }
    }
}

// ---------------------------------------------------------------------------
// Nested error types (error within error)
// ---------------------------------------------------------------------------

/// An API error that nests domain errors.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorized")]
    Unauthorized,

    #[error("not found")]
    NotFound,

    #[error("internal server error")]
    Internal {
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("service unavailable: {reason}")]
    ServiceUnavailable { reason: String, retry_after: u64 },
}

impl ApiError {
    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            ApiError::BadRequest(_) => 400,
            ApiError::Unauthorized => 401,
            ApiError::NotFound => 404,
            ApiError::Internal { .. } => 500,
            ApiError::ServiceUnavailable { .. } => 503,
        }
    }

    /// Check if the client should retry.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ApiError::ServiceUnavailable { .. } | ApiError::Internal { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_simple_error_display() {
        assert_eq!(SimpleError::NotFound.to_string(), "not found");
        assert_eq!(
            SimpleError::PermissionDenied("admin".into()).to_string(),
            "permission denied for resource 'admin'"
        );
        assert_eq!(
            SimpleError::Timeout(5000).to_string(),
            "operation timed out after 5000ms"
        );
    }

    #[test]
    fn test_io_error_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: IoError = io_err.into();
        assert!(matches!(err, IoError::Io(_)));
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn test_network_error_source_chain() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let err = NetworkError::ConnectionFailed {
            host: "example.com".into(),
            port: 443,
            source: io_err,
        };

        assert!(err.to_string().contains("example.com"));
        assert!(err.source().is_some());
    }

    #[test]
    fn test_tls_error_chain() {
        let tls_err = TlsError::InvalidCertificate("expired CA".into());
        let err = NetworkError::TlsError { source: tls_err };
        assert!(err.to_string().contains("TLS handshake"));

        // Walk the error chain
        let source = err.source().unwrap();
        assert!(source.to_string().contains("certificate invalid"));
    }

    #[test]
    fn test_parse_error_creation() {
        let err = ParseError::unexpected_token(";", "}", 10, 5);
        assert!(err.to_string().contains("';'"));
        assert!(err.to_string().contains("'}'"));
        assert!(err.to_string().contains("line 10"));
    }

    #[test]
    fn test_parse_error_at_line() {
        let err = ParseError::unexpected_token(";", "}", 10, 5);
        assert!(err.at_line(10));
        assert!(!err.at_line(11));
    }

    #[test]
    fn test_parse_error_overflow() {
        let err = ParseError::Overflow {
            value: "9999999999999".into(),
            max: "2147483647".into(),
        };
        assert!(!err.at_line(0));
        assert!(err.to_string().contains("overflow"));
    }

    #[test]
    fn test_config_error_missing_field() {
        let err = ConfigError::MissingField {
            field: "database.url".into(),
        };
        assert!(err.to_string().contains("database.url"));
    }

    #[test]
    fn test_config_error_parse_from() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err: ConfigError = json_err.into();
        assert!(matches!(err, ConfigError::ParseError(_)));
    }

    #[test]
    fn test_traced_error_at() {
        let err = TracedError::at("something failed", "p02_thiserror_patterns.rs:42");
        assert!(err.to_string().contains("something failed"));
        assert!(err.to_string().contains("p02_thiserror_patterns.rs:42"));
    }

    #[test]
    fn test_traced_error_wrap() {
        let inner = SimpleError::NotFound;
        let err = TracedError::wrap("loading config", inner);
        assert!(err.to_string().contains("loading config"));
        assert!(err.source().is_some());
    }

    #[test]
    fn test_api_error_status_codes() {
        assert_eq!(ApiError::BadRequest("x".into()).status_code(), 400);
        assert_eq!(ApiError::Unauthorized.status_code(), 401);
        assert_eq!(ApiError::NotFound.status_code(), 404);

        let inner = SimpleError::Timeout(1000);
        let internal = ApiError::Internal {
            source: Box::new(inner),
        };
        assert_eq!(internal.status_code(), 500);

        assert_eq!(
            ApiError::ServiceUnavailable {
                reason: "maintenance".into(),
                retry_after: 300,
            }
            .status_code(),
            503
        );
    }

    #[test]
    fn test_api_error_retryable() {
        assert!(!ApiError::BadRequest("x".into()).is_retryable());
        assert!(!ApiError::Unauthorized.is_retryable());
        assert!(!ApiError::NotFound.is_retryable());
        assert!(ApiError::ServiceUnavailable {
            reason: "x".into(),
            retry_after: 60,
        }
        .is_retryable());
    }

    #[test]
    fn test_error_source_chain_depth() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "root cause");
        let net_err = NetworkError::ConnectionFailed {
            host: "test".into(),
            port: 80,
            source: io_err,
        };
        let api_err = ApiError::Internal {
            source: Box::new(net_err),
        };

        // Walk: ApiError -> NetworkError -> io::Error
        let level1 = api_err.source().unwrap();
        let level2 = level1.source().unwrap();
        assert!(level2.to_string().contains("root cause"));
        assert!(level2.source().is_none());
    }
}
