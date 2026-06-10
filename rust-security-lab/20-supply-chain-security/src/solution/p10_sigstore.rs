//! # Lesson 10: Sigstore Concepts for Release Signing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

/// Compute a SHA-256 hash of an artifact.
pub fn hash_artifact(artifact: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(artifact);
    hex::encode(hasher.finalize())
}

/// Generate a simulated ephemeral key pair.
pub fn generate_ephemeral_keypair(
    identity: &SigningIdentity,
    not_before: &str,
    not_after: &str,
) -> EphemeralKeyPair {
    let mut pub_hasher = Sha256::new();
    pub_hasher.update(format!("{}{}{}", identity.email, not_before, not_after).as_bytes());
    let public_key = pub_hasher.finalize().to_vec();

    let mut priv_hasher = Sha256::new();
    priv_hasher.update(format!("private:{}{}{}", identity.email, not_before, not_after).as_bytes());
    let private_key = priv_hasher.finalize().to_vec();

    EphemeralKeyPair {
        public_key,
        private_key,
        not_before: not_before.to_string(),
        not_after: not_after.to_string(),
    }
}

/// Create a signing certificate from identity and key pair.
pub fn create_certificate(
    identity: &SigningIdentity,
    keypair: &EphemeralKeyPair,
) -> SigningCertificate {
    let mut hasher = Sha256::new();
    hasher.update(
        format!(
            "{}{}{}{}{}",
            identity.issuer,
            identity.subject,
            hex::encode(&keypair.public_key),
            keypair.not_before,
            keypair.not_after
        )
        .as_bytes(),
    );
    let certificate_hash = hex::encode(hasher.finalize());

    SigningCertificate {
        identity: identity.clone(),
        public_key: keypair.public_key.clone(),
        certificate_hash,
        not_before: keypair.not_before.clone(),
        not_after: keypair.not_after.clone(),
    }
}

/// Sign an artifact (simplified simulation).
pub fn sign_artifact(
    artifact: &[u8],
    keypair: &EphemeralKeyPair,
    identity: &SigningIdentity,
) -> Signature {
    let artifact_hash = hash_artifact(artifact);

    let mut sig_hasher = Sha256::new();
    sig_hasher.update(
        format!(
            "sign:{}{}",
            hex::encode(&keypair.private_key),
            artifact_hash
        )
        .as_bytes(),
    );
    let signature_bytes = sig_hasher.finalize().to_vec();

    let certificate = create_certificate(identity, keypair);

    Signature {
        artifact_hash,
        signature_bytes,
        certificate,
    }
}

/// Verify a signature.
pub fn verify_signature(
    artifact: &[u8],
    signature: &Signature,
) -> bool {
    // Verify artifact hash matches
    let computed_hash = hash_artifact(artifact);
    if computed_hash != signature.artifact_hash {
        return false;
    }

    // Recompute expected signature
    // We need the private key, which we derive from the certificate info
    let identity = &signature.certificate.identity;
    let not_before = &signature.certificate.not_before;
    let not_after = &signature.certificate.not_after;
    let keypair = generate_ephemeral_keypair(identity, not_before, not_after);

    let mut sig_hasher = Sha256::new();
    sig_hasher.update(
        format!(
            "sign:{}{}",
            hex::encode(&keypair.private_key),
            computed_hash
        )
        .as_bytes(),
    );
    let expected_sig = sig_hasher.finalize().to_vec();

    expected_sig == signature.signature_bytes
}

/// Create a transparency log entry for a signature.
pub fn create_log_entry(
    signature: Signature,
    log_index: u64,
    integrated_time: &str,
) -> LogEntry {
    let mut hasher = Sha256::new();
    hasher.update(
        format!(
            "{}{}{}{}",
            log_index,
            integrated_time,
            signature.artifact_hash,
            hex::encode(&signature.signature_bytes)
        )
        .as_bytes(),
    );
    let entry_hash = hex::encode(hasher.finalize());

    LogEntry {
        log_index,
        integrated_time: integrated_time.to_string(),
        signature,
        entry_hash,
        verified: true,
    }
}

/// Verify a log entry's integrity.
pub fn verify_log_entry(entry: &LogEntry) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(
        format!(
            "{}{}{}{}",
            entry.log_index,
            entry.integrated_time,
            entry.signature.artifact_hash,
            hex::encode(&entry.signature.signature_bytes)
        )
        .as_bytes(),
    );
    let expected_hash = hex::encode(hasher.finalize());
    expected_hash == entry.entry_hash
}

/// Generate a verification report.
pub fn verification_report(
    artifact: &[u8],
    entry: &LogEntry,
) -> String {
    let artifact_hash = hash_artifact(artifact);
    let valid = verify_signature(artifact, &entry.signature);
    let identity = &entry.signature.certificate.identity;

    format!(
        "SIGSTORE VERIFICATION REPORT\n\
         Artifact Hash: {}\n\
         Signed By: {}\n\
         Issuer: {}\n\
         Valid: {}\n\
         Transparency Log: entry #{} at {}\n\
         Entry Verified: {}",
        artifact_hash,
        identity.email,
        identity.issuer,
        if valid { "yes" } else { "no" },
        entry.log_index,
        entry.integrated_time,
        if entry.verified { "yes" } else { "no" }
    )
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
        entry.log_index = 43;
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
