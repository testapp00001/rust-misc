//! # Lesson 07: Build Provenance and SLSA Framework (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A provenance statement in in-toto format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceStatement {
    pub subject: Vec<Subject>,
    pub builder: Builder,
    pub recipe: Recipe,
    pub materials: Vec<Material>,
    pub metadata: ProvenanceMetadata,
}

/// The artifact that was built.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    pub name: String,
    pub digest: DigestValue,
}

/// A hash digest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestValue {
    pub sha256: String,
}

/// Information about the build system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Builder {
    pub id: String,
}

/// The build recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub r#type: String,
    pub entry_point: String,
    pub arguments: Vec<String>,
}

/// An input material used in the build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub uri: String,
    pub digest: DigestValue,
}

/// Metadata about the provenance itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceMetadata {
    pub build_started_on: String,
    pub build_finished_on: String,
    pub reproducible: bool,
    pub completeness: Completeness,
}

/// Completeness indicators for the provenance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completeness {
    pub parameters: bool,
    pub environment: bool,
    pub materials: bool,
}

/// SLSA security level.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SlsaLevel {
    None = 0,
    Level1 = 1,
    Level2 = 2,
    Level3 = 3,
    Level4 = 4,
}

/// Compute SHA-256 of build output for the subject digest.
pub fn compute_artifact_hash(artifact: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(artifact);
    hex::encode(hasher.finalize())
}

/// Create a ProvenanceStatement for a build.
pub fn create_provenance(
    artifact_name: &str,
    artifact: &[u8],
    source_uri: &str,
    source_hash: &str,
    builder_id: &str,
    entry_point: &str,
    started_on: &str,
    finished_on: &str,
) -> ProvenanceStatement {
    ProvenanceStatement {
        subject: vec![Subject {
            name: artifact_name.to_string(),
            digest: DigestValue {
                sha256: compute_artifact_hash(artifact),
            },
        }],
        builder: Builder {
            id: builder_id.to_string(),
        },
        recipe: Recipe {
            r#type: "https://slsa.dev/provenance/v1".to_string(),
            entry_point: entry_point.to_string(),
            arguments: vec!["--release".to_string()],
        },
        materials: vec![Material {
            uri: source_uri.to_string(),
            digest: DigestValue {
                sha256: source_hash.to_string(),
            },
        }],
        metadata: ProvenanceMetadata {
            build_started_on: started_on.to_string(),
            build_finished_on: finished_on.to_string(),
            reproducible: true,
            completeness: Completeness {
                parameters: true,
                environment: true,
                materials: true,
            },
        },
    }
}

/// Evaluate the SLSA level of a provenance statement.
pub fn evaluate_slsa_level(provenance: &ProvenanceStatement) -> SlsaLevel {
    let builder_present = !provenance.builder.id.is_empty();
    let has_materials = !provenance.materials.is_empty();
    let has_timestamps = !provenance.metadata.build_started_on.is_empty()
        && !provenance.metadata.build_finished_on.is_empty();
    let complete = provenance.metadata.completeness.parameters
        && provenance.metadata.completeness.environment
        && provenance.metadata.completeness.materials;
    let reproducible = provenance.metadata.reproducible;

    // Check highest level first
    if builder_present
        && has_materials
        && has_timestamps
        && complete
        && reproducible
        && provenance.materials.len() >= 2
    {
        return SlsaLevel::Level4;
    }
    if builder_present && has_materials && has_timestamps && complete && reproducible {
        return SlsaLevel::Level3;
    }
    if builder_present && has_materials && has_timestamps {
        return SlsaLevel::Level2;
    }
    if builder_present {
        return SlsaLevel::Level1;
    }
    SlsaLevel::None
}

/// Verify that a build artifact matches the subject in provenance.
pub fn verify_artifact(
    artifact: &[u8],
    provenance: &ProvenanceStatement,
) -> bool {
    let hash = compute_artifact_hash(artifact);
    provenance
        .subject
        .first()
        .map(|s| s.digest.sha256 == hash)
        .unwrap_or(false)
}

/// Serialize provenance to JSON.
pub fn serialize_provenance(provenance: &ProvenanceStatement) -> String {
    serde_json::to_string_pretty(provenance).unwrap_or_else(|_| "{}".to_string())
}

/// Deserialize provenance from JSON.
pub fn deserialize_provenance(json: &str) -> Option<ProvenanceStatement> {
    serde_json::from_str(json).ok()
}

/// Generate a provenance summary report.
pub fn provenance_report(provenance: &ProvenanceStatement) -> String {
    let (name, hash_prefix) = match provenance.subject.first() {
        Some(s) => {
            let prefix = if s.digest.sha256.len() >= 12 {
                &s.digest.sha256[..12]
            } else {
                &s.digest.sha256
            };
            (s.name.as_str(), prefix)
        }
        None => ("unknown", ""),
    };

    let level = evaluate_slsa_level(provenance);
    let reproducible = if provenance.metadata.reproducible {
        "yes"
    } else {
        "no"
    };

    format!(
        "PROVENANCE REPORT\n\
         Subject: {} (sha256:{}...)\n\
         Builder: {}\n\
         SLSA Level: {:?}\n\
         Materials: {} input(s)\n\
         Reproducible: {}",
        name,
        hash_prefix,
        provenance.builder.id,
        level,
        provenance.materials.len(),
        reproducible
    )
}

/// Validate provenance completeness.
pub fn validate_provenance(provenance: &ProvenanceStatement) -> Vec<String> {
    let mut issues = Vec::new();

    if provenance.subject.is_empty()
        || provenance.subject.iter().any(|s| s.name.is_empty() || s.digest.sha256.is_empty())
    {
        issues.push("Subject name or hash is empty".to_string());
    }
    if provenance.builder.id.is_empty() {
        issues.push("Builder ID is empty".to_string());
    }
    if provenance.materials.is_empty() {
        issues.push("No materials listed".to_string());
    }
    if provenance.metadata.build_started_on.is_empty()
        || provenance.metadata.build_finished_on.is_empty()
    {
        issues.push("Build timestamps are empty".to_string());
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_provenance() -> ProvenanceStatement {
        ProvenanceStatement {
            subject: vec![Subject {
                name: "my-app".to_string(),
                digest: DigestValue {
                    sha256: compute_artifact_hash(b"binary content"),
                },
            }],
            builder: Builder {
                id: "https://github.com/actions/runner".to_string(),
            },
            recipe: Recipe {
                r#type: "https://slsa.dev/provenance/v1".to_string(),
                entry_point: "build.sh".to_string(),
                arguments: vec!["--release".to_string()],
            },
            materials: vec![Material {
                uri: "https://github.com/org/repo".to_string(),
                digest: DigestValue {
                    sha256: "abc123".to_string(),
                },
            }],
            metadata: ProvenanceMetadata {
                build_started_on: "2024-01-01T00:00:00Z".to_string(),
                build_finished_on: "2024-01-01T00:05:00Z".to_string(),
                reproducible: true,
                completeness: Completeness {
                    parameters: true,
                    environment: true,
                    materials: true,
                },
            },
        }
    }

    #[test]
    fn test_compute_artifact_hash() {
        let hash = compute_artifact_hash(b"test binary");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, compute_artifact_hash(b"test binary"));
    }

    #[test]
    fn test_create_provenance() {
        let prov = create_provenance(
            "my-app",
            b"binary",
            "https://github.com/org/repo",
            "src123",
            "https://github.com/actions/runner",
            "build.sh",
            "2024-01-01T00:00:00Z",
            "2024-01-01T00:05:00Z",
        );
        assert_eq!(prov.subject.len(), 1);
        assert_eq!(prov.subject[0].name, "my-app");
        assert_eq!(prov.builder.id, "https://github.com/actions/runner");
        assert_eq!(prov.materials.len(), 1);
    }

    #[test]
    fn test_evaluate_slsa_level_none() {
        let mut prov = sample_provenance();
        prov.builder.id = String::new();
        assert_eq!(evaluate_slsa_level(&prov), SlsaLevel::None);
    }

    #[test]
    fn test_evaluate_slsa_level2() {
        let mut prov = sample_provenance();
        prov.metadata.reproducible = false;
        assert!(evaluate_slsa_level(&prov) >= SlsaLevel::Level2);
    }

    #[test]
    fn test_evaluate_slsa_level3() {
        let prov = sample_provenance();
        assert!(evaluate_slsa_level(&prov) >= SlsaLevel::Level3);
    }

    #[test]
    fn test_evaluate_slsa_level4() {
        let mut prov = sample_provenance();
        prov.materials.push(Material {
            uri: "https://github.com/org/dep-provenance".to_string(),
            digest: DigestValue { sha256: "dep456".to_string() },
        });
        assert_eq!(evaluate_slsa_level(&prov), SlsaLevel::Level4);
    }

    #[test]
    fn test_verify_artifact_match() {
        let prov = sample_provenance();
        assert!(verify_artifact(b"binary content", &prov));
    }

    #[test]
    fn test_verify_artifact_mismatch() {
        let prov = sample_provenance();
        assert!(!verify_artifact(b"tampered content", &prov));
    }

    #[test]
    fn test_provenance_roundtrip() {
        let prov = sample_provenance();
        let json = serialize_provenance(&prov);
        let restored = deserialize_provenance(&json).unwrap();
        assert_eq!(restored.subject[0].name, prov.subject[0].name);
        assert_eq!(restored.builder.id, prov.builder.id);
    }

    #[test]
    fn test_provenance_report_format() {
        let prov = sample_provenance();
        let report = provenance_report(&prov);
        assert!(report.contains("PROVENANCE REPORT"));
        assert!(report.contains("my-app"));
        assert!(report.contains("SLSA Level"));
    }

    #[test]
    fn test_validate_provenance_valid() {
        let prov = sample_provenance();
        let issues = validate_provenance(&prov);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_provenance_missing_builder() {
        let mut prov = sample_provenance();
        prov.builder.id = String::new();
        let issues = validate_provenance(&prov);
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.to_lowercase().contains("builder")));
    }
}
