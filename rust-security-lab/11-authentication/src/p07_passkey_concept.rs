//! # Lesson 07: Passkeys / WebAuthn — FIDO2 Public Key Credentials
//!
//! ## What are Passkeys?
//!
//! Passkeys replace passwords with public-key cryptography:
//! - **Registration**: Device generates a key pair. Private key stays on device.
//!   Public key is sent to the server.
//! - **Authentication**: Server sends a challenge. Device signs it with the private key.
//!   Server verifies with the stored public key.
//!
//! ## Why Passkeys?
//!
//! | Property | Passwords | Passkeys |
//! |----------|-----------|----------|
//! | Phishing | Vulnerable | Immune (bound to origin) |
//! | Reuse | Common | Impossible (unique per site) |
//! | Breach | Password leaked | Only public key on server |
//! | UX | Type/remember | Biometric/PIN |
//!
//! ## FIDO2 / WebAuthn Flow
//!
//! ```text
//! Registration:
//! 1. Client → Server: "I want to register"
//! 2. Server → Client: Challenge + relying party info
//! 3. Client: User authenticates (biometric), device creates key pair
//! 4. Client → Server: Public key + attestation
//! 5. Server: Stores public key + credential ID
//!
//! Authentication:
//! 1. Client → Server: "I want to log in"
//! 2. Server → Client: Challenge + allowed credentials
//! 3. Client: User authenticates, device signs challenge
//! 4. Client → Server: Signed challenge
//! 5. Server: Verifies signature with stored public key
//! ```

use ring::signature::{self, KeyPair};
use sha2::{Digest, Sha256};

/// A credential created during passkey registration.
#[derive(Debug, Clone)]
pub struct PasskeyCredential {
    /// Unique identifier for this credential
    pub credential_id: Vec<u8>,
    /// The public key (DER-encoded SubjectPublicKeyInfo for Ed25519)
    pub public_key_bytes: Vec<u8>,
    /// User this credential belongs to
    pub user_id: String,
    /// Sign counter (replay protection)
    pub sign_count: u32,
}

/// A challenge issued by the server for authentication.
#[derive(Debug, Clone)]
pub struct Challenge {
    pub challenge_bytes: Vec<u8>,
    pub relying_party_id: String,
    pub expires_at: u64,
}

/// Generate a cryptographic challenge for WebAuthn.
///
/// Challenges must be:
/// - At least 16 bytes (128 bits) of random data
/// - Unique per authentication attempt
/// - Time-limited
pub fn generate_challenge(length: usize) -> Vec<u8> {
    todo!("Generate a cryptographic random challenge")
}

/// Simulate passkey registration: generate a key pair and return the credential.
///
/// In a real system, the private key never leaves the authenticator device.
/// Here we simulate it by generating an Ed25519 key pair in memory.
pub fn register_passkey(
    user_id: &str,
    relying_party: &str,
) -> (PasskeyCredential, Vec<u8>) {
    todo!("Generate an Ed25519 key pair and create a PasskeyCredential")
}

/// Simulate passkey authentication: sign a challenge with the private key.
///
/// Returns the signature bytes and the updated sign counter.
pub fn authenticate_passkey(
    private_key_bytes: &[u8],
    challenge: &[u8],
    relying_party: &str,
) -> Result<(Vec<u8>, u32), String> {
    todo!("Sign a challenge with the private key")
}

/// Verify a passkey authentication response.
///
/// The server verifies:
/// 1. The signature is valid for the stored public key
/// 2. The challenge matches what was issued
/// 3. The relying party ID matches
/// 4. The sign counter has increased (replay protection)
pub fn verify_passkey_auth(
    public_key_bytes: &[u8],
    signature_bytes: &[u8],
    challenge: &[u8],
    relying_party: &str,
    current_sign_count: u32,
    new_sign_count: u32,
) -> Result<u32, String> {
    todo!("Verify a passkey authentication response")
}

/// Demonstrate origin binding — passkeys are bound to the relying party.
///
/// A signature made for "example.com" cannot be verified as being for "evil.com".
pub fn verify_origin_binding(
    public_key_bytes: &[u8],
    signature_bytes: &[u8],
    challenge: &[u8],
    correct_rp: &str,
    attacker_rp: &str,
) -> (bool, bool) {
    todo!("Verify that signatures are bound to the correct relying party")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_challenge_length() {
        let challenge = generate_challenge(32);
        assert_eq!(challenge.len(), 32);
    }

    #[test]
    fn test_generate_challenge_unique() {
        let c1 = generate_challenge(32);
        let c2 = generate_challenge(32);
        assert_ne!(c1, c2, "Challenges should be unique");
    }

    #[test]
    fn test_register_passkey_creates_credential() {
        let (credential, _private_key) = register_passkey("user123", "example.com");
        assert_eq!(credential.user_id, "user123");
        assert!(!credential.public_key_bytes.is_empty());
        assert!(!credential.credential_id.is_empty());
    }

    #[test]
    fn test_authenticate_and_verify_roundtrip() {
        let (credential, private_key) = register_passkey("user123", "example.com");
        let challenge = generate_challenge(32);

        let (signature, sign_count) =
            authenticate_passkey(&private_key, &challenge, "example.com").unwrap();

        let result = verify_passkey_auth(
            &credential.public_key_bytes,
            &signature,
            &challenge,
            "example.com",
            0,
            sign_count,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_wrong_challenge_fails() {
        let (credential, private_key) = register_passkey("user123", "example.com");
        let challenge = generate_challenge(32);
        let (signature, sign_count) =
            authenticate_passkey(&private_key, &challenge, "example.com").unwrap();

        let wrong_challenge = generate_challenge(32);
        let result = verify_passkey_auth(
            &credential.public_key_bytes,
            &signature,
            &wrong_challenge,
            "example.com",
            0,
            sign_count,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_origin_binding() {
        let (credential, private_key) = register_passkey("user123", "example.com");
        let challenge = generate_challenge(32);

        let (signature, _) =
            authenticate_passkey(&private_key, &challenge, "example.com").unwrap();

        let (correct_valid, attacker_valid) = verify_origin_binding(
            &credential.public_key_bytes,
            &signature,
            &challenge,
            "example.com",
            "evil.com",
        );

        assert!(correct_valid, "Signature should be valid for correct RP");
        assert!(
            !attacker_valid,
            "Signature should NOT be valid for attacker's RP"
        );
    }

    #[test]
    fn test_tampered_signature_fails() {
        let (credential, private_key) = register_passkey("user123", "example.com");
        let challenge = generate_challenge(32);
        let (mut signature, sign_count) =
            authenticate_passkey(&private_key, &challenge, "example.com").unwrap();

        // Tamper with signature
        if let Some(byte) = signature.first_mut() {
            *byte ^= 0xFF;
        }

        let result = verify_passkey_auth(
            &credential.public_key_bytes,
            &signature,
            &challenge,
            "example.com",
            0,
            sign_count,
        );
        assert!(result.is_err());
    }
}
