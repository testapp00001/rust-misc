//! # Lesson 01: Reproducible Builds (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use serde::{Deserialize, Serialize};

/// A build manifest recording all inputs that affect the build output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildManifest {
    pub source_hash: String,
    pub toolchain_version: String,
    pub target_triple: String,
    pub profile: String,
    pub lockfile_hash: String,
    pub env_overrides: Vec<(String, String)>,
}

/// Compute a deterministic build fingerprint from a manifest.
///
/// Serializes the manifest to JSON and hashes it with SHA-256.
/// The JSON serialization of serde is deterministic for the same data,
/// so the same manifest always produces the same fingerprint.
pub fn compute_build_fingerprint(manifest: &BuildManifest) -> String {
    let json = serde_json::to_string(manifest).expect("Failed to serialize manifest");
    let hash = digest::digest(&digest::SHA256, json.as_bytes());
    hex::encode(hash.as_ref())
}

/// Compare two build manifests by fingerprint.
pub fn manifests_match(a: &BuildManifest, b: &BuildManifest) -> bool {
    compute_build_fingerprint(a) == compute_build_fingerprint(b)
}

/// Audit a build manifest for non-reproducibility issues.
///
/// Checks for empty source hash, nightly toolchain, debug profile,
/// and RUSTFLAGS without --remap-path-prefix.
pub fn audit_manifest(manifest: &BuildManifest) -> Vec<String> {
    let mut problems = Vec::new();

    if manifest.source_hash.is_empty() {
        problems.push("source_hash is empty — cannot verify what was built".to_string());
    }

    if manifest.toolchain_version.is_empty() || manifest.toolchain_version == "nightly" {
        problems.push(format!(
            "toolchain_version '{}' is non-deterministic — pin a specific version",
            manifest.toolchain_version
        ));
    }

    if manifest.profile == "debug" {
        problems.push("profile is 'debug' — release builds are more likely reproducible".to_string());
    }

    for (key, value) in &manifest.env_overrides {
        if key == "RUSTFLAGS" && !value.contains("--remap-path-prefix") {
            problems.push(format!(
                "RUSTFLAGS '{}' missing --remap-path-prefix — paths will vary between machines",
                value
            ));
        }
    }

    problems
}

/// Normalize a build manifest for consistent comparison.
///
/// Sorts env_overrides by key and lowercases hex hashes.
pub fn normalize_manifest(manifest: &BuildManifest) -> BuildManifest {
    let mut normalized = manifest.clone();
    normalized.source_hash = normalized.source_hash.to_lowercase();
    normalized.lockfile_hash = normalized.lockfile_hash.to_lowercase();
    normalized.env_overrides.sort_by(|a, b| a.0.cmp(&b.0));
    normalized
}

/// Create a build manifest from raw inputs.
///
/// Hashes source_files and lockfile_content with SHA-256 to produce hex digests.
pub fn create_manifest(
    source_files: &[u8],
    lockfile_content: &[u8],
    toolchain_version: &str,
    target_triple: &str,
    profile: &str,
    env_overrides: Vec<(String, String)>,
) -> BuildManifest {
    let source_hash = hex::encode(
        digest::digest(&digest::SHA256, source_files).as_ref()
    );
    let lockfile_hash = hex::encode(
        digest::digest(&digest::SHA256, lockfile_content).as_ref()
    );
    BuildManifest {
        source_hash,
        toolchain_version: toolchain_version.to_string(),
        target_triple: target_triple.to_string(),
        profile: profile.to_string(),
        lockfile_hash,
        env_overrides,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> BuildManifest {
        BuildManifest {
            source_hash: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
            toolchain_version: "1.75.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            profile: "release".to_string(),
            lockfile_hash: "b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3".to_string(),
            env_overrides: vec![],
        }
    }

    #[test]
    fn test_build_fingerprint_deterministic() {
        let manifest = sample_manifest();
        let fp1 = compute_build_fingerprint(&manifest);
        let fp2 = compute_build_fingerprint(&manifest);
        assert_eq!(fp1, fp2, "Same manifest should produce same fingerprint");
    }

    #[test]
    fn test_build_fingerprint_is_hex() {
        let manifest = sample_manifest();
        let fp = compute_build_fingerprint(&manifest);
        assert_eq!(fp.len(), 64, "SHA-256 hex should be 64 chars");
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_different_manifests_different_fingerprints() {
        let m1 = sample_manifest();
        let mut m2 = sample_manifest();
        m2.source_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        assert_ne!(compute_build_fingerprint(&m1), compute_build_fingerprint(&m2));
    }

    #[test]
    fn test_manifests_match_true() {
        let m1 = sample_manifest();
        let m2 = sample_manifest();
        assert!(manifests_match(&m1, &m2));
    }

    #[test]
    fn test_manifests_match_false() {
        let m1 = sample_manifest();
        let mut m2 = sample_manifest();
        m2.toolchain_version = "1.74.0".to_string();
        assert!(!manifests_match(&m1, &m2));
    }

    #[test]
    fn test_audit_clean_manifest() {
        let manifest = sample_manifest();
        let problems = audit_manifest(&manifest);
        assert!(problems.is_empty(), "Clean manifest should have no problems, got: {:?}", problems);
    }

    #[test]
    fn test_audit_empty_source_hash() {
        let mut manifest = sample_manifest();
        manifest.source_hash = String::new();
        let problems = audit_manifest(&manifest);
        assert!(problems.iter().any(|p| p.contains("source_hash") || p.contains("source")));
    }

    #[test]
    fn test_audit_nightly_toolchain() {
        let mut manifest = sample_manifest();
        manifest.toolchain_version = "nightly".to_string();
        let problems = audit_manifest(&manifest);
        assert!(problems.iter().any(|p| p.contains("toolchain") || p.contains("nightly")));
    }

    #[test]
    fn test_audit_debug_profile() {
        let mut manifest = sample_manifest();
        manifest.profile = "debug".to_string();
        let problems = audit_manifest(&manifest);
        assert!(problems.iter().any(|p| p.contains("profile") || p.contains("debug")));
    }

    #[test]
    fn test_audit_rustflags_no_remap() {
        let mut manifest = sample_manifest();
        manifest.env_overrides = vec![
            ("RUSTFLAGS".to_string(), "-C opt-level=3".to_string()),
        ];
        let problems = audit_manifest(&manifest);
        assert!(problems.iter().any(|p| p.contains("remap") || p.contains("RUSTFLAGS")));
    }

    #[test]
    fn test_normalize_sorts_env_overrides() {
        let mut manifest = sample_manifest();
        manifest.env_overrides = vec![
            ("Z_VAR".to_string(), "z".to_string()),
            ("A_VAR".to_string(), "a".to_string()),
            ("M_VAR".to_string(), "m".to_string()),
        ];
        let normalized = normalize_manifest(&manifest);
        assert_eq!(normalized.env_overrides[0].0, "A_VAR");
        assert_eq!(normalized.env_overrides[1].0, "M_VAR");
        assert_eq!(normalized.env_overrides[2].0, "Z_VAR");
    }

    #[test]
    fn test_normalize_lowercase_hashes() {
        let mut manifest = sample_manifest();
        manifest.source_hash = manifest.source_hash.to_uppercase();
        let normalized = normalize_manifest(&manifest);
        assert!(normalized.source_hash.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_create_manifest_deterministic() {
        let source = b"fn main() {}";
        let lockfile = b"[[package]]\nname = \"test\"";
        let m1 = create_manifest(source, lockfile, "1.75.0", "x86_64-unknown-linux-gnu", "release", vec![]);
        let m2 = create_manifest(source, lockfile, "1.75.0", "x86_64-unknown-linux-gnu", "release", vec![]);
        assert_eq!(m1, m2, "Same inputs should produce same manifest");
    }

    #[test]
    fn test_create_manifest_different_source() {
        let lockfile = b"lockfile";
        let m1 = create_manifest(b"source A", lockfile, "1.75.0", "x86_64-unknown-linux-gnu", "release", vec![]);
        let m2 = create_manifest(b"source B", lockfile, "1.75.0", "x86_64-unknown-linux-gnu", "release", vec![]);
        assert_ne!(m1.source_hash, m2.source_hash);
    }
}
