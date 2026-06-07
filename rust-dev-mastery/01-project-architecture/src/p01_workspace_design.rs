//! # Lesson 1: Multi-Crate Workspace Design
//!
//! Workspaces let you manage multiple related crates in a single repository.
//! This lesson covers when to split code into separate crates, how to share
//! dependencies efficiently, and patterns for monorepo organization.

use serde::{Deserialize, Serialize};

/// Represents a crate within a workspace.
/// In a real monorepo, each member crate would live in its own directory
/// with its own Cargo.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrateManifest {
    pub name: String,
    pub version: String,
    pub path: String,
    pub kind: CrateKind,
    pub dependencies: Vec<String>,
}

/// Whether a crate is a library, binary, or both.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CrateKind {
    /// Library crate: can be depended upon by others.
    Library,
    /// Binary crate: produces an executable.
    Binary,
    /// Both a library and one or more binaries.
    LibAndBin,
    /// Proc-macro crate: generates code at compile time.
    ProcMacro,
}

/// Models a workspace with multiple member crates.
/// This demonstrates how a real monorepo is structured.
#[derive(Debug, Serialize, Deserialize)]
pub struct Workspace {
    pub root: String,
    pub resolver_version: u8,
    pub members: Vec<CrateManifest>,
    pub shared_dependencies: Vec<SharedDep>,
}

/// A dependency shared across workspace members via [workspace.dependencies].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedDep {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
    pub optional: bool,
}

impl Workspace {
    /// Create a new workspace with resolver v2 (Rust 2021+).
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            resolver_version: 2,
            members: Vec::new(),
            shared_dependencies: Vec::new(),
        }
    }

    /// Add a crate to the workspace. In practice, this means adding
    /// a directory with its own Cargo.toml and listing it in the
    /// workspace `members` array.
    pub fn add_crate(&mut self, manifest: CrateManifest) {
        self.members.push(manifest);
    }

    /// Add a shared dependency that all workspace members can reference
    /// with `dep.workspace = true` in their Cargo.toml.
    pub fn add_shared_dep(&mut self, dep: SharedDep) {
        self.shared_dependencies.push(dep);
    }

    /// Check if a crate with the given name exists in the workspace.
    pub fn has_crate(&self, name: &str) -> bool {
        self.members.iter().any(|m| m.name == name)
    }

    /// Find all crates that depend on a given crate name.
    /// Useful for understanding the dependency graph within the workspace.
    pub fn dependents_of(&self, crate_name: &str) -> Vec<&CrateManifest> {
        self.members
            .iter()
            .filter(|m| m.dependencies.iter().any(|d| d == crate_name))
            .collect()
    }

    /// Compute the total number of unique dependencies across all crates,
    /// excluding shared dependencies.
    pub fn unique_deps(&self) -> Vec<String> {
        let mut deps: Vec<String> = self
            .members
            .iter()
            .flat_map(|m| m.dependencies.iter().cloned())
            .collect();
        deps.sort();
        deps.dedup();
        deps
    }

    /// Determine whether splitting a set of modules into a new crate is
    /// justified. Heuristics: if >3 crates depend on the same set of
    /// modules, extracting them reduces compile times.
    pub fn should_extract(&self, module_names: &[&str]) -> ExtractionRecommendation {
        let affected = self
            .members
            .iter()
            .filter(|m| {
                module_names
                    .iter()
                    .any(|name| m.dependencies.iter().any(|d| d == *name))
            })
            .count();

        if affected >= 3 {
            ExtractionRecommendation {
                should_extract: true,
                reason: format!(
                    "{} crates depend on these modules; extracting reduces recompilation",
                    affected
                ),
                suggested_name: format!("shared-{}", module_names.join("-")),
            }
        } else {
            ExtractionRecommendation {
                should_extract: false,
                reason: "Not enough cross-crate usage to justify extraction".to_string(),
                suggested_name: String::new(),
            }
        }
    }
}

/// Result of analyzing whether to extract modules into a separate crate.
#[derive(Debug, PartialEq)]
pub struct ExtractionRecommendation {
    pub should_extract: bool,
    pub reason: String,
    pub suggested_name: String,
}

/// Helper to generate a workspace-level Cargo.toml string.
/// In a real project this would be written to disk.
pub fn generate_workspace_toml(workspace: &Workspace) -> String {
    let mut toml = String::from("[workspace]\n");
    toml.push_str("resolver = \"2\"\n");
    toml.push_str("members = [\n");
    for member in &workspace.members {
        toml.push_str(&format!("    \"{}\",\n", member.path));
    }
    toml.push_str("]\n\n");

    if !workspace.shared_dependencies.is_empty() {
        toml.push_str("[workspace.dependencies]\n");
        for dep in &workspace.shared_dependencies {
            if dep.features.is_empty() {
                toml.push_str(&format!("{} = \"{}\"\n", dep.name, dep.version));
            } else {
                let features = dep
                    .features
                    .iter()
                    .map(|f| format!("\"{}\"", f))
                    .collect::<Vec<_>>()
                    .join(", ");
                toml.push_str(&format!(
                    "{} = {{ version = \"{}\", features = [{}] }}\n",
                    dep.name, dep.version, features
                ));
            }
        }
    }

    toml
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_workspace() -> Workspace {
        let mut ws = Workspace::new("/home/dev/myproject");

        ws.add_crate(CrateManifest {
            name: "myapp-core".to_string(),
            version: "0.1.0".to_string(),
            path: "crates/core".to_string(),
            kind: CrateKind::Library,
            dependencies: vec!["serde".to_string(), "tokio".to_string()],
        });

        ws.add_crate(CrateManifest {
            name: "myapp-api".to_string(),
            version: "0.1.0".to_string(),
            path: "crates/api".to_string(),
            kind: CrateKind::Binary,
            dependencies: vec![
                "myapp-core".to_string(),
                "axum".to_string(),
                "serde".to_string(),
            ],
        });

        ws.add_crate(CrateManifest {
            name: "myapp-cli".to_string(),
            version: "0.1.0".to_string(),
            path: "crates/cli".to_string(),
            kind: CrateKind::Binary,
            dependencies: vec!["myapp-core".to_string(), "clap".to_string()],
        });

        ws.add_crate(CrateManifest {
            name: "myapp-macros".to_string(),
            version: "0.1.0".to_string(),
            path: "crates/macros".to_string(),
            kind: CrateKind::ProcMacro,
            dependencies: vec!["syn".to_string(), "quote".to_string()],
        });

        ws.add_shared_dep(SharedDep {
            name: "serde".to_string(),
            version: "1.0".to_string(),
            features: vec!["derive".to_string()],
            optional: false,
        });

        ws.add_shared_dep(SharedDep {
            name: "tokio".to_string(),
            version: "1.0".to_string(),
            features: vec!["full".to_string()],
            optional: false,
        });

        ws
    }

    #[test]
    fn test_workspace_creation() {
        let ws = Workspace::new("/tmp/test");
        assert_eq!(ws.root, "/tmp/test");
        assert_eq!(ws.resolver_version, 2);
        assert!(ws.members.is_empty());
    }

    #[test]
    fn test_add_crate() {
        let mut ws = sample_workspace();
        assert_eq!(ws.members.len(), 4);
        assert!(ws.has_crate("myapp-core"));
        assert!(ws.has_crate("myapp-api"));
        assert!(!ws.has_crate("nonexistent"));
    }

    #[test]
    fn test_dependents_of() {
        let ws = sample_workspace();
        let deps = ws.dependents_of("myapp-core");
        assert_eq!(deps.len(), 2);
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"myapp-api"));
        assert!(names.contains(&"myapp-cli"));
    }

    #[test]
    fn test_unique_deps() {
        let ws = sample_workspace();
        let deps = ws.unique_deps();
        // serde appears twice but should be deduplicated
        assert!(deps.contains(&"serde".to_string()));
        assert!(deps.contains(&"tokio".to_string()));
        assert!(deps.contains(&"axum".to_string()));
        assert!(deps.contains(&"clap".to_string()));
        assert!(deps.contains(&"syn".to_string()));
        assert!(deps.contains(&"quote".to_string()));
        assert!(deps.contains(&"myapp-core".to_string()));
    }

    #[test]
    fn test_extraction_recommendation() {
        let ws = sample_workspace();
        // "serde" is used by 2 crates, not enough for extraction
        let rec = ws.should_extract(&["serde"]);
        assert!(!rec.should_extract);

        // If we had 3+ crates depending on the same thing, extraction is recommended
        let mut big_ws = Workspace::new("/tmp/big");
        for i in 0..5 {
            big_ws.add_crate(CrateManifest {
                name: format!("crate-{}", i),
                version: "0.1.0".to_string(),
                path: format!("crates/crate-{}", i),
                kind: CrateKind::Library,
                dependencies: vec!["shared-logic".to_string()],
            });
        }
        let rec = big_ws.should_extract(&["shared-logic"]);
        assert!(rec.should_extract);
        assert!(rec.suggested_name.contains("shared-logic"));
    }

    #[test]
    fn test_generate_workspace_toml() {
        let ws = sample_workspace();
        let toml = generate_workspace_toml(&ws);
        assert!(toml.contains("[workspace]"));
        assert!(toml.contains("resolver = \"2\""));
        assert!(toml.contains("\"crates/core\""));
        assert!(toml.contains("[workspace.dependencies]"));
        assert!(toml.contains("serde"));
        assert!(toml.contains("derive"));
    }

    #[test]
    fn test_shared_dep_without_features() {
        let dep = SharedDep {
            name: "anyhow".to_string(),
            version: "1.0".to_string(),
            features: vec![],
            optional: false,
        };
        let mut ws = Workspace::new("/tmp");
        ws.add_shared_dep(dep);
        let toml = generate_workspace_toml(&ws);
        assert!(toml.contains("anyhow = \"1.0\""));
    }

    #[test]
    fn test_crate_kinds() {
        assert_eq!(CrateKind::Library, CrateKind::Library);
        assert_ne!(CrateKind::Library, CrateKind::Binary);
    }
}
