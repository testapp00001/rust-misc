//! # Lesson 04: CORS Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use std::collections::{HashMap, HashSet};

pub fn is_origin_allowed(origin: &str, allowed_origins: &HashSet<String>) -> bool {
    allowed_origins.contains(origin)
}

pub fn build_cors_headers(
    request_origin: &str,
    allowed_origins: &HashSet<String>,
) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    if is_origin_allowed(request_origin, allowed_origins) {
        headers.insert("Access-Control-Allow-Origin".to_string(), request_origin.to_string());
        headers.insert("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string());
        headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type, Authorization, X-Request-ID".to_string());
        headers.insert("Vary".to_string(), "Origin".to_string());
    }

    headers
}

pub fn build_cors_headers_with_credentials(
    request_origin: &str,
    allowed_origins: &HashSet<String>,
) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    // Reject null origin — can be spoofed by local files
    if request_origin == "null" {
        return headers;
    }

    if is_origin_allowed(request_origin, allowed_origins) {
        headers.insert("Access-Control-Allow-Origin".to_string(), request_origin.to_string());
        headers.insert("Access-Control-Allow-Methods".to_string(), "GET, POST, PUT, DELETE, OPTIONS".to_string());
        headers.insert("Access-Control-Allow-Headers".to_string(), "Content-Type, Authorization, X-Request-ID".to_string());
        headers.insert("Access-Control-Allow-Credentials".to_string(), "true".to_string());
        headers.insert("Vary".to_string(), "Origin".to_string());
    }

    headers
}

pub fn audit_cors_config(
    allow_credentials: bool,
    allowed_origins: &HashSet<String>,
) -> Vec<String> {
    let mut issues = Vec::new();

    if allowed_origins.is_empty() {
        issues.push("no origins configured".to_string());
        return issues;
    }

    for origin in allowed_origins {
        if origin == "*" {
            if allow_credentials {
                issues.push("wildcard origin '*' with credentials is insecure".to_string());
            } else {
                issues.push("wildcard origin '*' should be avoided".to_string());
            }
        }
        if origin == "null" {
            issues.push("null origin allowed — can be spoofed by local files".to_string());
        }
        if origin.starts_with("*.") {
            issues.push(format!("wildcard subdomain '{}' is too permissive", origin));
        }
        if origin.starts_with("http://") {
            issues.push(format!("http origin '{}' should use https", origin));
        }
    }

    issues
}

pub fn validate_origin(origin: &str) -> Result<String, &'static str> {
    if origin.is_empty() {
        return Err("empty origin");
    }

    if !origin.starts_with("https://") && !origin.starts_with("http://") {
        return Err("origin must have a scheme");
    }

    // After the scheme, find the authority (host:port)
    let after_scheme = if origin.starts_with("https://") {
        &origin[8..]
    } else {
        &origin[7..]
    };

    // The authority should not contain /, ?, or #
    if after_scheme.contains('/') || after_scheme.contains('?') || after_scheme.contains('#') {
        return Err("origin must not contain a path");
    }

    if after_scheme.is_empty() {
        return Err("origin must have a host");
    }

    Ok(origin.to_string())
}

pub fn validate_preflight(
    headers: &HashMap<String, String>,
    allowed_origins: &HashSet<String>,
    allowed_methods: &str,
) -> Result<(), &'static str> {
    let origin = match headers.get("Origin") {
        Some(o) => o,
        None => return Err("missing_origin"),
    };

    let method = match headers.get("Access-Control-Request-Method") {
        Some(m) => m,
        None => return Err("missing_method"),
    };

    if !is_origin_allowed(origin, allowed_origins) {
        return Err("origin_not_allowed");
    }

    let methods: Vec<&str> = allowed_methods.split(',').map(|m| m.trim()).collect();
    if !methods.contains(&method.as_str()) {
        return Err("method_not_allowed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_origins() -> HashSet<String> {
        let mut s = HashSet::new();
        s.insert("https://trusted-app.com".to_string());
        s.insert("https://admin.example.com".to_string());
        s
    }

    #[test]
    fn test_origin_allowed() {
        assert!(is_origin_allowed("https://trusted-app.com", &sample_origins()));
    }

    #[test]
    fn test_origin_not_allowed() {
        assert!(!is_origin_allowed("https://evil.com", &sample_origins()));
    }

    #[test]
    fn test_origin_scheme_matters() {
        assert!(!is_origin_allowed("http://trusted-app.com", &sample_origins()));
    }

    #[test]
    fn test_cors_headers_allowed() {
        let headers = build_cors_headers("https://trusted-app.com", &sample_origins());
        assert_eq!(headers.get("Access-Control-Allow-Origin").unwrap(), "https://trusted-app.com");
        assert!(headers.contains_key("Access-Control-Allow-Methods"));
        assert!(headers.contains_key("Vary"));
    }

    #[test]
    fn test_cors_headers_not_allowed() {
        let headers = build_cors_headers("https://evil.com", &sample_origins());
        assert!(!headers.contains_key("Access-Control-Allow-Origin"));
    }

    #[test]
    fn test_cors_credentials_allowed() {
        let headers = build_cors_headers_with_credentials("https://trusted-app.com", &sample_origins());
        assert_eq!(headers.get("Access-Control-Allow-Origin").unwrap(), "https://trusted-app.com");
        assert_eq!(headers.get("Access-Control-Allow-Credentials").unwrap(), "true");
    }

    #[test]
    fn test_cors_credentials_null_origin_rejected() {
        let mut origins = sample_origins();
        origins.insert("null".to_string());
        let headers = build_cors_headers_with_credentials("null", &origins);
        assert!(!headers.contains_key("Access-Control-Allow-Origin"));
    }

    #[test]
    fn test_audit_finds_wildcard() {
        let mut origins = HashSet::new();
        origins.insert("*".to_string());
        let issues = audit_cors_config(true, &origins);
        assert!(issues.iter().any(|i| i.contains("wildcard")));
    }

    #[test]
    fn test_audit_finds_null_origin() {
        let mut origins = HashSet::new();
        origins.insert("null".to_string());
        let issues = audit_cors_config(false, &origins);
        assert!(issues.iter().any(|i| i.contains("null")));
    }

    #[test]
    fn test_audit_finds_http_origin() {
        let mut origins = HashSet::new();
        origins.insert("http://example.com".to_string());
        let issues = audit_cors_config(false, &origins);
        assert!(issues.iter().any(|i| i.contains("http")));
    }

    #[test]
    fn test_audit_clean_config() {
        let issues = audit_cors_config(false, &sample_origins());
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_origin_valid() {
        assert_eq!(validate_origin("https://example.com").unwrap(), "https://example.com");
    }

    #[test]
    fn test_validate_origin_with_port() {
        assert_eq!(validate_origin("https://example.com:8080").unwrap(), "https://example.com:8080");
    }

    #[test]
    fn test_validate_origin_with_path() {
        assert!(validate_origin("https://example.com/path").is_err());
    }

    #[test]
    fn test_validate_origin_no_scheme() {
        assert!(validate_origin("example.com").is_err());
    }

    #[test]
    fn test_validate_origin_empty() {
        assert!(validate_origin("").is_err());
    }

    #[test]
    fn test_preflight_valid() {
        let mut headers = HashMap::new();
        headers.insert("Origin".to_string(), "https://trusted-app.com".to_string());
        headers.insert("Access-Control-Request-Method".to_string(), "POST".to_string());
        let result = validate_preflight(&headers, &sample_origins(), "GET, POST, PUT, DELETE, OPTIONS");
        assert!(result.is_ok());
    }

    #[test]
    fn test_preflight_missing_origin() {
        let mut headers = HashMap::new();
        headers.insert("Access-Control-Request-Method".to_string(), "POST".to_string());
        let result = validate_preflight(&headers, &sample_origins(), "GET, POST");
        assert_eq!(result.unwrap_err(), "missing_origin");
    }

    #[test]
    fn test_preflight_method_not_allowed() {
        let mut headers = HashMap::new();
        headers.insert("Origin".to_string(), "https://trusted-app.com".to_string());
        headers.insert("Access-Control-Request-Method".to_string(), "PATCH".to_string());
        let result = validate_preflight(&headers, &sample_origins(), "GET, POST, PUT");
        assert_eq!(result.unwrap_err(), "method_not_allowed");
    }
}
