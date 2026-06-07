//! # Lesson 7: Dependency Management
//!
//! Managing dependencies effectively is critical for project health.
//! This lesson covers SemVer, version pinning, dependency resolution,
//! cargo audit, cargo tree, and strategies for keeping dependencies updated.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A semantic version with major.minor.patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Parse a "major.minor.patch" string.
    pub fn parse(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Expected major.minor.patch, got '{}'", s));
        }
        let major = parts[0]
            .parse::<u32>()
            .map_err(|e| format!("Invalid major: {}", e))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|e| format!("Invalid minor: {}", e))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|e| format!("Invalid patch: {}", e))?;
        Ok(Self::new(major, minor, patch))
    }

    /// Check if this version is compatible with a caret requirement (^).
    /// ^1.2.3 means >=1.2.3 and <2.0.0
    pub fn is_compatible_caret(&self, requirement: &SemVer) -> bool {
        if self.major != requirement.major {
            return false;
        }
        if self.major == 0 && requirement.major == 0 {
            // 0.x.y: only patch updates are compatible
            self.minor == requirement.minor && self.patch >= requirement.patch
        } else {
            *self >= *requirement
        }
    }

    /// Check if this version satisfies a tilde requirement (~).
    /// ~1.2.3 means >=1.2.3 and <1.3.0
    pub fn is_compatible_tilde(&self, requirement: &SemVer) -> bool {
        self.major == requirement.major
            && self.minor == requirement.minor
            && self.patch >= requirement.patch
    }

    /// Compute the next version bump.
    pub fn bump_major(&self) -> Self {
        Self::new(self.major + 1, 0, 0)
    }

    pub fn bump_minor(&self) -> Self {
        Self::new(self.major, self.minor + 1, 0)
    }

    pub fn bump_patch(&self) -> Self {
        Self::new(self.major, self.minor, self.patch + 1)
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A dependency with version constraints and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub required_version: SemVer,
    pub resolved_version: SemVer,
    pub source: DependencySource,
    pub is_dev: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencySource {
    CratesIo,
    Git(String),
    Path(String),
}

/// A dependency lock file entry (simplified Cargo.lock).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockEntry {
    pub name: String,
    pub version: SemVer,
    pub checksum: String,
    pub dependencies: Vec<String>,
}

/// A dependency tree showing what depends on what.
#[derive(Debug, Serialize, Deserialize)]
pub struct DependencyTree {
    pub root: String,
    pub entries: BTreeMap<String, Vec<DependencyEdge>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub version_req: String,
    pub is_dev: bool,
}

impl DependencyTree {
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            entries: BTreeMap::new(),
        }
    }

    pub fn add_edge(&mut self, edge: DependencyEdge) {
        self.entries
            .entry(edge.from.clone())
            .or_default()
            .push(edge);
    }

    /// Get all direct dependencies of a given crate.
    pub fn direct_deps(&self, crate_name: &str) -> Vec<&DependencyEdge> {
        self.entries
            .get(crate_name)
            .map(|edges| edges.iter().collect())
            .unwrap_or_default()
    }

    /// Count total unique dependencies (transitive).
    pub fn total_unique_deps(&self) -> usize {
        let mut seen = std::collections::HashSet::new();
        for edges in self.entries.values() {
            for edge in edges {
                seen.insert(&edge.to);
            }
        }
        seen.len()
    }

    /// Find the longest dependency chain.
    pub fn max_depth(&self) -> usize {
        self.max_depth_from(&self.root, &mut std::collections::HashSet::new())
    }

    fn max_depth_from(
        &self,
        name: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> usize {
        if !visited.insert(name.to_string()) {
            return 0; // cycle detection
        }
        let deps = self.direct_deps(name);
        if deps.is_empty() {
            visited.remove(name);
            return 0;
        }
        let max_child = deps
            .iter()
            .map(|dep| self.max_depth_from(&dep.to, visited))
            .max()
            .unwrap_or(0);
        visited.remove(name);
        1 + max_child
    }

    /// Detect circular dependencies.
    pub fn has_cycles(&self) -> bool {
        let mut visited = std::collections::HashSet::new();
        let mut in_stack = std::collections::HashSet::new();
        self.detect_cycle(&self.root, &mut visited, &mut in_stack)
    }

    fn detect_cycle(
        &self,
        name: &str,
        visited: &mut std::collections::HashSet<String>,
        in_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        if in_stack.contains(name) {
            return true;
        }
        if !visited.insert(name.to_string()) {
            return false;
        }
        in_stack.insert(name.to_string());
        for dep in self.direct_deps(name) {
            if self.detect_cycle(&dep.to, visited, in_stack) {
                return true;
            }
        }
        in_stack.remove(name);
        false
    }
}

/// Represents an audit finding (simulating cargo audit output).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub advisory_id: String,
    pub package: String,
    pub version: SemVer,
    pub severity: Severity,
    pub title: String,
    pub patched_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Run a simulated audit against a list of dependencies.
pub fn simulate_audit(
    deps: &[Dependency],
    advisories: &[AuditFinding],
) -> Vec<AuditFinding> {
    advisories
        .iter()
        .filter(|advisory| {
            deps.iter().any(|dep| {
                dep.name == advisory.package && dep.resolved_version == advisory.version
            })
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v, SemVer::new(1, 2, 3));
    }

    #[test]
    fn test_semver_parse_error() {
        assert!(SemVer::parse("1.2").is_err());
        assert!(SemVer::parse("a.b.c").is_err());
        assert!(SemVer::parse("1.2.3.4").is_err());
    }

    #[test]
    fn test_semver_display() {
        let v = SemVer::new(2, 10, 0);
        assert_eq!(v.to_string(), "2.10.0");
    }

    #[test]
    fn test_semver_ordering() {
        assert!(SemVer::new(1, 0, 0) < SemVer::new(2, 0, 0));
        assert!(SemVer::new(1, 0, 0) < SemVer::new(1, 1, 0));
        assert!(SemVer::new(1, 0, 0) < SemVer::new(1, 0, 1));
        assert!(SemVer::new(1, 2, 3) == SemVer::new(1, 2, 3));
    }

    #[test]
    fn test_caret_compatibility() {
        let req = SemVer::new(1, 2, 3);
        assert!(SemVer::new(1, 2, 3).is_compatible_caret(&req));
        assert!(SemVer::new(1, 3, 0).is_compatible_caret(&req));
        assert!(SemVer::new(1, 99, 99).is_compatible_caret(&req));
        assert!(!SemVer::new(2, 0, 0).is_compatible_caret(&req));
        assert!(!SemVer::new(1, 2, 2).is_compatible_caret(&req));
    }

    #[test]
    fn test_caret_compatibility_zero_major() {
        let req = SemVer::new(0, 2, 3);
        assert!(SemVer::new(0, 2, 3).is_compatible_caret(&req));
        assert!(SemVer::new(0, 2, 5).is_compatible_caret(&req));
        assert!(!SemVer::new(0, 3, 0).is_compatible_caret(&req));
        assert!(!SemVer::new(1, 0, 0).is_compatible_caret(&req));
    }

    #[test]
    fn test_tilde_compatibility() {
        let req = SemVer::new(1, 2, 3);
        assert!(SemVer::new(1, 2, 3).is_compatible_tilde(&req));
        assert!(SemVer::new(1, 2, 99).is_compatible_tilde(&req));
        assert!(!SemVer::new(1, 3, 0).is_compatible_tilde(&req));
        assert!(!SemVer::new(2, 0, 0).is_compatible_tilde(&req));
    }

    #[test]
    fn test_version_bumps() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.bump_major(), SemVer::new(2, 0, 0));
        assert_eq!(v.bump_minor(), SemVer::new(1, 3, 0));
        assert_eq!(v.bump_patch(), SemVer::new(1, 2, 4));
    }

    #[test]
    fn test_dependency_tree_direct_deps() {
        let mut tree = DependencyTree::new("myapp");
        tree.add_edge(DependencyEdge {
            from: "myapp".to_string(),
            to: "serde".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        tree.add_edge(DependencyEdge {
            from: "myapp".to_string(),
            to: "tokio".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });

        let deps = tree.direct_deps("myapp");
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_dependency_tree_total_unique() {
        let mut tree = DependencyTree::new("myapp");
        tree.add_edge(DependencyEdge {
            from: "myapp".to_string(),
            to: "serde".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        tree.add_edge(DependencyEdge {
            from: "serde".to_string(),
            to: "serde_derive".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        assert_eq!(tree.total_unique_deps(), 2);
    }

    #[test]
    fn test_dependency_tree_max_depth() {
        let mut tree = DependencyTree::new("myapp");
        tree.add_edge(DependencyEdge {
            from: "myapp".to_string(),
            to: "a".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        tree.add_edge(DependencyEdge {
            from: "a".to_string(),
            to: "b".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        tree.add_edge(DependencyEdge {
            from: "b".to_string(),
            to: "c".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        assert_eq!(tree.max_depth(), 3);
    }

    #[test]
    fn test_dependency_tree_no_cycles() {
        let mut tree = DependencyTree::new("myapp");
        tree.add_edge(DependencyEdge {
            from: "myapp".to_string(),
            to: "serde".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        assert!(!tree.has_cycles());
    }

    #[test]
    fn test_dependency_tree_cycles() {
        let mut tree = DependencyTree::new("a");
        tree.add_edge(DependencyEdge {
            from: "a".to_string(),
            to: "b".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        tree.add_edge(DependencyEdge {
            from: "b".to_string(),
            to: "c".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        tree.add_edge(DependencyEdge {
            from: "c".to_string(),
            to: "a".to_string(),
            version_req: "1.0".to_string(),
            is_dev: false,
        });
        assert!(tree.has_cycles());
    }

    #[test]
    fn test_simulate_audit() {
        let deps = vec![
            Dependency {
                name: "serde".to_string(),
                required_version: SemVer::new(1, 0, 0),
                resolved_version: SemVer::new(1, 0, 150),
                source: DependencySource::CratesIo,
                is_dev: false,
                is_optional: false,
            },
            Dependency {
                name: "old-crate".to_string(),
                required_version: SemVer::new(0, 1, 0),
                resolved_version: SemVer::new(0, 1, 5),
                source: DependencySource::CratesIo,
                is_dev: false,
                is_optional: false,
            },
        ];

        let advisories = vec![AuditFinding {
            advisory_id: "RUSTSEC-2023-0001".to_string(),
            package: "old-crate".to_string(),
            version: SemVer::new(0, 1, 5),
            severity: Severity::High,
            title: "Vulnerability in old-crate".to_string(),
            patched_versions: vec![">=0.2.0".to_string()],
        }];

        let findings = simulate_audit(&deps, &advisories);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].advisory_id, "RUSTSEC-2023-0001");
    }

    #[test]
    fn test_simulate_audit_no_findings() {
        let deps = vec![Dependency {
            name: "serde".to_string(),
            required_version: SemVer::new(1, 0, 0),
            resolved_version: SemVer::new(1, 0, 150),
            source: DependencySource::CratesIo,
            is_dev: false,
            is_optional: false,
        }];

        let findings = simulate_audit(&deps, &[]);
        assert!(findings.is_empty());
    }
}
