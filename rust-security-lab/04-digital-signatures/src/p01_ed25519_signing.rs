//! # Lesson 01: Ed25519 Signing
//!
//! ## What is Ed25519?
//!
//! Ed25519 is an Edwards-curve Digital Signature Algorithm (EdDSA) signature scheme
//! based on Curve25519. It is the recommended modern signature scheme for most applications.
//!
//! Properties:
//! - **Deterministic**: Same private key + message always produces the same signature (no RNG needed)
//! - **Fast**: Key generation, signing, and verification are all very fast
//! - **Small keys**: 32-byte private key, 32-byte public key, 64-byte signature
//! - **Resistant to side-channel attacks**: Constant-time by design
//!
//! ## Why Ed25519 over ECDSA?
//!
//! 1. No random nonce required — eliminates the catastrophic nonce-reuse attack
//! 2. Simpler to implement correctly
//! 3. Faster verification
//! 4. Supports batch verification (verify many signatures at once)
//!
//! ## Attack Scenario
//!
//! An attacker intercepts a signed message in transit. They want to:
//! 1. Modify the message without detection
//! 2. Forge a signature from the legitimate sender
//! 3. Replay an old signature with a new message
//!
//! Ed25519 prevents all three: the signature is bound to the exact message content.

use ed25519_dalek::{Signer, Verifier};

/// Exercise 1: Generate an Ed25519 keypair.
///
/// Returns (signing_key_bytes, verifying_key_bytes) where each is a Vec<u8>.
///
/// Hints:
/// - Use `ed25519_dalek::SigningKey::generate(&mut rng)` with `rand::rngs::OsRng`
/// - `SigningKey::to_bytes()` returns `[u8; 32]`
/// - `VerifyingKey::from(&signing_key)` to get the public key
/// - `VerifyingKey::to_bytes()` returns `[u8; 32]`
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    todo!("Generate an Ed25519 keypair and return (private_key_bytes, public_key_bytes)")
}

/// Exercise 2: Sign a message using Ed25519.
///
/// Takes the signing key bytes and a message, returns the 64-byte signature.
///
/// Hints:
/// - Reconstruct the key: `SigningKey::from_bytes(&key_bytes.try_into().unwrap())`
/// - Sign: `signing_key.sign(message)` returns a `Signature`
/// - Convert: `signature.to_bytes()` returns `[u8; 64]`
pub fn sign_message(signing_key_bytes: &[u8], message: &[u8]) -> Vec<u8> {
    todo!("Sign a message with Ed25519")
}

/// Exercise 3: Verify an Ed25519 signature.
///
/// Returns true if the signature is valid for the given message and public key.
///
/// Hints:
/// - Reconstruct: `VerifyingKey::from_bytes(&key_bytes.try_into().unwrap())`
/// - Reconstruct signature: `Signature::from_bytes(&sig_bytes.try_into().unwrap())`
/// - Verify: `verifying_key.verify(message, &signature).is_ok()`
pub fn verify_signature(verifying_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> bool {
    todo!("Verify an Ed25519 signature")
}

/// Exercise 4: Sign and serialize a JSON payload.
///
/// Sign the JSON-serialized version of the data and return (json_bytes, signature_bytes).
///
/// Hints:
/// - Serialize with `serde_json::to_vec(&data)?`
/// - Sign the serialized bytes
/// - Return both the serialized data and the signature
pub fn sign_json_payload<T: serde::Serialize>(
    signing_key_bytes: &[u8],
    data: &T,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    todo!("Sign a JSON-serialized payload")
}

/// Exercise 5: Verify a signed JSON payload.
///
/// Deserialize the JSON, verify the signature, and return the deserialized data.
///
/// Hints:
/// - Verify the signature over the raw json_bytes (not the deserialized data)
/// - Use `serde_json::from_slice` to deserialize after verification
/// - Return Err if verification fails
pub fn verify_json_payload<T: serde::de::DeserializeOwned>(
    verifying_key_bytes: &[u8],
    json_bytes: &[u8],
    signature_bytes: &[u8],
) -> Result<T, String> {
    todo!("Verify and deserialize a signed JSON payload")
}

/// Exercise 6: Create a signed token (compact representation).
///
/// Encode the public key + signature + message as a single base64 string.
/// Format: base64(public_key || signature || message)
///
/// Hints:
/// - Concatenate the bytes: public_key (32) + signature (64) + message
/// - Encode with `base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&combined)`
pub fn create_signed_token(signing_key_bytes: &[u8], message: &[u8]) -> String {
    todo!("Create a compact signed token")
}

/// Exercise 7: Parse and verify a signed token.
///
/// Decode the base64 token, extract public key + signature + message,
/// verify the signature, and return the message bytes.
///
/// Hints:
/// - Decode base64
/// - First 32 bytes = public key, next 64 bytes = signature, rest = message
/// - Verify signature over message using the public key
pub fn verify_signed_token(token: &str) -> Result<Vec<u8>, String> {
    todo!("Parse and verify a signed token")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair_size() {
        let (priv_key, pub_key) = generate_keypair();
        assert_eq!(priv_key.len(), 32, "Ed25519 private key should be 32 bytes");
        assert_eq!(pub_key.len(), 32, "Ed25519 public key should be 32 bytes");
    }

    #[test]
    fn test_sign_and_verify() {
        let (priv_key, pub_key) = generate_keypair();
        let message = b"Hello, Ed25519!";
        let sig = sign_message(&priv_key, message);
        assert_eq!(sig.len(), 64, "Ed25519 signature should be 64 bytes");
        assert!(verify_signature(&pub_key, message, &sig));
    }

    #[test]
    fn test_verify_wrong_message() {
        let (priv_key, pub_key) = generate_keypair();
        let sig = sign_message(&priv_key, b"original message");
        assert!(!verify_signature(&pub_key, b"tampered message", &sig));
    }

    #[test]
    fn test_verify_wrong_key() {
        let (priv_key, _) = generate_keypair();
        let (_, wrong_pub_key) = generate_keypair();
        let sig = sign_message(&priv_key, b"message");
        assert!(!verify_signature(&wrong_pub_key, b"message", &sig));
    }

    #[test]
    fn test_verify_corrupted_signature() {
        let (priv_key, pub_key) = generate_keypair();
        let mut sig = sign_message(&priv_key, b"message");
        sig[0] ^= 0xFF; // corrupt first byte
        assert!(!verify_signature(&pub_key, b"message", &sig));
    }

    #[test]
    fn test_sign_json_roundtrip() {
        let (priv_key, pub_key) = generate_keypair();
        let data = serde_json::json!({"user": "alice", "role": "admin"});
        let (json_bytes, sig) = sign_json_payload(&priv_key, &data).unwrap();
        let result: serde_json::Value =
            verify_json_payload(&pub_key, &json_bytes, &sig).unwrap();
        assert_eq!(result["user"], "alice");
        assert_eq!(result["role"], "admin");
    }

    #[test]
    fn test_signed_token_roundtrip() {
        let (priv_key, _) = generate_keypair();
        let message = b"authenticated request";
        let token = create_signed_token(&priv_key, message);
        let recovered = verify_signed_token(&token).unwrap();
        assert_eq!(recovered, message);
    }

    #[test]
    fn test_signed_token_tampered() {
        let (priv_key, _) = generate_keypair();
        let mut token = create_signed_token(&priv_key, b"original");
        // Tamper with the base64 string
        let last = token.pop().unwrap();
        token.push(if last == 'A' { 'B' } else { 'A' });
        assert!(verify_signed_token(&token).is_err());
    }
}
