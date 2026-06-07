//! # Lesson 6: Client-Facing Errors
//!
//! When exposing errors to API clients, you need to hide internal details
//! while providing useful feedback. This lesson covers error serialization,
//! status codes, and separating internal vs external error representations.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Internal error (never exposed to clients)
// ---------------------------------------------------------------------------

/// Internal error with full details. Never serialize this for clients.
#[derive(Debug, thiserror::Error)]
pub enum InternalError {
    #[error("database query failed")]
    Database(#[source] std::io::Error),

    #[error("cache miss on key '{key}'")]
    CacheMiss { key: String },

    #[error("service '{service}' unavailable")]
    ServiceUnavailable { service: String },

    #[error("internal state inconsistency: {0}")]
    InternalState(String),
}

// ---------------------------------------------------------------------------
// Client-facing error (safe to serialize and send)
// ---------------------------------------------------------------------------

/// A safe error type that can be serialized and sent to clients.
/// Internal details are logged but never exposed.
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientError {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub status: u16,
}

impl ClientError {
    pub fn not_found(resource: &str) -> Self {
        Self {
            error: ErrorBody {
                code: "NOT_FOUND".into(),
                message: format!("The requested {} was not found", resource),
                details: None,
                request_id: None,
                status: 404,
            },
        }
    }

    pub fn bad_request(message: &str) -> Self {
        Self {
            error: ErrorBody {
                code: "BAD_REQUEST".into(),
                message: message.into(),
                details: None,
                request_id: None,
                status: 400,
            },
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            error: ErrorBody {
                code: "UNAUTHORIZED".into(),
                message: "Authentication required".into(),
                details: None,
                request_id: None,
                status: 401,
            },
        }
    }

    pub fn forbidden() -> Self {
        Self {
            error: ErrorBody {
                code: "FORBIDDEN".into(),
                message: "You do not have permission to perform this action".into(),
                details: None,
                request_id: None,
                status: 403,
            },
        }
    }

    pub fn internal_error() -> Self {
        Self {
            error: ErrorBody {
                code: "INTERNAL_ERROR".into(),
                message: "An unexpected error occurred. Please try again later.".into(),
                details: None,
                request_id: None,
                status: 500,
            },
        }
    }

    pub fn validation_error(errors: Vec<FieldError>) -> Self {
        Self {
            error: ErrorBody {
                code: "VALIDATION_ERROR".into(),
                message: "The request contains invalid fields".into(),
                details: Some(serde_json::json!({ "fields": errors })),
                request_id: None,
                status: 422,
            },
        }
    }

    pub fn with_request_id(mut self, id: impl Into<String>) -> Self {
        self.error.request_id = Some(id.into());
        self
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
    pub code: String,
}

/// Convert an internal error to a client-safe error.
/// This is the critical function that separates internal from external.
pub fn to_client_error(err: &InternalError, request_id: Option<&str>) -> ClientError {
    let mut client_err = match err {
        InternalError::Database(_) => ClientError::internal_error(),
        InternalError::CacheMiss { .. } => ClientError::internal_error(),
        InternalError::ServiceUnavailable { .. } => {
            let mut e = ClientError::internal_error();
            e.error.code = "SERVICE_UNAVAILABLE".into();
            e.error.status = 503;
            e.error.message = "The service is temporarily unavailable. Please retry.".into();
            e
        }
        InternalError::InternalState(_) => ClientError::internal_error(),
    };

    if let Some(id) = request_id {
        client_err = client_err.with_request_id(id);
    }

    client_err
}

// ---------------------------------------------------------------------------
// Error response builder with logging integration
// ---------------------------------------------------------------------------

/// Builds an error response and logs the internal error for debugging.
pub struct ErrorResponseBuilder {
    log_entries: Vec<LogEntry>,
}

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub error_type: String,
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
}

impl ErrorResponseBuilder {
    pub fn new() -> Self {
        Self {
            log_entries: Vec::new(),
        }
    }

    /// Handle an error: log it internally and return a safe client response.
    pub fn handle(
        &mut self,
        err: InternalError,
        request_id: Option<&str>,
    ) -> ClientError {
        // Log the full internal error
        self.log_entries.push(LogEntry {
            level: LogLevel::Error,
            message: format!("{:?}", err),
            error_type: std::any::type_name_of_val(&err).to_string(),
            request_id: request_id.map(|s| s.to_string()),
        });

        // Return a safe client error
        to_client_error(&err, request_id)
    }

    /// Get accumulated log entries (for testing).
    pub fn log_entries(&self) -> &[LogEntry] {
        &self.log_entries
    }
}

impl Default for ErrorResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Problem Details (RFC 7807) implementation
// ---------------------------------------------------------------------------

/// Implements RFC 7807 Problem Details for HTTP APIs.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub problem_type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
}

impl ProblemDetails {
    pub fn new(status: u16, title: &str, detail: &str) -> Self {
        Self {
            problem_type: format!("https://httpstatuses.com/{}", status),
            title: title.into(),
            status,
            detail: detail.into(),
            instance: None,
        }
    }

    pub fn with_instance(mut self, path: &str) -> Self {
        self.instance = Some(path.into());
        self
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_error_not_found() {
        let err = ClientError::not_found("user");
        assert_eq!(err.error.code, "NOT_FOUND");
        assert_eq!(err.error.status, 404);
        assert!(err.error.message.contains("user"));
        assert!(err.error.request_id.is_none());
    }

    #[test]
    fn test_client_error_bad_request() {
        let err = ClientError::bad_request("missing field 'name'");
        assert_eq!(err.error.code, "BAD_REQUEST");
        assert_eq!(err.error.status, 400);
    }

    #[test]
    fn test_client_error_unauthorized() {
        let err = ClientError::unauthorized();
        assert_eq!(err.error.code, "UNAUTHORIZED");
        assert_eq!(err.error.status, 401);
    }

    #[test]
    fn test_client_error_forbidden() {
        let err = ClientError::forbidden();
        assert_eq!(err.error.code, "FORBIDDEN");
        assert_eq!(err.error.status, 403);
    }

    #[test]
    fn test_client_error_internal() {
        let err = ClientError::internal_error();
        assert_eq!(err.error.code, "INTERNAL_ERROR");
        assert_eq!(err.error.status, 500);
        // Should not leak internal details
        assert!(!err.error.message.contains("debug"));
    }

    #[test]
    fn test_client_error_validation() {
        let fields = vec![
            FieldError {
                field: "email".into(),
                message: "invalid format".into(),
                code: "INVALID_FORMAT".into(),
            },
            FieldError {
                field: "age".into(),
                message: "must be positive".into(),
                code: "OUT_OF_RANGE".into(),
            },
        ];
        let err = ClientError::validation_error(fields);
        assert_eq!(err.error.code, "VALIDATION_ERROR");
        assert_eq!(err.error.status, 422);
        assert!(err.error.details.is_some());
    }

    #[test]
    fn test_client_error_with_request_id() {
        let err = ClientError::internal_error().with_request_id("req-abc-123");
        assert_eq!(err.error.request_id, Some("req-abc-123".into()));
    }

    #[test]
    fn test_to_client_error_hides_internals() {
        let internal = InternalError::Database(std::io::Error::new(
            std::io::ErrorKind::Other,
            "connection to db-internal:5432 failed",
        ));
        let client = to_client_error(&internal, Some("req-1"));
        // Should NOT contain internal details
        assert!(!client.error.message.contains("db-internal"));
        assert!(!client.error.message.contains("5432"));
        assert_eq!(client.error.request_id, Some("req-1".into()));
    }

    #[test]
    fn test_to_client_error_service_unavailable() {
        let internal = InternalError::ServiceUnavailable {
            service: "payment-gateway".into(),
        };
        let client = to_client_error(&internal, None);
        assert_eq!(client.error.code, "SERVICE_UNAVAILABLE");
        assert_eq!(client.error.status, 503);
    }

    #[test]
    fn test_error_response_builder() {
        let mut builder = ErrorResponseBuilder::new();
        let internal = InternalError::InternalState("inconsistent".into());
        let client = builder.handle(internal, Some("req-42"));

        assert_eq!(client.error.status, 500);
        assert_eq!(builder.log_entries().len(), 1);
        assert!(builder.log_entries()[0].message.contains("inconsistent"));
    }

    #[test]
    fn test_error_response_builder_default() {
        let builder = ErrorResponseBuilder::default();
        assert!(builder.log_entries().is_empty());
    }

    #[test]
    fn test_problem_details() {
        let pd = ProblemDetails::new(404, "Not Found", "The user was not found")
            .with_instance("/api/users/42");

        assert_eq!(pd.status, 404);
        assert_eq!(pd.title, "Not Found");
        assert_eq!(pd.detail, "The user was not found");
        assert_eq!(pd.instance, Some("/api/users/42".into()));

        let json = pd.to_json();
        assert!(json.contains("404"));
        assert!(json.contains("Not Found"));
    }

    #[test]
    fn test_problem_details_serialization() {
        let pd = ProblemDetails::new(400, "Bad Request", "invalid input");
        let json: serde_json::Value = serde_json::from_str(&pd.to_json()).unwrap();
        assert_eq!(json["status"], 400);
        assert_eq!(json["title"], "Bad Request");
    }

    #[test]
    fn test_client_error_serialization() {
        let err = ClientError::not_found("user");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("NOT_FOUND"));
        assert!(json.contains("404"));
        // request_id should be skipped when None
        assert!(!json.contains("request_id"));
    }

    #[test]
    fn test_client_error_serialization_with_request_id() {
        let err = ClientError::internal_error().with_request_id("req-1");
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("request_id"));
        assert!(json.contains("req-1"));
    }
}
