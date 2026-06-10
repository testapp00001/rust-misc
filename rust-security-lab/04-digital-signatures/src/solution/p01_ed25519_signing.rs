//! # Lesson 01: Ed25519 Signing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;

/// Generate an Ed25519 keypair.
///
/// Uses `OsRng` for cryptographically secure random key generation.
/// The signing key is 32 bytes; the verifying key is derived from it.
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    (
        signing_key.to_bytes().to_vec(),
        verifying_key.to_bytes().to_vec(),
    )
}

/// Sign a message using Ed25519.
///
/// Ed25519 signing is deterministic: the same key + message always produces
/// the same signature. No random nonce is needed, which eliminates an entire
/// class of vulnerabilities present in ECDSA.
pub fn sign_message(signing_key_bytes: &[u8], message: &[u8]) -> Vec<u8> {
    let key_bytes: [u8; 32] = signing_key_bytes
        .try_into()
        .expect("Ed25519 signing key must be 32 bytes");
    let signing_key = SigningKey::from_bytes(&key_bytes);
    signing_key.sign(message).to_bytes().to_vec()
}

/// Verify an Ed25519 signature.
///
/// Returns false on any error — invalid key format, invalid signature format,
/// or failed verification. Never panics on bad input.
pub fn verify_signature(verifying_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> bool {
    let key_bytes: [u8; 32] = match verifying_key_bytes.try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let sig_bytes: [u8; 64] = match signature_bytes.try_into() {
        Ok(b) => b,
        Err(_) => return false,
    };
    let verifying_key = match VerifyingKey::from_bytes(&key_bytes) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
    verifying_key.verify(message, &signature).is_ok()
}

/// Sign a JSON-serialized payload.
///
/// SECURITY NOTE: Always sign the serialized bytes, not a re-serialized version.
/// If you sign the struct and later serialize it differently (different field order,
/// whitespace, etc.), verification will fail.
pub fn sign_json_payload<T: serde::Serialize>(
    signing_key_bytes: &[u8],
    data: &T,
) -> Result<(Vec<u8>, Vec<u8>), String> {
    let json_bytes = serde_json::to_vec(data).map_err(|e| e.to_string())?;
    let sig = sign_message(signing_key_bytes, &json_bytes);
    Ok((json_bytes, sig))
}

/// Verify a signed JSON payload.
///
/// SECURITY NOTE: Verify the signature over the raw bytes BEFORE deserializing.
/// This prevents processing untrusted data. If the signature is invalid,
/// the data is never deserialized.
pub fn verify_json_payload<T: serde::de::DeserializeOwned>(
    verifying_key_bytes: &[u8],
    json_bytes: &[u8],
    signature_bytes: &[u8],
) -> Result<T, String> {
    if !verify_signature(verifying_key_bytes, json_bytes, signature_bytes) {
        return Err("Signature verification failed".to_string());
    }
    serde_json::from_slice(json_bytes).map_err(|e| e.to_string())
}

/// Create a signed token: base64(public_key || signature || message).
///
/// This is a compact, self-contained signed message. The verifier needs no
/// external state — the public key is embedded in the token.
pub fn create_signed_token(signing_key_bytes: &[u8], message: &[u8]) -> String {
    let key_bytes: [u8; 32] = signing_key_bytes
        .try_into()
        .expect("Ed25519 signing key must be 32 bytes");
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = VerifyingKey::from(&signing_key);
    let signature = signing_key.sign(message);

    let mut combined = Vec::with_capacity(32 + 64 + message.len());
    combined.extend_from_slice(&verifying_key.to_bytes());
    combined.extend_from_slice(&signature.to_bytes());
    combined.extend_from_slice(message);

    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&combined)
}

/// Parse and verify a signed token.
///
/// Extracts public key (32 bytes), signature (64 bytes), and message (remaining),
/// then verifies the signature over the message.
pub fn verify_signed_token(token: &str) -> Result<Vec<u8>, String> {
    let combined = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| "Invalid base64 token")?;

    if combined.len() < 96 {
        return Err("Token too short: need at least 32 (key) + 64 (sig) bytes".to_string());
    }

    let verifying_key_bytes: [u8; 32] = combined[..32].try_into().unwrap();
    let signature_bytes: [u8; 64] = combined[32..96].try_into().unwrap();
    let message = &combined[96..];

    if verify_signature(&verifying_key_bytes, message, &signature_bytes) {
        Ok(message.to_vec())
    } else {
        Err("Signature verification failed".to_string())
    }
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
        sig[0] ^= 0xFF;
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
        let last = token.pop().unwrap();
        token.push(if last == 'A' { 'B' } else { 'A' });
        assert!(verify_signed_token(&token).is_err());
    }
}
