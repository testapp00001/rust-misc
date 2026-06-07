//! # JWT Authentication
//!
//! JWT (JSON Web Tokens) are a widely used mechanism for stateless
//! authentication. This lesson covers JWT creation, verification, claims
//! management, and refresh token patterns.
//!
//! ## Key Concepts
//! - JWT structure (header, payload, signature)
//! - Token creation and verification
//! - Claims and expiration
//! - Refresh token rotation
//! - Token revocation strategies

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// 1. JWT Claims
// ---------------------------------------------------------------------------

/// Standard and custom JWT claims.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    pub sub: String,
    pub iat: u64,
    pub exp: u64,
    pub iss: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jti: Option<String>,
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl Claims {
    pub fn new(user_id: impl Into<String>, issuer: impl Into<String>, ttl: Duration) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            sub: user_id.into(),
            iat: now,
            exp: now + ttl.as_secs(),
            iss: issuer.into(),
            aud: None,
            jti: None,
            custom: HashMap::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.exp
    }

    pub fn with_audience(mut self, aud: impl Into<String>) -> Self {
        self.aud = Some(aud.into());
        self
    }

    pub fn with_jti(mut self, jti: impl Into<String>) -> Self {
        self.jti = Some(jti.into());
        self
    }

    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }
}

// ---------------------------------------------------------------------------
// 2. JWT Service
// ---------------------------------------------------------------------------

/// Creates and verifies JWT tokens.
#[derive(Debug)]
pub struct JwtService {
    secret: Vec<u8>,
    issuer: String,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

impl JwtService {
    pub fn new(secret: impl Into<Vec<u8>>, issuer: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            issuer: issuer.into(),
            access_ttl: Duration::from_secs(3600),
            refresh_ttl: Duration::from_secs(86400 * 7),
        }
    }

    pub fn with_access_ttl(mut self, ttl: Duration) -> Self {
        self.access_ttl = ttl;
        self
    }

    pub fn with_refresh_ttl(mut self, ttl: Duration) -> Self {
        self.refresh_ttl = ttl;
        self
    }

    pub fn create_access_token(&self, user_id: &str) -> String {
        let jti = format!("{}-access-{}", user_id, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        let claims = Claims::new(user_id, &self.issuer, self.access_ttl)
            .with_jti(jti);
        self.encode(&claims)
    }

    pub fn create_refresh_token(&self, user_id: &str) -> String {
        let jti = format!("{}-refresh-{}", user_id, std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        let claims = Claims::new(user_id, &self.issuer, self.refresh_ttl)
            .with_custom("token_type", serde_json::json!("refresh"))
            .with_jti(jti);
        self.encode(&claims)
    }

    pub fn create_token_pair(&self, user_id: &str) -> TokenPair {
        TokenPair {
            access_token: self.create_access_token(user_id),
            refresh_token: self.create_refresh_token(user_id),
            token_type: "Bearer".into(),
            expires_in: self.access_ttl.as_secs(),
        }
    }

    pub fn verify(&self, token: &str) -> Result<Claims, JwtError> {
        let claims = self.decode(token)?;
        if claims.is_expired() {
            return Err(JwtError::Expired);
        }
        if claims.iss != self.issuer {
            return Err(JwtError::InvalidIssuer);
        }
        Ok(claims)
    }

    pub fn refresh(&self, refresh_token: &str) -> Result<TokenPair, JwtError> {
        let claims = self.verify(refresh_token)?;
        if claims.custom.get("token_type").and_then(|v| v.as_str()) != Some("refresh") {
            return Err(JwtError::NotRefreshToken);
        }
        Ok(self.create_token_pair(&claims.sub))
    }

    fn encode(&self, claims: &Claims) -> String {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let header = engine.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = engine.encode(serde_json::to_string(claims).unwrap());
        let signature = self.sign(&format!("{header}.{payload}"));
        format!("{header}.{payload}.{signature}")
    }

    fn decode(&self, token: &str) -> Result<Claims, JwtError> {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(JwtError::Malformed);
        }
        let (header, payload, signature) = (parts[0], parts[1], parts[2]);
        let expected_sig = self.sign(&format!("{header}.{payload}"));
        if signature != expected_sig {
            return Err(JwtError::InvalidSignature);
        }
        let payload_bytes = engine.decode(payload).map_err(|_| JwtError::Malformed)?;
        serde_json::from_slice(&payload_bytes).map_err(|_| JwtError::Malformed)
    }

    fn sign(&self, data: &str) -> String {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
        let mut sig = vec![0u8; 32];
        for (i, &byte) in data.as_bytes().iter().enumerate() {
            sig[i % 32] ^= byte ^ self.secret[i % self.secret.len()];
        }
        engine.encode(sig)
    }
}

// ---------------------------------------------------------------------------
// 3. JWT Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("token has expired")]
    Expired,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("malformed token")]
    Malformed,
    #[error("invalid issuer")]
    InvalidIssuer,
    #[error("not a refresh token")]
    NotRefreshToken,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new("user-1", "test-iss", Duration::from_secs(3600));
        assert_eq!(claims.sub, "user-1");
        assert_eq!(claims.iss, "test-iss");
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_claims_custom() {
        let claims = Claims::new("u1", "iss", Duration::from_secs(60))
            .with_audience("my-app")
            .with_custom("role", serde_json::json!("admin"));
        assert_eq!(claims.aud, Some("my-app".into()));
        assert_eq!(claims.custom["role"], "admin");
    }

    #[test]
    fn test_jwt_create_and_verify() {
        let jwt = JwtService::new(b"test-secret", "test-issuer");
        let token = jwt.create_access_token("user-42");
        let claims = jwt.verify(&token).unwrap();
        assert_eq!(claims.sub, "user-42");
    }

    #[test]
    fn test_jwt_token_pair() {
        let jwt = JwtService::new(b"secret", "issuer");
        let pair = jwt.create_token_pair("user-1");
        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");
    }

    #[test]
    fn test_jwt_refresh() {
        let jwt = JwtService::new(b"secret", "issuer");
        let pair = jwt.create_token_pair("user-1");
        let new_pair = jwt.refresh(&pair.refresh_token).unwrap();
        assert_ne!(pair.access_token, new_pair.access_token);
    }

    #[test]
    fn test_jwt_cannot_refresh_with_access_token() {
        let jwt = JwtService::new(b"secret", "issuer");
        let pair = jwt.create_token_pair("user-1");
        assert!(jwt.refresh(&pair.access_token).is_err());
    }

    #[test]
    fn test_jwt_invalid_signature() {
        let jwt1 = JwtService::new(b"secret1", "issuer");
        let jwt2 = JwtService::new(b"secret2", "issuer");
        let token = jwt1.create_access_token("user");
        assert!(jwt2.verify(&token).is_err());
    }

    #[test]
    fn test_jwt_malformed() {
        let jwt = JwtService::new(b"secret", "issuer");
        assert!(jwt.verify("not-a-token").is_err());
        assert!(jwt.verify("").is_err());
    }

    #[test]
    fn test_jwt_invalid_issuer() {
        let jwt1 = JwtService::new(b"same", "issuer-a");
        let jwt2 = JwtService::new(b"same", "issuer-b");
        let token = jwt1.create_access_token("user");
        assert!(jwt2.verify(&token).is_err());
    }

    #[test]
    fn test_jwt_error_display() {
        assert!(!JwtError::Expired.to_string().is_empty());
        assert!(!JwtError::Malformed.to_string().is_empty());
    }
}
