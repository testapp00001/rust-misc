//! # Lesson 05: CSRF Protection
//!
//! ## What Is CSRF?
//!
//! Cross-Site Request Forgery (CSRF) tricks an authenticated user's browser
//! into making an unwanted request to your API. The victim's cookies are
//! automatically included by the browser.
//!
//! ## Attack Scenario
//!
//! ```text
//! 1. User logs into bank.com → receives session cookie
//! 2. User visits evil-site.com (while still logged into bank.com)
//! 3. evil-site.com contains:
//!    <form action="https://bank.com/transfer" method="POST">
//!      <input name="to" value="attacker">
//!      <input name="amount" value="10000">
//!    </form>
//!    <script>document.forms[0].submit();</script>
//! 4. Browser sends the form POST to bank.com WITH the session cookie
//! 5. bank.com processes the transfer as if the user initiated it
//! ```
//!
//! ## Defense Layers
//!
//! ### 1. SameSite Cookies
//!
//! ```text
//! Set-Cookie: session=abc123; SameSite=Strict; Secure; HttpOnly
//! ```
//!
//! - `Strict`: Cookie never sent on cross-site requests
//! - `Lax`: Cookie sent on top-level GET navigations only
//! - `None`: Cookie always sent (requires `Secure`)
//!
//! ### 2. CSRF Tokens
//!
//! A random token embedded in each form. The server validates that the token
//! in the form matches the token in the session.
//!
//! ```text
//! 1. Server generates random token, stores in session, embeds in form
//! 2. User submits form → browser sends cookie + form data (with token)
//! 3. Server checks: does form token match session token?
//! ```
//!
//! Attacker can't read the token from another origin (same-origin policy).
//!
//! ### 3. Double-Submit Cookie
//!
//! Token is set as a cookie AND sent in a header. The server checks they match.
//! An attacker can't set cookies for another origin.
//!
//! ## Defense
//!
//! 1. Use `SameSite=Strict` or `SameSite=Lax` on session cookies
//! 2. Require CSRF tokens for all state-changing requests (POST, PUT, DELETE)
//! 3. Don't rely solely on cookies — use Authorization headers for APIs
//! 4. Check `Origin` and `Referer` headers as additional validation

use rand::Rng;
use std::collections::HashMap;

/// Exercise 1: Generate a CSRF token.
///
/// Generate a cryptographically random CSRF token as a hex-encoded string.
/// The token should be 32 bytes (256 bits) of randomness.
///
/// Hints:
/// - Use `rand::rngs::OsRng` for cryptographic randomness
/// - Generate 32 random bytes
/// - Encode with `hex::encode`
pub fn generate_csrf_token() -> String {
    todo!("Generate a cryptographically random CSRF token")
}

/// Exercise 2: Build a Set-Cookie header with SameSite attribute.
///
/// Given a cookie name, value, and settings, build a Set-Cookie header string.
///
/// Format:
/// ```text
/// "{name}={value}; SameSite={samesite}; Path={path}{; Secure}{; HttpOnly}"
/// ```
///
/// Parameters:
/// - `name`: cookie name
/// - `value`: cookie value
/// - `same_site`: "Strict", "Lax", or "None"
/// - `secure`: whether to add the Secure flag
/// - `http_only`: whether to add the HttpOnly flag
/// - `path`: the cookie path (e.g., "/")
///
/// Security rules:
/// - If SameSite=None, Secure MUST be true (browser requirement)
/// - If secure is true and SameSite=None is set without Secure, return an error
///
/// Hints:
/// - Build the string parts conditionally
/// - Use String::new() and push_str for building
pub fn build_set_cookie(
    name: &str,
    value: &str,
    same_site: &str,
    secure: bool,
    http_only: bool,
    path: &str,
) -> Result<String, &'static str> {
    todo!("Build a Set-Cookie header with SameSite attribute")
}

/// Exercise 3: Validate a CSRF token from a form submission.
///
/// The session stores the expected CSRF token. The form submission includes
/// a token. Validate they match using constant-time comparison.
///
/// Returns `true` if the token is valid, `false` otherwise.
///
/// Security rules:
/// - Use constant-time comparison to prevent timing attacks
/// - A missing token (empty string) should always fail
/// - Both the session token and form token must be non-empty
///
/// Hints:
/// - Check both tokens are non-empty first
/// - Use `ring::constant_time::verify_slices_are_equal`
pub fn validate_csrf_token(session_token: &str, form_token: &str) -> bool {
    todo!("Validate a CSRF token using constant-time comparison")
}

/// Exercise 4: Extract and validate a CSRF token from request headers.
///
/// For API-style CSRF protection, the token is sent in a custom HTTP header
/// (e.g., `X-CSRF-Token`) rather than form data.
///
/// Given request headers and the session token, validate the CSRF token.
///
/// Returns:
/// - `Ok(())` if valid
/// - `Err("missing_header")` if the X-CSRF-Token header is not present
/// - `Err("invalid_token")` if the token doesn't match
///
/// Hints:
/// - Get "X-CSRF-Token" from the headers HashMap
/// - Use validate_csrf_token for the comparison
pub fn validate_csrf_from_headers(
    headers: &HashMap<String, String>,
    session_token: &str,
) -> Result<(), &'static str> {
    todo!("Validate CSRF token from request headers")
}

/// Exercise 5: Implement double-submit cookie pattern.
///
/// In this pattern:
/// 1. The CSRF token is stored in a cookie AND sent in a header
/// 2. The server verifies the cookie value matches the header value
/// 3. An attacker can't set cookies for another origin, so they can't match
///
/// Given the request headers, extract the CSRF token from both:
/// - Cookie: parse from the `Cookie` header (format: "name1=val1; name2=val2")
/// - Header: from `X-CSRF-Token`
///
/// They must match. Returns `true` if they match, `false` otherwise.
///
/// Hints:
/// - Parse the Cookie header to find the "csrf_token" cookie value
/// - Split on "; ", then find the entry starting with "csrf_token="
/// - Compare with the X-CSRF-Token header value
pub fn validate_double_submit(headers: &HashMap<String, String>) -> bool {
    todo!("Validate double-submit CSRF cookie pattern")
}

/// Exercise 6: Check if a request method requires CSRF protection.
///
/// State-changing methods (POST, PUT, DELETE, PATCH) require CSRF protection.
/// Safe methods (GET, HEAD, OPTIONS) do not.
///
/// Returns `true` if the method requires CSRF protection.
///
/// Hints:
/// - Match on the method string (case-insensitive)
/// - GET, HEAD, OPTIONS are safe → false
/// - POST, PUT, DELETE, PATCH require protection → true
pub fn requires_csrf_protection(method: &str) -> bool {
    todo!("Check if HTTP method requires CSRF protection")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_length() {
        let token = generate_csrf_token();
        // 32 bytes = 64 hex characters
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_generate_token_is_hex() {
        let token = generate_csrf_token();
        assert!(hex::decode(&token).is_ok());
    }

    #[test]
    fn test_generate_tokens_unique() {
        let t1 = generate_csrf_token();
        let t2 = generate_csrf_token();
        assert_ne!(t1, t2);
    }

    #[test]
    fn test_set_cookie_strict() {
        let header = build_set_cookie("session", "abc123", "Strict", true, true, "/").unwrap();
        assert!(header.contains("SameSite=Strict"));
        assert!(header.contains("Secure"));
        assert!(header.contains("HttpOnly"));
        assert!(header.contains("session=abc123"));
    }

    #[test]
    fn test_set_cookie_none_requires_secure() {
        let result = build_set_cookie("session", "abc123", "None", false, true, "/");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_cookie_none_with_secure() {
        let header = build_set_cookie("session", "abc123", "None", true, false, "/").unwrap();
        assert!(header.contains("SameSite=None"));
        assert!(header.contains("Secure"));
        assert!(!header.contains("HttpOnly"));
    }

    #[test]
    fn test_validate_csrf_token_valid() {
        assert!(validate_csrf_token("token123", "token123"));
    }

    #[test]
    fn test_validate_csrf_token_mismatch() {
        assert!(!validate_csrf_token("token123", "wrong_token"));
    }

    #[test]
    fn test_validate_csrf_token_empty() {
        assert!(!validate_csrf_token("", ""));
        assert!(!validate_csrf_token("token123", ""));
        assert!(!validate_csrf_token("", "token123"));
    }

    #[test]
    fn test_validate_csrf_from_headers_valid() {
        let mut headers = HashMap::new();
        headers.insert("X-CSRF-Token".to_string(), "mytoken".to_string());
        assert!(validate_csrf_from_headers(&headers, "mytoken").is_ok());
    }

    #[test]
    fn test_validate_csrf_from_headers_missing() {
        let headers = HashMap::new();
        assert_eq!(validate_csrf_from_headers(&headers, "token").unwrap_err(), "missing_header");
    }

    #[test]
    fn test_validate_csrf_from_headers_invalid() {
        let mut headers = HashMap::new();
        headers.insert("X-CSRF-Token".to_string(), "wrong".to_string());
        assert_eq!(validate_csrf_from_headers(&headers, "correct").unwrap_err(), "invalid_token");
    }

    #[test]
    fn test_double_submit_valid() {
        let mut headers = HashMap::new();
        headers.insert("Cookie".to_string(), "session=abc; csrf_token=mytoken".to_string());
        headers.insert("X-CSRF-Token".to_string(), "mytoken".to_string());
        assert!(validate_double_submit(&headers));
    }

    #[test]
    fn test_double_submit_mismatch() {
        let mut headers = HashMap::new();
        headers.insert("Cookie".to_string(), "csrf_token=token_a".to_string());
        headers.insert("X-CSRF-Token".to_string(), "token_b".to_string());
        assert!(!validate_double_submit(&headers));
    }

    #[test]
    fn test_double_submit_missing_cookie() {
        let mut headers = HashMap::new();
        headers.insert("X-CSRF-Token".to_string(), "token".to_string());
        assert!(!validate_double_submit(&headers));
    }

    #[test]
    fn test_requires_csrf_post() {
        assert!(requires_csrf_protection("POST"));
        assert!(requires_csrf_protection("PUT"));
        assert!(requires_csrf_protection("DELETE"));
        assert!(requires_csrf_protection("PATCH"));
    }

    #[test]
    fn test_no_csrf_get() {
        assert!(!requires_csrf_protection("GET"));
        assert!(!requires_csrf_protection("HEAD"));
        assert!(!requires_csrf_protection("OPTIONS"));
    }
}
