//! # Lesson 05: Software Bill of Materials (SBOM) Generation
//!
//! ## What is an SBOM?
//!
//! A Software Bill of Materials is a formal, machine-readable inventory of all
//! components, libraries, and dependencies used in building a software artifact.
//! Think of it as an "ingredients list" for software.
//!
//! ## Why SBOMs Matter
//!
//! - **Incident response**: When a vulnerability is found in a library, SBOMs let you
//!   instantly determine which of your products are affected
//! - **Regulatory compliance**: Executive Order 14028 (US) requires SBOMs for
//!   software sold to the federal government
//! - **License compliance**: Track all license obligations across the supply chain
//!
//! ## SBOM Formats
//!
//! | Format | Standard | Organization |
//! |--------|----------|-------------|
//! | CycloneDX | OWASP | Application security focused |
//! | SPDX | Linux Foundation / ISO | License compliance focused |
//! | SWID | ISO/IEC 19770-2 | Tag-based identification |
//!
//! ## Key Fields
//!
//! Every SBOM component includes:
//! - **Name** and **Version**: Identifies the component
//! - **Supplier**: Who provides it
//! - **Hash**: Cryptographic integrity check
//! - **License**: Legal obligations
//! - **Dependencies**: Relationships to other components
//!
//! ## Defense: Automated SBOM Generation
//!
//! In this lesson, you will implement SBOM data structures, dependency tree
//! resolution, and a simplified SBOM generator.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// A component in the Software Bill of Materials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomComponent {
    pub name: String,
    pub version: String,
    pub supplier: String,
    pub license: String,
    pub hash: String,
    pub purl: String,
    pub dependencies: Vec<String>,
}

/// A complete Software Bill of Materials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    pub metadata: SbomMetadata,
    pub components: Vec<SbomComponent>,
}

/// Metadata about the SBOM itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomMetadata {
    pub tool_name: String,
    pub tool_version: String,
    pub format: String,
    pub timestamp: String,
    pub project_name: String,
    pub project_version: String,
}

/// Exercise 1: Compute a SHA-256 hash of source code content.
///
/// This simulates hashing a crate's source code for the SBOM hash field.
/// Return the hash as a lowercase hex string.
///
/// Hints:
/// - Create a `Sha256` hasher
/// - Feed the input bytes
/// - Finalize and hex-encode
pub fn hash_content(content: &[u8]) -> String {
    todo!("Compute SHA-256 hash of content")
}

/// Exercise 2: Generate a Package URL (purl) for a crate.
///
/// A purl for a Rust crate follows this format:
/// `pkg:cargo/{name}@{version}`
///
/// Example: `pkg:cargo/serde@1.0.188`
///
/// Hints:
/// - Use `format!()` with the purl template
pub fn generate_purl(name: &str, version: &str) -> String {
    todo!("Generate Package URL for a crate")
}

/// Exercise 3: Build an SbomComponent from raw dependency data.
///
/// Given name, version, supplier, license, and source code bytes,
/// compute the hash and purl, and construct the component.
///
/// Hints:
/// - Use `hash_content` for the hash field
/// - Use `generate_purl` for the purl field
/// - Dependencies are not set here (they are resolved separately)
pub fn build_component(
    name: &str,
    version: &str,
    supplier: &str,
    license: &str,
    source: &[u8],
) -> SbomComponent {
    todo!("Build an SbomComponent from raw data")
}

/// Exercise 4: Resolve the dependency tree and add dependency info to components.
///
/// Given a list of components and a list of (parent, child) dependency relationships,
/// populate each component's `dependencies` field.
///
/// Hints:
/// - Create a mutable clone of the components
/// - For each (parent, child) pair, find the parent component and push child to its dependencies
pub fn resolve_dependencies(
    components: &[SbomComponent],
    relationships: &[(String, String)],
) -> Vec<SbomComponent> {
    todo!("Resolve dependency tree relationships")
}

/// Exercise 5: Generate a complete SBOM from components and metadata.
///
/// Given project name, version, and a list of components,
/// create a complete `Sbom` with standard metadata.
///
/// Hints:
/// - Create `SbomMetadata` with:
///   - tool_name: "rust-security-lab"
///   - tool_version: "1.0.0"
///   - format: "CycloneDX"
///   - timestamp: use the provided timestamp string
///   - project_name and project_version from parameters
pub fn generate_sbom(
    project_name: &str,
    project_version: &str,
    timestamp: &str,
    components: Vec<SbomComponent>,
) -> Sbom {
    todo!("Generate a complete SBOM")
}

/// Exercise 6: Serialize an SBOM to JSON string.
///
/// Use pretty-printed JSON for human readability.
///
/// Hints:
/// - Use `serde_json::to_string_pretty`
pub fn serialize_sbom(sbom: &Sbom) -> String {
    todo!("Serialize SBOM to JSON")
}

/// Exercise 7: Find all components with a specific license.
///
/// Return references to components whose license matches the given SPDX identifier.
///
/// Hints:
/// - Filter components by license string equality
pub fn find_by_license<'a>(sbom: &'a Sbom, license: &str) -> Vec<&'a SbomComponent> {
    todo!("Find components by license")
}

/// Exercise 8: Calculate the total number of unique dependencies (transitive).
///
/// Starting from a root component, follow the dependency chain to count
/// all unique components in the tree.
///
/// Hints:
/// - Use a `HashSet<String>` to track visited component names
/// - BFS or DFS from the root component
/// - For each component, follow its `dependencies` list
pub fn count_transitive_deps(sbom: &Sbom, root_name: &str) -> usize {
    todo!("Count transitive dependencies")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_component(name: &str, version: &str, deps: Vec<String>) -> SbomComponent {
        SbomComponent {
            name: name.to_string(),
            version: version.to_string(),
            supplier: "crates.io".to_string(),
            license: "MIT".to_string(),
            hash: hash_content(name.as_bytes()),
            purl: generate_purl(name, version),
            dependencies: deps,
        }
    }

    #[test]
    fn test_hash_content_deterministic() {
        let h1 = hash_content(b"test data");
        let h2 = hash_content(b"test data");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_hash_content_different_inputs() {
        let h1 = hash_content(b"hello");
        let h2 = hash_content(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_generate_purl() {
        let purl = generate_purl("serde", "1.0.188");
        assert_eq!(purl, "pkg:cargo/serde@1.0.188");
    }

    #[test]
    fn test_build_component() {
        let comp = build_component("serde", "1.0.188", "crates.io", "MIT", b"source code");
        assert_eq!(comp.name, "serde");
        assert_eq!(comp.purl, "pkg:cargo/serde@1.0.188");
        assert!(!comp.hash.is_empty());
        assert_eq!(comp.supplier, "crates.io");
    }

    #[test]
    fn test_resolve_dependencies() {
        let components = vec![
            sample_component("my-app", "1.0.0", vec![]),
            sample_component("serde", "1.0.188", vec![]),
        ];
        let rels = vec![
            ("my-app".to_string(), "serde".to_string()),
        ];
        let resolved = resolve_dependencies(&components, &rels);
        assert_eq!(resolved[0].dependencies, vec!["serde"]);
        assert!(resolved[1].dependencies.is_empty());
    }

    #[test]
    fn test_generate_sbom() {
        let components = vec![
            sample_component("serde", "1.0.188", vec![]),
        ];
        let sbom = generate_sbom("my-project", "1.0.0", "2024-01-01T00:00:00Z", components);
        assert_eq!(sbom.metadata.project_name, "my-project");
        assert_eq!(sbom.metadata.format, "CycloneDX");
        assert_eq!(sbom.components.len(), 1);
    }

    #[test]
    fn test_serialize_sbom_is_valid_json() {
        let components = vec![
            sample_component("serde", "1.0.188", vec![]),
        ];
        let sbom = generate_sbom("test", "1.0.0", "2024-01-01T00:00:00Z", components);
        let json = serialize_sbom(&sbom);
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_find_by_license() {
        let mut comp_mit = sample_component("serde", "1.0.188", vec![]);
        comp_mit.license = "MIT".to_string();
        let mut comp_gpl = sample_component("some-gpl", "1.0.0", vec![]);
        comp_gpl.license = "GPL-3.0".to_string();

        let sbom = generate_sbom("test", "1.0.0", "2024-01-01T00:00:00Z", vec![comp_mit, comp_gpl]);
        let mit_only = find_by_license(&sbom, "MIT");
        assert_eq!(mit_only.len(), 1);
        assert_eq!(mit_only[0].name, "serde");
    }

    #[test]
    fn test_count_transitive_deps() {
        let components = vec![
            sample_component("app", "1.0.0", vec!["lib-a".to_string(), "lib-b".to_string()]),
            sample_component("lib-a", "1.0.0", vec!["lib-c".to_string()]),
            sample_component("lib-b", "1.0.0", vec![]),
            sample_component("lib-c", "1.0.0", vec![]),
        ];
        let sbom = generate_sbom("test", "1.0.0", "2024-01-01T00:00:00Z", components);
        // app -> lib-a, lib-b, lib-c = 3 deps (not counting root)
        let count = count_transitive_deps(&sbom, "app");
        assert_eq!(count, 3);
    }

    #[test]
    fn test_count_transitive_deps_no_deps() {
        let components = vec![
            sample_component("standalone", "1.0.0", vec![]),
        ];
        let sbom = generate_sbom("test", "1.0.0", "2024-01-01T00:00:00Z", components);
        let count = count_transitive_deps(&sbom, "standalone");
        assert_eq!(count, 0);
    }
}
