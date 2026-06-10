//! # Lesson 04: Release Signing
//!
//! ## Attack: Binary Tampering
//!
//! An attacker gains access to your release server and replaces the official binary
//! with a backdoored version. Users download it, verify... nothing, because there's
//! no signature. The backdoor spreads to thousands of installations.
//!
//! This is how the **HandBrake** malware spread in 2017: the download server was
//! compromised and a trojanized DMG replaced the legitimate one.
//!
//! ## Defend: Cryptographic Signing
//!
//! Sign every release artifact with a private key. Publish the signature alongside
//! the artifact. Users verify with the public key before trusting the artifact.
//!
//! ```text
//! Signing:  signature = Sign(private_key, SHA256(artifact))
//! Verify:   Verify(public_key, SHA256(artifact), signature)
//! ```
//!
//! ## Audit: Signing Checklist
//!
//! - [ ] Every release artifact is signed
//! - [ ] Public key is published via a separate channel (website, keybase)
//! - [ ] Users are instructed to verify before running
//! [ ] Signing keys are stored in HSM or cloud KMS
//! - [ ] Key rotation procedure exists
//! - [ ] Compromised keys are revoked

use ring::signature::{self, KeyPair};

/// A signed release artifact.
#[derive(Debug, Clone)]
pub struct SignedArtifact {
    /// Name of the artifact (e.g., "myapp-v1.0.0-linux-amd64")
    pub name: String,
    /// SHA-256 digest of the artifact
    pub digest: Vec<u8>,
    /// Ed25519 signature over the digest
    pub signature: Vec<u8>,
    /// Hex-encoded public key that can verify this signature
    pub public_key_hex: String,
}

/// Exercise 1: Generate an Ed25519 signing keypair.
///
/// Return (private_key_bytes, public_key_bytes) as Vec<u8>.
///
/// Hints:
/// - Use `ring::signature::Ed25519KeyPair::generate_pkcs8(rng)` to generate
/// - Use `ring::rand::SystemRandom::new()` for the RNG
/// - The PKCS8 document contains the private key: `.as_ref()` gives `&[u8]`
/// - Get the public key with `pair.public_key().as_ref()`
pub fn generate_signing_keypair() -> (Vec<u8>, Vec<u8>) {
    todo!("Generate an Ed25519 keypair for release signing")
}

/// Exercise 2: Sign an artifact digest.
///
/// Given a PKCS8 private key and a SHA-256 digest, produce an Ed25519 signature.
///
/// Hints:
/// - Parse the key with `Ed25519KeyPair::from_pkcs8(private_key_bytes)`
/// - Sign with `pair.sign(digest)`
/// - Return the signature bytes
pub fn sign_digest(pkcs8_private_key: &[u8], digest: &[u8]) -> Vec<u8> {
    todo!("Sign a digest with Ed25519")
}

/// Exercise 3: Verify an artifact signature.
///
/// Given a public key, digest, and signature, verify the signature is valid.
///
/// Hints:
/// - Parse the public key with `signature::UnparsedPublicKey::new(&signature::ED25519, public_key)`
/// - Verify with `public_key.verify(digest, signature)`
/// - Return true if verification succeeds
pub fn verify_signature(public_key: &[u8], digest: &[u8], sig: &[u8]) -> bool {
    todo!("Verify an Ed25519 signature")
}

/// Exercise 4: Create a complete signed artifact.
///
/// Given an artifact's raw bytes and a PKCS8 private key:
/// 1. Compute SHA-256 of the artifact bytes
/// 2. Sign the digest
/// 3. Return a `SignedArtifact` with all fields populated
///
/// The public key should be hex-encoded.
pub fn create_signed_artifact(
    name: &str,
    artifact_bytes: &[u8],
    pkcs8_private_key: &[u8],
) -> SignedArtifact {
    todo!("Create a complete signed artifact")
}

/// Exercise 5: Verify a signed artifact end-to-end.
///
/// Given a `SignedArtifact`, verify:
/// 1. The signature is valid for the digest and public key
/// 2. The digest is 32 bytes (SHA-256 output length)
/// 3. The public key hex decodes to 32 bytes (Ed25519 public key length)
///
/// Return Ok(()) if valid, Err with description if any check fails.
pub fn verify_signed_artifact(artifact: &SignedArtifact) -> Result<(), String> {
    todo!("Verify a signed artifact end-to-end")
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::digest;

    fn sha256(data: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA256, data).as_ref().to_vec()
    }

    #[test]
    fn test_generate_keypair() {
        let (privkey, pubkey) = generate_signing_keypair();
        assert!(!privkey.is_empty(), "Private key should not be empty");
        assert!(!pubkey.is_empty(), "Public key should not be empty");
        assert_ne!(privkey, pubkey, "Private and public keys should differ");
    }

    #[test]
    fn test_sign_and_verify() {
        let (privkey, pubkey) = generate_signing_keypair();
        let digest = sha256(b"release artifact v1.0.0");
        let sig = sign_digest(&privkey, &digest);
        assert!(verify_signature(&pubkey, &digest, &sig));
    }

    #[test]
    fn test_verify_wrong_key_fails() {
        let (privkey1, _) = generate_signing_keypair();
        let (_, pubkey2) = generate_signing_keypair();
        let digest = sha256(b"artifact");
        let sig = sign_digest(&privkey1, &digest);
        assert!(!verify_signature(&pubkey2, &digest, &sig));
    }

    #[test]
    fn test_verify_tampered_digest_fails() {
        let (privkey, pubkey) = generate_signing_keypair();
        let digest = sha256(b"original");
        let sig = sign_digest(&privkey, &digest);
        let tampered = sha256(b"tampered");
        assert!(!verify_signature(&pubkey, &tampered, &sig));
    }

    #[test]
    fn test_create_signed_artifact() {
        let (privkey, pubkey) = generate_signing_keypair();
        let artifact = create_signed_artifact("myapp-1.0.0", b"binary content here", &privkey);
        assert_eq!(artifact.name, "myapp-1.0.0");
        assert_eq!(artifact.digest.len(), 32);
        assert!(!artifact.signature.is_empty());
        assert!(!artifact.public_key_hex.is_empty());
        // Verify the public key hex matches
        assert_eq!(artifact.public_key_hex, hex::encode(&pubkey));
    }

    #[test]
    fn test_verify_signed_artifact_valid() {
        let (privkey, _) = generate_signing_keypair();
        let artifact = create_signed_artifact("myapp", b"binary", &privkey);
        assert!(verify_signed_artifact(&artifact).is_ok());
    }

    #[test]
    fn test_verify_signed_artifact_tampered() {
        let (privkey, _) = generate_signing_keypair();
        let mut artifact = create_signed_artifact("myapp", b"binary", &privkey);
        artifact.digest[0] ^= 0xFF; // Tamper with digest
        assert!(verify_signed_artifact(&artifact).is_err());
    }

    #[test]
    fn test_signature_is_deterministic() {
        let (privkey, _) = generate_signing_keypair();
        let digest = sha256(b"deterministic test");
        let sig1 = sign_digest(&privkey, &digest);
        let sig2 = sign_digest(&privkey, &digest);
        assert_eq!(sig1, sig2, "Ed25519 signatures should be deterministic");
    }
}
