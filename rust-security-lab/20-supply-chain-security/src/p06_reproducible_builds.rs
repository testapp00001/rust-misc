//! # Lesson 06: Reproducible Builds Verification
//!
//! ## What Are Reproducible Builds?
//!
//! A build is reproducible if, given the same source code and build environment,
//! it produces bit-for-bit identical output every time. This means:
//!
//! - Same compiler version
//! - Same target platform
//! - Same environment variables (or environment-isolated builds)
//! - Same dependency versions (locked)
//!
//! ## Why Reproducibility Matters
//!
//! - **Trust**: Verify that distributed binaries match published source code
//! - **Tamper detection**: Any difference in output proves the build was modified
//! - **Independent verification**: Third parties can rebuild and compare hashes
//! - **Supply chain integrity**: Prevent compromised build infrastructure from injecting malware
//!
//! ## Challenges in Rust
//!
//! - `CARGO_ENCODED_RUSTFLAGS` affects code generation
//! - Build scripts (`build.rs`) can produce non-deterministic output
//! - Procedural macros may embed timestamps or random values
//! - LTO settings and codegen-units affect binary layout
//!
//! ## Defense: Build Verification Pipeline
//!
//! In this lesson, you will implement build fingerprinting, comparison logic,
//! and reproducibility verification.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Metadata captured from a build for reproducibility verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildFingerprint {
    pub source_hash: String,
    pub dependency_hash: String,
    pub compiler_version: String,
    pub target_triple: String,
    pub output_hash: String,
    pub build_timestamp: String,
    pub flags_hash: String,
}

/// Result of comparing two build fingerprints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReproducibilityResult {
    /// Builds are bit-for-bit identical.
    Reproducible,
    /// Builds differ in one or more fields.
    NotReproducible { differences: Vec<String> },
}

/// Exercise 1: Compute a deterministic hash of a build input set.
///
/// Hash the concatenation of: source_hash + dependency_hash + compiler_version + flags_hash.
/// Each field is separated by a "|" delimiter.
///
/// Return a lowercase hex string.
///
/// Hints:
/// - Use Sha256
/// - Concatenate fields with "|" separator
/// - Feed to hasher, finalize, hex-encode
pub fn compute_build_input_hash(fingerprint: &BuildFingerprint) -> String {
    todo!("Compute deterministic hash of build inputs")
}

/// Exercise 2: Compare two build fingerprints for reproducibility.
///
/// Compare all fields EXCEPT build_timestamp (timestamps are expected to differ).
///
/// Return `Reproducible` if all non-timestamp fields match.
/// Return `NotReproducible` with a list of differing field names otherwise.
///
/// Hints:
/// - Compare source_hash, dependency_hash, compiler_version, target_triple,
///   output_hash, and flags_hash
/// - Push field name to differences vec for each mismatch
pub fn check_reproducibility(
    build_a: &BuildFingerprint,
    build_b: &BuildFingerprint,
) -> ReproducibilityResult {
    todo!("Compare two build fingerprints")
}

/// Exercise 3: Create a build verification certificate.
///
/// Returns a formatted string containing:
/// ```text
/// BUILD VERIFICATION CERTIFICATE
/// ===============================
/// Source Hash: {source_hash}
/// Dependency Hash: {dependency_hash}
/// Compiler: {compiler_version}
/// Target: {target_triple}
/// Output Hash: {output_hash}
/// Status: {Reproducible / NOT Reproducible}
/// ===============================
/// ```
///
/// Hints:
/// - Use `format!()` with multiline template
/// - Call `check_reproducibility` to determine status
pub fn create_verification_certificate(
    build_a: &BuildFingerprint,
    build_b: &BuildFingerprint,
) -> String {
    todo!("Create build verification certificate")
}

/// Exercise 4: Validate that a build fingerprint contains no empty fields.
///
/// All string fields must be non-empty. Return a list of empty field names.
///
/// Hints:
/// - Check each field with `.is_empty()`
/// - Collect names of empty fields
pub fn validate_fingerprint(fingerprint: &BuildFingerprint) -> Vec<String> {
    todo!("Validate build fingerprint has no empty fields")
}

/// Exercise 5: Check if a build is "environment-isolated" (best practice).
///
/// A build is environment-isolated if:
/// - The flags_hash is deterministic (not empty)
/// - The build uses a known target triple (starts with "x86_64-", "aarch64-", or "wasm")
///
/// Hints:
/// - Check flags_hash is not empty
/// - Check target_triple starts with known prefixes
pub fn is_environment_isolated(fingerprint: &BuildFingerprint) -> bool {
    todo!("Check if build uses environment isolation")
}

/// Exercise 6: Compare output hashes from multiple builds of the same source.
///
/// Given a slice of build fingerprints from independent builds of the same code,
/// check if ALL output hashes are identical.
///
/// Return (all_match: bool, unique_hashes: Vec<String>).
///
/// Hints:
/// - Collect all unique output_hash values into a HashSet
/// - all_match is true if there's exactly 1 unique hash
pub fn verify_multi_build(fingerprints: &[BuildFingerprint]) -> (bool, Vec<String>) {
    todo!("Verify multiple builds produce identical output")
}

/// Exercise 7: Generate a reproducibility score (0.0 to 1.0) for a set of builds.
///
/// Score = (number of matching pairs) / (total number of pairs).
/// A pair is matching if their output_hash values are equal.
///
/// For 3 builds (A, B, C): pairs are (A,B), (A,C), (B,C) = 3 pairs.
/// If all match, score = 1.0. If 2 of 3 match, score = 0.667.
///
/// Return 1.0 for 0 or 1 builds (trivially reproducible).
///
/// Hints:
/// - Generate all pairs (i, j) where i < j
/// - Count matching output_hash pairs
/// - Divide by total pairs
pub fn reproducibility_score(fingerprints: &[BuildFingerprint]) -> f64 {
    todo!("Calculate reproducibility score")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_fingerprint() -> BuildFingerprint {
        BuildFingerprint {
            source_hash: "abc123".to_string(),
            dependency_hash: "def456".to_string(),
            compiler_version: "rustc 1.72.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            output_hash: "789ghi".to_string(),
            build_timestamp: "2024-01-01T00:00:00Z".to_string(),
            flags_hash: "jkl012".to_string(),
        }
    }

    fn different_fingerprint() -> BuildFingerprint {
        BuildFingerprint {
            source_hash: "abc123".to_string(),
            dependency_hash: "def456".to_string(),
            compiler_version: "rustc 1.72.0".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            output_hash: "DIFFERENT".to_string(),
            build_timestamp: "2024-01-02T00:00:00Z".to_string(),
            flags_hash: "jkl012".to_string(),
        }
    }

    #[test]
    fn test_build_input_hash_deterministic() {
        let fp = sample_fingerprint();
        let h1 = compute_build_input_hash(&fp);
        let h2 = compute_build_input_hash(&fp);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_build_input_hash_differs() {
        let fp1 = sample_fingerprint();
        let mut fp2 = fp1.clone();
        fp2.flags_hash = "different".to_string();
        assert_ne!(compute_build_input_hash(&fp1), compute_build_input_hash(&fp2));
    }

    #[test]
    fn test_check_reproducibility_identical() {
        let fp = sample_fingerprint();
        let mut fp2 = fp.clone();
        fp2.build_timestamp = "2024-06-01T00:00:00Z".to_string(); // timestamp can differ
        assert_eq!(check_reproducibility(&fp, &fp2), ReproducibilityResult::Reproducible);
    }

    #[test]
    fn test_check_reproducibility_different_output() {
        let fp = sample_fingerprint();
        let fp2 = different_fingerprint();
        let result = check_reproducibility(&fp, &fp2);
        assert!(matches!(result, ReproducibilityResult::NotReproducible { .. }));
        if let ReproducibilityResult::NotReproducible { differences } = result {
            assert!(differences.contains(&"output_hash".to_string()));
        }
    }

    #[test]
    fn test_create_verification_certificate_reproducible() {
        let fp = sample_fingerprint();
        let mut fp2 = fp.clone();
        fp2.build_timestamp = "different".to_string();
        let cert = create_verification_certificate(&fp, &fp2);
        assert!(cert.contains("Reproducible"));
        assert!(cert.contains("BUILD VERIFICATION CERTIFICATE"));
    }

    #[test]
    fn test_create_verification_certificate_not_reproducible() {
        let fp = sample_fingerprint();
        let fp2 = different_fingerprint();
        let cert = create_verification_certificate(&fp, &fp2);
        assert!(cert.contains("NOT Reproducible"));
    }

    #[test]
    fn test_validate_fingerprint_valid() {
        let fp = sample_fingerprint();
        assert!(validate_fingerprint(&fp).is_empty());
    }

    #[test]
    fn test_validate_fingerprint_empty_fields() {
        let mut fp = sample_fingerprint();
        fp.source_hash = String::new();
        fp.output_hash = String::new();
        let empty = validate_fingerprint(&fp);
        assert_eq!(empty.len(), 2);
        assert!(empty.contains(&"source_hash".to_string()));
        assert!(empty.contains(&"output_hash".to_string()));
    }

    #[test]
    fn test_is_environment_isolated_true() {
        let fp = sample_fingerprint();
        assert!(is_environment_isolated(&fp));
    }

    #[test]
    fn test_is_environment_isolated_no_flags() {
        let mut fp = sample_fingerprint();
        fp.flags_hash = String::new();
        assert!(!is_environment_isolated(&fp));
    }

    #[test]
    fn test_is_environment_isolated_unknown_target() {
        let mut fp = sample_fingerprint();
        fp.target_triple = "unknown-platform".to_string();
        assert!(!is_environment_isolated(&fp));
    }

    #[test]
    fn test_verify_multi_build_all_match() {
        let fp = sample_fingerprint();
        let mut fp2 = fp.clone();
        fp2.build_timestamp = "different".to_string();
        let (all_match, unique) = verify_multi_build(&[fp, fp2]);
        assert!(all_match);
        assert_eq!(unique.len(), 1);
    }

    #[test]
    fn test_verify_multi_build_mismatch() {
        let fp = sample_fingerprint();
        let fp2 = different_fingerprint();
        let (all_match, unique) = verify_multi_build(&[fp, fp2]);
        assert!(!all_match);
        assert_eq!(unique.len(), 2);
    }

    #[test]
    fn test_reproducibility_score_all_match() {
        let fp = sample_fingerprint();
        let fingerprints: Vec<_> = (0..3).map(|_| {
            let mut f = fp.clone();
            f.build_timestamp = format!("ts-{}", rand_suffix());
            f
        }).collect();
        let score = reproducibility_score(&fingerprints);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    fn rand_suffix() -> String {
        static mut COUNTER: u32 = 0;
        unsafe {
            COUNTER += 1;
            COUNTER.to_string()
        }
    }

    #[test]
    fn test_reproducibility_score_single_build() {
        let fp = sample_fingerprint();
        assert!((reproducibility_score(&[fp]) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reproducibility_score_empty() {
        assert!((reproducibility_score(&[]) - 1.0).abs() < f64::EPSILON);
    }
}
