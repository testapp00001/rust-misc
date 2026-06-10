//! # Lesson 05: Hardware Security Tokens — FIDO2, YubiKey (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::hmac;
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};

/// A credential registered with a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub user_id: String,
    pub origin: String,
    pub private_key: Vec<u8>,
}

/// Simulated hardware security token.
#[derive(Debug, Default)]
pub struct HardwareToken {
    credentials: Vec<Credential>,
    counter: u64,
}

impl HardwareToken {
    /// Register a new credential on the token.
    pub fn register(&mut self, user_id: &str, origin: &str) -> (String, Vec<u8>) {
        let rng = ring::rand::SystemRandom::new();
        let mut private_key = vec![0u8; 32];
        rng.fill(&mut private_key).unwrap();

        self.counter += 1;
        let cred_id = format!("cred-{}", self.counter);

        self.credentials.push(Credential {
            id: cred_id.clone(),
            user_id: user_id.to_string(),
            origin: origin.to_string(),
            private_key: private_key.clone(),
        });

        (cred_id, private_key)
    }

    /// Sign a challenge with origin binding: HMAC-SHA256(key, origin || challenge).
    pub fn authenticate(&self, credential_id: &str, challenge: &[u8]) -> Option<Vec<u8>> {
        let cred = self.credentials.iter().find(|c| c.id == credential_id)?;

        let mut message = cred.origin.as_bytes().to_vec();
        message.extend_from_slice(challenge);

        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &cred.private_key);
        Some(hmac::sign(&hmac_key, &message).as_ref().to_vec())
    }

    /// Verify an authentication signature.
    pub fn verify(
        public_key: &[u8],
        origin: &str,
        challenge: &[u8],
        signature: &[u8],
    ) -> bool {
        let mut message = origin.as_bytes().to_vec();
        message.extend_from_slice(challenge);

        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, public_key);
        hmac::verify(&hmac_key, &message, signature).is_ok()
    }

    /// Check if a credential exists on this token.
    pub fn has_credential(&self, credential_id: &str) -> bool {
        self.credentials.iter().any(|c| c.id == credential_id)
    }

    /// Get the number of credentials on this token.
    pub fn credential_count(&self) -> usize {
        self.credentials.len()
    }
}

/// A server-side credential store (relying party).
#[derive(Debug, Default)]
pub struct RelyingParty {
    credentials: Vec<(String, String, String, Vec<u8>)>,
}

impl RelyingParty {
    /// Register a credential on the server.
    pub fn register(&mut self, credential_id: String, user_id: String, origin: String, public_key: Vec<u8>) {
        self.credentials.push((credential_id, user_id, origin, public_key));
    }

    /// Verify an authentication attempt.
    pub fn verify_authentication(
        &self,
        credential_id: &str,
        origin: &str,
        challenge: &[u8],
        signature: &[u8],
    ) -> bool {
        let cred = self.credentials.iter().find(|(id, _, _, _)| id == credential_id);
        let (_, _, stored_origin, public_key) = match cred {
            Some(c) => c,
            None => return false,
        };
        if stored_origin != origin {
            return false;
        }
        HardwareToken::verify(public_key, origin, challenge, signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_creates_credential() {
        let mut token = HardwareToken::default();
        let (cred_id, pub_key) = token.register("user@example.com", "https://example.com");
        assert!(!cred_id.is_empty());
        assert_eq!(pub_key.len(), 32);
        assert!(token.has_credential(&cred_id));
    }

    #[test]
    fn test_authenticate_and_verify() {
        let mut token = HardwareToken::default();
        let (cred_id, pub_key) = token.register("user@example.com", "https://example.com");

        let challenge = b"random-server-challenge";
        let sig = token.authenticate(&cred_id, challenge).unwrap();

        assert!(HardwareToken::verify(&pub_key, "https://example.com", challenge, &sig));
    }

    #[test]
    fn test_origin_binding_prevents_phishing() {
        let mut token = HardwareToken::default();
        let (cred_id, pub_key) = token.register("user@example.com", "https://example.com");

        let challenge = b"challenge";
        let sig = token.authenticate(&cred_id, challenge).unwrap();

        assert!(HardwareToken::verify(&pub_key, "https://example.com", challenge, &sig));
        assert!(!HardwareToken::verify(&pub_key, "https://evil.com", challenge, &sig));
    }

    #[test]
    fn test_wrong_challenge_fails() {
        let mut token = HardwareToken::default();
        let (cred_id, pub_key) = token.register("user", "https://example.com");

        let sig = token.authenticate(&cred_id, b"real-challenge").unwrap();
        assert!(!HardwareToken::verify(&pub_key, "https://example.com", b"wrong-challenge", &sig));
    }

    #[test]
    fn test_authenticate_nonexistent_credential() {
        let token = HardwareToken::default();
        assert!(token.authenticate("nonexistent", b"challenge").is_none());
    }

    #[test]
    fn test_server_registration_and_verification() {
        let mut token = HardwareToken::default();
        let mut server = RelyingParty::default();

        let (cred_id, pub_key) = token.register("user@example.com", "https://example.com");
        server.register(cred_id.clone(), "user@example.com".into(), "https://example.com".into(), pub_key);

        let challenge = b"server-challenge";
        let sig = token.authenticate(&cred_id, challenge).unwrap();

        assert!(server.verify_authentication(&cred_id, "https://example.com", challenge, &sig));
    }

    #[test]
    fn test_credential_count() {
        let mut token = HardwareToken::default();
        assert_eq!(token.credential_count(), 0);
        token.register("user1", "https://a.com");
        assert_eq!(token.credential_count(), 1);
        token.register("user2", "https://b.com");
        assert_eq!(token.credential_count(), 2);
    }
}
