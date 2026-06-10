//! # Lesson 03: SBOM Generation in CI
//!
//! ## Attack: Hidden Dependencies
//!
//! Your application uses 200+ transitive dependencies. An attacker compromises a
//! maintainer account for a rarely-updated crate deep in your dependency tree.
//! Without an SBOM (Software Bill of Materials), you don't even know you're affected,
//! let alone which versions are vulnerable.
//!
//! ## Defend: Software Bill of Materials (SBOM)
//!
//! An SBOM is a formal record of all components, dependencies, and versions used
//! to build a software artifact. It enables:
//!
//! - **Vulnerability tracking**: When a CVE is published, instantly know if you're affected
//! - **License compliance**: Ensure no GPL code sneaks into your MIT project
//! - **Audit trail**: Prove what was in each release
//!
//! ## Audit: SBOM Quality Checklist
//!
//! - [ ] All direct dependencies listed
//! [ ] All transitive dependencies listed
//! - [ ] Version numbers are exact (not ranges)
//! - [ ] Source URLs are included
//! - [ ] License information is present
//! - [ ] SBOM is generated in CI (not locally)
//! - [ ] SBOM is signed or attested

use serde::{Deserialize, Serialize};

/// A component in an SBOM.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SbomComponent {
    /// Package name
    pub name: String,
    /// Exact version
    pub version: String,
    /// Package URL (e.g., "pkg:cargo/serde@1.0.193")
    pub purl: String,
    /// SPDX license identifier (e.g., "MIT", "Apache-2.0")
    pub license: String,
    /// Source repository URL
    pub source_url: String,
    /// SHA-256 hash of the source archive
    pub hash: String,
    /// Whether this is a direct or transitive dependency
    pub is_direct: bool,
}

/// A complete Software Bill of Materials.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sbom {
    /// SBOM format version
    pub spec_version: String,
    /// Name of the software this SBOM describes
    pub name: String,
    /// Version of the software
    pub version: String,
    /// All components
    pub components: Vec<SbomComponent>,
    /// Timestamp of SBOM generation (ISO 8601)
    pub timestamp: String,
    /// Tool that generated this SBOM
    pub generator: String,
}

/// Exercise 1: Create an SBOM component.
///
/// Construct an `SbomComponent` with the given fields. The `purl` should be
/// generated in the format: `pkg:cargo/{name}@{version}`
pub fn create_component(
    name: &str,
    version: &str,
    license: &str,
    source_url: &str,
    hash: &str,
    is_direct: bool,
) -> SbomComponent {
    todo!("Create an SbomComponent with auto-generated purl")
}

/// Exercise 2: Validate an SBOM for completeness.
///
/// Check for these issues:
/// 1. `spec_version` is empty
/// 2. `name` is empty
/// 3. `components` is empty
/// 4. Any component has empty `name`, `version`, or `license`
/// 5. Any component has `license` == "UNKNOWN"
/// 6. Any component has empty `hash`
///
/// Return a list of issue descriptions. Empty = valid SBOM.
pub fn validate_sbom(sbom: &Sbom) -> Vec<String> {
    todo!("Validate SBOM for completeness and correctness")
}

/// Exercise 3: Find components with a specific license.
///
/// Return all components whose license matches the given SPDX identifier.
/// The comparison should be case-insensitive.
pub fn find_by_license<'a>(sbom: &'a Sbom, license: &str) -> Vec<&'a SbomComponent> {
    todo!("Find components matching a license identifier")
}

/// Exercise 4: Check for copyleft license contamination.
///
/// Copyleft licenses that require derivative works to be open source:
/// "GPL-2.0", "GPL-3.0", "AGPL-3.0", "LGPL-2.1", "LGPL-3.0"
/// (and their "-only" and "-or-later" variants)
///
/// Return the names of components with copyleft licenses.
/// The check should be case-insensitive and handle variants like "GPL-3.0-only".
pub fn find_copyleft_components(sbom: &Sbom) -> Vec<String> {
    todo!("Find components with copyleft licenses")
}

/// Exercise 5: Generate a summary of the SBOM.
///
/// Return a string with:
/// - Total number of components
/// - Number of direct vs transitive dependencies
/// - Number of unique licenses
/// - Whether any copyleft licenses are present
///
/// Format:
/// ```text
/// SBOM Summary: {name} v{version}
/// Total components: {count}
/// Direct: {direct_count}, Transitive: {transitive_count}
/// Unique licenses: {license_count}
/// Copyleft present: {yes/no}
/// ```
pub fn sbom_summary(sbom: &Sbom) -> String {
    todo!("Generate a human-readable SBOM summary")
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
        // ring has "MIT", serde has "MIT OR Apache-2.0" which contains "MIT"
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
