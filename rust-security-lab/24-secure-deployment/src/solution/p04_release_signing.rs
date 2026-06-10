//! # Lesson 04: Release Signing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use ring::signature::{self, Ed25519KeyPair, KeyPair};

/// A signed release artifact.
#[derive(Debug, Clone)]
pub struct SignedArtifact {
    pub name: String,
    pub digest: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key_hex: String,
}

/// Generate an Ed25519 keypair for release signing.
///
/// Returns (pkcs8_private_key_bytes, public_key_bytes).
pub fn generate_signing_keypair() -> (Vec<u8>, Vec<u8>) {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("Failed to generate key");
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("Failed to parse key");
    let pubkey = pair.public_key().as_ref().to_vec();
    (pkcs8.as_ref().to_vec(), pubkey)
}

/// Sign a digest with Ed25519.
pub fn sign_digest(pkcs8_private_key: &[u8], digest: &[u8]) -> Vec<u8> {
    let pair = Ed25519KeyPair::from_pkcs8(pkcs8_private_key).expect("Failed to parse key");
    pair.sign(digest).as_ref().to_vec()
}

/// Verify an Ed25519 signature.
pub fn verify_signature(public_key: &[u8], digest: &[u8], sig: &[u8]) -> bool {
    let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, public_key);
    public_key.verify(digest, sig).is_ok()
}

/// Create a complete signed artifact.
pub fn create_signed_artifact(
    name: &str,
    artifact_bytes: &[u8],
    pkcs8_private_key: &[u8],
) -> SignedArtifact {
    let digest = digest::digest(&digest::SHA256, artifact_bytes);
    let digest_bytes = digest.as_ref().to_vec();
    let sig = sign_digest(pkcs8_private_key, &digest_bytes);

    let pair = Ed25519KeyPair::from_pkcs8(pkcs8_private_key).expect("Failed to parse key");
    let public_key_hex = hex::encode(pair.public_key().as_ref());

    SignedArtifact {
        name: name.to_string(),
        digest: digest_bytes,
        signature: sig,
        public_key_hex,
    }
}

/// Verify a signed artifact end-to-end.
pub fn verify_signed_artifact(artifact: &SignedArtifact) -> Result<(), String> {
    if artifact.digest.len() != 32 {
        return Err(format!("Digest length is {} bytes, expected 32 (SHA-256)", artifact.digest.len()));
    }

    let public_key_bytes = hex::decode(&artifact.public_key_hex)
        .map_err(|_| "Public key hex is not valid hex".to_string())?;
    if public_key_bytes.len() != 32 {
        return Err(format!("Public key is {} bytes, expected 32 (Ed25519)", public_key_bytes.len()));
    }

    if !verify_signature(&public_key_bytes, &artifact.digest, &artifact.signature) {
        return Err("Signature verification failed".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let digest = digest::digest(&digest::SHA256, b"release artifact v1.0.0");
        let digest_bytes = digest.as_ref().to_vec();
        let sig = sign_digest(&privkey, &digest_bytes);
        assert!(verify_signature(&pubkey, &digest_bytes, &sig));
    }

    #[test]
    fn test_verify_wrong_key_fails() {
        let (privkey1, _) = generate_signing_keypair();
        let (_, pubkey2) = generate_signing_keypair();
        let digest = digest::digest(&digest::SHA256, b"artifact");
        let digest_bytes = digest.as_ref().to_vec();
        let sig = sign_digest(&privkey1, &digest_bytes);
        assert!(!verify_signature(&pubkey2, &digest_bytes, &sig));
    }

    #[test]
    fn test_verify_tampered_digest_fails() {
        let (privkey, pubkey) = generate_signing_keypair();
        let digest = digest::digest(&digest::SHA256, b"original");
        let digest_bytes = digest.as_ref().to_vec();
        let sig = sign_digest(&privkey, &digest_bytes);
        let tampered = digest::digest(&digest::SHA256, b"tampered");
        assert!(!verify_signature(&pubkey, tampered.as_ref(), &sig));
    }

    #[test]
    fn test_create_signed_artifact() {
        let (privkey, pubkey) = generate_signing_keypair();
        let artifact = create_signed_artifact("myapp-1.0.0", b"binary content here", &privkey);
        assert_eq!(artifact.name, "myapp-1.0.0");
        assert_eq!(artifact.digest.len(), 32);
        assert!(!artifact.signature.is_empty());
        assert!(!artifact.public_key_hex.is_empty());
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
        artifact.digest[0] ^= 0xFF;
        assert!(verify_signed_artifact(&artifact).is_err());
    }

    #[test]
    fn test_signature_is_deterministic() {
        let (privkey, _) = generate_signing_keypair();
        let digest = digest::digest(&digest::SHA256, b"deterministic test");
        let digest_bytes = digest.as_ref().to_vec();
        let sig1 = sign_digest(&privkey, &digest_bytes);
        let sig2 = sign_digest(&privkey, &digest_bytes);
        assert_eq!(sig1, sig2, "Ed25519 signatures should be deterministic");
    }
}
