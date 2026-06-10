//! # Lesson 05: CSRF Protection (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use rand::Rng;
use std::collections::HashMap;

pub fn generate_csrf_token() -> String {
    let mut rng = rand::rngs::OsRng;
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes);
    hex::encode(bytes)
}

pub fn build_set_cookie(
    name: &str,
    value: &str,
    same_site: &str,
    secure: bool,
    http_only: bool,
    path: &str,
) -> Result<String, &'static str> {
    // SameSite=None requires Secure
    if same_site == "None" && !secure {
        return Err("SameSite=None requires Secure flag");
    }

    let mut cookie = format!("{}={}; SameSite={}; Path={}", name, value, same_site, path);

    if secure {
        cookie.push_str("; Secure");
    }
    if http_only {
        cookie.push_str("; HttpOnly");
    }

    Ok(cookie)
}

pub fn validate_csrf_token(session_token: &str, form_token: &str) -> bool {
    if session_token.is_empty() || form_token.is_empty() {
        return false;
    }

    ring::constant_time::verify_slices_are_equal(
        session_token.as_bytes(),
        form_token.as_bytes(),
    ).is_ok()
}

pub fn validate_csrf_from_headers(
    headers: &HashMap<String, String>,
    session_token: &str,
) -> Result<(), &'static str> {
    let token = match headers.get("X-CSRF-Token") {
        Some(t) => t,
        None => return Err("missing_header"),
    };

    if validate_csrf_token(session_token, token) {
        Ok(())
    } else {
        Err("invalid_token")
    }
}

pub fn validate_double_submit(headers: &HashMap<String, String>) -> bool {
    let cookie_header = match headers.get("Cookie") {
        Some(c) => c,
        None => return false,
    };

    let cookie_token = cookie_header
        .split("; ")
        .find(|part| part.starts_with("csrf_token="))
        .and_then(|part| part.strip_prefix("csrf_token="));

    let header_token = headers.get("X-CSRF-Token");

    match (cookie_token, header_token) {
        (Some(ct), Some(ht)) => ct == ht,
        _ => false,
    }
}

pub fn requires_csrf_protection(method: &str) -> bool {
    match method.to_uppercase().as_str() {
        "GET" | "HEAD" | "OPTIONS" => false,
        "POST" | "PUT" | "DELETE" | "PATCH" => true,
        _ => true, // Unknown methods should require protection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_length() {
        let token = generate_csrf_token();
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
