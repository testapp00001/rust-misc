//! # Lesson 05: Build Attestation
//!
//! ## Attack: Forged Build Metadata
//!
//! An attacker compromises your build server. They build a backdoored binary but
//! generate "official-looking" build metadata claiming it came from the clean source.
//! Without cryptographic attestation, there's no way to verify the metadata is truthful.
//!
//! ## Defend: SLSA Build Attestation
//!
//! SLSA (Supply-chain Levels for Software Artifacts) uses cryptographic attestation
//! to prove:
//! - **What** was built (source repo, commit, build instructions)
//! - **Who** built it (builder identity)
//! - **How** it was built (build platform, environment)
//!
//! The attestation is signed by the build platform, making it non-falsifiable by
//! individual developers or compromised build scripts.
//!
//! ## Audit: Attestation Verification
//!
//! 1. Verify the attestation signature
//! 2. Check that the builder is trusted
//! 3. Verify the source repository and commit match expectations
//! 4. Check the SLSA level meets requirements
//! 5. Verify the artifact hash matches

use serde::{Deserialize, Serialize};

/// SLSA build levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SlsaLevel {
    /// No guarantees
    Level0 = 0,
    /// Build process is documented
    Level1 = 1,
    /// Hosted build platform, signed provenance
    Level2 = 2,
    /// Hardened build platform, non-falsifiable provenance
    Level3 = 3,
    /// Hermetic, reproducible, two-person review
    Level4 = 4,
}

/// A build provenance attestation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildAttestation {
    /// SLSA level achieved
    pub slsa_level: SlsaLevel,
    /// Identity of the builder (e.g., "https://github.com/actions/runner")
    pub builder_id: String,
    /// Source repository URL
    pub source_uri: String,
    /// Git commit SHA
    pub source_commit: String,
    /// SHA-256 digest of the build artifact
    pub artifact_digest: String,
    /// Build invocation parameters
    pub build_invocation: String,
    /// Timestamp of the build (ISO 8601)
    pub timestamp: String,
    /// Whether the build was hermetic (no network access)
    pub hermetic: bool,
    /// Whether the build was reproducible
    pub reproducible: bool,
}

/// Exercise 1: Validate a build attestation.
///
/// Check for these issues:
/// 1. `builder_id` is empty
/// 2. `source_uri` is empty or doesn't start with "https://"
/// 3. `source_commit` is not a valid 40-character hex string
/// 4. `artifact_digest` is not a valid 64-character hex string
/// 5. `timestamp` is empty
///
/// Return a list of issue descriptions. Empty = valid.
pub fn validate_attestation(att: &BuildAttestation) -> Vec<String> {
    todo!("Validate build attestation fields")
}

/// Exercise 2: Check if an attestation meets minimum SLSA requirements.
///
/// Given a required minimum SLSA level, return true if the attestation meets it.
/// Also verify:
/// - For Level 2+: builder_id must start with "https://"
/// - For Level 3+: hermetic must be true
/// - For Level 4+: reproducible must be true
pub fn meets_slsa_requirement(att: &BuildAttestation, required: SlsaLevel) -> bool {
    todo!("Check SLSA level requirements")
}

/// Exercise 3: Verify artifact provenance.
///
/// Given an attestation and the actual artifact bytes:
/// 1. Compute SHA-256 of the artifact bytes
/// 2. Convert to hex string
/// 3. Compare with `att.artifact_digest`
///
/// Return true if the artifact matches the attestation.
pub fn verify_artifact_provenance(att: &BuildAttestation, artifact_bytes: &[u8]) -> bool {
    todo!("Verify artifact matches attestation digest")
}

/// Exercise 4: Generate an attestation for an artifact.
///
/// Create a `BuildAttestation` with the given parameters. The `artifact_digest`
/// should be computed as SHA-256 of `artifact_bytes` (hex-encoded).
///
/// Default values:
/// - `hermetic`: true if SLSA level >= 3
/// - `reproducible`: true if SLSA level >= 4
pub fn generate_attestation(
    slsa_level: SlsaLevel,
    builder_id: &str,
    source_uri: &str,
    source_commit: &str,
    artifact_bytes: &[u8],
    build_invocation: &str,
    timestamp: &str,
) -> BuildAttestation {
    todo!("Generate a build attestation")
}

/// Exercise 5: Compare two attestations and find differences.
///
/// Return a list of fields that differ between the two attestations.
/// Each entry should be the field name (e.g., "source_commit", "builder_id").
///
/// Compare all fields except `timestamp` (builds at different times are expected).
pub fn diff_attestations(a: &BuildAttestation, b: &BuildAttestation) -> Vec<String> {
    todo!("Compare two attestations and list differing fields")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_attestation() -> BuildAttestation {
        BuildAttestation {
            slsa_level: SlsaLevel::Level3,
            builder_id: "https://github.com/actions/runner/v2.311.0".to_string(),
            source_uri: "https://github.com/org/repo".to_string(),
            source_commit: "a".repeat(40),
            artifact_digest: "b".repeat(64),
            build_invocation: "cargo build --release --locked".to_string(),
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            hermetic: true,
            reproducible: false,
        }
    }

    #[test]
    fn test_validate_valid_attestation() {
        let att = sample_attestation();
        let issues = validate_attestation(&att);
        assert!(issues.is_empty(), "Valid attestation should pass, got: {:?}", issues);
    }

    #[test]
    fn test_validate_empty_builder() {
        let mut att = sample_attestation();
        att.builder_id = String::new();
        let issues = validate_attestation(&att);
        assert!(issues.iter().any(|i| i.contains("builder")));
    }

    #[test]
    fn test_validate_bad_source_uri() {
        let mut att = sample_attestation();
        att.source_uri = "ftp://example.com".to_string();
        let issues = validate_attestation(&att);
        assert!(issues.iter().any(|i| i.contains("source") || i.contains("https")));
    }

    #[test]
    fn test_validate_bad_commit_hash() {
        let mut att = sample_attestation();
        att.source_commit = "not-a-hash".to_string();
        let issues = validate_attestation(&att);
        assert!(issues.iter().any(|i| i.contains("commit")));
    }

    #[test]
    fn test_meets_slsa_requirement_pass() {
        let att = sample_attestation();
        assert!(meets_slsa_requirement(&att, SlsaLevel::Level1));
        assert!(meets_slsa_requirement(&att, SlsaLevel::Level2));
        assert!(meets_slsa_requirement(&att, SlsaLevel::Level3));
    }

    #[test]
    fn test_meets_slsa_requirement_fail() {
        let att = sample_attestation();
        assert!(!meets_slsa_requirement(&att, SlsaLevel::Level4));
    }

    #[test]
    fn test_verify_artifact_provenance() {
        let att = sample_attestation();
        // The attestation has artifact_digest = "b" * 64
        // We need to provide bytes whose SHA-256 hash equals "b" * 64
        // Instead, let's create an attestation with a known artifact
        let artifact = b"test artifact";
        let digest = ring::digest::digest(&ring::digest::SHA256, artifact);
        let digest_hex = hex::encode(digest.as_ref());
        let mut att2 = sample_attestation();
        att2.artifact_digest = digest_hex;
        assert!(verify_artifact_provenance(&att2, artifact));
    }

    #[test]
    fn test_verify_artifact_provenance_mismatch() {
        let att = sample_attestation();
        assert!(!verify_artifact_provenance(&att, b"different artifact"));
    }

    #[test]
    fn test_generate_attestation() {
        let artifact = b"my binary";
        let att = generate_attestation(
            SlsaLevel::Level3,
            "https://github.com/actions/runner",
            "https://github.com/org/repo",
            &"a".repeat(40),
            artifact,
            "cargo build --release",
            "2024-01-15T10:30:00Z",
        );
        assert_eq!(att.slsa_level, SlsaLevel::Level3);
        assert!(att.hermetic, "Level 3 should require hermetic builds");
        assert!(!att.reproducible, "Level 3 should not require reproducible builds");
        // Verify the digest matches
        let expected_digest = hex::encode(
            ring::digest::digest(&ring::digest::SHA256, artifact).as_ref()
        );
        assert_eq!(att.artifact_digest, expected_digest);
    }

    #[test]
    fn test_diff_identical_attestations() {
        let a = sample_attestation();
        let mut b = sample_attestation();
        b.timestamp = "different time".to_string(); // timestamp is excluded from diff
        let diff = diff_attestations(&a, &b);
        assert!(diff.is_empty(), "Identical attestations (ignoring time) should have no diff");
    }

    #[test]
    fn test_diff_different_source() {
        let a = sample_attestation();
        let mut b = sample_attestation();
        b.source_commit = "f".repeat(40);
        let diff = diff_attestations(&a, &b);
        assert!(diff.contains(&"source_commit".to_string()));
    }
}
