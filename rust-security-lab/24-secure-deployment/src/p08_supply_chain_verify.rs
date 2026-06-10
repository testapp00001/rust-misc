//! # Lesson 08: Supply Chain Verification
//!
//! ## Attack: Dependency Confusion
//!
//! An attacker publishes a malicious crate with the same name as your private
//! internal dependency. Cargo resolves the public version instead of your private
//! one, and the attacker's code runs in your build and production environments.
//!
//! This attack has been demonstrated against Apple, Microsoft, and Tesla.
//!
//! ## Defend: Verify Dependency Integrity
//!
//! 1. **Lock files**: Commit `Cargo.lock` and use `--locked`
//! 2. **Checksums**: Verify downloaded crate checksums against the registry
//! 3. **Source pinning**: Use `[patch]` or git dependencies for internal crates
//! 4. **Audit**: Run `cargo audit` to check for known vulnerabilities
//! 5. **Minimal dependencies**: Each dependency is an attack surface
//!
//! ## Audit: Supply Chain Checklist
//!
//! - [ ] Cargo.lock committed to version control
//! - [ ] `cargo build --locked` used in CI
//! - [ ] `cargo audit` runs on every PR
//! - [ ] No wildcard version requirements (">=*")
//! - [ ] All git dependencies pin to specific commits
//! - [ ] Dependencies have been reviewed for trustworthiness

use serde::{Deserialize, Serialize};

/// A dependency entry from Cargo.lock.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dependency {
    /// Package name
    pub name: String,
    /// Exact version
    pub version: String,
    /// Source: "registry", "git", or "path"
    pub source: String,
    /// SHA-256 checksum (for registry crates)
    pub checksum: String,
    /// Git commit hash (for git dependencies)
    pub git_commit: String,
}

/// A Cargo.lock file representation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LockFile {
    /// Lock file format version
    pub version: u32,
    /// All dependencies
    pub dependencies: Vec<Dependency>,
}

/// Exercise 1: Verify that a dependency matches its expected checksum.
///
/// Given a dependency and an expected SHA-256 checksum, return true if they match.
/// The comparison should be case-insensitive (hex is case-insensitive).
///
/// If the dependency has an empty checksum (e.g., git or path deps), return true.
pub fn verify_checksum(dep: &Dependency, expected_checksum: &str) -> bool {
    todo!("Verify dependency checksum")
}

/// Exercise 2: Audit a lock file for security issues.
///
/// Check for these problems:
/// 1. Any dependency with version containing "*" (wildcard)
/// 2. Any git dependency without a specific commit hash (empty `git_commit`)
/// 3. Any registry dependency with an empty checksum
/// 4. Any dependency with a pre-release version (contains "-alpha", "-beta", "-rc")
///
/// Return a list of issue descriptions. Each should mention the dependency name.
pub fn audit_lock_file(lock_file: &LockFile) -> Vec<String> {
    todo!("Audit lock file for security issues")
}

/// Exercise 3: Find all git dependencies.
///
/// Return references to all dependencies where `source` is "git".
pub fn find_git_dependencies(lock_file: &LockFile) -> Vec<&Dependency> {
    todo!("Find all git dependencies in a lock file")
}

/// Exercise 4: Check for duplicate package names with different versions.
///
/// This is a potential supply chain attack vector (dependency confusion).
/// Return the names of packages that appear more than once with different versions.
pub fn find_duplicates(lock_file: &LockFile) -> Vec<String> {
    todo!("Find duplicate package names with different versions")
}

/// Exercise 5: Generate a dependency inventory summary.
///
/// Return a string with:
/// - Total number of dependencies
/// - Number of registry vs git vs path dependencies
/// - Number of unique publishers (count unique checksum prefixes, first 8 chars)
/// - List of git dependencies with their commit hashes
///
/// Format:
/// ```text
/// Dependency Inventory
/// ====================
/// Total: {count}
/// Registry: {count}, Git: {count}, Path: {count}
/// Unique sources: {count}
///
/// Git Dependencies:
///   {name} @ {commit_hash}
///   ...
/// ```
pub fn dependency_inventory(lock_file: &LockFile) -> String {
    todo!("Generate a dependency inventory summary")
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
        let dep = &sample_lock_file().dependencies[1]; // git dep
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
