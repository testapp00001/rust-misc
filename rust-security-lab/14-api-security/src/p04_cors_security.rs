//! # Lesson 04: CORS Security
//!
//! ## What Is CORS?
//!
//! Cross-Origin Resource Sharing (CORS) is a browser mechanism that controls
//! which web origins can access your API. Without CORS, a malicious website
//! could make requests to your API using the victim's cookies.
//!
//! ## Same-Origin Policy
//!
//! Browsers block cross-origin requests by default. An "origin" is:
//!
//! ```text
//! scheme + host + port
//! https://trusted-app.com:443  ← one origin
//! http://evil-site.com:80      ← different origin
//! ```
//!
//! CORS headers tell the browser: "It's OK for these other origins to access me."
//!
//! ## The Danger of Misconfigured CORS
//!
//! ```rust
//! // DANGEROUS — reflects any origin
//! Access-Control-Allow-Origin: <reflects Origin header>
//! Access-Control-Allow-Credentials: true
//!
//! // This allows ANY website to make authenticated requests to your API!
//! // evil-site.com can read the victim's data using their cookies.
//! ```
//!
//! ## Attack: CORS Origin Reflection
//!
//! 1. Victim visits evil-site.com
//! 2. evil-site.com JavaScript makes a fetch to your-api.com
//! 3. Your API reflects evil-site.com in `Access-Control-Allow-Origin`
//! 4. Browser allows evil-site.com to read the response
//! 5. Attacker now has the victim's data
//!
//! ## Defense
//!
//! 1. Whitelist specific origins — never reflect the Origin header blindly
//! 2. Don't use `*` with credentials
//! 3. Don't allow `null` origin (local files send this)
//! 4. Validate origin format (must be a valid URL with scheme)
//! 5. Use `Vary: Origin` to prevent cache poisoning

use std::collections::HashSet;

/// Exercise 1: Validate an origin against a whitelist.
///
/// Check if the given origin is in the allowed set. Return `true` if allowed.
///
/// The origin is a URL like "https://trusted-app.com". The whitelist is a set
/// of allowed origins.
///
/// Hints:
/// - Use `HashSet::contains` for O(1) lookup
/// - Origin must exactly match (including scheme and port)
pub fn is_origin_allowed(origin: &str, allowed_origins: &HashSet<String>) -> bool {
    todo!("Check if origin is in the whitelist")
}

/// Exercise 2: Build CORS response headers.
///
/// Given the request's `Origin` header and a whitelist, return the appropriate
/// CORS response headers as a `HashMap`.
///
/// Rules:
/// - If origin is in the whitelist, set `Access-Control-Allow-Origin` to that origin
/// - If origin is NOT in the whitelist, return an empty HashMap (no CORS headers)
/// - Always set `Access-Control-Allow-Methods` to "GET, POST, PUT, DELETE, OPTIONS"
/// - Always set `Access-Control-Allow-Headers` to "Content-Type, Authorization, X-Request-ID"
/// - Set `Vary: Origin` to prevent cache poisoning
///
/// Security rules:
/// - NEVER reflect the Origin if it's not in the whitelist
/// - NEVER use `*` as the allowed origin
///
/// Hints:
/// - Use `is_origin_allowed` to check the whitelist
/// - Return `HashMap<String, String>`
pub fn build_cors_headers(
    request_origin: &str,
    allowed_origins: &HashSet<String>,
) -> std::collections::HashMap<String, String> {
    todo!("Build CORS response headers based on origin whitelist")
}

/// Exercise 3: Build CORS headers with credentials support.
///
/// Same as `build_cors_headers` but also sets `Access-Control-Allow-Credentials: true`
/// when the origin is allowed.
///
/// IMPORTANT: When `Allow-Credentials` is `true`, the origin MUST NOT be `*`.
/// It must be a specific origin. This is a browser security requirement.
///
/// Additional security check:
/// - Reject origin "null" (sent by local files, can be spoofed)
///
/// Hints:
/// - Add "Access-Control-Allow-Credentials" = "true"
/// - Reject "null" origin explicitly
pub fn build_cors_headers_with_credentials(
    request_origin: &str,
    allowed_origins: &HashSet<String>,
) -> std::collections::HashMap<String, String> {
    todo!("Build CORS headers with credentials support")
}

/// Exercise 4: Detect insecure CORS configurations.
///
/// Audit a CORS configuration and return a list of security issues found.
///
/// Check for these problems:
/// 1. Origin `*` is in the allowed list → "wildcard origin with credentials"
/// 2. `null` origin is in the allowed list → "null origin allowed"
/// 3. Allowed origins contain a wildcard subdomain like `*.example.com` → "wildcard subdomain"
/// 4. Any origin starts with `http://` → "http origin (not https)"
/// 5. The allowed origins set is empty → "no origins configured"
///
/// Return a Vec of problem description strings. Empty Vec means no issues.
///
/// Hints:
/// - Iterate over allowed_origins and check each one
/// - Use `.starts_with()`, `.contains()`, etc.
pub fn audit_cors_config(
    allow_credentials: bool,
    allowed_origins: &HashSet<String>,
) -> Vec<String> {
    todo!("Audit CORS configuration for security issues")
}

/// Exercise 5: Validate and normalize an origin URL.
///
/// An origin is valid if:
/// 1. It has a scheme (http:// or https://)
/// 2. It has a host
/// 3. It does NOT contain a path, query string, or fragment
///
/// Return `Ok(normalized_origin)` if valid, `Err("reason")` if not.
///
/// Examples:
/// - "https://example.com" → Ok("https://example.com")
/// - "https://example.com:8080" → Ok("https://example.com:8080")
/// - "https://example.com/path" → Err("origin must not contain a path")
/// - "example.com" → Err("origin must have a scheme")
/// - "" → Err("empty origin")
///
/// Hints:
/// - Check that it starts with "http://" or "https://"
/// - After the scheme+host portion, there should be no '/', '?', or '#'
/// - Parse the URL manually: find the scheme, then the authority (host:port)
pub fn validate_origin(origin: &str) -> Result<String, &'static str> {
    todo!("Validate and normalize an origin URL")
}

/// Exercise 6: Check if a CORS preflight request is valid.
///
/// A CORS preflight is an OPTIONS request that the browser sends before the
/// actual request to check if CORS is allowed. Validate the preflight:
///
/// Rules:
/// 1. `Origin` header must be present
/// 2. `Access-Control-Request-Method` must be present and be a valid HTTP method
/// 3. Origin must be in the allowed list
/// 4. The requested method must be in the allowed methods list
///
/// Returns:
/// - `Ok(())` if the preflight is valid
/// - `Err("missing_origin")` if Origin header is missing
/// - `Err("missing_method")` if Request-Method header is missing
/// - `Err("origin_not_allowed")` if origin is not whitelisted
/// - `Err("method_not_allowed")` if the method is not in the allowed list
///
/// Hints:
/// - Check headers for "Origin" and "Access-Control-Request-Method"
/// - Use `is_origin_allowed` for origin validation
/// - Check if the method is in "GET, POST, PUT, DELETE, OPTIONS"
pub fn validate_preflight(
    headers: &std::collections::HashMap<String, String>,
    allowed_origins: &HashSet<String>,
    allowed_methods: &str,
) -> Result<(), &'static str> {
    todo!("Validate a CORS preflight request")
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
        // http vs https should not match
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
        let mut headers = std::collections::HashMap::new();
        headers.insert("Origin".to_string(), "https://trusted-app.com".to_string());
        headers.insert("Access-Control-Request-Method".to_string(), "POST".to_string());
        let result = validate_preflight(&headers, &sample_origins(), "GET, POST, PUT, DELETE, OPTIONS");
        assert!(result.is_ok());
    }

    #[test]
    fn test_preflight_missing_origin() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Access-Control-Request-Method".to_string(), "POST".to_string());
        let result = validate_preflight(&headers, &sample_origins(), "GET, POST");
        assert_eq!(result.unwrap_err(), "missing_origin");
    }

    #[test]
    fn test_preflight_method_not_allowed() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("Origin".to_string(), "https://trusted-app.com".to_string());
        headers.insert("Access-Control-Request-Method".to_string(), "PATCH".to_string());
        let result = validate_preflight(&headers, &sample_origins(), "GET, POST, PUT");
        assert_eq!(result.unwrap_err(), "method_not_allowed");
    }
}
