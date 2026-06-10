//! # Lesson 03: Capability Tokens (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CapabilityAction {
    Read,
    Write,
    Delete,
    Share,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub subject: String,
    pub resource: String,
    pub actions: Vec<CapabilityAction>,
    pub expires_at: u64,
    pub issued_at: u64,
    pub issuer: String,
    pub signature: Vec<u8>,
}

pub struct CapabilityAuthority {
    signing_key: Vec<u8>,
}

impl CapabilityAuthority {
    pub fn new(signing_key: Vec<u8>) -> Self {
        Self { signing_key }
    }

    pub fn issue_token(
        &self,
        subject: &str,
        resource: &str,
        actions: Vec<CapabilityAction>,
        ttl_seconds: u64,
    ) -> CapabilityToken {
        let now = Self::current_timestamp();
        let token_data = (
            subject,
            resource,
            &actions,
            now + ttl_seconds,
            now,
            "rust-security-lab",
        );
        let data = serde_json::to_vec(&token_data).unwrap();
        let signature = self.compute_signature(&data);

        CapabilityToken {
            subject: subject.to_string(),
            resource: resource.to_string(),
            actions,
            expires_at: now + ttl_seconds,
            issued_at: now,
            issuer: "rust-security-lab".to_string(),
            signature,
        }
    }

    /// Simple keyed hash: SHA256(key || data). In production, use HMAC-SHA256.
    fn compute_signature(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(&self.signing_key);
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Serialize the signable fields (everything except the signature).
    fn serialize_fields(token: &CapabilityToken) -> Vec<u8> {
        let data = (
            &token.subject,
            &token.resource,
            &token.actions,
            token.expires_at,
            token.issued_at,
            &token.issuer,
        );
        serde_json::to_vec(&data).unwrap()
    }

    /// Constant-time comparison to prevent timing attacks on signature verification.
    fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut diff = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            diff |= x ^ y;
        }
        diff == 0
    }

    pub fn verify_token(&self, token: &CapabilityToken) -> Result<(), String> {
        // Check expiry
        let now = Self::current_timestamp();
        if now > token.expires_at {
            return Err("Token has expired".to_string());
        }

        // Verify signature
        let data = Self::serialize_fields(token);
        let expected = self.compute_signature(&data);

        if Self::constant_time_eq(&token.signature, &expected) {
            Ok(())
        } else {
            Err("Invalid signature".to_string())
        }
    }

    pub fn check_access(
        &self,
        token: &CapabilityToken,
        action: &CapabilityAction,
    ) -> Result<(), String> {
        self.verify_token(token)?;

        if token.actions.contains(action) {
            Ok(())
        } else {
            Err(format!("Action {:?} not permitted by this token", action))
        }
    }

    pub fn attenuate(
        &self,
        original: &CapabilityToken,
        new_actions: Vec<CapabilityAction>,
        new_expires_at: u64,
    ) -> Result<CapabilityToken, String> {
        // Verify the original token first
        self.verify_token(original)?;

        // New actions must be a subset of original
        for action in &new_actions {
            if !original.actions.contains(action) {
                return Err(format!(
                    "Action {:?} is not in the original token",
                    action
                ));
            }
        }

        // New expiry must be <= original
        if new_expires_at > original.expires_at {
            return Err("New expiry cannot be later than original".to_string());
        }

        let mut new_token = CapabilityToken {
            subject: original.subject.clone(),
            resource: original.resource.clone(),
            actions: new_actions,
            expires_at: new_expires_at,
            issued_at: Self::current_timestamp(),
            issuer: format!("attenuated:{}", original.issuer),
            signature: Vec::new(),
        };

        let data = Self::serialize_fields(&new_token);
        new_token.signature = self.compute_signature(&data);

        Ok(new_token)
    }

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
        token.expires_at = 1;
        let data = CapabilityAuthority::serialize_fields(&token);
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
