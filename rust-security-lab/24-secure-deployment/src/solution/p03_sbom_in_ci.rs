//! # Lesson 03: SBOM Generation in CI (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SbomComponent {
    pub name: String,
    pub version: String,
    pub purl: String,
    pub license: String,
    pub source_url: String,
    pub hash: String,
    pub is_direct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sbom {
    pub spec_version: String,
    pub name: String,
    pub version: String,
    pub components: Vec<SbomComponent>,
    pub timestamp: String,
    pub generator: String,
}

/// Create an SbomComponent with auto-generated purl.
pub fn create_component(
    name: &str,
    version: &str,
    license: &str,
    source_url: &str,
    hash: &str,
    is_direct: bool,
) -> SbomComponent {
    SbomComponent {
        name: name.to_string(),
        version: version.to_string(),
        purl: format!("pkg:cargo/{}@{}", name, version),
        license: license.to_string(),
        source_url: source_url.to_string(),
        hash: hash.to_string(),
        is_direct,
    }
}

/// Validate an SBOM for completeness and correctness.
pub fn validate_sbom(sbom: &Sbom) -> Vec<String> {
    let mut issues = Vec::new();

    if sbom.spec_version.is_empty() {
        issues.push("spec_version is empty".to_string());
    }

    if sbom.name.is_empty() {
        issues.push("name is empty".to_string());
    }

    if sbom.components.is_empty() {
        issues.push("components list is empty".to_string());
    }

    for comp in &sbom.components {
        if comp.name.is_empty() {
            issues.push("component has empty name".to_string());
        }
        if comp.version.is_empty() {
            issues.push(format!("component '{}' has empty version", comp.name));
        }
        if comp.license.is_empty() {
            issues.push(format!("component '{}' has empty license", comp.name));
        }
        if comp.license.to_uppercase() == "UNKNOWN" {
            issues.push(format!("component '{}' has UNKNOWN license", comp.name));
        }
        if comp.hash.is_empty() {
            issues.push(format!("component '{}' has empty hash", comp.name));
        }
    }

    issues
}

/// Find components matching a license identifier (case-insensitive).
pub fn find_by_license<'a>(sbom: &'a Sbom, license: &str) -> Vec<&'a SbomComponent> {
    let license_lower = license.to_lowercase();
    sbom.components.iter()
        .filter(|c| c.license.to_lowercase().contains(&license_lower))
        .collect()
}

/// Find components with copyleft licenses.
///
/// Handles variants like "GPL-3.0", "GPL-3.0-only", "GPL-3.0-or-later".
pub fn find_copyleft_components(sbom: &Sbom) -> Vec<String> {
    let copyleft_prefixes = ["gpl-2.0", "gpl-3.0", "agpl-3.0", "lgpl-2.1", "lgpl-3.0"];

    sbom.components.iter()
        .filter(|c| {
            let license_lower = c.license.to_lowercase();
            copyleft_prefixes.iter().any(|prefix| license_lower.contains(prefix))
        })
        .map(|c| c.name.clone())
        .collect()
}

/// Generate a human-readable SBOM summary.
pub fn sbom_summary(sbom: &Sbom) -> String {
    let total = sbom.components.len();
    let direct = sbom.components.iter().filter(|c| c.is_direct).count();
    let transitive = total - direct;

    let mut unique_licenses: Vec<String> = sbom.components.iter()
        .map(|c| c.license.clone())
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .collect();
    unique_licenses.sort();

    let copyleft = find_copyleft_components(sbom);
    let copyleft_present = if copyleft.is_empty() { "no" } else { "yes" };

    format!(
        "SBOM Summary: {} v{}\nTotal components: {}\nDirect: {}, Transitive: {}\nUnique licenses: {}\nCopyleft present: {}",
        sbom.name, sbom.version, total, direct, transitive, unique_licenses.len(), copyleft_present
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_sbom() -> Sbom {
        Sbom {
            spec_version: "1.4".to_string(),
            name: "myapp".to_string(),
            version: "1.0.0".to_string(),
            components: vec![
                SbomComponent {
                    name: "serde".to_string(),
                    version: "1.0.193".to_string(),
                    purl: "pkg:cargo/serde@1.0.193".to_string(),
                    license: "MIT OR Apache-2.0".to_string(),
                    source_url: "https://github.com/serde-rs/serde".to_string(),
                    hash: "abc123".to_string(),
                    is_direct: true,
                },
                SbomComponent {
                    name: "ring".to_string(),
                    version: "0.17.7".to_string(),
                    purl: "pkg:cargo/ring@0.17.7".to_string(),
                    license: "MIT".to_string(),
                    source_url: "https://github.com/briansmith/ring".to_string(),
                    hash: "def456".to_string(),
                    is_direct: true,
                },
                SbomComponent {
                    name: "some-gpl-lib".to_string(),
                    version: "0.1.0".to_string(),
                    purl: "pkg:cargo/some-gpl-lib@0.1.0".to_string(),
                    license: "GPL-3.0".to_string(),
                    source_url: "https://example.com/gpl".to_string(),
                    hash: "ghi789".to_string(),
                    is_direct: false,
                },
            ],
            timestamp: "2024-01-15T10:30:00Z".to_string(),
            generator: "cargo-sbom".to_string(),
        }
    }

    #[test]
    fn test_create_component() {
        let comp = create_component("serde", "1.0.193", "MIT", "https://example.com", "hash123", true);
        assert_eq!(comp.name, "serde");
        assert_eq!(comp.version, "1.0.193");
        assert_eq!(comp.purl, "pkg:cargo/serde@1.0.193");
        assert!(comp.is_direct);
    }

    #[test]
    fn test_validate_sbom_valid() {
        let sbom = sample_sbom();
        let issues = validate_sbom(&sbom);
        assert!(issues.is_empty(), "Valid SBOM should pass, got: {:?}", issues);
    }

    #[test]
    fn test_validate_sbom_empty_name() {
        let mut sbom = sample_sbom();
        sbom.name = String::new();
        let issues = validate_sbom(&sbom);
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_validate_sbom_unknown_license() {
        let mut sbom = sample_sbom();
        sbom.components[0].license = "UNKNOWN".to_string();
        let issues = validate_sbom(&sbom);
        assert!(issues.iter().any(|i| i.contains("UNKNOWN") || i.contains("license")));
    }

    #[test]
    fn test_find_by_license() {
        let sbom = sample_sbom();
        let mit = find_by_license(&sbom, "MIT");
        assert!(mit.iter().any(|c| c.name == "ring"));
    }

    #[test]
    fn test_find_by_license_case_insensitive() {
        let sbom = sample_sbom();
        let mit = find_by_license(&sbom, "mit");
        assert!(mit.iter().any(|c| c.name == "ring"));
    }

    #[test]
    fn test_find_copyleft_components() {
        let sbom = sample_sbom();
        let copyleft = find_copyleft_components(&sbom);
        assert!(copyleft.contains(&"some-gpl-lib".to_string()));
        assert!(!copyleft.contains(&"serde".to_string()));
    }

    #[test]
    fn test_sbom_summary() {
        let sbom = sample_sbom();
        let summary = sbom_summary(&sbom);
        assert!(summary.contains("3"), "Should list 3 components");
        assert!(summary.contains("2"), "Should list 2 direct deps");
        assert!(summary.contains("1"), "Should list 1 transitive dep");
        assert!(summary.contains("yes") || summary.contains("Yes") || summary.contains("true"),
            "Should indicate copyleft is present");
    }
}
