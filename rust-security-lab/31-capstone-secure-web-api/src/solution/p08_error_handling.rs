//! # Lesson 08: Secure Error Handling — Solution
//!
//! Internal detail suppression, correlation IDs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    RateLimited,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeErrorResponse {
    pub error_code: ErrorCode,
    pub message: String,
    pub correlation_id: String,
}

#[derive(Debug, Clone)]
pub struct InternalErrorDetails {
    pub correlation_id: String,
    pub internal_message: String,
    pub source: String,
    pub timestamp: u64,
}

#[derive(Debug)]
pub struct SecureErrorHandler {
    next_id: u64,
    pub internal_log: Vec<InternalErrorDetails>,
}

impl SecureErrorHandler {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            internal_log: Vec::new(),
        }
    }
}

pub fn generate_correlation_id(handler: &mut SecureErrorHandler) -> String {
    let id = format!("err-{}", handler.next_id);
    handler.next_id += 1;
    id
}

pub fn create_safe_error(
    error_code: ErrorCode,
    correlation_id: &str,
) -> SafeErrorResponse {
    let message = match error_code {
        ErrorCode::BadRequest => "The request was malformed or invalid.",
        ErrorCode::Unauthorized => "Authentication is required.",
        ErrorCode::Forbidden => "You do not have permission to perform this action.",
        ErrorCode::NotFound => "The requested resource was not found.",
        ErrorCode::RateLimited => "Too many requests. Please try again later.",
        ErrorCode::InternalError => "An internal error occurred. Please try again later.",
    };

    SafeErrorResponse {
        error_code,
        message: message.to_string(),
        correlation_id: correlation_id.to_string(),
    }
}

pub fn handle_internal_error(
    handler: &mut SecureErrorHandler,
    internal_message: &str,
    source: &str,
    timestamp: u64,
) -> SafeErrorResponse {
    let correlation_id = generate_correlation_id(handler);

    // Log internal details on the server
    handler.internal_log.push(InternalErrorDetails {
        correlation_id: correlation_id.clone(),
        internal_message: internal_message.to_string(),
        source: source.to_string(),
        timestamp,
    });

    // Return safe response to client
    create_safe_error(ErrorCode::InternalError, &correlation_id)
}

pub fn sanitize_error_message(message: &str) -> String {
    // Process line by line: remove stack trace lines first
    let lines: Vec<&str> = message
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            !trimmed.starts_with("at ") && !trimmed.starts_with("  at ")
        })
        .collect();

    let mut result = lines.join("\n");

    // Remove file paths: sequences starting with / containing at least one more /
    let mut sanitized = String::new();
    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '/' && i + 1 < chars.len() {
            let mut j = i + 1;
            let mut found_slash = false;
            while j < chars.len() && !chars[j].is_whitespace() {
                if chars[j] == '/' {
                    found_slash = true;
                }
                j += 1;
            }
            if found_slash {
                sanitized.push_str("[REDACTED_PATH]");
                i = j;
                continue;
            }
        }
        sanitized.push(chars[i]);
        i += 1;
    }
    result = sanitized;

    // Remove IP addresses: pattern like ddd.ddd.ddd.ddd
    let mut final_result = String::new();
    let tokens: Vec<&str> = result.split(|c: char| c.is_whitespace() || c == ',').collect();
    for (idx, token) in tokens.iter().enumerate() {
        if idx > 0 {
            final_result.push(' ');
        }
        if is_ip_like(token) {
            final_result.push_str("[REDACTED_IP]");
        } else {
            final_result.push_str(token);
        }
    }
    result = final_result;

    if result.trim().is_empty() {
        "An error occurred.".to_string()
    } else {
        result
    }
}

fn is_ip_like(s: &str) -> bool {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

pub fn status_to_error_code(status: u16) -> ErrorCode {
    match status {
        400 => ErrorCode::BadRequest,
        401 => ErrorCode::Unauthorized,
        403 => ErrorCode::Forbidden,
        404 => ErrorCode::NotFound,
        429 => ErrorCode::RateLimited,
        500..=599 => ErrorCode::InternalError,
        _ => ErrorCode::BadRequest,
    }
}

pub fn error_handler_middleware(
    handler: &mut SecureErrorHandler,
    status: u16,
    internal_message: &str,
    source: &str,
    timestamp: u64,
) -> SafeErrorResponse {
    let error_code = status_to_error_code(status);
    let correlation_id = generate_correlation_id(handler);

    // Always log internal details
    handler.internal_log.push(InternalErrorDetails {
        correlation_id: correlation_id.clone(),
        internal_message: internal_message.to_string(),
        source: source.to_string(),
        timestamp,
    });

    // Return generic message for all error codes
    create_safe_error(error_code, &correlation_id)
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
        let resp = handle_internal_error(
            &mut handler,
            "NullPointerException at line 42",
            "UserService",
            1700000000,
        );

        assert_eq!(resp.error_code, ErrorCode::InternalError);
        assert!(!resp.message.contains("NullPointerException"));
        assert!(!resp.message.contains("line 42"));

        assert_eq!(handler.internal_log.len(), 1);
        assert!(handler.internal_log[0]
            .internal_message
            .contains("NullPointerException"));
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
        let result = sanitize_error_message(
            "Error occurred\n  at com.app.Service.method(Service.java:42)\n  at com.app.Main.main",
        );
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
            &mut handler,
            500,
            "Database connection pool exhausted",
            "db",
            1700000000,
        );
        assert_eq!(resp.error_code, ErrorCode::InternalError);
        assert!(!resp.message.contains("Database"));
        assert!(!resp.message.contains("pool"));
    }

    #[test]
    fn test_error_handler_middleware_4xx() {
        let mut handler = SecureErrorHandler::new();
        let resp =
            error_handler_middleware(&mut handler, 401, "Invalid JWT token", "auth", 1700000000);
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
