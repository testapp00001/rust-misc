//! # Lesson 05: Build Attestation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SlsaLevel {
    Level0 = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildAttestation {
    pub slsa_level: SlsaLevel,
    pub builder_id: String,
    pub source_uri: String,
    pub source_commit: String,
    pub artifact_digest: String,
    pub build_invocation: String,
    pub timestamp: String,
    pub hermetic: bool,
    pub reproducible: bool,
}

/// Validate build attestation fields.
pub fn validate_attestation(att: &BuildAttestation) -> Vec<String> {
    let mut issues = Vec::new();

    if att.builder_id.is_empty() {
        issues.push("builder_id is empty".to_string());
    }

    if att.source_uri.is_empty() || !att.source_uri.starts_with("https://") {
        issues.push(format!("source_uri '{}' must start with https://", att.source_uri));
    }

    if att.source_commit.len() != 40 || !att.source_commit.chars().all(|c| c.is_ascii_hexdigit()) {
        issues.push(format!(
            "source_commit '{}' is not a valid 40-char hex SHA",
            att.source_commit
        ));
    }

    if att.artifact_digest.len() != 64 || !att.artifact_digest.chars().all(|c| c.is_ascii_hexdigit()) {
        issues.push(format!(
            "artifact_digest '{}' is not a valid 64-char hex SHA-256",
            att.artifact_digest
        ));
    }

    if att.timestamp.is_empty() {
        issues.push("timestamp is empty".to_string());
    }

    issues
}

/// Check if an attestation meets minimum SLSA requirements.
pub fn meets_slsa_requirement(att: &BuildAttestation, required: SlsaLevel) -> bool {
    if att.slsa_level < required {
        return false;
    }

    if required >= SlsaLevel::Level2 && !att.builder_id.starts_with("https://") {
        return false;
    }

    if required >= SlsaLevel::Level3 && !att.hermetic {
        return false;
    }

    if required >= SlsaLevel::Level4 && !att.reproducible {
        return false;
    }

    true
}

/// Verify artifact matches attestation digest.
pub fn verify_artifact_provenance(att: &BuildAttestation, artifact_bytes: &[u8]) -> bool {
    let computed = digest::digest(&digest::SHA256, artifact_bytes);
    let computed_hex = hex::encode(computed.as_ref());
    att.artifact_digest == computed_hex
}

/// Generate a build attestation.
pub fn generate_attestation(
    slsa_level: SlsaLevel,
    builder_id: &str,
    source_uri: &str,
    source_commit: &str,
    artifact_bytes: &[u8],
    build_invocation: &str,
    timestamp: &str,
) -> BuildAttestation {
    let artifact_digest = hex::encode(
        digest::digest(&digest::SHA256, artifact_bytes).as_ref()
    );

    BuildAttestation {
        slsa_level,
        builder_id: builder_id.to_string(),
        source_uri: source_uri.to_string(),
        source_commit: source_commit.to_string(),
        artifact_digest,
        build_invocation: build_invocation.to_string(),
        timestamp: timestamp.to_string(),
        hermetic: slsa_level >= SlsaLevel::Level3,
        reproducible: slsa_level >= SlsaLevel::Level4,
    }
}

/// Compare two attestations and list differing fields (excluding timestamp).
pub fn diff_attestations(a: &BuildAttestation, b: &BuildAttestation) -> Vec<String> {
    let mut diffs = Vec::new();

    if a.slsa_level != b.slsa_level { diffs.push("slsa_level".to_string()); }
    if a.builder_id != b.builder_id { diffs.push("builder_id".to_string()); }
    if a.source_uri != b.source_uri { diffs.push("source_uri".to_string()); }
    if a.source_commit != b.source_commit { diffs.push("source_commit".to_string()); }
    if a.artifact_digest != b.artifact_digest { diffs.push("artifact_digest".to_string()); }
    if a.build_invocation != b.build_invocation { diffs.push("build_invocation".to_string()); }
    if a.hermetic != b.hermetic { diffs.push("hermetic".to_string()); }
    if a.reproducible != b.reproducible { diffs.push("reproducible".to_string()); }

    diffs
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
        let artifact = b"test artifact";
        let digest_hex = hex::encode(
            digest::digest(&digest::SHA256, artifact).as_ref()
        );
        let mut att = sample_attestation();
        att.artifact_digest = digest_hex;
        assert!(verify_artifact_provenance(&att, artifact));
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
        let expected_digest = hex::encode(
            digest::digest(&digest::SHA256, artifact).as_ref()
        );
        assert_eq!(att.artifact_digest, expected_digest);
    }

    #[test]
    fn test_diff_identical_attestations() {
        let a = sample_attestation();
        let mut b = sample_attestation();
        b.timestamp = "different time".to_string();
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
