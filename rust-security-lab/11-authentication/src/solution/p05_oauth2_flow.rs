//! # Lesson 05: OAuth2 Authorization Code Flow
//!
//! ## OAuth2 in a Nutshell
//!
//! OAuth2 lets users authorize third-party apps WITHOUT sharing their password.
//! Instead of giving GitHub credentials to a CI tool, you grant it a scoped token.
//!
//! ## Authorization Code Flow (the secure one)
//!
//! ```text
//! 1. Client → Auth Server: "Redirect user to login + consent"
//! 2. User → Auth Server:   Logs in, grants permission
//! 3. Auth Server → Client: Authorization code (one-time, 10min lifetime)
//! 4. Client → Auth Server: Exchange code + client_secret for tokens
//! 5. Auth Server → Client: access_token + refresh_token
//! ```
//!
//! ## Why Authorization Code (not Implicit)?
//!
//! The Implicit flow puts tokens in the URL fragment — visible in browser history,
//! referrer headers, and logs. The Authorization Code flow keeps tokens server-side.
//!
//! PKCE (Proof Key for Code Exchange) extends this for public clients (mobile/SPA):
//! - Client generates a random `code_verifier`
//! - Sends `code_challenge = SHA256(code_verifier)` in the auth request
//! - Sends `code_verifier` in the token exchange
//! - Attacker who intercepts the code can't exchange it without the verifier

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents an OAuth2 client registration.
#[derive(Debug, Clone)]
pub struct OAuthClient {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub allowed_scopes: Vec<String>,
}

/// An authorization code issued by the auth server.
#[derive(Debug, Clone)]
pub struct AuthorizationCode {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub user_id: String,
    pub expires_at: u64,
    /// PKCE: SHA256 hash of the code_verifier
    pub code_challenge: Option<String>,
    /// Whether this code has been exchanged (single-use)
    pub used: bool,
}

/// Tokens returned after successful exchange.
#[derive(Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
}

/// PKCE code verifier/challenge pair.
#[derive(Debug, Clone)]
pub struct PkcePair {
    pub code_verifier: String,
    pub code_challenge: String,
}

/// Generate a PKCE code_verifier and code_challenge pair.
///
/// code_verifier: 43-128 chars of [A-Z][a-z][0-9]-._~
/// code_challenge: BASE64URL(SHA256(code_verifier))
pub fn generate_pkce_pair() -> PkcePair {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~"
        .chars()
        .collect();
    let verifier: String = (0..64).map(|_| chars[rng.gen_range(0..chars.len())]).collect();

    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));

    PkcePair {
        code_verifier: verifier,
        code_challenge: challenge,
    }
}

/// Simulate a simple OAuth2 authorization server.
pub struct AuthorizationServer {
    pub clients: HashMap<String, OAuthClient>,
    pub issued_codes: HashMap<String, AuthorizationCode>,
    pub code_lifetime_secs: u64,
}

impl AuthorizationServer {
    pub fn new(code_lifetime_secs: u64) -> Self {
        Self {
            clients: HashMap::new(),
            issued_codes: HashMap::new(),
            code_lifetime_secs,
        }
    }

    /// Register an OAuth2 client.
    pub fn register_client(&mut self, client: OAuthClient) {
        self.clients.insert(client.client_id.clone(), client);
    }

    /// Step 1-3: Issue an authorization code after user consent.
    ///
    /// In a real system, this happens after the user logs in and grants consent.
    /// Here we simulate it directly.
    pub fn issue_authorization_code(
        &mut self,
        client_id: &str,
        redirect_uri: &str,
        scopes: &[&str],
        user_id: &str,
        code_challenge: Option<&str>,
    ) -> Result<String, String> {
        let client = self
            .clients
            .get(client_id)
            .ok_or("Unknown client_id")?;

        if client.redirect_uri != redirect_uri {
            return Err("redirect_uri mismatch".to_string());
        }

        // Validate scopes
        for scope in scopes {
            if !client.allowed_scopes.contains(&scope.to_string()) {
                return Err(format!("Scope '{}' not allowed", scope));
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let code = AuthorizationCode {
            code: generate_random_string(32),
            client_id: client_id.to_string(),
            redirect_uri: redirect_uri.to_string(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            user_id: user_id.to_string(),
            expires_at: now + self.code_lifetime_secs,
            code_challenge: code_challenge.map(|s| s.to_string()),
            used: false,
        };

        let code_str = code.code.clone();
        self.issued_codes.insert(code_str.clone(), code);
        Ok(code_str)
    }

    /// Step 4-5: Exchange authorization code for tokens.
    ///
    /// Validates:
    /// - Code exists and hasn't expired
    /// - Code hasn't been used before (single-use)
    /// - client_id matches
    /// - redirect_uri matches
    /// - PKCE code_verifier matches code_challenge (if PKCE was used)
    pub fn exchange_code(
        &mut self,
        code: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
        code_verifier: Option<&str>,
    ) -> Result<TokenResponse, String> {
        let client = self
            .clients
            .get(client_id)
            .ok_or("Unknown client_id")?;

        if client.client_secret != client_secret {
            return Err("Invalid client_secret".to_string());
        }

        let auth_code = self
            .issued_codes
            .get_mut(code)
            .ok_or("Invalid authorization code")?;

        if auth_code.used {
            return Err("Authorization code already used".to_string());
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if auth_code.expires_at < now {
            return Err("Authorization code expired".to_string());
        }

        if auth_code.client_id != client_id {
            return Err("client_id mismatch".to_string());
        }

        if auth_code.redirect_uri != redirect_uri {
            return Err("redirect_uri mismatch".to_string());
        }

        // PKCE verification
        if let Some(ref challenge) = auth_code.code_challenge {
            let verifier = code_verifier.ok_or("PKCE code_verifier required")?;
            let computed_challenge =
                URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
            if computed_challenge != *challenge {
                return Err("PKCE verification failed".to_string());
            }
        }

        // Mark code as used (single-use)
        auth_code.used = true;

        Ok(TokenResponse {
            access_token: generate_random_string(64),
            refresh_token: generate_random_string(64),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: auth_code.scopes.join(" "),
        })
    }
}

fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    (0..length)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_server() -> AuthorizationServer {
        let mut server = AuthorizationServer::new(600);
        server.register_client(OAuthClient {
            client_id: "test-client".to_string(),
            client_secret: "test-secret".to_string(),
            redirect_uri: "https://app.example.com/callback".to_string(),
            allowed_scopes: vec!["read".to_string(), "write".to_string()],
        });
        server
    }

    #[test]
    fn test_authorization_code_flow_basic() {
        let mut server = setup_server();
        let code = server
            .issue_authorization_code(
                "test-client",
                "https://app.example.com/callback",
                &["read"],
                "user123",
                None,
            )
            .unwrap();
        assert!(!code.is_empty());

        let tokens = server
            .exchange_code(
                &code,
                "test-client",
                "test-secret",
                "https://app.example.com/callback",
                None,
            )
            .unwrap();
        assert!(!tokens.access_token.is_empty());
        assert!(!tokens.refresh_token.is_empty());
        assert_eq!(tokens.token_type, "Bearer");
    }

    #[test]
    fn test_code_single_use() {
        let mut server = setup_server();
        let code = server
            .issue_authorization_code(
                "test-client",
                "https://app.example.com/callback",
                &["read"],
                "user123",
                None,
            )
            .unwrap();

        // First exchange succeeds
        let result1 = server.exchange_code(
            &code,
            "test-client",
            "test-secret",
            "https://app.example.com/callback",
            None,
        );
        assert!(result1.is_ok());

        // Second exchange fails (code already used)
        let result2 = server.exchange_code(
            &code,
            "test-client",
            "test-secret",
            "https://app.example.com/callback",
            None,
        );
        assert!(result2.unwrap_err().contains("already used"));
    }

    #[test]
    fn test_wrong_client_secret_rejected() {
        let mut server = setup_server();
        let code = server
            .issue_authorization_code(
                "test-client",
                "https://app.example.com/callback",
                &["read"],
                "user123",
                None,
            )
            .unwrap();

        let result = server.exchange_code(
            &code,
            "test-client",
            "wrong-secret",
            "https://app.example.com/callback",
            None,
        );
        assert!(result.unwrap_err().contains("client_secret"));
    }

    #[test]
    fn test_redirect_uri_mismatch_rejected() {
        let mut server = setup_server();
        let code = server
            .issue_authorization_code(
                "test-client",
                "https://app.example.com/callback",
                &["read"],
                "user123",
                None,
            )
            .unwrap();

        let result = server.exchange_code(
            &code,
            "test-client",
            "test-secret",
            "https://evil.com/callback",
            None,
        );
        assert!(result.unwrap_err().contains("redirect_uri"));
    }

    #[test]
    fn test_invalid_scope_rejected() {
        let mut server = setup_server();
        let result = server.issue_authorization_code(
            "test-client",
            "https://app.example.com/callback",
            &["admin"], // not in allowed scopes
            "user123",
            None,
        );
        assert!(result.unwrap_err().contains("not allowed"));
    }

    #[test]
    fn test_pkce_flow() {
        let mut server = setup_server();
        let pkce = generate_pkce_pair();

        let code = server
            .issue_authorization_code(
                "test-client",
                "https://app.example.com/callback",
                &["read"],
                "user123",
                Some(&pkce.code_challenge),
            )
            .unwrap();

        // Correct verifier succeeds
        let result = server.exchange_code(
            &code,
            "test-client",
            "test-secret",
            "https://app.example.com/callback",
            Some(&pkce.code_verifier),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_pkce_wrong_verifier_rejected() {
        let mut server = setup_server();
        let pkce = generate_pkce_pair();

        let code = server
            .issue_authorization_code(
                "test-client",
                "https://app.example.com/callback",
                &["read"],
                "user123",
                Some(&pkce.code_challenge),
            )
            .unwrap();

        let result = server.exchange_code(
            &code,
            "test-client",
            "test-secret",
            "https://app.example.com/callback",
            Some("wrong-verifier"),
        );
        assert!(result.unwrap_err().contains("PKCE"));
    }

    #[test]
    fn test_pkce_pair_deterministic() {
        let pkce = generate_pkce_pair();
        let computed = URL_SAFE_NO_PAD.encode(Sha256::digest(pkce.code_verifier.as_bytes()));
        assert_eq!(computed, pkce.code_challenge);
    }
}
