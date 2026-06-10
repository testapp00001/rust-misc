//! # Lesson 10: Sigstore Concepts for Release Signing
//!
//! ## What is Sigstore?
//!
//! [Sigstore](https://www.sigstore.dev/) is a set of tools and services for signing,
//! verifying, and protecting software supply chains. It solves the key management
//! problem: developers no longer need to manage long-lived signing keys.
//!
//! ## How Sigstore Works
//!
//! 1. **Identity-based signing**: Sign with your identity (email/OIDC) instead of a key
//! 2. **Ephemeral keys**: Generate a short-lived key pair, sign, then discard
//! 3. **Transparency log**: Every signature is recorded in a public, append-only log (Rekor)
//! 4. **Certificate authority**: Fulcio issues short-lived certificates binding identity to keys
//!
//! ## Key Components
//!
//! | Component | Purpose |
//! |-----------|---------|
//! | **Fulcio** | Certificate authority for ephemeral signing certificates |
//! | **Rekor** | Transparency log for signature records |
//! | **Cosign** | CLI tool for signing and verifying container images and files |
//! | **OIDC** | OpenID Connect for identity verification |
//!
//! ## Attack: Signature Expiry Bypass
//!
//! If a signing key is compromised and no transparency log exists, an attacker
//! can create signatures that appear legitimate. Sigstore's transparency log
//! makes every signature publicly auditable.
//!
//! ## Defense: Signature and Transparency Verification
//!
//! In this lesson, you will implement signature creation, verification,
//! and transparency log concepts using SHA-256 and basic cryptographic primitives.

use ring::digest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A signing identity (simplified OIDC representation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningIdentity {
    pub issuer: String,
    pub subject: String,
    pub email: String,
}

/// An ephemeral signing key pair (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub not_before: String,
    pub not_after: String,
}

/// A signing certificate binding identity to a key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningCertificate {
    pub identity: SigningIdentity,
    pub public_key: Vec<u8>,
    pub certificate_hash: String,
    pub not_before: String,
    pub not_after: String,
}

/// A signature over an artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub artifact_hash: String,
    pub signature_bytes: Vec<u8>,
    pub certificate: SigningCertificate,
}

/// A transparency log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub log_index: u64,
    pub integrated_time: String,
    pub signature: Signature,
    pub entry_hash: String,
    pub verified: bool,
}

/// Exercise 1: Compute a SHA-256 hash of an artifact.
///
/// Return lowercase hex string.
///
/// Hints:
/// - Use `Sha256` from `sha2`
/// - Feed bytes, finalize, hex-encode
pub fn hash_artifact(artifact: &[u8]) -> String {
    todo!("Compute SHA-256 hash of artifact")
}

/// Exercise 2: Generate a simulated ephemeral key pair.
///
/// For this exercise, "generate" a key pair by:
/// - public_key = SHA-256 of (identity.email + not_before + not_after) as bytes
/// - private_key = SHA-256 of ("private:" + identity.email + not_before + not_after)
///
/// This simulates the concept — real Sigstore uses ECDSA or Ed25519.
///
/// Hints:
/// - Concatenate the strings and hash
/// - Use the hash output as the "key" bytes
pub fn generate_ephemeral_keypair(
    identity: &SigningIdentity,
    not_before: &str,
    not_after: &str,
) -> EphemeralKeyPair {
    todo!("Generate simulated ephemeral key pair")
}

/// Exercise 3: Create a signing certificate from identity and key pair.
///
/// The certificate_hash is SHA-256 of:
/// identity.issuer + identity.subject + public_key hex + not_before + not_after
///
/// Hints:
/// - Concatenate the fields
/// - Hash the concatenation
pub fn create_certificate(
    identity: &SigningIdentity,
    keypair: &EphemeralKeyPair,
) -> SigningCertificate {
    todo!("Create signing certificate")
}

/// Exercise 4: Sign an artifact (simplified simulation).
///
/// For this simulation, the "signature" is SHA-256 of:
/// "sign:" + private_key_hex + artifact_hash
///
/// This demonstrates the concept — real signing uses asymmetric crypto.
///
/// Hints:
/// - Hash the artifact first
/// - Concatenate "sign:" + hex(private_key) + artifact_hash
/// - Hash the concatenation
pub fn sign_artifact(
    artifact: &[u8],
    keypair: &EphemeralKeyPair,
    identity: &SigningIdentity,
) -> Signature {
    todo!("Sign an artifact (simulated)")
}

/// Exercise 5: Verify a signature.
///
/// For this simulation, re-compute the expected signature and compare.
/// Also verify the artifact hash matches.
///
/// Return true if the signature is valid.
///
/// Hints:
/// - Recompute the expected signature using the same algorithm as `sign_artifact`
/// - Compare the recomputed signature bytes with the stored ones
/// - Also verify the artifact hash matches
pub fn verify_signature(
    artifact: &[u8],
    signature: &Signature,
) -> bool {
    todo!("Verify a signature")
}

/// Exercise 6: Create a transparency log entry for a signature.
///
/// The entry_hash is SHA-256 of:
/// log_index (as string) + integrated_time + artifact_hash + signature hex
///
/// Hints:
/// - Format log_index as string
/// - Concatenate all fields
/// - Hash the concatenation
pub fn create_log_entry(
    signature: Signature,
    log_index: u64,
    integrated_time: &str,
) -> LogEntry {
    todo!("Create transparency log entry")
}

/// Exercise 7: Verify a log entry's integrity.
///
/// Recompute the entry_hash and compare with the stored one.
///
/// Hints:
/// - Use the same formula as `create_log_entry`
/// - Compare hashes
pub fn verify_log_entry(entry: &LogEntry) -> bool {
    todo!("Verify log entry integrity")
}

/// Exercise 8: Generate a verification report.
///
/// Format:
/// ```text
/// SIGSTORE VERIFICATION REPORT
/// Artifact Hash: {hash}
/// Signed By: {email}
/// Issuer: {issuer}
/// Valid: {yes/no}
/// Transparency Log: entry #{index} at {time}
/// Entry Verified: {yes/no}
/// ```
pub fn verification_report(
    artifact: &[u8],
    entry: &LogEntry,
) -> String {
    todo!("Generate Sigstore verification report")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_identity() -> SigningIdentity {
        SigningIdentity {
            issuer: "https://accounts.google.com".to_string(),
            subject: "123456789".to_string(),
            email: "developer@example.com".to_string(),
        }
    }

    #[test]
    fn test_hash_artifact() {
        let hash = hash_artifact(b"test binary");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, hash_artifact(b"test binary"));
    }

    #[test]
    fn test_hash_artifact_different() {
        assert_ne!(hash_artifact(b"a"), hash_artifact(b"b"));
    }

    #[test]
    fn test_generate_ephemeral_keypair() {
        let identity = sample_identity();
        let kp = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        assert!(!kp.public_key.is_empty());
        assert!(!kp.private_key.is_empty());
        assert_ne!(kp.public_key, kp.private_key);
    }

    #[test]
    fn test_keypair_deterministic() {
        let identity = sample_identity();
        let kp1 = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        let kp2 = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        assert_eq!(kp1.public_key, kp2.public_key);
        assert_eq!(kp1.private_key, kp2.private_key);
    }

    #[test]
    fn test_create_certificate() {
        let identity = sample_identity();
        let kp = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        let cert = create_certificate(&identity, &kp);
        assert_eq!(cert.identity.email, "developer@example.com");
        assert!(!cert.certificate_hash.is_empty());
    }

    #[test]
    fn test_sign_and_verify() {
        let identity = sample_identity();
        let kp = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        let artifact = b"my release binary";
        let sig = sign_artifact(artifact, &kp, &identity);
        assert!(verify_signature(artifact, &sig));
    }

    #[test]
    fn test_verify_tampered_artifact() {
        let identity = sample_identity();
        let kp = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        let sig = sign_artifact(b"original", &kp, &identity);
        assert!(!verify_signature(b"tampered", &sig));
    }

    #[test]
    fn test_create_and_verify_log_entry() {
        let identity = sample_identity();
        let kp = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        let sig = sign_artifact(b"release", &kp, &identity);
        let entry = create_log_entry(sig, 42, "2024-01-01T12:00:00Z");
        assert_eq!(entry.log_index, 42);
        assert!(verify_log_entry(&entry));
    }

    #[test]
    fn test_log_entry_tampered() {
        let identity = sample_identity();
        let kp = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        let sig = sign_artifact(b"release", &kp, &identity);
        let mut entry = create_log_entry(sig, 42, "2024-01-01T12:00:00Z");
        entry.log_index = 43; // tamper
        assert!(!verify_log_entry(&entry));
    }

    #[test]
    fn test_verification_report() {
        let identity = sample_identity();
        let kp = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        let sig = sign_artifact(b"release", &kp, &identity);
        let entry = create_log_entry(sig, 1, "2024-01-01T12:00:00Z");
        let report = verification_report(b"release", &entry);
        assert!(report.contains("SIGSTORE VERIFICATION REPORT"));
        assert!(report.contains("developer@example.com"));
        assert!(report.contains("entry #1"));
    }

    #[test]
    fn test_signature_serialization() {
        let identity = sample_identity();
        let kp = generate_ephemeral_keypair(&identity, "2024-01-01", "2024-01-02");
        let sig = sign_artifact(b"test", &kp, &identity);
        let json = serde_json::to_string(&sig).unwrap();
        let restored: Signature = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.artifact_hash, sig.artifact_hash);
    }
}
