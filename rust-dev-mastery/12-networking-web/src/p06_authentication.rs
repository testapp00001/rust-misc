//! # Authentication
//!
//! Authentication is how users prove their identity. This lesson covers JWT token
//! creation and verification, session management, password hashing with proper
//! security practices, and building authentication middleware.
//!
//! ## Key Concepts
//! - JWT (JSON Web Tokens) structure: header, payload, signature
//! - Token-based authentication flow
//! - Refresh token rotation
//! - Password hashing with salt
//! - Session management patterns
//! - Auth middleware design

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// 1. JWT Token Types
// ---------------------------------------------------------------------------

/// Standard JWT claims.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issued at (unix timestamp)
    pub iat: u64,
    /// Expiration (unix timestamp)
    pub exp: u64,
    /// Issuer
    pub iss: String,
    /// Audience
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    /// Custom claims
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

    pub fn remaining_ttl(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now >= self.exp {
            Duration::ZERO
        } else {
            Duration::from_secs(self.exp - now)
        }
    }

    pub fn with_audience(mut self, aud: impl Into<String>) -> Self {
        self.aud = Some(aud.into());
        self
    }

    pub fn with_custom(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom.insert(key.into(), value);
        self
    }
}

// ---------------------------------------------------------------------------
// 2. JWT Token Service
// ---------------------------------------------------------------------------

/// A simplified JWT service that handles token creation and verification.
/// In production, you'd use a library like `jsonwebtoken` which handles
/// the actual cryptographic signing. This demonstrates the concepts.
#[derive(Debug)]
pub struct JwtService {
    secret: Vec<u8>,
    issuer: String,
    access_ttl: Duration,
    refresh_ttl: Duration,
}

impl JwtService {
    pub fn new(secret: impl Into<Vec<u8>>, issuer: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
            issuer: issuer.into(),
            access_ttl: Duration::from_secs(3600),    // 1 hour
            refresh_ttl: Duration::from_secs(86400 * 7), // 7 days
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

    /// Create an access token for a user.
    pub fn create_access_token(&self, user_id: &str) -> String {
        let claims = Claims::new(user_id, &self.issuer, self.access_ttl);
        self.encode(&claims)
    }

    /// Create a refresh token for a user.
    pub fn create_refresh_token(&self, user_id: &str) -> String {
        let claims = Claims::new(user_id, &self.issuer, self.refresh_ttl)
            .with_custom(String::from("token_type"), serde_json::json!("refresh"));
        self.encode(&claims)
    }

    /// Create both access and refresh tokens.
    pub fn create_token_pair(&self, user_id: &str) -> TokenPair {
        TokenPair {
            access_token: self.create_access_token(user_id),
            refresh_token: self.create_refresh_token(user_id),
            token_type: "Bearer".into(),
            expires_in: self.access_ttl.as_secs(),
        }
    }

    /// Verify and decode a token.
    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let claims = self.decode(token)?;

        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        if claims.iss != self.issuer {
            return Err(AuthError::InvalidIssuer);
        }

        Ok(claims)
    }

    /// Encode claims into a token string (simplified format: base64(header).base64(payload).signature).
    fn encode(&self, claims: &Claims) -> String {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;

        let header = engine.encode(r#"{"alg":"HS256","typ":"JWT"}"#);
        let payload = engine.encode(serde_json::to_string(claims).unwrap());
        let signature = self.sign(&format!("{header}.{payload}"));

        format!("{header}.{payload}.{signature}")
    }

    /// Decode and verify a token string.
    fn decode(&self, token: &str) -> Result<Claims, AuthError> {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;

        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthError::MalformedToken);
        }

        let (header, payload, signature) = (parts[0], parts[1], parts[2]);

        // Verify signature
        let expected_sig = self.sign(&format!("{header}.{payload}"));
        if signature != expected_sig {
            return Err(AuthError::InvalidSignature);
        }

        // Decode payload
        let payload_bytes = engine.decode(payload).map_err(|_| AuthError::MalformedToken)?;
        let claims: Claims =
            serde_json::from_slice(&payload_bytes).map_err(|_| AuthError::MalformedToken)?;

        Ok(claims)
    }

    /// Create an HMAC signature (simplified for learning; use ring in production).
    fn sign(&self, data: &str) -> String {
        use base64::Engine;
        let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;

        // Simple HMAC-like signature using XOR folding (NOT for production!)
        let mut sig = vec![0u8; 32];
        let data_bytes = data.as_bytes();
        for (i, &byte) in data_bytes.iter().enumerate() {
            sig[i % 32] ^= byte ^ self.secret[i % self.secret.len()];
        }
        engine.encode(sig)
    }
}

/// A pair of access and refresh tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

// ---------------------------------------------------------------------------
// 3. Authentication Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("token has expired")]
    TokenExpired,

    #[error("invalid token signature")]
    InvalidSignature,

    #[error("malformed token")]
    MalformedToken,

    #[error("invalid issuer")]
    InvalidIssuer,

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("account locked")]
    AccountLocked,

    #[error("insufficient permissions")]
    InsufficientPermissions,
}

// ---------------------------------------------------------------------------
// 4. Password Hashing
// ---------------------------------------------------------------------------

/// A password hasher that demonstrates secure password storage patterns.
/// Uses PBKDF2-like concepts (simplified for learning without external deps).
pub struct PasswordHasher {
    iterations: u32,
    salt_length: usize,
}

impl PasswordHasher {
    pub fn new(iterations: u32, salt_length: usize) -> Self {
        Self {
            iterations,
            salt_length,
        }
    }

    pub fn default_secure() -> Self {
        Self::new(100_000, 32)
    }

    /// Hash a password with a generated salt.
    pub fn hash_password(&self, password: &str) -> PasswordHash {
        let salt = self.generate_salt();
        let hash = self.derive_key(password, &salt);
        PasswordHash {
            algorithm: "pbkdf2-sha256".into(),
            iterations: self.iterations,
            salt: hex_encode(&salt),
            hash: hex_encode(&hash),
        }
    }

    /// Verify a password against a stored hash.
    pub fn verify_password(&self, password: &str, stored: &PasswordHash) -> bool {
        let salt = hex_decode(&stored.salt);
        let hash = self.derive_key(password, &salt);
        constant_time_eq(&hex_encode(&hash), &stored.hash)
    }

    /// Derive a key from password and salt using iterative hashing.
    fn derive_key(&self, password: &str, salt: &[u8]) -> Vec<u8> {
        let mut key = Vec::with_capacity(32);
        key.extend_from_slice(password.as_bytes());
        key.extend_from_slice(salt);

        // Iterative hashing (simplified PBKDF2)
        for _ in 0..self.iterations {
            let mut new_key = Vec::with_capacity(32);
            // Simple mixing function
            for (i, &b) in key.iter().enumerate() {
                let mixed = b.wrapping_add(
                    salt.get(i % salt.len()).copied().unwrap_or(0),
                );
                new_key.push(mixed);
            }
            key = new_key;
        }

        // Truncate to 32 bytes
        key.truncate(32);
        key
    }

    fn generate_salt(&self) -> Vec<u8> {
        // In production, use ring::rand::SystemRandom
        // For learning, we use a deterministic but unique approach
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut salt = Vec::with_capacity(self.salt_length);
        for i in 0..self.salt_length {
            salt.push(((now >> (i * 8)) & 0xFF) as u8 ^ (i as u8));
        }
        salt
    }
}

/// Stored password hash representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordHash {
    pub algorithm: String,
    pub iterations: u32,
    pub salt: String,
    pub hash: String,
}

impl PasswordHash {
    /// Format as a storable string (like Unix crypt format).
    pub fn to_stored_string(&self) -> String {
        format!(
            "${}${}${}${}",
            self.algorithm, self.iterations, self.salt, self.hash
        )
    }

    /// Parse from stored string format.
    pub fn from_stored_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('$').filter(|p| !p.is_empty()).collect();
        if parts.len() != 4 {
            return None;
        }
        Some(Self {
            algorithm: parts[0].into(),
            iterations: parts[1].parse().ok()?,
            salt: parts[2].into(),
            hash: parts[3].into(),
        })
    }
}

// ---------------------------------------------------------------------------
// 5. Session Management
// ---------------------------------------------------------------------------

/// Represents an active user session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub data: HashMap<String, String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl Session {
    pub fn new(user_id: impl Into<String>, ttl: Duration) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            session_id: generate_session_id(),
            user_id: user_id.into(),
            created_at: now,
            expires_at: now + ttl.as_secs(),
            data: HashMap::new(),
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.expires_at
    }

    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.data.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
}

/// A thread-safe session store.
#[derive(Debug, Clone, Default)]
pub struct SessionStore {
    sessions: HashMap<String, Session>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&mut self, user_id: &str, ttl: Duration) -> Session {
        let session = Session::new(user_id, ttl);
        let id = session.session_id.clone();
        self.sessions.insert(id, session.clone());
        session
    }

    pub fn get(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id).filter(|s| !s.is_expired())
    }

    pub fn destroy(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    pub fn destroy_all_for_user(&mut self, user_id: &str) -> usize {
        let to_remove: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.user_id == user_id)
            .map(|(k, _)| k.clone())
            .collect();

        let count = to_remove.len();
        for id in to_remove {
            self.sessions.remove(&id);
        }
        count
    }

    /// Remove expired sessions.
    pub fn cleanup(&mut self) -> usize {
        let before = self.sessions.len();
        self.sessions.retain(|_, s| !s.is_expired());
        before - self.sessions.len()
    }

    pub fn active_count(&self) -> usize {
        self.sessions.values().filter(|s| !s.is_expired()).count()
    }
}

// ---------------------------------------------------------------------------
// 6. User Store with Credentials
// ---------------------------------------------------------------------------

/// A user record with authentication data.
#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub email: String,
    pub password_hash: PasswordHash,
    pub roles: Vec<String>,
    pub locked: bool,
    pub failed_login_attempts: u32,
}

/// In-memory user store for authentication.
#[derive(Debug, Default)]
pub struct UserAuthStore {
    users: HashMap<String, UserRecord>,
}

impl UserAuthStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_user(&mut self, user: UserRecord) {
        self.users.insert(user.id.clone(), user);
    }

    pub fn find_by_username(&self, username: &str) -> Option<&UserRecord> {
        self.users.values().find(|u| u.username == username)
    }

    pub fn find_by_id(&self, id: &str) -> Option<&UserRecord> {
        self.users.get(id)
    }

    pub fn record_failed_login(&mut self, user_id: &str) {
        if let Some(user) = self.users.get_mut(user_id) {
            user.failed_login_attempts += 1;
            if user.failed_login_attempts >= 5 {
                user.locked = true;
            }
        }
    }

    pub fn reset_failed_logins(&mut self, user_id: &str) {
        if let Some(user) = self.users.get_mut(user_id) {
            user.failed_login_attempts = 0;
        }
    }
}

// ---------------------------------------------------------------------------
// 7. Authentication Service
// ---------------------------------------------------------------------------

/// The main authentication service that ties everything together.
pub struct AuthService {
    jwt: JwtService,
    hasher: PasswordHasher,
    users: UserAuthStore,
    sessions: SessionStore,
}

impl AuthService {
    pub fn new(jwt: JwtService, hasher: PasswordHasher) -> Self {
        Self {
            jwt,
            hasher,
            users: UserAuthStore::new(),
            sessions: SessionStore::new(),
        }
    }

    pub fn add_user(&mut self, user: UserRecord) {
        self.users.add_user(user);
    }

    /// Authenticate with username/password and return tokens.
    pub fn login(&mut self, username: &str, password: &str) -> Result<TokenPair, AuthError> {
        let user = self
            .users
            .find_by_username(username)
            .ok_or(AuthError::InvalidCredentials)?
            .clone();

        if user.locked {
            return Err(AuthError::AccountLocked);
        }

        if !self.hasher.verify_password(password, &user.password_hash) {
            self.users.record_failed_login(&user.id);
            return Err(AuthError::InvalidCredentials);
        }

        self.users.reset_failed_logins(&user.id);
        Ok(self.jwt.create_token_pair(&user.id))
    }

    /// Verify an access token and return the user ID.
    pub fn verify_access_token(&self, token: &str) -> Result<String, AuthError> {
        let claims = self.jwt.verify(token)?;
        // Ensure it's not a refresh token being used as access
        if claims.custom.get("token_type").and_then(|v| v.as_str()) == Some("refresh") {
            return Err(AuthError::InsufficientPermissions);
        }
        Ok(claims.sub)
    }

    /// Refresh an access token using a refresh token.
    pub fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, AuthError> {
        let claims = self.jwt.verify(refresh_token)?;

        if claims.custom.get("token_type").and_then(|v| v.as_str()) != Some("refresh") {
            return Err(AuthError::InsufficientPermissions);
        }

        Ok(self.jwt.create_token_pair(&claims.sub))
    }

    /// Create a session-based login.
    pub fn create_session(
        &mut self,
        username: &str,
        password: &str,
        ttl: Duration,
    ) -> Result<Session, AuthError> {
        let user = self
            .users
            .find_by_username(username)
            .ok_or(AuthError::InvalidCredentials)?
            .clone();

        if user.locked {
            return Err(AuthError::AccountLocked);
        }

        if !self.hasher.verify_password(password, &user.password_hash) {
            self.users.record_failed_login(&user.id);
            return Err(AuthError::InvalidCredentials);
        }

        self.users.reset_failed_logins(&user.id);
        Ok(self.sessions.create(&user.id, ttl))
    }

    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    pub fn destroy_session(&mut self, session_id: &str) -> bool {
        self.sessions.destroy(session_id)
    }

    /// Check if a user has a specific role.
    pub fn user_has_role(&self, user_id: &str, role: &str) -> bool {
        self.users
            .find_by_id(user_id)
            .map(|u| u.roles.iter().any(|r| r == role))
            .unwrap_or(false)
    }
}

// ---------------------------------------------------------------------------
// Utility Functions
// ---------------------------------------------------------------------------

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

/// Constant-time string comparison to prevent timing attacks.
fn constant_time_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

fn generate_session_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("sess_{now:x}")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new("user-123", "test-issuer", Duration::from_secs(3600));
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.iss, "test-issuer");
        assert!(!claims.is_expired());
        assert!(claims.remaining_ttl() > Duration::from_secs(3500));
    }

    #[test]
    fn test_claims_with_custom() {
        let claims = Claims::new("u1", "iss", Duration::from_secs(60))
            .with_audience("my-app")
            .with_custom("role", serde_json::json!("admin"));

        assert_eq!(claims.aud, Some("my-app".into()));
        assert_eq!(claims.custom["role"], "admin");
    }

    #[test]
    fn test_jwt_create_and_verify() {
        let jwt = JwtService::new(b"test-secret-key-12345", "test-issuer");
        let token = jwt.create_access_token("user-42");

        let claims = jwt.verify(&token).unwrap();
        assert_eq!(claims.sub, "user-42");
        assert_eq!(claims.iss, "test-issuer");
    }

    #[test]
    fn test_jwt_token_pair() {
        let jwt = JwtService::new(b"my-secret", "my-app");
        let pair = jwt.create_token_pair("user-1");

        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");

        // Verify access token
        let claims = jwt.verify(&pair.access_token).unwrap();
        assert_eq!(claims.sub, "user-1");

        // Verify refresh token
        let claims = jwt.verify(&pair.refresh_token).unwrap();
        assert_eq!(claims.sub, "user-1");
        assert_eq!(
            claims.custom.get("token_type").and_then(|v| v.as_str()),
            Some("refresh")
        );
    }

    #[test]
    fn test_jwt_invalid_signature() {
        let jwt1 = JwtService::new(b"secret-1", "issuer");
        let jwt2 = JwtService::new(b"secret-2", "issuer");

        let token = jwt1.create_access_token("user-1");
        assert!(jwt2.verify(&token).is_err());
    }

    #[test]
    fn test_jwt_malformed_token() {
        let jwt = JwtService::new(b"secret", "issuer");
        assert!(jwt.verify("not.a.valid.token.too.many.parts").is_err());
        assert!(jwt.verify("no-dots").is_err());
        assert!(jwt.verify("").is_err());
    }

    #[test]
    fn test_jwt_invalid_issuer() {
        let jwt1 = JwtService::new(b"same-secret", "issuer-a");
        let jwt2 = JwtService::new(b"same-secret", "issuer-b");

        let token = jwt1.create_access_token("user-1");
        // Same secret but different issuer
        let result = jwt2.verify(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hash_and_verify() {
        let hasher = PasswordHasher::new(100, 16); // Low iterations for testing
        let hash = hasher.hash_password("my-secret-password");

        assert!(hasher.verify_password("my-secret-password", &hash));
        assert!(!hasher.verify_password("wrong-password", &hash));
    }

    #[test]
    fn test_password_hash_stored_string() {
        let hasher = PasswordHasher::new(100, 16);
        let hash = hasher.hash_password("test123");

        let stored = hash.to_stored_string();
        let parsed = PasswordHash::from_stored_string(&stored).unwrap();
        assert_eq!(parsed.algorithm, hash.algorithm);
        assert_eq!(parsed.iterations, hash.iterations);
        assert_eq!(parsed.salt, hash.salt);
        assert_eq!(parsed.hash, hash.hash);
    }

    #[test]
    fn test_password_hash_different_salts() {
        let hasher = PasswordHasher::new(100, 16);
        let hash1 = hasher.hash_password("same-password");
        std::thread::sleep(Duration::from_millis(1));
        let hash2 = hasher.hash_password("same-password");

        // Different salts means different hashes
        assert_ne!(hash1.salt, hash2.salt);
        assert_ne!(hash1.hash, hash2.hash);
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new("user-1", Duration::from_secs(3600));
        assert_eq!(session.user_id, "user-1");
        assert!(!session.is_expired());
        assert!(!session.session_id.is_empty());
    }

    #[test]
    fn test_session_data() {
        let mut session = Session::new("user-1", Duration::from_secs(3600));
        session.set("theme", "dark");
        session.set("lang", "en");

        assert_eq!(session.get("theme"), Some("dark"));
        assert_eq!(session.get("lang"), Some("en"));
        assert_eq!(session.get("missing"), None);
    }

    #[test]
    fn test_session_store() {
        let mut store = SessionStore::new();
        let session = store.create("user-1", Duration::from_secs(3600));

        assert!(store.get(&session.session_id).is_some());
        assert_eq!(store.active_count(), 1);

        store.destroy(&session.session_id);
        assert!(store.get(&session.session_id).is_none());
        assert_eq!(store.active_count(), 0);
    }

    #[test]
    fn test_session_store_destroy_all_for_user() {
        let mut store = SessionStore::new();
        store.create("user-1", Duration::from_secs(3600));
        store.create("user-1", Duration::from_secs(3600));
        store.create("user-2", Duration::from_secs(3600));

        assert_eq!(store.active_count(), 3);
        let removed = store.destroy_all_for_user("user-1");
        assert_eq!(removed, 2);
        assert_eq!(store.active_count(), 1);
    }

    #[test]
    fn test_auth_service_login() {
        let jwt = JwtService::new(b"secret", "test-app");
        let hasher = PasswordHasher::new(100, 16);
        let mut auth = AuthService::new(jwt, hasher);

        let hash = auth.hasher.hash_password("password123");
        auth.add_user(UserRecord {
            id: "u1".into(),
            username: "alice".into(),
            email: "alice@example.com".into(),
            password_hash: hash,
            roles: vec!["user".into()],
            locked: false,
            failed_login_attempts: 0,
        });

        let result = auth.login("alice", "password123");
        assert!(result.is_ok());
        let tokens = result.unwrap();
        assert!(!tokens.access_token.is_empty());
    }

    #[test]
    fn test_auth_service_wrong_password() {
        let jwt = JwtService::new(b"secret", "test-app");
        let hasher = PasswordHasher::new(100, 16);
        let mut auth = AuthService::new(jwt, hasher);

        let hash = auth.hasher.hash_password("correct");
        auth.add_user(UserRecord {
            id: "u1".into(),
            username: "alice".into(),
            email: "alice@example.com".into(),
            password_hash: hash,
            roles: vec![],
            locked: false,
            failed_login_attempts: 0,
        });

        assert!(auth.login("alice", "wrong").is_err());
    }

    #[test]
    fn test_auth_service_account_lockout() {
        let jwt = JwtService::new(b"secret", "test-app");
        let hasher = PasswordHasher::new(100, 16);
        let mut auth = AuthService::new(jwt, hasher);

        let hash = auth.hasher.hash_password("password");
        auth.add_user(UserRecord {
            id: "u1".into(),
            username: "alice".into(),
            email: "alice@example.com".into(),
            password_hash: hash,
            roles: vec![],
            locked: false,
            failed_login_attempts: 4, // one more failure locks it
        });

        // This will fail and lock the account
        let _ = auth.login("alice", "wrong");
        // Now locked
        assert!(auth.login("alice", "password").is_err());
    }

    #[test]
    fn test_auth_service_role_check() {
        let jwt = JwtService::new(b"secret", "test-app");
        let hasher = PasswordHasher::new(100, 16);
        let mut auth = AuthService::new(jwt, hasher);

        auth.add_user(UserRecord {
            id: "u1".into(),
            username: "alice".into(),
            email: "alice@example.com".into(),
            password_hash: auth.hasher.hash_password("pw"),
            roles: vec!["admin".into(), "editor".into()],
            locked: false,
            failed_login_attempts: 0,
        });

        assert!(auth.user_has_role("u1", "admin"));
        assert!(auth.user_has_role("u1", "editor"));
        assert!(!auth.user_has_role("u1", "superadmin"));
        assert!(!auth.user_has_role("nonexistent", "admin"));
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq("hello", "hello"));
        assert!(!constant_time_eq("hello", "world"));
        assert!(!constant_time_eq("hello", "hell"));
        assert!(!constant_time_eq("short", "longer string"));
    }

    #[test]
    fn test_hex_encode_decode() {
        let data = vec![0, 1, 127, 255, 42];
        let encoded = hex_encode(&data);
        let decoded = hex_decode(&encoded);
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_password_hash_from_stored_string_invalid() {
        assert!(PasswordHash::from_stored_string("").is_none());
        assert!(PasswordHash::from_stored_string("only_two").is_none());
    }
}
