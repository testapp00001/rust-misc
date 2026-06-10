//! # Lesson 01: JWT Creation — Claims, Expiration, HMAC-SHA256 Signing
//!
//! ## What is a JWT?
//!
//! A JSON Web Token (JWT) is a compact, URL-safe token format for securely transmitting
//! information between parties. A JWT has three parts separated by dots:
//!
//! ```text
//! header.payload.signature
//! ```
//!
//! Each part is Base64URL-encoded.
//!
//! **Header**: `{"alg": "HS256", "typ": "JWT"}`
//! **Payload (Claims)**: `{"sub": "user123", "exp": 1700000000, "iat": 1699996400}`
//! **Signature**: `HMACSHA256(base64url(header) + "." + base64url(payload), secret)`
//!
//! ## Why HMAC-SHA256?
//!
//! HMAC-SHA256 (HS256) is the simplest JWT signing algorithm:
//! - Symmetric: same key signs and verifies
//! - Fast: ~10x faster than RSA
//! - Secure: if the secret is strong (256+ bits random)
//!
//! For asymmetric needs (public verification), use RS256 or ES256 instead.
//!
//! ## Attack Context
//!
//! A JWT's security depends entirely on:
//! 1. **Secret strength** — weak secrets can be brute-forced (Lesson 03)
//! 2. **Algorithm validation** — `alg=none` bypass (Lesson 03)
//! 3. **Claim verification** — expired/invalid claims must be rejected (Lesson 02)

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Standard JWT header fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

/// Standard JWT claims (payload).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JwtClaims {
    /// Subject — who this token is about (typically user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp, seconds)
    pub exp: u64,
    /// Issued-at time (Unix timestamp, seconds)
    pub iat: u64,
    /// Optional: issuer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// Optional: audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
}

/// A parsed JWT with its three components.
#[derive(Debug, Clone, PartialEq)]
pub struct Jwt {
    pub header: JwtHeader,
    pub claims: JwtClaims,
    pub signature: Vec<u8>,
}

impl Jwt {
    /// Serialize the header to JSON bytes.
    pub fn header_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self.header).expect("header should serialize")
    }

    /// Serialize the claims to JSON bytes.
    pub fn claims_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self.claims).expect("claims should serialize")
    }

    /// The signing input: base64url(header) + "." + base64url(payload)
    pub fn signing_input(&self) -> String {
        let header_b64 = URL_SAFE_NO_PAD.encode(self.header_json());
        let claims_b64 = URL_SAFE_NO_PAD.encode(self.claims_json());
        format!("{}.{}", header_b64, claims_b64)
    }

    /// Encode the full JWT string: header.payload.signature
    pub fn encode(&self) -> String {
        let sig_b64 = URL_SAFE_NO_PAD.encode(&self.signature);
        format!("{}.{}", self.signing_input(), sig_b64)
    }
}

/// Create a JWT with the given claims, signed with HMAC-SHA256.
///
/// Steps:
/// 1. Build header with `alg: "HS256"`, `typ: "JWT"`
/// 2. Serialize header and claims to JSON
/// 3. Base64URL-encode both (no padding)
/// 4. Compute HMAC-SHA256 of `base64(header).base64(claims)` using the secret
/// 5. Return the JWT struct with signature attached
pub fn create_jwt(claims: JwtClaims, secret: &[u8]) -> Jwt {
    let header = JwtHeader {
        alg: "HS256".to_string(),
        typ: "JWT".to_string(),
    };

    let jwt = Jwt {
        header,
        claims,
        signature: Vec::new(), // placeholder
    };

    let signing_input = jwt.signing_input();

    let mut mac =
        HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(signing_input.as_bytes());
    let signature = mac.finalize().into_bytes().to_vec();

    Jwt {
        header: jwt.header,
        claims: jwt.claims,
        signature,
    }
}

/// Encode a JWT into its compact string representation.
///
/// Format: `base64url(header).base64url(payload).base64url(signature)`
pub fn encode_jwt(jwt: &Jwt) -> String {
    jwt.encode()
}

/// Base64URL-encode bytes (no padding).
pub fn base64url_encode(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

/// Base64URL-decode a string back to bytes.
pub fn base64url_decode(encoded: &str) -> Result<Vec<u8>, String> {
    URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|e| format!("Base64URL decode error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"super-secret-key-at-least-32-bytes-long!!";

    fn sample_claims() -> JwtClaims {
        JwtClaims {
            sub: "user123".to_string(),
            exp: 2000000000,
            iat: 1700000000,
            iss: Some("rust-security-lab".to_string()),
            aud: None,
        }
    }

    #[test]
    fn test_create_jwt_returns_valid_structure() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        assert_eq!(jwt.header.alg, "HS256");
        assert_eq!(jwt.header.typ, "JWT");
        assert_eq!(jwt.claims.sub, "user123");
        assert_eq!(jwt.signature.len(), 32, "HMAC-SHA256 signature is 32 bytes");
    }

    #[test]
    fn test_jwt_encode_format() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let encoded = encode_jwt(&jwt);
        let parts: Vec<&str> = encoded.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have 3 dot-separated parts");
        // Each part should be valid base64url
        assert!(base64url_decode(parts[0]).is_ok());
        assert!(base64url_decode(parts[1]).is_ok());
        assert!(base64url_decode(parts[2]).is_ok());
    }

    #[test]
    fn test_jwt_header_roundtrip() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let header_json = jwt.header_json();
        let decoded: JwtHeader = serde_json::from_slice(&header_json).unwrap();
        assert_eq!(decoded, jwt.header);
    }

    #[test]
    fn test_jwt_claims_roundtrip() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let claims_json = jwt.claims_json();
        let decoded: JwtClaims = serde_json::from_slice(&claims_json).unwrap();
        assert_eq!(decoded, jwt.claims);
    }

    #[test]
    fn test_jwt_deterministic() {
        let jwt1 = create_jwt(sample_claims(), TEST_SECRET);
        let jwt2 = create_jwt(sample_claims(), TEST_SECRET);
        assert_eq!(jwt1.signature, jwt2.signature, "Same input should produce same JWT");
        assert_eq!(encode_jwt(&jwt1), encode_jwt(&jwt2));
    }

    #[test]
    fn test_jwt_different_secrets_produce_different_signatures() {
        let jwt1 = create_jwt(sample_claims(), b"secret-one-32-bytes-padding-here!");
        let jwt2 = create_jwt(sample_claims(), b"different-key-32-bytes-padding-ok!");
        assert_ne!(jwt1.signature, jwt2.signature);
    }

    #[test]
    fn test_jwt_different_claims_produce_different_signatures() {
        let mut claims2 = sample_claims();
        claims2.sub = "user456".to_string();
        let jwt1 = create_jwt(sample_claims(), TEST_SECRET);
        let jwt2 = create_jwt(claims2, TEST_SECRET);
        assert_ne!(jwt1.signature, jwt2.signature);
    }

    #[test]
    fn test_base64url_encode_decode_roundtrip() {
        let data = b"hello world! This is a test payload for JWT.";
        let encoded = base64url_encode(data);
        // Base64URL should not contain '+' '/' or '='
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert!(!encoded.contains('='));
        let decoded = base64url_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_signing_input_consistency() {
        let jwt = create_jwt(sample_claims(), TEST_SECRET);
        let input = jwt.signing_input();
        // Should be base64url(header).base64url(claims)
        let parts: Vec<&str> = input.split('.').collect();
        assert_eq!(parts.len(), 2);
        let header_decoded = base64url_decode(parts[0]).unwrap();
        let header: JwtHeader = serde_json::from_slice(&header_decoded).unwrap();
        assert_eq!(header.alg, "HS256");
    }
}
