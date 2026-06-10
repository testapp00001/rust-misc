//! # Lesson 02: JWT Authentication Middleware
//!
//! ## Authentication vs Authorization
//!
//! - **Authentication**: "Who are you?" — Verify identity via JWT token
//! - **Authorization**: "What can you do?" — Check permissions (Lesson 03)
//!
//! This lesson focuses on authentication: validating JWTs and extracting claims.
//!
//! ## JWT Authentication Flow
//!
//! ```text
//! 1. Client sends request with header: Authorization: Bearer <token>
//! 2. Middleware extracts the token string
//! 3. Middleware splits the token into header.payload.signature
//! 4. Middleware verifies the HMAC-SHA256 signature
//! 5. Middleware checks expiration (exp > now)
//! 6. Middleware extracts claims (sub, roles, etc.)
//! 7. Middleware attaches claims to the request context
//! 8. Handler uses the authenticated user context
//! ```
//!
//! ## Attack Context
//!
//! - **Missing token**: Request has no Authorization header — reject with 401
//! - **Malformed token**: Token is not valid JWT format — reject with 401
//! - **Invalid signature**: Token was tampered with — reject with 401
//! - **Expired token**: Token has expired — reject with 401
//! - **Algorithm confusion**: Token claims `alg: none` — always enforce expected algorithm

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Standard JWT header.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

/// JWT claims extracted from a validated token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: u64,
    /// Issued-at time (Unix timestamp)
    pub iat: u64,
    /// User roles for authorization
    #[serde(default)]
    pub roles: Vec<String>,
    /// Optional issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
}

/// Represents an authenticated request context.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// The authenticated user's ID
    pub user_id: String,
    /// The user's roles
    pub roles: Vec<String>,
    /// When the token was issued
    pub issued_at: u64,
    /// When the token expires
    pub expires_at: u64,
}

/// Errors that can occur during authentication.
#[derive(Debug, Clone, PartialEq)]
pub enum AuthError {
    /// No Authorization header present
    MissingToken,
    /// Token is not in valid JWT format (not 3 dot-separated parts)
    MalformedToken,
    /// HMAC signature verification failed
    InvalidSignature,
    /// Token has expired
    TokenExpired,
    /// Token claims could not be deserialized
    InvalidClaims,
    /// Algorithm in token is not allowed
    DisallowedAlgorithm,
}

/// Exercise 1: Create a JWT token with the given claims.
///
/// Steps:
/// 1. Build header with `alg: "HS256"`, `typ: "JWT"`
/// 2. Serialize header and claims to JSON
/// 3. Base64URL-encode both (no padding)
/// 4. Compute HMAC-SHA256 of `base64(header).base64(claims)` using the secret
/// 5. Return the full token string: `header.payload.signature`
pub fn create_token(claims: &Claims, secret: &[u8]) -> String {
    todo!("Create a signed JWT token")
}

/// Exercise 2: Extract the token from an Authorization header value.
///
/// The header value should be in the format: "Bearer <token>"
/// Return the token string portion, or None if the header is missing/malformed.
///
/// Examples:
/// - "Bearer eyJhbG..." -> Some("eyJhbG...")
/// - "Basic dXNlcj..." -> None (not Bearer scheme)
/// - "" -> None
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    todo!("Extract Bearer token from Authorization header")
}

/// Exercise 3: Validate a JWT token and extract the authentication context.
///
/// Steps:
/// 1. Split the token by '.' into exactly 3 parts
/// 2. Decode the header from base64url and verify alg is "HS256"
/// 3. Recompute the HMAC-SHA256 signature and compare (constant-time via hmac crate)
/// 4. Decode the payload from base64url and deserialize into Claims
/// 5. Check that the token has not expired (exp > current_time)
/// 6. Return an AuthContext with the extracted information
///
/// The `expected_algorithm` parameter must match the header's alg field.
pub fn validate_token(
    token: &str,
    secret: &[u8],
    current_time: u64,
    expected_algorithm: &str,
) -> Result<AuthContext, AuthError> {
    todo!("Validate JWT token and extract claims")
}

/// Exercise 4: Create an authentication middleware function.
///
/// Given an Authorization header value, the HMAC secret, and the current time,
/// perform the full authentication flow:
/// 1. Extract the bearer token
/// 2. Validate the token
/// 3. Return the AuthContext or an AuthError
pub fn authenticate(
    auth_header: &str,
    secret: &[u8],
    current_time: u64,
) -> Result<AuthContext, AuthError> {
    todo!("Implement full authentication middleware")
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
        assert_eq!(parts.len(), 3, "JWT should have 3 parts");

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
        claims.exp = 1000000000; // Already expired
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
        // Tamper with the payload
        let mut parts: Vec<&str> = token.split('.').collect();
        parts[1] = "dGFtcGVyZWQ"; // tampered payload
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
