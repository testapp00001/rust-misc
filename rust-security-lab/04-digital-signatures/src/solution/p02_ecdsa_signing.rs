//! # Lesson 02: ECDSA Signing with P-256 (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use p256::ecdsa::{
    signature::Signer, signature::Verifier, Signature, SigningKey, VerifyingKey,
};
use rand::rngs::OsRng;

/// Generate an ECDSA P-256 keypair.
///
/// Returns (32-byte private key scalar, 33-byte compressed public key).
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = VerifyingKey::from(&signing_key);
    (
        signing_key.to_bytes().to_vec(),
        verifying_key.to_encoded_point(true).as_bytes().to_vec(),
    )
}

/// Sign a message using ECDSA P-256.
///
/// SECURITY NOTE: Each call generates a fresh random nonce. If the RNG is
/// broken or predictable, the private key is compromised. This is why
/// Ed25519 (deterministic) is preferred for new applications.
pub fn sign_message(signing_key_bytes: &[u8], message: &[u8]) -> Vec<u8> {
    let scalar = p256::NonZeroScalar::try_from(signing_key_bytes)
        .expect("invalid signing key bytes");
    let signing_key = SigningKey::from(scalar);
    let signature: Signature = signing_key.sign(message);
    signature.to_bytes().to_vec()
}

/// Verify an ECDSA P-256 signature.
///
/// Returns false on any error (invalid key, invalid signature, verification failure).
pub fn verify_signature(verifying_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> bool {
    let point = match p256::EncodedPoint::from_bytes(verifying_key_bytes) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let verifying_key = match VerifyingKey::from_encoded_point(&point) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let signature = match Signature::from_slice(signature_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };
    verifying_key.verify(message, &signature).is_ok()
}

/// Extract the (r, s) components from an ECDSA signature.
///
/// ECDSA signatures are 64 bytes: 32 bytes for r, 32 bytes for s.
/// Both are big-endian encoded integers.
pub fn extract_signature_components(signature_bytes: &[u8]) -> (Vec<u8>, Vec<u8>) {
    assert_eq!(signature_bytes.len(), 64, "ECDSA P-256 signature is 64 bytes");
    (signature_bytes[..32].to_vec(), signature_bytes[32..].to_vec())
}

/// Demonstrate that ECDSA signatures can be deterministic (RFC 6979) or non-deterministic.
///
/// The `p256` crate uses RFC 6979 by default, which derives the nonce deterministically
/// from the private key and message. This means the same key + message produces the
/// same signature — similar to Ed25519.
///
/// This is actually GOOD: it eliminates the catastrophic nonce-reuse attack.
/// We return both signatures and note they are identical with RFC 6979.
pub fn demonstrate_nondeterminism(message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let signing_key = SigningKey::random(&mut OsRng);
    let sig1: Signature = signing_key.sign(message);
    let sig2: Signature = signing_key.sign(message);
    // With RFC 6979 (used by p256 crate), these are IDENTICAL.
    // With a naive RNG-based nonce, they would differ.
    (sig1.to_bytes().to_vec(), sig2.to_bytes().to_vec())
}

/// Create a signed certificate (simplified).
///
/// A CA signs the binding between a subject name and their public key.
/// This is the core idea behind X.509 certificates and PKI.
pub fn create_certificate(
    signing_key_bytes: &[u8],
    subject: &str,
    subject_pubkey: &[u8],
) -> (String, Vec<u8>, Vec<u8>) {
    let scalar = p256::NonZeroScalar::try_from(signing_key_bytes)
        .expect("invalid signing key bytes");
    let signing_key = SigningKey::from(scalar);
    let verifying_key = VerifyingKey::from(&signing_key);

    let mut data = Vec::new();
    data.extend_from_slice(subject.as_bytes());
    data.extend_from_slice(subject_pubkey);

    let signature: Signature = signing_key.sign(&data);
    (subject.to_string(), verifying_key.to_encoded_point(true).as_bytes().to_vec(), signature.to_bytes().to_vec())
}

/// Verify a certificate against a CA's public key.
///
/// SECURITY NOTE: In a real system, you would also check:
/// - Certificate expiration
/// - Certificate revocation (CRL/OCSP)
/// - Key usage constraints
/// - Chain of trust to a root CA
pub fn verify_certificate(
    ca_pubkey_bytes: &[u8],
    subject: &str,
    subject_pubkey: &[u8],
    signature_bytes: &[u8],
) -> bool {
    let mut data = Vec::new();
    data.extend_from_slice(subject.as_bytes());
    data.extend_from_slice(subject_pubkey);
    verify_signature(ca_pubkey_bytes, &data, signature_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (priv_key, pub_key) = generate_keypair();
        assert_eq!(priv_key.len(), 32, "P-256 private key should be 32 bytes");
        assert_eq!(pub_key.len(), 33, "Compressed P-256 public key should be 33 bytes");
    }

    #[test]
    fn test_sign_and_verify() {
        let (priv_key, pub_key) = generate_keypair();
        let message = b"ECDSA test message";
        let sig = sign_message(&priv_key, message);
        assert!(verify_signature(&pub_key, message, &sig));
    }

    #[test]
    fn test_verify_wrong_message() {
        let (priv_key, pub_key) = generate_keypair();
        let sig = sign_message(&priv_key, b"original");
        assert!(!verify_signature(&pub_key, b"tampered", &sig));
    }

    #[test]
    fn test_verify_wrong_key() {
        let (priv_key, _) = generate_keypair();
        let (_, wrong_pub) = generate_keypair();
        let sig = sign_message(&priv_key, b"message");
        assert!(!verify_signature(&wrong_pub, b"message", &sig));
    }

    #[test]
    fn test_extract_components() {
        let (priv_key, _) = generate_keypair();
        let sig = sign_message(&priv_key, b"test");
        let (r, s) = extract_signature_components(&sig);
        assert_eq!(r.len(), 32);
        assert_eq!(s.len(), 32);
    }

    #[test]
    fn test_nondeterminism() {
        let (sig1, sig2) = demonstrate_nondeterminism(b"same message");
        assert_ne!(sig1, sig2, "ECDSA signatures should differ (random nonce)");
    }

    #[test]
    fn test_certificate_roundtrip() {
        let (ca_priv, ca_pub) = generate_keypair();
        let (subject, subject_pub, sig) =
            create_certificate(&ca_priv, "example.com", b"subject_public_key_here");
        assert!(verify_certificate(&ca_pub, &subject, &subject_pub, &sig));
    }

    #[test]
    fn test_certificate_wrong_ca() {
        let (ca_priv, _) = generate_keypair();
        let (_, wrong_ca_pub) = generate_keypair();
        let (subject, subject_pub, sig) =
            create_certificate(&ca_priv, "example.com", b"subject_key");
        assert!(!verify_certificate(&wrong_ca_pub, &subject, &subject_pub, &sig));
    }
}
