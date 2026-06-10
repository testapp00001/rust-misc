//! # Lesson 02: ECDSA Signing with P-256
//!
//! ## What is ECDSA?
//!
//! ECDSA (Elliptic Curve Digital Signature Algorithm) is the most widely deployed
//! signature scheme on the internet. It is used in TLS certificates, Bitcoin, and
//! many other systems.
//!
//! P-256 (also called secp256r1 or prime256v1) is the most common curve.
//!
//! ## ECDSA vs Ed25519
//!
//! ECDSA requires a random nonce `k` for each signature. If `k` is:
//! - **Reused**: Private key can be recovered from two signatures (Sony PS3 hack)
//! - **Predictable**: Private key can be recovered
//! - **Biased**: Private key can be partially recovered
//!
//! Ed25519 does NOT have this weakness — it derives `k` deterministically.
//!
//! ## Attack Scenario: Nonce Reuse
//!
//! Given two ECDSA signatures with the same `k`:
//! s1 = k^-1 * (z1 + r*privkey) mod n
//! s2 = k^-1 * (z2 + r*privkey) mod n
//!
//! Then: k = (z1 - z2) / (s1 - s2) mod n
//! And:  privkey = (s1*k - z1) / r mod n
//!
//! This is exactly how the PlayStation 3's master key was extracted.

use p256::ecdsa::{SigningKey, Signature, signature::Signer, signature::Verifier};
use rand::rngs::OsRng;

/// Exercise 1: Generate an ECDSA P-256 keypair.
///
/// Returns (private_key_bytes, public_key_bytes_compressed).
///
/// Hints:
/// - Use `SigningKey::random(&mut OsRng)` to generate a private key
/// - `signing_key.to_bytes()` gives the 32-byte scalar (use `.as_slice().to_vec()`)
/// - `VerifyingKey::from(&signing_key)` for the public key
/// - `verifying_key.to_encoded_point(true)` for compressed encoding
/// - `.as_bytes().to_vec()` to get the byte vector
pub fn generate_keypair() -> (Vec<u8>, Vec<u8>) {
    todo!("Generate an ECDSA P-256 keypair")
}

/// Exercise 2: Sign a message using ECDSA P-256.
///
/// Returns the DER-encoded signature bytes.
///
/// Hints:
/// - Reconstruct: `SigningKey::from_bytes(scalar_bytes.into()).unwrap()`
/// - Sign: `signing_key.sign(message)` returns a `Signature`
/// - DER encode: `signature.to_bytes()` (this gives fixed 64-byte encoding)
pub fn sign_message(signing_key_bytes: &[u8], message: &[u8]) -> Vec<u8> {
    todo!("Sign a message with ECDSA P-256")
}

/// Exercise 3: Verify an ECDSA P-256 signature.
///
/// Returns true if the signature is valid.
///
/// Hints:
/// - Reconstruct verifying key from compressed bytes
/// - Reconstruct signature from bytes
/// - Use `verifying_key.verify(message, &signature).is_ok()`
pub fn verify_signature(verifying_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> bool {
    todo!("Verify an ECDSA P-256 signature")
}

/// Exercise 4: Extract the (r, s) components from an ECDSA signature.
///
/// ECDSA signatures consist of two 32-byte integers (r, s).
/// Return them as (r_bytes, s_bytes).
///
/// Hints:
/// - Parse signature: `Signature::from_slice(signature_bytes).unwrap()`
/// - The signature bytes are 64 bytes: first 32 = r, last 32 = s
pub fn extract_signature_components(signature_bytes: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Extract r and s components from ECDSA signature")
}

/// Exercise 5: Demonstrate that ECDSA signatures are NOT deterministic.
///
/// Sign the same message twice and return both signatures.
/// They will be different because a new random nonce k is used each time.
///
/// Hints:
/// - Generate one keypair
/// - Sign the same message twice
/// - Return both signatures
pub fn demonstrate_nondeterminism(message: &[u8]) -> (Vec<u8>, Vec<u8>) {
    todo!("Sign the same message twice to show ECDSA is non-deterministic")
}

/// Exercise 6: Create a signed certificate-like structure.
///
/// Sign the concatenation of (subject_name || public_key) and return
/// the certificate as (subject, pubkey, signature_bytes).
///
/// Hints:
/// - Concatenate subject bytes and public key bytes
/// - Sign the concatenation
pub fn create_certificate(
    signing_key_bytes: &[u8],
    subject: &str,
    subject_pubkey: &[u8],
) -> (String, Vec<u8>, Vec<u8>) {
    todo!("Create a signed certificate")
}

/// Exercise 7: Verify a certificate.
///
/// Given a CA's public key and a certificate (subject, pubkey, sig),
/// verify the CA signed it.
///
/// Hints:
/// - Reconstruct the signed data: subject.as_bytes() || pubkey
/// - Verify the signature over that data using the CA's public key
pub fn verify_certificate(
    ca_pubkey_bytes: &[u8],
    subject: &str,
    subject_pubkey: &[u8],
    signature_bytes: &[u8],
) -> bool {
    todo!("Verify a certificate against CA public key")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (priv_key, pub_key) = generate_keypair();
        assert_eq!(priv_key.len(), 32, "P-256 private key should be 32 bytes");
        // Compressed P-256 public key is 33 bytes (0x02/0x03 prefix + 32 bytes x-coord)
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
