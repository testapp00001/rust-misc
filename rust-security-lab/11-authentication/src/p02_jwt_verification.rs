//! # Lesson 02: JWT Verification — Signature, Expiration, Claims Validation
//!
//! ## Verification Order Matters!
//!
//! A JWT must be verified in this specific order:
//!
//! 1. **Parse** the three parts (header, payload, signature)
//! 2. **Validate algorithm** — reject if not in your allowlist
//! 3. **Verify signature** — reject if invalid
//! 4. **Check expiration** — reject if `exp < now`
//! 5. **Validate claims** — issuer, audience, subject
//!
//! **NEVER** trust claims before verifying the signature. The whole point of
//! signing is to ensure claims haven't been tampered with.
//!
//! ## Attack Context
//!
//! Skipping signature verification is the most common JWT vulnerability.
//! An attacker can modify any claim (sub, role, exp) and the server accepts it.
//!
//! Also: always validate `alg` against an allowlist. If you accept whatever the
//! token says, the `alg=none` attack works (Lesson 03).

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

use super::p01_jwt_creation::{Jwt, JwtClaims, JwtHeader};

/// Errors that can occur during JWT verification.
#[derive(Debug, Clone, PartialEq)]
pub enum JwtError {
    /// Token has wrong number of parts
    MalformedToken,
    /// Base64 decoding failed
    InvalidEncoding(String),
    /// JSON parsing failed
    InvalidJson(String),
    /// Algorithm not in allowlist
    AlgorithmNotAllowed { got: String, allowed: Vec<String> },
    /// Signature verification failed
    InvalidSignature,
    /// Token has expired (exp < now)
    TokenExpired { exp: u64, now: u64 },
    /// Token is not yet valid (nbf > now, if present)
    NotYetValid { nbf: u64, now: u64 },
    /// Issuer mismatch
    IssuerMismatch { got: String, expected: String },
    /// Audience mismatch
    AudienceMismatch { got: String, expected: String },
}

/// Parse a JWT string into its three components without verifying.
///
/// This is a low-level function. Always verify after parsing!
pub fn parse_jwt(token: &str) -> Result<Jwt, JwtError> {
    todo!("Parse a JWT string into header, claims, and signature")
}

/// Verify the HMAC-SHA256 signature of a JWT.
///
/// This is the critical security check. Without it, anyone can forge tokens.
///
/// Steps:
/// 1. Parse the JWT
/// 2. Reconstruct the signing input: `base64url(header).base64url(claims)`
/// 3. Compute HMAC-SHA256 with the secret
/// 4. Compare with the token's signature (constant-time)
pub fn verify_signature(token: &str, secret: &[u8]) -> Result<Jwt, JwtError> {
    todo!("Verify the HMAC-SHA256 signature of a JWT")
}

/// Check if a JWT has expired.
pub fn check_expiration(claims: &JwtClaims, now: u64) -> Result<(), JwtError> {
    todo!("Check if the JWT has expired")
}

/// Verify a JWT completely: signature, algorithm, expiration, and claims.
///
/// This is the function you'd use in production.
pub fn verify_jwt(
    token: &str,
    secret: &[u8],
    allowed_algorithms: &[&str],
    expected_issuer: Option<&str>,
    expected_audience: Option<&str>,
    now: u64,
) -> Result<Jwt, JwtError> {
    todo!("Fully verify a JWT: signature + algorithm + expiry + claims")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p01_jwt_creation::{create_jwt, encode_jwt, JwtClaims};

    const TEST_SECRET: &[u8] = b"super-secret-key-at-least-32-bytes-long!!";

    fn sample_claims() -> JwtClaims {
        JwtClaims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1700000000,
            iss: Some("rust-security-lab".to_string()),
            aud: Some("api.example.com".to_string()),
        }
    }

    #[test]
    fn test_parse_valid_jwt() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let token = encode_jwt(&jwt);
        let parsed = parse_jwt(&token).unwrap();
        assert_eq!(parsed.header.alg, "HS256");
        assert_eq!(parsed.claims.sub, "user123");
    }

    #[test]
    fn test_parse_malformed_token() {
        assert_eq!(parse_jwt("not.a.valid.token.parts"), Err(JwtError::MalformedToken));
        assert_eq!(parse_jwt("only-two"), Err(JwtError::MalformedToken));
        assert_eq!(parse_jwt(""), Err(JwtError::MalformedToken));
    }

    #[test]
    fn test_verify_signature_valid() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let token = encode_jwt(&jwt);
        let result = verify_signature(&token, TEST_SECRET);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_signature_wrong_secret() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let token = encode_jwt(&jwt);
        let result = verify_signature(&token, b"wrong-secret-key-32-bytes-padding!");
        assert_eq!(result, Err(JwtError::InvalidSignature));
    }

    #[test]
    fn test_verify_signature_tampered_payload() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let token = encode_jwt(&jwt);
        // Tamper with the payload (change last byte so JSON stays parseable)
        let parts: Vec<&str> = token.split('.').collect();
        let mut tampered_claims = URL_SAFE_NO_PAD.decode(parts[1]).unwrap();
        let last = tampered_claims.len() - 1;
        tampered_claims[last] ^= 0xFF; // flip bits in last byte
        let tampered = format!(
            "{}.{}.{}",
            parts[0],
            URL_SAFE_NO_PAD.encode(&tampered_claims),
            parts[2]
        );
        let result = verify_signature(&tampered, TEST_SECRET);
        assert!(result.is_err(), "Tampered payload should fail verification");
    }

    #[test]
    fn test_check_expiration_valid() {
        let claims = sample_claims();
        assert!(check_expiration(&claims, 1700000000).is_ok());
    }

    #[test]
    fn test_check_expiration_expired() {
        let claims = sample_claims();
        let result = check_expiration(&claims, 2000000001);
        assert!(matches!(result, Err(JwtError::TokenExpired { .. })));
    }

    #[test]
    fn test_verify_jwt_full() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let token = encode_jwt(&jwt);
        let result = verify_jwt(
            &token,
            TEST_SECRET,
            &["HS256"],
            Some("rust-security-lab"),
            Some("api.example.com"),
            1700000000,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_jwt_wrong_algorithm() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let token = encode_jwt(&jwt);
        let result = verify_jwt(
            &token,
            TEST_SECRET,
            &["RS256"], // only allow RS256
            None,
            None,
            1700000000,
        );
        assert!(matches!(result, Err(JwtError::AlgorithmNotAllowed { .. })));
    }

    #[test]
    fn test_verify_jwt_issuer_mismatch() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let token = encode_jwt(&jwt);
        let result = verify_jwt(
            &token,
            TEST_SECRET,
            &["HS256"],
            Some("evil-attacker.com"),
            None,
            1700000000,
        );
        assert!(matches!(result, Err(JwtError::IssuerMismatch { .. })));
    }
}
