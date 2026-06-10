//! # Lesson 08: Supply Chain Verification (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: String,
    pub git_commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockFile {
    pub version: u32,
    pub dependencies: Vec<Dependency>,
}

/// Verify that a dependency matches its expected checksum.
pub fn verify_checksum(dep: &Dependency, expected_checksum: &str) -> bool {
    if dep.checksum.is_empty() {
        return true; // Git/path deps don't have checksums
    }
    dep.checksum.to_lowercase() == expected_checksum.to_lowercase()
}

/// Audit a lock file for security issues.
pub fn audit_lock_file(lock_file: &LockFile) -> Vec<String> {
    let mut issues = Vec::new();

    for dep in &lock_file.dependencies {
        // Check for wildcard versions
        if dep.version.contains('*') {
            issues.push(format!("{}: version '{}' contains wildcard", dep.name, dep.version));
        }

        // Check for git deps without specific commit
        if dep.source == "git" && dep.git_commit.is_empty() {
            issues.push(format!("{}: git dependency without specific commit hash", dep.name));
        }

        // Check for registry deps without checksum
        if dep.source == "registry" && dep.checksum.is_empty() {
            issues.push(format!("{}: registry dependency without checksum", dep.name));
        }

        // Check for pre-release versions
        if dep.version.contains("-alpha") || dep.version.contains("-beta")
            || dep.version.contains("-rc") || dep.version.contains("-pre")
        {
            issues.push(format!("{}: pre-release version '{}'", dep.name, dep.version));
        }
    }

    issues
}

/// Find all git dependencies in a lock file.
pub fn find_git_dependencies(lock_file: &LockFile) -> Vec<&Dependency> {
    lock_file.dependencies.iter()
        .filter(|d| d.source == "git")
        .collect()
}

/// Find duplicate package names with different versions.
pub fn find_duplicates(lock_file: &LockFile) -> Vec<String> {
    use std::collections::HashMap;

    let mut versions_by_name: HashMap<&str, Vec<&str>> = HashMap::new();
    for dep in &lock_file.dependencies {
        versions_by_name.entry(&dep.name)
            .or_default()
            .push(&dep.version);
    }

    versions_by_name.iter()
        .filter(|(_, versions)| {
            let unique: std::collections::HashSet<&str> = versions.iter().copied().collect();
            unique.len() > 1
        })
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Generate a dependency inventory summary.
pub fn dependency_inventory(lock_file: &LockFile) -> String {
    let total = lock_file.dependencies.len();
    let registry = lock_file.dependencies.iter().filter(|d| d.source == "registry").count();
    let git = lock_file.dependencies.iter().filter(|d| d.source == "git").count();
    let path = lock_file.dependencies.iter().filter(|d| d.source == "path").count();

    // Count unique checksum prefixes (first 8 chars) as a proxy for unique sources
    let mut unique_sources: std::collections::HashSet<String> = std::collections::HashSet::new();
    for dep in &lock_file.dependencies {
        if dep.checksum.len() >= 8 {
            unique_sources.insert(dep.checksum[..8].to_string());
        } else if dep.source == "git" && !dep.git_commit.is_empty() {
            unique_sources.insert(format!("git:{}", &dep.git_commit[..8.min(dep.git_commit.len())]));
        }
    }

    let mut git_lines = String::new();
    let git_deps = find_git_dependencies(lock_file);
    for dep in &git_deps {
        git_lines.push_str(&format!("  {} @ {}\n", dep.name, dep.git_commit));
    }

    format!(
        "Dependency Inventory\n\
         ====================\n\
         Total: {}\n\
         Registry: {}, Git: {}, Path: {}\n\
         Unique sources: {}\n\n\
         Git Dependencies:\n{}",
        total, registry, git, path, unique_sources.len(), git_lines
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lock_file() -> LockFile {
        LockFile {
            version: 3,
            dependencies: vec![
                Dependency {
                    name: "serde".to_string(),
                    version: "1.0.193".to_string(),
                    source: "registry".to_string(),
                    checksum: "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string(),
                    git_commit: String::new(),
                },
                Dependency {
                    name: "my-internal-lib".to_string(),
                    version: "0.1.0".to_string(),
                    source: "git".to_string(),
                    checksum: String::new(),
                    git_commit: "abc123def456abc123def456abc123def456abc1".to_string(),
                },
                Dependency {
                    name: "ring".to_string(),
                    version: "0.17.7".to_string(),
                    source: "registry".to_string(),
                    checksum: "b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3".to_string(),
                    git_commit: String::new(),
                },
                Dependency {
                    name: "local-utils".to_string(),
                    version: "0.2.0".to_string(),
                    source: "path".to_string(),
                    checksum: String::new(),
                    git_commit: String::new(),
                },
            ],
        }
    }

    #[test]
    fn test_verify_checksum_match() {
        let dep = &sample_lock_file().dependencies[0];
        assert!(verify_checksum(dep, &dep.checksum));
    }

    #[test]
    fn test_verify_checksum_case_insensitive() {
        let dep = &sample_lock_file().dependencies[0];
        assert!(verify_checksum(dep, &dep.checksum.to_uppercase()));
    }

    #[test]
    fn test_verify_checksum_mismatch() {
        let dep = &sample_lock_file().dependencies[0];
        assert!(!verify_checksum(dep, "0000000000000000000000000000000000000000000000000000000000000000"));
    }

    #[test]
    fn test_verify_checksum_empty_for_git() {
        let dep = &sample_lock_file().dependencies[1];
        assert!(verify_checksum(dep, "anything"));
    }

    #[test]
    fn test_audit_lock_file_clean() {
        let lock_file = sample_lock_file();
        let issues = audit_lock_file(&lock_file);
        assert!(issues.is_empty(), "Clean lock file should pass, got: {:?}", issues);
    }

    #[test]
    fn test_audit_lock_file_git_no_commit() {
        let mut lock_file = sample_lock_file();
        lock_file.dependencies[1].git_commit = String::new();
        let issues = audit_lock_file(&lock_file);
        assert!(issues.iter().any(|i| i.contains("commit") || i.contains("my-internal-lib")));
    }

    #[test]
    fn test_audit_lock_file_missing_checksum() {
        let mut lock_file = sample_lock_file();
        lock_file.dependencies[0].checksum = String::new();
        let issues = audit_lock_file(&lock_file);
        assert!(issues.iter().any(|i| i.contains("checksum") || i.contains("serde")));
    }

    #[test]
    fn test_audit_lock_file_prerelease() {
        let mut lock_file = sample_lock_file();
        lock_file.dependencies[0].version = "2.0.0-alpha.1".to_string();
        let issues = audit_lock_file(&lock_file);
        assert!(issues.iter().any(|i| i.contains("pre-release") || i.contains("alpha") || i.contains("serde")));
    }

    #[test]
    fn test_find_git_dependencies() {
        let lock_file = sample_lock_file();
        let git_deps = find_git_dependencies(&lock_file);
        assert_eq!(git_deps.len(), 1);
        assert_eq!(git_deps[0].name, "my-internal-lib");
    }

    #[test]
    fn test_find_duplicates() {
        let mut lock_file = sample_lock_file();
        lock_file.dependencies.push(Dependency {
            name: "serde".to_string(),
            version: "1.0.190".to_string(),
            source: "registry".to_string(),
            checksum: "different".to_string(),
            git_commit: String::new(),
        });
        let dupes = find_duplicates(&lock_file);
        assert!(dupes.contains(&"serde".to_string()));
    }

    #[test]
    fn test_dependency_inventory() {
        let lock_file = sample_lock_file();
        let inventory = dependency_inventory(&lock_file);
        assert!(inventory.contains("4"), "Should list 4 total dependencies");
        assert!(inventory.contains("my-internal-lib"), "Should list git dependencies");
    }
}
