//! # Lesson 05: Hardware Security Tokens — FIDO2, YubiKey Concepts
//!
//! ## What are Hardware Tokens?
//!
//! Hardware security tokens are physical devices used for authentication.
//! They store private keys and perform cryptographic operations without exposing keys.
//!
//! ### FIDO2 / WebAuthn
//!
//! FIDO2 is a phishing-resistant authentication standard:
//! 1. **Registration**: User's token generates a key pair. Public key sent to server.
//! 2. **Authentication**: Server sends a challenge. Token signs it with the private key.
//! 3. **Verification**: Server verifies the signature with the stored public key.
//!
//! Key properties:
//! - **Origin-bound**: Key is bound to the website origin (prevents phishing)
//! - **User verification**: Requires PIN or biometric (presence check)
//! - **No shared secrets**: Server only stores public keys (no password breaches)
//!
//! ### Challenge-Response Protocol
//!
//! ```
//! Server                    Token
//!   |--- challenge (nonce) --->|
//!   |                         | sign(challenge, private_key)
//!   |<-- signature ------------|
//!   | verify(signature, public_key)
//! ```
//!
//! ## Attack: Token Cloning
//!
//! If an attacker extracts the private key from a token (e.g., via fault injection
//! or side-channel), they can clone the token. Defense: use secure elements
//! that store keys in non-extractable form with tamper protection.
//!
//! ## Attack: Relay/Man-in-the-Middle
//!
//! An attacker relays the challenge from the real server to a victim's token,
//! then forwards the signature back. Defense: origin binding — the token signs
//! the origin (URL) along with the challenge.

use ring::digest;
use ring::hmac;
use serde::{Deserialize, Serialize};

/// A credential registered with a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Unique credential ID.
    pub id: String,
    /// User identifier (e.g., email).
    pub user_id: String,
    /// The website origin this credential is bound to.
    pub origin: String,
    /// HMAC key (simulates the private key in the token).
    pub private_key: Vec<u8>,
}

/// Simulated hardware security token (e.g., YubiKey).
#[derive(Debug, Default)]
pub struct HardwareToken {
    /// Credentials stored on this token.
    credentials: Vec<Credential>,
}

impl HardwareToken {
    /// Exercise 1: Register a new credential on the token.
    ///
    /// Generate a random key pair (we use HMAC key as the private key).
    /// Bind the credential to the user ID and origin (anti-phishing).
    ///
    /// Returns the credential ID and public key material (same as private key in HMAC sim).
    ///
    /// Hints:
    /// - Generate 32 random bytes for the key
    /// - Create a Credential with a unique ID (use a counter or hash)
    /// - Store it in the credentials vector
    pub fn register(&mut self, user_id: &str, origin: &str) -> (String, Vec<u8>) {
        todo!("Register a new FIDO2 credential on the token")
    }

    /// Exercise 2: Authenticate — sign a challenge for a specific credential.
    ///
    /// The token signs: origin || challenge with the credential's private key.
    /// This binds the signature to the origin, preventing relay attacks.
    ///
    /// Returns the signature, or None if the credential doesn't exist.
    ///
    /// Hints:
    /// - Find the credential by ID
    /// - Construct message: origin_bytes || challenge
    /// - Compute HMAC-SHA256 with the credential's private key
    pub fn authenticate(&self, credential_id: &str, challenge: &[u8]) -> Option<Vec<u8>> {
        todo!("Sign challenge with origin binding")
    }

    /// Exercise 3: Verify an authentication response.
    ///
    /// Given a credential (with its public key), challenge, origin, and signature,
    /// verify that the signature is valid.
    ///
    /// Returns true if verification succeeds.
    pub fn verify(
        public_key: &[u8],
        origin: &str,
        challenge: &[u8],
        signature: &[u8],
    ) -> bool {
        todo!("Verify FIDO2 authentication signature")
    }

    /// Exercise 4: Check if a credential exists on this token.
    pub fn has_credential(&self, credential_id: &str) -> bool {
        todo!("Check credential existence")
    }

    /// Exercise 5: Get the number of credentials on this token.
    pub fn credential_count(&self) -> usize {
        todo!("Return number of stored credentials")
    }
}

/// A server-side credential store (relying party).
#[derive(Debug, Default)]
pub struct RelyingParty {
    /// Stored credentials: (credential_id, user_id, origin, public_key).
    credentials: Vec<(String, String, String, Vec<u8>)>,
}

impl RelyingParty {
    /// Exercise 6: Register a credential on the server.
    ///
    /// Store the credential ID, user ID, origin, and public key.
    pub fn register(&mut self, credential_id: String, user_id: String, origin: String, public_key: Vec<u8>) {
        todo!("Store credential on server side")
    }

    /// Exercise 7: Verify an authentication attempt.
    ///
    /// Find the credential by ID and verify the signature against
    /// the stored public key, origin, and challenge.
    pub fn verify_authentication(
        &self,
        credential_id: &str,
        origin: &str,
        challenge: &[u8],
        signature: &[u8],
    ) -> bool {
        todo!("Verify authentication against stored public key")
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

        // Correct origin verifies
        assert!(HardwareToken::verify(&pub_key, "https://example.com", challenge, &sig));

        // Phishing site — different origin — should fail
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
