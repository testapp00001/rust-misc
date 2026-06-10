//! # Lesson 05: Software Bill of Materials (SBOM) Generation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet, VecDeque};

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

/// Compute a SHA-256 hash of source code content.
pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

/// Generate a Package URL (purl) for a crate.
pub fn generate_purl(name: &str, version: &str) -> String {
    format!("pkg:cargo/{}@{}", name, version)
}

/// Build an SbomComponent from raw dependency data.
pub fn build_component(
    name: &str,
    version: &str,
    supplier: &str,
    license: &str,
    source: &[u8],
) -> SbomComponent {
    SbomComponent {
        name: name.to_string(),
        version: version.to_string(),
        supplier: supplier.to_string(),
        license: license.to_string(),
        hash: hash_content(source),
        purl: generate_purl(name, version),
        dependencies: Vec::new(),
    }
}

/// Resolve the dependency tree and add dependency info to components.
pub fn resolve_dependencies(
    components: &[SbomComponent],
    relationships: &[(String, String)],
) -> Vec<SbomComponent> {
    let mut result = components.to_vec();
    for (parent, child) in relationships {
        if let Some(comp) = result.iter_mut().find(|c| c.name == *parent) {
            if !comp.dependencies.contains(child) {
                comp.dependencies.push(child.clone());
            }
        }
    }
    result
}

/// Generate a complete SBOM from components and metadata.
pub fn generate_sbom(
    project_name: &str,
    project_version: &str,
    timestamp: &str,
    components: Vec<SbomComponent>,
) -> Sbom {
    Sbom {
        metadata: SbomMetadata {
            tool_name: "rust-security-lab".to_string(),
            tool_version: "1.0.0".to_string(),
            format: "CycloneDX".to_string(),
            timestamp: timestamp.to_string(),
            project_name: project_name.to_string(),
            project_version: project_version.to_string(),
        },
        components,
    }
}

/// Serialize an SBOM to JSON string.
pub fn serialize_sbom(sbom: &Sbom) -> String {
    serde_json::to_string_pretty(sbom).unwrap_or_else(|_| "{}".to_string())
}

/// Find all components with a specific license.
pub fn find_by_license<'a>(sbom: &'a Sbom, license: &str) -> Vec<&'a SbomComponent> {
    sbom.components
        .iter()
        .filter(|c| c.license == license)
        .collect()
}

/// Count transitive dependencies starting from a root component.
pub fn count_transitive_deps(sbom: &Sbom, root_name: &str) -> usize {
    let dep_map: HashMap<&str, &SbomComponent> = sbom
        .components
        .iter()
        .map(|c| (c.name.as_str(), c))
        .collect();

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // Start from root's dependencies (not the root itself)
    if let Some(root) = dep_map.get(root_name) {
        for dep in &root.dependencies {
            if visited.insert(dep.as_str()) {
                queue.push_back(dep.as_str());
            }
        }
    }

    // BFS
    while let Some(current) = queue.pop_front() {
        if let Some(comp) = dep_map.get(current) {
            for dep in &comp.dependencies {
                if visited.insert(dep.as_str()) {
                    queue.push_back(dep.as_str());
                }
            }
        }
    }

    visited.len()
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
        assert_eq!(h1.len(), 64);
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
