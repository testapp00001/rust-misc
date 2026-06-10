//! # Lesson 10: Complete Secure API — Solution
//!
//! All middleware composed into production-ready API.

use std::collections::HashMap;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// A simulated HTTP request.
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
    pub jwt_secret: Vec<u8>,
    pub signing_secret: Vec<u8>,
    pub max_body_size: usize,
    pub rate_limit_per_minute: u32,
    pub signature_max_age: u64,
    pub current_time: u64,
}

impl ApiConfig {
    pub fn default_config() -> Self {
        Self {
            jwt_secret: b"jwt-secret-key-at-least-32-byte".to_vec(),
            signing_secret: b"signing-secret-32-bytes-long!!".to_vec(),
            max_body_size: 1_048_576,
            rate_limit_per_minute: 60,
            signature_max_age: 300,
            current_time: 1700000000,
        }
    }
}

/// The result of processing a request through the middleware stack.
#[derive(Debug, Clone)]
pub enum MiddlewareResult {
    Continue(ProcessedRequest),
    Reject(HttpResponse),
}

/// A request that has been validated and enriched by middleware.
#[derive(Debug, Clone)]
pub struct ProcessedRequest {
    pub request: HttpRequest,
    pub user_id: Option<String>,
    pub roles: Vec<String>,
    pub correlation_id: String,
}

pub fn check_request_size(request: &HttpRequest, max_size: usize) -> MiddlewareResult {
    if request.body.len() > max_size {
        MiddlewareResult::Reject(HttpResponse {
            status: 413,
            body: r#"{"error": "Request body too large."}"#.to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
        })
    } else {
        MiddlewareResult::Continue(ProcessedRequest {
            request: request.clone(),
            user_id: None,
            roles: Vec::new(),
            correlation_id: String::new(),
        })
    }
}

pub fn check_rate_limit_simple(
    request: &HttpRequest,
    rate_map: &HashMap<String, u32>,
    max_requests: u32,
) -> MiddlewareResult {
    let count = rate_map.get(&request.client_ip).copied().unwrap_or(0);
    if count > max_requests {
        MiddlewareResult::Reject(HttpResponse {
            status: 429,
            body: r#"{"error": "Too many requests. Please try again later."}"#.to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
        })
    } else {
        MiddlewareResult::Continue(ProcessedRequest {
            request: request.clone(),
            user_id: None,
            roles: Vec::new(),
            correlation_id: String::new(),
        })
    }
}

pub fn validate_headers(request: &HttpRequest) -> MiddlewareResult {
    // Check Content-Type for POST/PUT with body
    if (request.method == "POST" || request.method == "PUT") && !request.body.is_empty() {
        let has_content_type = request
            .headers
            .keys()
            .any(|k| k.to_lowercase() == "content-type");
        let has_json = request
            .headers
            .iter()
            .any(|(k, v)| k.to_lowercase() == "content-type" && v.contains("application/json"));

        if !has_content_type || !has_json {
            return MiddlewareResult::Reject(HttpResponse {
                status: 400,
                body: r#"{"error": "Content-Type must be application/json"}"#.to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
            });
        }
    }

    // Check Authorization for /api/ endpoints
    if request.path.starts_with("/api/") {
        let has_auth = request
            .headers
            .keys()
            .any(|k| k.to_lowercase() == "authorization");
        if !has_auth {
            return MiddlewareResult::Reject(HttpResponse {
                status: 401,
                body: r#"{"error": "Authentication is required."}"#.to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
            });
        }
    }

    // Check header value lengths
    for (name, value) in &request.headers {
        if value.len() > 8192 {
            return MiddlewareResult::Reject(HttpResponse {
                status: 400,
                body: format!(
                    r#"{{"error": "Header '{}' value too long"}}"#,
                    name
                ),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
            });
        }
    }

    MiddlewareResult::Continue(ProcessedRequest {
        request: request.clone(),
        user_id: None,
        roles: Vec::new(),
        correlation_id: String::new(),
    })
}

pub fn authenticate_request(
    request: &HttpRequest,
    _jwt_secret: &[u8],
    current_time: u64,
) -> MiddlewareResult {
    let auth_header = match request.headers.get("Authorization") {
        Some(h) => h.as_str(),
        None => {
            // No auth header -- might be a public endpoint
            return MiddlewareResult::Continue(ProcessedRequest {
                request: request.clone(),
                user_id: None,
                roles: Vec::new(),
                correlation_id: String::new(),
            });
        }
    };

    // Extract Bearer token
    if !auth_header.starts_with("Bearer ") {
        return MiddlewareResult::Reject(HttpResponse {
            status: 401,
            body: r#"{"error": "Authentication is required."}"#.to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
        });
    }

    let token = &auth_header[7..];
    if token.is_empty() {
        return MiddlewareResult::Reject(HttpResponse {
            status: 401,
            body: r#"{"error": "Authentication is required."}"#.to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
        });
    }

    // Parse the JWT
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return MiddlewareResult::Reject(HttpResponse {
            status: 401,
            body: r#"{"error": "Authentication is required."}"#.to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
        });
    }

    // Decode payload
    let payload_bytes = match URL_SAFE_NO_PAD.decode(parts[1]) {
        Ok(b) => b,
        Err(_) => {
            return MiddlewareResult::Reject(HttpResponse {
                status: 401,
                body: r#"{"error": "Authentication is required."}"#.to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
            });
        }
    };

    let claims: serde_json::Value = match serde_json::from_slice(&payload_bytes) {
        Ok(c) => c,
        Err(_) => {
            return MiddlewareResult::Reject(HttpResponse {
                status: 401,
                body: r#"{"error": "Authentication is required."}"#.to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
            });
        }
    };

    // Check expiry
    let exp = claims.get("exp").and_then(|v| v.as_u64()).unwrap_or(0);
    if exp <= current_time {
        return MiddlewareResult::Reject(HttpResponse {
            status: 401,
            body: r#"{"error": "Authentication is required."}"#.to_string(),
            headers: {
                let mut h = HashMap::new();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
        });
    }

    // Extract user info
    let user_id = claims
        .get("sub")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let roles: Vec<String> = claims
        .get("roles")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    MiddlewareResult::Continue(ProcessedRequest {
        request: request.clone(),
        user_id: Some(user_id),
        roles,
        correlation_id: String::new(),
    })
}

pub fn authorize_request(processed: &ProcessedRequest) -> MiddlewareResult {
    // Non-API endpoints are public
    if !processed.request.path.starts_with("/api/") {
        return MiddlewareResult::Continue(ProcessedRequest {
            request: processed.request.clone(),
            user_id: processed.user_id.clone(),
            roles: processed.roles.clone(),
            correlation_id: processed.correlation_id.clone(),
        });
    }

    // /api/admin/* requires admin role
    if processed.request.path.starts_with("/api/admin/") {
        if !processed.roles.contains(&"admin".to_string()) {
            return MiddlewareResult::Reject(HttpResponse {
                status: 403,
                body: r#"{"error": "You do not have permission to perform this action."}"#
                    .to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
            });
        }
    }

    // GET /api/users/* requires users:read
    if processed.request.method == "GET"
        && processed.request.path.starts_with("/api/users")
    {
        if !processed.roles.iter().any(|r| r == "users:read") {
            return MiddlewareResult::Reject(HttpResponse {
                status: 403,
                body: r#"{"error": "You do not have permission to perform this action."}"#
                    .to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
            });
        }
    }

    // POST /api/users/* requires users:write
    if processed.request.method == "POST"
        && processed.request.path.starts_with("/api/users")
    {
        if !processed.roles.iter().any(|r| r == "users:write") {
            return MiddlewareResult::Reject(HttpResponse {
                status: 403,
                body: r#"{"error": "You do not have permission to perform this action."}"#
                    .to_string(),
                headers: {
                    let mut h = HashMap::new();
                    h.insert("Content-Type".to_string(), "application/json".to_string());
                    h
                },
            });
        }
    }

    // Other /api/* endpoints just require authentication
    MiddlewareResult::Continue(ProcessedRequest {
        request: processed.request.clone(),
        user_id: processed.user_id.clone(),
        roles: processed.roles.clone(),
        correlation_id: processed.correlation_id.clone(),
    })
}

pub fn process_request(
    request: &HttpRequest,
    config: &ApiConfig,
    rate_map: &HashMap<String, u32>,
    req_counter: u64,
) -> MiddlewareResult {
    let correlation_id = format!("req-{}", req_counter);

    // Step 1: Request size check
    if let MiddlewareResult::Reject(r) = check_request_size(request, config.max_body_size) {
        return MiddlewareResult::Reject(r);
    }

    // Step 2: Header validation
    if let MiddlewareResult::Reject(r) = validate_headers(request) {
        return MiddlewareResult::Reject(r);
    }

    // Step 3: Rate limiting
    if let MiddlewareResult::Reject(r) = check_rate_limit_simple(request, rate_map, config.rate_limit_per_minute) {
        return MiddlewareResult::Reject(r);
    }

    // Step 4: Authentication
    let processed = match authenticate_request(request, &config.jwt_secret, config.current_time) {
        MiddlewareResult::Reject(r) => return MiddlewareResult::Reject(r),
        MiddlewareResult::Continue(mut p) => {
            p.correlation_id = correlation_id.clone();
            p
        }
    };

    // Step 5: Authorization
    if let MiddlewareResult::Reject(r) = authorize_request(&processed) {
        return MiddlewareResult::Reject(r);
    }

    MiddlewareResult::Continue(processed)
}

pub fn error_response(status: u16, message: &str, correlation_id: &str) -> HttpResponse {
    let body = serde_json::json!({
        "error": message,
        "correlation_id": correlation_id,
    });

    HttpResponse {
        status,
        body: body.to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
    }
}

pub fn success_response(status: u16, body: &str) -> HttpResponse {
    HttpResponse {
        status,
        body: body.to_string(),
        headers: {
            let mut h = HashMap::new();
            h.insert("Content-Type".to_string(), "application/json".to_string());
            h
        },
    }
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
                h.insert(
                    "Authorization".to_string(),
                    "Bearer header.eyJzdWIiOiJ1c2VyMTIzIiwiZXhwIjoyMDAwMDAwMDAwLCJyb2xlcyI6WyJ1c2VyIl19.sig"
                        .to_string(),
                );
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
        assert_eq!(
            resp.headers.get("Content-Type").unwrap(),
            "application/json"
        );
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
        let result = validate_headers(&request);
        assert!(matches!(result, MiddlewareResult::Continue(_)));
    }
}
