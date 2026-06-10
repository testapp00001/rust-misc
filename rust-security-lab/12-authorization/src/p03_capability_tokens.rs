//! # Lesson 03: Capability Tokens
//!
//! ## What are Capability Tokens?
//!
//! A capability token is a signed document that says: "Holder of this token may perform
//! THESE actions on THIS resource until THIS expiry." Unlike RBAC where permissions are
//! looked up server-side, the token ITSELF carries the authorization.
//!
//! ## Capability Token Structure
//!
//! ```json
//! {
//!   "subject": "alice",
//!   "resource": "/documents/report-42",
//!   "actions": ["read", "write"],
//!   "expires_at": "2025-12-31T23:59:59Z",
//!   "issued_at": "2025-01-01T00:00:00Z",
//!   "issuer": "auth-server"
//! }
//! ```
//!
//! The token is signed with HMAC-SHA256 to prevent tampering.
//!
//! ## Capability Tokens vs JWTs
//!
//! Both are signed tokens, but:
//! - JWTs typically encode IDENTITY (who you are) → used for authentication
//! - Capability tokens encode AUTHORITY (what you can do) → used for authorization
//! - Capability tokens are attenuable: you can create a MORE restricted token from a broader one
//!
//! ## 🔴 Attack: Token Forgery
//!
//! Without cryptographic signatures, an attacker can forge tokens claiming any permissions.
//! Always verify the signature before trusting token contents.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Actions that can be granted in a capability token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapabilityAction {
    Read,
    Write,
    Delete,
    Share,
}

/// A capability token granting specific actions on a resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    /// Who this token is issued to.
    pub subject: String,
    /// The resource path this token grants access to.
    pub resource: String,
    /// Actions this token permits.
    pub actions: Vec<CapabilityAction>,
    /// Unix timestamp when this token expires.
    pub expires_at: u64,
    /// Unix timestamp when this token was issued.
    pub issued_at: u64,
    /// Who issued this token.
    pub issuer: String,
    /// HMAC-SHA256 signature over the token fields.
    pub signature: Vec<u8>,
}

/// Signs and verifies capability tokens using HMAC-SHA256.
pub struct CapabilityAuthority {
    /// The secret key used for HMAC signing.
    signing_key: Vec<u8>,
}

impl CapabilityAuthority {
    /// Create a new authority with the given signing key.
    pub fn new(signing_key: Vec<u8>) -> Self {
        todo!("Store the signing key")
    }

    /// Issue a new capability token.
    ///
    /// 1. Build the token fields (subject, resource, actions, timestamps, issuer).
    /// 2. Serialize the token fields (excluding signature) to JSON.
    /// 3. Compute HMAC-SHA256 over the serialized data using the signing key.
    /// 4. Attach the signature to the token.
    pub fn issue_token(
        &self,
        subject: &str,
        resource: &str,
        actions: Vec<CapabilityAction>,
        ttl_seconds: u64,
    ) -> CapabilityToken {
        todo!("Create and sign a capability token")
    }

    /// Compute the HMAC-SHA256 signature for token data.
    ///
    /// Since we don't have the `hmac` crate in dependencies, use a simple approach:
    /// SHA256(signing_key || data) as a keyed hash. In production, use proper HMAC.
    fn compute_signature(&self, data: &[u8]) -> Vec<u8> {
        todo!("Compute SHA256(signing_key || data)")
    }

    /// Verify a capability token's signature and expiry.
    ///
    /// 1. Re-serialize the token fields (excluding signature) to JSON.
    /// 2. Recompute the signature.
    /// 3. Compare signatures using constant-time comparison.
    /// 4. Check that the token has not expired.
    ///
    /// Returns Ok(()) if valid, Err with a description if invalid.
    pub fn verify_token(&self, token: &CapabilityToken) -> Result<(), String> {
        todo!("Verify signature and expiry")
    }

    /// Check if a token grants a specific action.
    ///
    /// First verify the token, then check if the action is in the token's action list.
    pub fn check_access(
        &self,
        token: &CapabilityToken,
        action: &CapabilityAction,
    ) -> Result<(), String> {
        todo!("Verify token then check action")
    }

    /// Create an attenuated (restricted) token from an existing valid token.
    ///
    /// The new token can only have a SUBSET of the original's actions and
    /// a SHORTER expiry. This enables safe delegation.
    ///
    /// Returns an error if the new actions are not a subset of the original
    /// or if the new expiry is later than the original.
    pub fn attenuate(
        &self,
        original: &CapabilityToken,
        new_actions: Vec<CapabilityAction>,
        new_expires_at: u64,
    ) -> Result<CapabilityToken, String> {
        todo!("Create a more restricted token from the original")
    }

    /// Get the current Unix timestamp.
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &[u8] = b"super-secret-signing-key-for-testing";

    fn authority() -> CapabilityAuthority {
        CapabilityAuthority::new(TEST_KEY.to_vec())
    }

    #[test]
    fn test_issue_and_verify() {
        let auth = authority();
        let token = auth.issue_token(
            "alice",
            "/docs/report",
            vec![CapabilityAction::Read, CapabilityAction::Write],
            3600,
        );
        assert!(auth.verify_token(&token).is_ok());
    }

    #[test]
    fn test_token_fields_correct() {
        let auth = authority();
        let token = auth.issue_token("bob", "/files/secret", vec![CapabilityAction::Read], 600);
        assert_eq!(token.subject, "bob");
        assert_eq!(token.resource, "/files/secret");
        assert!(token.actions.contains(&CapabilityAction::Read));
        assert!(token.expires_at > token.issued_at);
    }

    #[test]
    fn test_check_access_granted() {
        let auth = authority();
        let token = auth.issue_token(
            "alice",
            "/docs/1",
            vec![CapabilityAction::Read, CapabilityAction::Write],
            3600,
        );
        assert!(auth.check_access(&token, &CapabilityAction::Read).is_ok());
        assert!(auth.check_access(&token, &CapabilityAction::Write).is_ok());
    }

    #[test]
    fn test_check_access_denied() {
        let auth = authority();
        let token = auth.issue_token("alice", "/docs/1", vec![CapabilityAction::Read], 3600);
        assert!(auth.check_access(&token, &CapabilityAction::Write).is_err());
    }

    #[test]
    fn test_tampered_token_fails() {
        let auth = authority();
        let mut token =
            auth.issue_token("alice", "/docs/1", vec![CapabilityAction::Read], 3600);
        // Tamper with the resource
        token.resource = "/docs/2".to_string();
        assert!(auth.verify_token(&token).is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let auth1 = CapabilityAuthority::new(b"key-1".to_vec());
        let auth2 = CapabilityAuthority::new(b"key-2".to_vec());
        let token = auth1.issue_token("alice", "/docs/1", vec![CapabilityAction::Read], 3600);
        assert!(auth2.verify_token(&token).is_err());
    }

    #[test]
    fn test_expired_token_fails() {
        let auth = authority();
        let mut token = auth.issue_token("alice", "/docs/1", vec![CapabilityAction::Read], 3600);
        // Set expiry to the past
        token.expires_at = 1;
        // Re-sign with correct data
        let data = serde_json::to_vec(&(&token.subject, &token.resource, &token.actions, token.expires_at, token.issued_at, &token.issuer)).unwrap();
        token.signature = auth.compute_signature(&data);
        assert!(auth.verify_token(&token).is_err());
    }

    #[test]
    fn test_attenuate_reduces_actions() {
        let auth = authority();
        let original = auth.issue_token(
            "alice",
            "/docs/1",
            vec![CapabilityAction::Read, CapabilityAction::Write, CapabilityAction::Delete],
            3600,
        );
        let restricted = auth
            .attenuate(&original, vec![CapabilityAction::Read], original.expires_at)
            .unwrap();
        assert!(restricted.actions.contains(&CapabilityAction::Read));
        assert!(!restricted.actions.contains(&CapabilityAction::Write));
        assert!(auth.verify_token(&restricted).is_ok());
    }

    #[test]
    fn test_attenuate_cannot_expand() {
        let auth = authority();
        let original = auth.issue_token("alice", "/docs/1", vec![CapabilityAction::Read], 3600);
        let result = auth.attenuate(
            &original,
            vec![CapabilityAction::Read, CapabilityAction::Delete],
            original.expires_at,
        );
        assert!(result.is_err());
    }
}
