//! # Lesson 08: Secure Error Handling
//!
//! ## The Problem
//!
//! When an API returns an error, it must balance two needs:
//! 1. **Help the developer**: Detailed error information for debugging
//! 2. **Protect the system**: Never leak internal details to attackers
//!
//! A stack trace, database error message, or file path in a client response gives
//! attackers a roadmap to your vulnerabilities.
//!
//! ## What Attackers Learn from Verbose Errors
//!
//! | Error Detail | What Attacker Learns |
//! |--------------|---------------------|
//! | "Connection to PostgreSQL failed" | Database type (target SQL injection) |
//! | "File not found: /var/www/app/config.yml" | Server paths (target path traversal) |
//! | "NullPointerException at com.app.UserService" | Technology stack (target known CVEs) |
//! | "Redis connection refused on :6379" | Internal services and ports |
//! | "JWT decode failed: invalid algorithm 'none'" | Authentication implementation |
//!
//! ## Secure Error Handling Pattern
//!
//! ```text
//! 1. Catch the error
//! 2. Generate a unique correlation ID
//! 3. Log the FULL error internally (with stack trace, context)
//! 4. Return a GENERIC error to the client with the correlation ID
//!
//! Client sees: "Internal server error. Reference: req-abc123"
//! Server log:  "req-abc123: NullPointerException at UserService.java:42 ..."
//! ```
//!
//! ## Attack Context
//!
//! - **Information disclosure**: Verbose errors reveal implementation details
//! - **Error-based SQL injection**: Database errors that echo back user input
//! - **Panic information**: Rust panics can include file paths and line numbers
//! - **Timing attacks**: Different error paths take different times, revealing information

use serde::{Deserialize, Serialize};

/// Public-safe error codes that do NOT reveal internal details.
/// These are what the client sees.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Generic bad request (malformed input)
    BadRequest,
    /// Authentication required
    Unauthorized,
    /// Permission denied
    Forbidden,
    /// Resource not found
    NotFound,
    /// Too many requests
    RateLimited,
    /// Internal server error (never reveals details)
    InternalError,
}

/// A safe error response sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeErrorResponse {
    /// Machine-readable error code
    pub error_code: ErrorCode,
    /// Human-readable message (generic, no internal details)
    pub message: String,
    /// Correlation ID for support/debugging reference
    pub correlation_id: String,
}

/// Internal error details logged on the server (never sent to clients).
#[derive(Debug, Clone)]
pub struct InternalErrorDetails {
    /// The correlation ID matching the client response
    pub correlation_id: String,
    /// The actual error message (may contain sensitive details)
    pub internal_message: String,
    /// Error category/source
    pub source: String,
    /// Timestamp when the error occurred
    pub timestamp: u64,
}

/// The secure error handler.
#[derive(Debug)]
pub struct SecureErrorHandler {
    /// Counter for generating correlation IDs
    next_id: u64,
    /// Internal error log (never sent to clients)
    pub internal_log: Vec<InternalErrorDetails>,
}

impl SecureErrorHandler {
    /// Create a new error handler.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            internal_log: Vec::new(),
        }
    }
}

/// Exercise 1: Generate a correlation ID.
///
/// Format: "err-{number}" where number starts at 1 and increments.
/// Each call should return a unique ID.
pub fn generate_correlation_id(handler: &mut SecureErrorHandler) -> String {
    todo!("Generate a unique correlation ID")
}

/// Exercise 2: Create a safe error response.
///
/// Given an error code, create a SafeErrorResponse with a generic message.
/// Use these default messages:
/// - BadRequest: "The request was malformed or invalid."
/// - Unauthorized: "Authentication is required."
/// - Forbidden: "You do not have permission to perform this action."
/// - NotFound: "The requested resource was not found."
/// - RateLimited: "Too many requests. Please try again later."
/// - InternalError: "An internal error occurred. Please try again later."
///
/// Always include the correlation_id.
pub fn create_safe_error(
    error_code: ErrorCode,
    correlation_id: &str,
) -> SafeErrorResponse {
    todo!("Create a safe error response with generic message")
}

/// Exercise 3: Handle an internal error securely.
///
/// Steps:
/// 1. Generate a correlation ID
/// 2. Log the internal error details (internal_message, source, timestamp)
/// 3. Create a safe error response (InternalError variant)
/// 4. Return the safe response (what the client sees)
///
/// The internal details are logged on the server only.
pub fn handle_internal_error(
    handler: &mut SecureErrorHandler,
    internal_message: &str,
    source: &str,
    timestamp: u64,
) -> SafeErrorResponse {
    todo!("Handle internal error securely")
}

/// Exercise 4: Sanitize an error message for client response.
///
/// Given a raw error message, remove information that should not be exposed:
/// - Remove file paths (anything matching "/path/to/something" pattern)
/// - Remove IP addresses (anything matching "x.x.x.x" where x is 1-3 digits)
/// - Remove port numbers (anything matching ":1234" after an IP or hostname)
/// - Remove stack trace lines (lines starting with "at " or "  at ")
///
/// If the message is completely sanitized away, return "An error occurred."
pub fn sanitize_error_message(message: &str) -> String {
    todo!("Sanitize error message for client response")
}

/// Exercise 5: Map an HTTP status code to a safe ErrorCode.
///
/// - 400 -> BadRequest
/// - 401 -> Unauthorized
/// - 403 -> Forbidden
/// - 404 -> NotFound
/// - 429 -> RateLimited
/// - 500..=599 -> InternalError
/// - Any other -> BadRequest
pub fn status_to_error_code(status: u16) -> ErrorCode {
    todo!("Map HTTP status code to safe error code")
}

/// Exercise 6: Create a complete error handling middleware function.
///
/// Given an HTTP status code, an internal error message, a source, and a timestamp:
/// 1. Map the status code to an error code
/// 2. Generate a correlation ID
/// 3. Log the internal details
/// 4. If the status is 5xx, use a generic message (never expose internal details)
/// 5. For 4xx errors, use the safe default message for the error code
/// 6. Return the safe error response
pub fn error_handler_middleware(
    handler: &mut SecureErrorHandler,
    status: u16,
    internal_message: &str,
    source: &str,
    timestamp: u64,
) -> SafeErrorResponse {
    todo!("Implement complete error handling middleware")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_increments() {
        let mut handler = SecureErrorHandler::new();
        let id1 = generate_correlation_id(&mut handler);
        let id2 = generate_correlation_id(&mut handler);
        assert_eq!(id1, "err-1");
        assert_eq!(id2, "err-2");
    }

    #[test]
    fn test_create_safe_error_messages() {
        let resp = create_safe_error(ErrorCode::BadRequest, "err-1");
        assert_eq!(resp.error_code, ErrorCode::BadRequest);
        assert!(resp.message.contains("malformed") || resp.message.contains("invalid"));
        assert_eq!(resp.correlation_id, "err-1");

        let resp = create_safe_error(ErrorCode::Unauthorized, "err-2");
        assert!(resp.message.contains("Authentication") || resp.message.contains("authentication"));
    }

    #[test]
    fn test_handle_internal_error_logs_details() {
        let mut handler = SecureErrorHandler::new();
        let resp = handle_internal_error(&mut handler, "NullPointerException at line 42", "UserService", 1700000000);

        // Client gets safe response
        assert_eq!(resp.error_code, ErrorCode::InternalError);
        assert!(!resp.message.contains("NullPointerException"));
        assert!(!resp.message.contains("line 42"));

        // Server has the internal details
        assert_eq!(handler.internal_log.len(), 1);
        assert!(handler.internal_log[0].internal_message.contains("NullPointerException"));
    }

    #[test]
    fn test_sanitize_file_paths() {
        let result = sanitize_error_message("File not found: /var/www/app/config.yml");
        assert!(!result.contains("/var/www"));
    }

    #[test]
    fn test_sanitize_ip_addresses() {
        let result = sanitize_error_message("Connection failed to 192.168.1.100");
        assert!(!result.contains("192.168.1.100"));
    }

    #[test]
    fn test_sanitize_stack_traces() {
        let result = sanitize_error_message("Error occurred\n  at com.app.Service.method(Service.java:42)\n  at com.app.Main.main");
        assert!(!result.contains("at com.app"));
    }

    #[test]
    fn test_status_to_error_code() {
        assert_eq!(status_to_error_code(400), ErrorCode::BadRequest);
        assert_eq!(status_to_error_code(401), ErrorCode::Unauthorized);
        assert_eq!(status_to_error_code(403), ErrorCode::Forbidden);
        assert_eq!(status_to_error_code(404), ErrorCode::NotFound);
        assert_eq!(status_to_error_code(429), ErrorCode::RateLimited);
        assert_eq!(status_to_error_code(500), ErrorCode::InternalError);
        assert_eq!(status_to_error_code(503), ErrorCode::InternalError);
        assert_eq!(status_to_error_code(200), ErrorCode::BadRequest);
    }

    #[test]
    fn test_error_handler_middleware_5xx() {
        let mut handler = SecureErrorHandler::new();
        let resp = error_handler_middleware(
            &mut handler, 500, "Database connection pool exhausted", "db", 1700000000,
        );
        assert_eq!(resp.error_code, ErrorCode::InternalError);
        // Should NOT contain database details
        assert!(!resp.message.contains("Database"));
        assert!(!resp.message.contains("pool"));
    }

    #[test]
    fn test_error_handler_middleware_4xx() {
        let mut handler = SecureErrorHandler::new();
        let resp = error_handler_middleware(
            &mut handler, 401, "Invalid JWT token", "auth", 1700000000,
        );
        assert_eq!(resp.error_code, ErrorCode::Unauthorized);
        assert_eq!(handler.internal_log.len(), 1);
    }

    #[test]
    fn test_safe_error_serializes() {
        let resp = SafeErrorResponse {
            error_code: ErrorCode::NotFound,
            message: "The requested resource was not found.".to_string(),
            correlation_id: "err-42".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("NotFound"));
        assert!(json.contains("err-42"));
    }
}
