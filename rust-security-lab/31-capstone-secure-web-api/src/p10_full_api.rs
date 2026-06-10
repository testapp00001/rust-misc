//! # Lesson 10: Complete Secure API
//!
//! ## Putting It All Together
//!
//! This lesson combines all security middleware into a single request processing
//! pipeline. Each layer from Lessons 01-09 is composed into a defense-in-depth
//! architecture.
//!
//! ## Request Processing Pipeline
//!
//! ```text
//! Incoming Request
//!     |
//!     v
//! [1] Rate Limiting    -- reject floods (p04)
//!     |
//!     v
//! [2] Audit Log        -- log the attempt (p07)
//!     |
//!     v
//! [3] Signature Check  -- verify integrity (p06)
//!     |
//!     v
//! [4] Input Validation -- reject bad data (p05)
//!     |
//!     v
//! [5] Authentication   -- identify user (p02)
//!     |
//!     v
//! [6] Authorization    -- check permissions (p03)
//!     |
//!     v
//! [7] Handler          -- business logic
//!     |
//!     v
//! [8] Error Handling   -- safe error responses (p08)
//! ```
//!
//! ## Defense in Depth
//!
//! Every layer assumes the previous layers might have failed:
//! - Even if rate limiting has a bug, authentication blocks unauthorized users
//! - Even if authentication is bypassed, authorization restricts actions
//! - Even if authorization fails, input validation prevents injection
//! - Even if everything fails, audit logging records the incident
//!
//! This is defense in depth: no single point of failure.

use std::collections::HashMap;

/// A simulated HTTP request (for testing without a real HTTP framework).
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub client_ip: String,
}

/// A simulated HTTP response.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

/// Configuration for the secure API middleware stack.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// HMAC secret for JWT signing (in production, use a real secret manager)
    pub jwt_secret: Vec<u8>,
    /// HMAC secret for request signing
    pub signing_secret: Vec<u8>,
    /// Maximum request body size in bytes
    pub max_body_size: usize,
    /// Maximum requests per minute per IP
    pub rate_limit_per_minute: u32,
    /// Request signature max age in seconds
    pub signature_max_age: u64,
    /// Current server time (for testing)
    pub current_time: u64,
}

impl ApiConfig {
    /// Create a default API configuration.
    pub fn default_config() -> Self {
        Self {
            jwt_secret: b"jwt-secret-key-at-least-32-byte".to_vec(),
            signing_secret: b"signing-secret-32-bytes-long!!".to_vec(),
            max_body_size: 1_048_576, // 1 MB
            rate_limit_per_minute: 60,
            signature_max_age: 300, // 5 minutes
            current_time: 1700000000,
        }
    }
}

/// The result of processing a request through the middleware stack.
#[derive(Debug, Clone)]
pub enum MiddlewareResult {
    /// Request passed all checks, proceed to handler
    Continue(ProcessedRequest),
    /// Request was rejected by middleware, return the response
    Reject(HttpResponse),
}

/// A request that has been validated and enriched by middleware.
#[derive(Debug, Clone)]
pub struct ProcessedRequest {
    /// The original request
    pub request: HttpRequest,
    /// The authenticated user ID (if authentication passed)
    pub user_id: Option<String>,
    /// The user's roles (if authentication passed)
    pub roles: Vec<String>,
    /// The correlation ID for this request
    pub correlation_id: String,
}

/// Exercise 1: Implement the request size check middleware.
///
/// Reject the request if the body exceeds max_size.
/// Return MiddlewareResult::Reject with status 413 and message "Request body too large."
/// Otherwise return MiddlewareResult::Continue.
pub fn check_request_size(request: &HttpRequest, max_size: usize) -> MiddlewareResult {
    todo!("Check request body size")
}

/// Exercise 2: Implement a simple rate limiter middleware.
///
/// Track request counts per IP address using a HashMap.
/// If an IP has made more than max_requests in the current window, reject.
///
/// For simplicity, use a fixed window: if current_time / 60 is the same window,
/// count requests. If a new window starts, reset the count.
///
/// Return Reject with status 429 if rate limited, Continue otherwise.
///
/// Note: Since we can't mutate the HashMap across calls in this exercise structure,
/// use a simplified approach: just check if the count stored in the map exceeds the limit.
/// The caller is responsible for updating the map.
pub fn check_rate_limit_simple(
    request: &HttpRequest,
    rate_map: &HashMap<String, u32>,
    max_requests: u32,
) -> MiddlewareResult {
    todo!("Implement simple rate limit check")
}

/// Exercise 3: Validate request headers.
///
/// Check that:
/// 1. Content-Type is "application/json" for POST/PUT requests (if body is not empty)
/// 2. Authorization header is present for protected endpoints (paths starting with "/api/")
/// 3. No header value exceeds 8192 bytes
///
/// Return Reject with appropriate error if validation fails, Continue otherwise.
pub fn validate_headers(request: &HttpRequest) -> MiddlewareResult {
    todo!("Validate request headers")
}

/// Exercise 4: Implement authentication middleware.
///
/// Extract the Bearer token from the Authorization header.
/// Parse the JWT (split by '.', decode payload, check expiry).
/// For this exercise, use a simplified check:
/// 1. Extract token from "Bearer <token>"
/// 2. Split token by '.' -- must have exactly 3 parts
/// 3. Decode the payload (part 1) from base64url
/// 4. Parse it as JSON and extract "sub" (user_id) and "exp" (expiry)
/// 5. Check that exp > current_time
/// 6. If valid, return Continue with user_id populated
/// 7. If invalid, return Reject with status 401
pub fn authenticate_request(
    request: &HttpRequest,
    _jwt_secret: &[u8],
    current_time: u64,
) -> MiddlewareResult {
    todo!("Authenticate the request using JWT")
}

/// Exercise 5: Implement authorization middleware.
///
/// Given the processed request (with user_id and roles), check if the user
/// is authorized for the requested endpoint.
///
/// Use these rules:
/// - GET /api/users/* requires "users:read" role
/// - POST /api/users/* requires "users:write" role
/// - GET /api/admin/* requires "admin" role
/// - All other /api/* endpoints require any authenticated user
/// - Non /api/* endpoints are public (no authorization needed)
///
/// If authorized, return Continue. If not, return Reject with status 403.
pub fn authorize_request(processed: &ProcessedRequest) -> MiddlewareResult {
    todo!("Authorize the request based on roles and path")
}

/// Exercise 6: Build the complete middleware pipeline.
///
/// Process a request through all middleware layers in order:
/// 1. Request size check
/// 2. Header validation
/// 3. Rate limiting (use rate_map parameter)
/// 4. Authentication
/// 5. Authorization
///
/// If any layer rejects, return that rejection immediately.
/// If all layers pass, return Continue with the fully populated ProcessedRequest.
///
/// Generate a correlation ID in the format "req-{counter}" using the req_counter parameter.
pub fn process_request(
    request: &HttpRequest,
    config: &ApiConfig,
    rate_map: &HashMap<String, u32>,
    req_counter: u64,
) -> MiddlewareResult {
    todo!("Build the complete middleware pipeline")
}

/// Exercise 7: Create a safe error response.
///
/// Given a status code and message, create an HttpResponse:
/// - Set the status code
/// - Set Content-Type: application/json header
/// - Set body to JSON: {"error": "<message>", "correlation_id": "<correlation_id>"}
pub fn error_response(status: u16, message: &str, correlation_id: &str) -> HttpResponse {
    todo!("Create a safe error response")
}

/// Exercise 8: Create a success response.
///
/// Given a status code and body, create an HttpResponse:
/// - Set the status code
/// - Set Content-Type: application/json header
/// - Set the body
pub fn success_response(status: u16, body: &str) -> HttpResponse {
    todo!("Create a success response")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> HttpRequest {
        HttpRequest {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Authorization".to_string(), "Bearer header.eyJzdWIiOiJ1c2VyMTIzIiwiZXhwIjoyMDAwMDAwMDAwLCJyb2xlcyI6WyJ1c2VyIl19.sig".to_string());
                h
            },
            body: Vec::new(),
            client_ip: "10.0.0.1".to_string(),
        }
    }

    fn sample_config() -> ApiConfig {
        ApiConfig::default_config()
    }

    #[test]
    fn test_check_request_size_ok() {
        let request = HttpRequest {
            method: "POST".to_string(),
            path: "/api/data".to_string(),
            headers: HashMap::new(),
            body: b"small body".to_vec(),
            client_ip: "10.0.0.1".to_string(),
        };
        let result = check_request_size(&request, 1024);
        assert!(matches!(result, MiddlewareResult::Continue(_)));
    }

    #[test]
    fn test_check_request_size_too_large() {
        let request = HttpRequest {
            method: "POST".to_string(),
            path: "/api/data".to_string(),
            headers: HashMap::new(),
            body: vec![0u8; 2048],
            client_ip: "10.0.0.1".to_string(),
        };
        let result = check_request_size(&request, 1024);
        assert!(matches!(result, MiddlewareResult::Reject(_)));
        if let MiddlewareResult::Reject(resp) = result {
            assert_eq!(resp.status, 413);
        }
    }

    #[test]
    fn test_validate_headers_post_without_content_type() {
        let request = HttpRequest {
            method: "POST".to_string(),
            path: "/api/data".to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Authorization".to_string(), "Bearer token".to_string());
                h
            },
            body: b"{}".to_vec(),
            client_ip: "10.0.0.1".to_string(),
        };
        let result = validate_headers(&request);
        assert!(matches!(result, MiddlewareResult::Reject(_)));
    }

    #[test]
    fn test_validate_headers_api_without_auth() {
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/api/users".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            client_ip: "10.0.0.1".to_string(),
        };
        let result = validate_headers(&request);
        assert!(matches!(result, MiddlewareResult::Reject(_)));
    }

    #[test]
    fn test_rate_limit_simple_ok() {
        let mut rate_map = HashMap::new();
        rate_map.insert("10.0.0.1".to_string(), 5);
        let request = sample_request();
        let result = check_rate_limit_simple(&request, &rate_map, 60);
        assert!(matches!(result, MiddlewareResult::Continue(_)));
    }

    #[test]
    fn test_rate_limit_simple_exceeded() {
        let mut rate_map = HashMap::new();
        rate_map.insert("10.0.0.1".to_string(), 61);
        let request = sample_request();
        let result = check_rate_limit_simple(&request, &rate_map, 60);
        assert!(matches!(result, MiddlewareResult::Reject(_)));
        if let MiddlewareResult::Reject(resp) = result {
            assert_eq!(resp.status, 429);
        }
    }

    #[test]
    fn test_error_response_format() {
        let resp = error_response(400, "Bad request", "req-1");
        assert_eq!(resp.status, 400);
        assert!(resp.body.contains("Bad request"));
        assert!(resp.body.contains("req-1"));
        assert_eq!(resp.headers.get("Content-Type").unwrap(), "application/json");
    }

    #[test]
    fn test_success_response_format() {
        let resp = success_response(200, r#"{"users": []}"#);
        assert_eq!(resp.status, 200);
        assert!(resp.body.contains("users"));
    }

    #[test]
    fn test_authorize_users_read() {
        let processed = ProcessedRequest {
            request: HttpRequest {
                method: "GET".to_string(),
                path: "/api/users".to_string(),
                headers: HashMap::new(),
                body: Vec::new(),
                client_ip: "10.0.0.1".to_string(),
            },
            user_id: Some("alice".to_string()),
            roles: vec!["users:read".to_string()],
            correlation_id: "req-1".to_string(),
        };
        let result = authorize_request(&processed);
        assert!(matches!(result, MiddlewareResult::Continue(_)));
    }

    #[test]
    fn test_authorize_users_read_denied() {
        let processed = ProcessedRequest {
            request: HttpRequest {
                method: "GET".to_string(),
                path: "/api/users".to_string(),
                headers: HashMap::new(),
                body: Vec::new(),
                client_ip: "10.0.0.1".to_string(),
            },
            user_id: Some("bob".to_string()),
            roles: vec!["viewer".to_string()],
            correlation_id: "req-2".to_string(),
        };
        let result = authorize_request(&processed);
        assert!(matches!(result, MiddlewareResult::Reject(_)));
    }

    #[test]
    fn test_authorize_admin_endpoint() {
        let processed = ProcessedRequest {
            request: HttpRequest {
                method: "GET".to_string(),
                path: "/api/admin/settings".to_string(),
                headers: HashMap::new(),
                body: Vec::new(),
                client_ip: "10.0.0.1".to_string(),
            },
            user_id: Some("admin".to_string()),
            roles: vec!["admin".to_string()],
            correlation_id: "req-3".to_string(),
        };
        let result = authorize_request(&processed);
        assert!(matches!(result, MiddlewareResult::Continue(_)));
    }

    #[test]
    fn test_process_request_rejects_oversized() {
        let mut request = sample_request();
        request.body = vec![0u8; 2_000_000];
        let config = sample_config();
        let rate_map = HashMap::new();
        let result = process_request(&request, &config, &rate_map, 1);
        assert!(matches!(result, MiddlewareResult::Reject(_)));
    }

    #[test]
    fn test_public_endpoint_no_auth_required() {
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/health".to_string(),
            headers: HashMap::new(),
            body: Vec::new(),
            client_ip: "10.0.0.1".to_string(),
        };
        // Public endpoint should pass header validation without auth
        let result = validate_headers(&request);
        assert!(matches!(result, MiddlewareResult::Continue(_)));
    }
}
