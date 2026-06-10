//! # Lesson 06: Reproducible Builds Verification (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

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
    Reproducible,
    NotReproducible { differences: Vec<String> },
}

/// Compute a deterministic hash of build inputs.
pub fn compute_build_input_hash(fingerprint: &BuildFingerprint) -> String {
    let mut hasher = Sha256::new();
    let input = format!(
        "{}|{}|{}|{}",
        fingerprint.source_hash,
        fingerprint.dependency_hash,
        fingerprint.compiler_version,
        fingerprint.flags_hash
    );
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Compare two build fingerprints for reproducibility.
pub fn check_reproducibility(
    build_a: &BuildFingerprint,
    build_b: &BuildFingerprint,
) -> ReproducibilityResult {
    let mut differences = Vec::new();

    if build_a.source_hash != build_b.source_hash {
        differences.push("source_hash".to_string());
    }
    if build_a.dependency_hash != build_b.dependency_hash {
        differences.push("dependency_hash".to_string());
    }
    if build_a.compiler_version != build_b.compiler_version {
        differences.push("compiler_version".to_string());
    }
    if build_a.target_triple != build_b.target_triple {
        differences.push("target_triple".to_string());
    }
    if build_a.output_hash != build_b.output_hash {
        differences.push("output_hash".to_string());
    }
    if build_a.flags_hash != build_b.flags_hash {
        differences.push("flags_hash".to_string());
    }

    if differences.is_empty() {
        ReproducibilityResult::Reproducible
    } else {
        ReproducibilityResult::NotReproducible { differences }
    }
}

/// Create a build verification certificate.
pub fn create_verification_certificate(
    build_a: &BuildFingerprint,
    build_b: &BuildFingerprint,
) -> String {
    let result = check_reproducibility(build_a, build_b);
    let status = match &result {
        ReproducibilityResult::Reproducible => "Reproducible",
        ReproducibilityResult::NotReproducible { .. } => "NOT Reproducible",
    };

    format!(
        "BUILD VERIFICATION CERTIFICATE\n\
         ===============================\n\
         Source Hash: {}\n\
         Dependency Hash: {}\n\
         Compiler: {}\n\
         Target: {}\n\
         Output Hash: {}\n\
         Status: {}\n\
         ===============================",
        build_a.source_hash,
        build_a.dependency_hash,
        build_a.compiler_version,
        build_a.target_triple,
        build_a.output_hash,
        status
    )
}

/// Validate that a build fingerprint contains no empty fields.
pub fn validate_fingerprint(fingerprint: &BuildFingerprint) -> Vec<String> {
    let mut empty = Vec::new();
    if fingerprint.source_hash.is_empty() {
        empty.push("source_hash".to_string());
    }
    if fingerprint.dependency_hash.is_empty() {
        empty.push("dependency_hash".to_string());
    }
    if fingerprint.compiler_version.is_empty() {
        empty.push("compiler_version".to_string());
    }
    if fingerprint.target_triple.is_empty() {
        empty.push("target_triple".to_string());
    }
    if fingerprint.output_hash.is_empty() {
        empty.push("output_hash".to_string());
    }
    if fingerprint.flags_hash.is_empty() {
        empty.push("flags_hash".to_string());
    }
    empty
}

/// Check if a build uses environment isolation.
pub fn is_environment_isolated(fingerprint: &BuildFingerprint) -> bool {
    if fingerprint.flags_hash.is_empty() {
        return false;
    }
    let target = &fingerprint.target_triple;
    target.starts_with("x86_64-")
        || target.starts_with("aarch64-")
        || target.starts_with("wasm")
}

/// Verify multiple builds produce identical output.
pub fn verify_multi_build(fingerprints: &[BuildFingerprint]) -> (bool, Vec<String>) {
    let unique: HashSet<&str> = fingerprints
        .iter()
        .map(|f| f.output_hash.as_str())
        .collect();
    let unique_vec: Vec<String> = unique.into_iter().map(String::from).collect();
    (unique_vec.len() <= 1, unique_vec)
}

/// Calculate reproducibility score for a set of builds.
pub fn reproducibility_score(fingerprints: &[BuildFingerprint]) -> f64 {
    let n = fingerprints.len();
    if n <= 1 {
        return 1.0;
    }

    let total_pairs = (n * (n - 1)) / 2;
    let matching_pairs = fingerprints
        .iter()
        .enumerate()
        .flat_map(|(i, a)| {
            fingerprints[i + 1..]
                .iter()
                .map(move |b| a.output_hash == b.output_hash)
        })
        .filter(|&m| m)
        .count();

    matching_pairs as f64 / total_pairs as f64
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
        fp2.build_timestamp = "2024-06-01T00:00:00Z".to_string();
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
        let fp2 = BuildFingerprint {
            build_timestamp: "different".to_string(),
            ..fp.clone()
        };
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
        let fingerprints: Vec<_> = (0..3)
            .map(|i| BuildFingerprint {
                build_timestamp: format!("ts-{}", i),
                ..fp.clone()
            })
            .collect();
        let score = reproducibility_score(&fingerprints);
        assert!((score - 1.0).abs() < f64::EPSILON);
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
