//! # Lesson 01: Reproducible Builds
//!
//! ## Attack: Non-Reproducible Build Poisoning
//!
//! An attacker compromises a build server. Because builds are not reproducible,
//! nobody can tell whether the binary on the release page was built from the
//! official source code or from a backdoored version. The attacker's trojanized
//! binary ships to thousands of users.
//!
//! This is exactly what happened in the **SolarWinds** attack (2020): the build
//! system was compromised, and malicious code was injected during compilation.
//!
//! ## Defend: Deterministic Compilation
//!
//! A reproducible build ensures that the same source code + same build environment
//! always produces bit-identical output. Any deviation means tampering.
//!
//! ## Audit: What to Check
//!
//! 1. Are timestamps stripped or fixed?
//! 2. Are build paths remapped to a canonical prefix?
//! 3. Is the toolchain version pinned?
//! 4. Is `Cargo.lock` committed and used with `--locked`?
//! 5. Can two independent builders produce the same hash?

use serde::{Deserialize, Serialize};

/// A build manifest recording all inputs that affect the build output.
///
/// If any field changes, the build output should change too.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildManifest {
    /// Source code hash (SHA-256 of the git tree)
    pub source_hash: String,
    /// Rust toolchain version (e.g., "1.75.0")
    pub toolchain_version: String,
    /// Target triple (e.g., "x86_64-unknown-linux-gnu")
    pub target_triple: String,
    /// Build profile ("debug" or "release")
    pub profile: String,
    /// Cargo.lock hash
    pub lockfile_hash: String,
    /// Environment variable overrides that affect codegen
    pub env_overrides: Vec<(String, String)>,
}

/// Exercise 1: Compute a deterministic build fingerprint.
///
/// Given a `BuildManifest`, produce a single SHA-256 hash that uniquely identifies
/// this exact build configuration. The hash must be deterministic: the same manifest
/// always produces the same hash.
///
/// The hash should be computed over a canonical JSON representation of the manifest.
///
/// Hints:
/// - Serialize the manifest to JSON with `serde_json::to_string`
/// - The JSON serialization of serde is deterministic for the same data
/// - Hash the JSON bytes with SHA-256
/// - Return the hash as a hex string
pub fn compute_build_fingerprint(manifest: &BuildManifest) -> String {
    todo!("Compute deterministic build fingerprint from manifest")
}

/// Exercise 2: Verify that two build manifests are identical.
///
/// Two manifests are identical if their build fingerprints match.
///
/// Hints:
/// - Use `compute_build_fingerprint` on both manifests
/// - Compare the resulting hashes
pub fn manifests_match(a: &BuildManifest, b: &BuildManifest) -> bool {
    todo!("Compare two build manifests by fingerprint")
}

/// Exercise 3: Detect non-reproducible fields in a build manifest.
///
/// A "non-reproducible field" is one that varies between builds of the same source code
/// when it should not. Check for these problems:
///
/// 1. `source_hash` is empty
/// 2. `toolchain_version` is empty or "nightly" (nightly is non-deterministic)
/// 3. `profile` is "debug" (release builds are more likely reproducible)
/// 4. `env_overrides` contains RUSTFLAGS without `--remap-path-prefix`
///
/// Return a list of problem descriptions. Empty list means the manifest looks good.
pub fn audit_manifest(manifest: &BuildManifest) -> Vec<String> {
    todo!("Audit build manifest for non-reproducibility issues")
}

/// Exercise 4: Normalize a build manifest for comparison.
///
/// Remove or canonicalize fields that vary between builds but don't affect the output:
/// - Sort `env_overrides` alphabetically by key
/// - Ensure `source_hash` and `lockfile_hash` are lowercase hex
///
/// Return a new manifest with normalized fields.
pub fn normalize_manifest(manifest: &BuildManifest) -> BuildManifest {
    todo!("Normalize manifest fields for consistent comparison")
}

/// Exercise 5: Create a build manifest from raw inputs.
///
/// This is a convenience constructor that computes the hashes itself.
/// `source_files` and `lockfile_content` should be hashed with SHA-256.
///
/// Hints:
/// - Hash each input with `ring::digest::digest(&ring::digest::SHA256, data)`
/// - Convert to hex string
pub fn create_manifest(
    source_files: &[u8],
    lockfile_content: &[u8],
    toolchain_version: &str,
    target_triple: &str,
    profile: &str,
    env_overrides: Vec<(String, String)>,
) -> BuildManifest {
    todo!("Create a BuildManifest from raw inputs")
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
