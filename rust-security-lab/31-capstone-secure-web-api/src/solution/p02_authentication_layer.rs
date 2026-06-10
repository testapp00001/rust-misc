//! # Lesson 02: JWT Authentication Middleware — Solution
//!
//! Token validation, claims extraction, middleware pattern.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthContext {
    pub user_id: String,
    pub roles: Vec<String>,
    pub issued_at: u64,
    pub expires_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthError {
    MissingToken,
    MalformedToken,
    InvalidSignature,
    TokenExpired,
    InvalidClaims,
    DisallowedAlgorithm,
}

pub fn create_token(claims: &Claims, secret: &[u8]) -> String {
    let header = JwtHeader {
        alg: "HS256".to_string(),
        typ: "JWT".to_string(),
    };

    let header_json = serde_json::to_vec(&header).expect("header should serialize");
    let claims_json = serde_json::to_vec(&claims).expect("claims should serialize");

    let header_b64 = URL_SAFE_NO_PAD.encode(&header_json);
    let claims_b64 = URL_SAFE_NO_PAD.encode(&claims_json);
    let signing_input = format!("{}.{}", header_b64, claims_b64);

    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(signing_input.as_bytes());
    let signature = mac.finalize().into_bytes();
    let sig_b64 = URL_SAFE_NO_PAD.encode(&signature);

    format!("{}.{}", signing_input, sig_b64)
}

pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    if auth_header.starts_with("Bearer ") {
        let token = &auth_header[7..];
        if token.is_empty() {
            None
        } else {
            Some(token)
        }
    } else {
        None
    }
}

pub fn validate_token(
    token: &str,
    secret: &[u8],
    current_time: u64,
    expected_algorithm: &str,
) -> Result<AuthContext, AuthError> {
    // Step 1: Split into 3 parts
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::MalformedToken);
    }

    // Step 2: Decode and verify header algorithm
    let header_bytes = URL_SAFE_NO_PAD
        .decode(parts[0])
        .map_err(|_| AuthError::MalformedToken)?;
    let header: JwtHeader =
        serde_json::from_slice(&header_bytes).map_err(|_| AuthError::MalformedToken)?;

    if header.alg != expected_algorithm {
        return Err(AuthError::DisallowedAlgorithm);
    }

    // Step 3: Verify signature (constant-time via hmac crate)
    let signing_input = format!("{}.{}", parts[0], parts[1]);
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(signing_input.as_bytes());

    let sig_bytes = URL_SAFE_NO_PAD
        .decode(parts[2])
        .map_err(|_| AuthError::MalformedToken)?;
    mac.verify_slice(&sig_bytes)
        .map_err(|_| AuthError::InvalidSignature)?;

    // Step 4: Decode and deserialize claims
    let claims_bytes = URL_SAFE_NO_PAD
        .decode(parts[1])
        .map_err(|_| AuthError::MalformedToken)?;
    let claims: Claims =
        serde_json::from_slice(&claims_bytes).map_err(|_| AuthError::InvalidClaims)?;

    // Step 5: Check expiration
    if claims.exp <= current_time {
        return Err(AuthError::TokenExpired);
    }

    // Step 6: Return auth context
    Ok(AuthContext {
        user_id: claims.sub,
        roles: claims.roles,
        issued_at: claims.iat,
        expires_at: claims.exp,
    })
}

pub fn authenticate(
    auth_header: &str,
    secret: &[u8],
    current_time: u64,
) -> Result<AuthContext, AuthError> {
    let token = extract_bearer_token(auth_header).ok_or(AuthError::MissingToken)?;
    validate_token(token, secret, current_time, "HS256")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"test-secret-key-at-least-32-bytes!";

    fn sample_claims() -> Claims {
        Claims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1700000000,
            roles: vec!["user".to_string()],
            iss: Some("security-lab".to_string()),
        }
    }

    #[test]
    fn test_create_and_validate_token() {
        let token = create_token(&sample_claims(), TEST_SECRET);
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3);

        let ctx = validate_token(&token, TEST_SECRET, 1700000001, "HS256").unwrap();
        assert_eq!(ctx.user_id, "user123");
        assert_eq!(ctx.roles, vec!["user"]);
    }

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(
            extract_bearer_token("Bearer eyJhbGciOiJIUzI1NiJ9"),
            Some("eyJhbGciOiJIUzI1NiJ9")
        );
        assert_eq!(extract_bearer_token("Basic dXNlcjpwYXNz"), None);
        assert_eq!(extract_bearer_token(""), None);
        assert_eq!(extract_bearer_token("Bearer"), None);
    }

    #[test]
    fn test_validate_expired_token() {
        let mut claims = sample_claims();
        claims.exp = 1000000000;
        let token = create_token(&claims, TEST_SECRET);
        let result = validate_token(&token, TEST_SECRET, 1700000000, "HS256");
        assert_eq!(result, Err(AuthError::TokenExpired));
    }

    #[test]
    fn test_validate_wrong_secret() {
        let token = create_token(&sample_claims(), TEST_SECRET);
        let wrong_secret = b"wrong-secret-key-at-least-32-byte";
        let result = validate_token(&token, wrong_secret, 1700000001, "HS256");
        assert_eq!(result, Err(AuthError::InvalidSignature));
    }

    #[test]
    fn test_validate_malformed_token() {
        let result = validate_token("not.a.jwt.token", TEST_SECRET, 1700000001, "HS256");
        assert_eq!(result, Err(AuthError::MalformedToken));
    }

    #[test]
    fn test_validate_tampered_token() {
        let token = create_token(&sample_claims(), TEST_SECRET);
        let mut parts: Vec<&str> = token.split('.').collect();
        parts[1] = "dGFtcGVyZWQ";
        let tampered = parts.join(".");
        let result = validate_token(&tampered, TEST_SECRET, 1700000001, "HS256");
        assert_eq!(result, Err(AuthError::InvalidSignature));
    }

    #[test]
    fn test_authenticate_full_flow() {
        let token = create_token(&sample_claims(), TEST_SECRET);
        let auth_header = format!("Bearer {}", token);
        let ctx = authenticate(&auth_header, TEST_SECRET, 1700000001).unwrap();
        assert_eq!(ctx.user_id, "user123");
    }

    #[test]
    fn test_authenticate_missing_header() {
        let result = authenticate("", TEST_SECRET, 1700000001);
        assert_eq!(result, Err(AuthError::MissingToken));
    }

    #[test]
    fn test_validate_wrong_algorithm() {
        let token = create_token(&sample_claims(), TEST_SECRET);
        let result = validate_token(&token, TEST_SECRET, 1700000001, "RS256");
        assert_eq!(result, Err(AuthError::DisallowedAlgorithm));
    }
}
