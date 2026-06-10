//! # Lesson 03: Dependency Pinning and Cargo.lock
//!
//! ## Why Pin Dependencies?
//!
//! When you specify a dependency version as `serde = "1.0"`, Cargo resolves this to
//! the latest compatible version. This means:
//!
//! - Different builds may get different versions
//! - A compromised version could be silently pulled in
//! - Builds are not reproducible across machines or time
//!
//! ## Cargo.lock: Your First Line of Defense
//!
//! `Cargo.lock` records the exact versions resolved for every dependency.
//! For **applications** (not libraries), you MUST commit `Cargo.lock` to version control.
//!
//! ## SemVer Ranges in Cargo.toml
//!
//! | Syntax | Meaning | Risk |
//! |--------|---------|------|
//! | `"1.0"` | `>=1.0.0, <2.0.0` | Wide range, accepts minor updates |
//! | `"1.0.5"` | `>=1.0.5, <2.0.0` | Still accepts patches |
//! | `"=1.0.5"` | Exactly `1.0.5` | Maximum pinning |
//! | `">=1.0, <1.5"` | Explicit range | Controlled range |
//!
//! ## Attack: Dependency Confusion + Lock File Manipulation
//!
//! 1. Attacker publishes a malicious version to crates.io
//! 2. If `Cargo.lock` is not committed or is regenerated, the next build
//!    may pull the malicious version
//! 3. Even with a committed lock file, `cargo update` can be tricked
//!    if the version satisfies the range in `Cargo.toml`
//!
//! ## Defense: Lock File Verification
//!
//! In this lesson, you will implement lock file parsing, integrity checking,
//! and pinning strategies to ensure deterministic builds.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Represents a single resolved dependency from a lock file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedDependency {
    pub name: String,
    pub version: String,
    /// SHA-256 checksum of the source archive (if available)
    pub checksum: Option<String>,
    /// The source registry or path
    pub source: String,
}

/// Exercise 1: Parse a simplified lock file format.
///
/// Input format (one dependency per line):
/// ```text
/// name=serde;version=1.0.188;source=crates.io;checksum=abc123
/// name=tokio;version=1.32.0;source=crates.io;checksum=def456
/// ```
///
/// Each line has `key=value` pairs separated by `;`.
/// Not all keys are required (checksum may be missing).
///
/// Hints:
/// - Split input on `\n`, skip empty lines
/// - Split each line on `;` to get pairs
/// - Split each pair on `=` (first `=` only, in case value contains `=`)
/// - Use a match or if-else to populate fields
pub fn parse_lockfile(input: &str) -> Vec<LockedDependency> {
    todo!("Parse simplified lock file format")
}

/// Exercise 2: Compute a hash of the lock file contents for integrity verification.
///
/// The hash covers the name, version, and source of each dependency,
/// in the order they appear. Format each entry as "name@version@source"
/// and hash the concatenation with SHA-256.
///
/// Hints:
/// - Create a `Sha256` hasher
/// - For each dep, feed `format!("{}@{}@{}", name, version, source)` as bytes
/// - Finalize and return the hex-encoded hash
pub fn compute_lockfile_hash(deps: &[LockedDependency]) -> String {
    todo!("Compute integrity hash of lock file")
}

/// Exercise 3: Verify a lock file's integrity against an expected hash.
///
/// Compute the hash and compare with the expected value.
/// Use constant-time comparison to prevent timing attacks.
///
/// Hints:
/// - Call `compute_lockfile_hash`
/// - Compare strings (constant-time comparison not strictly needed for hex,
///   but good practice — use `==` for simplicity here)
pub fn verify_lockfile_integrity(
    deps: &[LockedDependency],
    expected_hash: &str,
) -> bool {
    todo!("Verify lock file integrity against expected hash")
}

/// Exercise 4: Detect if a lock file has been tampered with by comparing
/// two snapshots.
///
/// Return a list of changes:
/// - Dependencies that were added (present in new, not in old)
/// - Dependencies that were removed (present in old, not in new)
/// - Dependencies whose version changed
///
/// Hints:
/// - Build HashMaps from both lockfiles keyed by name
/// - Check for additions, removals, and version changes
pub fn detect_lockfile_changes(
    old: &[LockedDependency],
    new: &[LockedDependency],
) -> Vec<LockfileChange> {
    todo!("Detect changes between two lock file snapshots")
}

/// A change detected between two lock file snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockfileChange {
    Added { name: String, version: String },
    Removed { name: String, version: String },
    VersionChanged { name: String, old_version: String, new_version: String },
}

/// Exercise 5: Validate that all dependencies have checksums (supply chain integrity).
///
/// Return the names of dependencies that are missing checksums.
///
/// Hints:
/// - Filter deps where `checksum` is None
/// - Collect their names
pub fn find_missing_checksums(deps: &[LockedDependency]) -> Vec<String> {
    todo!("Find dependencies missing checksums")
}

/// Exercise 6: Check if a specific dependency version is pinned to an exact version
/// in the Cargo.toml spec.
///
/// An exact pin uses `=` prefix, e.g., "=1.0.5".
/// A caret range like "1.0" or "1.0.5" is NOT an exact pin.
///
/// Hints:
/// - Check if the spec string starts with '='
pub fn is_exact_pin(version_spec: &str) -> bool {
    todo!("Check if a version spec is an exact pin")
}

/// Exercise 7: Recommend pinning strategy for a set of dependencies.
///
/// For each dependency, recommend:
/// - "EXACT" if it has no checksum (suggest exact pinning)
/// - "RANGE" if it has a checksum (range is acceptable with integrity check)
///
/// Return Vec of (name, recommendation) pairs.
///
/// Hints:
/// - Iterate deps, check if checksum is Some or None
pub fn recommend_pinning_strategy(deps: &[LockedDependency]) -> Vec<(String, String)> {
    todo!("Recommend pinning strategy for dependencies")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lockfile() -> Vec<LockedDependency> {
        vec![
            LockedDependency {
                name: "serde".to_string(),
                version: "1.0.188".to_string(),
                checksum: Some("a1b2c3".to_string()),
                source: "crates.io".to_string(),
            },
            LockedDependency {
                name: "tokio".to_string(),
                version: "1.32.0".to_string(),
                checksum: Some("d4e5f6".to_string()),
                source: "crates.io".to_string(),
            },
            LockedDependency {
                name: "ring".to_string(),
                version: "0.17.0".to_string(),
                checksum: None,
                source: "crates.io".to_string(),
            },
        ]
    }

    #[test]
    fn test_parse_lockfile_basic() {
        let input = "name=serde;version=1.0.188;source=crates.io;checksum=a1b2c3\nname=tokio;version=1.32.0;source=crates.io;checksum=d4e5f6";
        let deps = parse_lockfile(input);
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "serde");
        assert_eq!(deps[0].version, "1.0.188");
        assert_eq!(deps[1].name, "tokio");
    }

    #[test]
    fn test_parse_lockfile_no_checksum() {
        let input = "name=ring;version=0.17.0;source=crates.io";
        let deps = parse_lockfile(input);
        assert_eq!(deps.len(), 1);
        assert!(deps[0].checksum.is_none());
    }

    #[test]
    fn test_parse_lockfile_empty() {
        let deps = parse_lockfile("");
        assert!(deps.is_empty());
    }

    #[test]
    fn test_lockfile_hash_deterministic() {
        let deps = sample_lockfile();
        let h1 = compute_lockfile_hash(&deps);
        let h2 = compute_lockfile_hash(&deps);
        assert_eq!(h1, h2);
        assert!(!h1.is_empty());
    }

    #[test]
    fn test_lockfile_hash_changes_on_modification() {
        let deps1 = sample_lockfile();
        let mut deps2 = deps1.clone();
        deps2[0].version = "1.0.189".to_string();
        assert_ne!(compute_lockfile_hash(&deps1), compute_lockfile_hash(&deps2));
    }

    #[test]
    fn test_verify_integrity_pass() {
        let deps = sample_lockfile();
        let hash = compute_lockfile_hash(&deps);
        assert!(verify_lockfile_integrity(&deps, &hash));
    }

    #[test]
    fn test_verify_integrity_fail() {
        let deps = sample_lockfile();
        assert!(!verify_lockfile_integrity(&deps, "wrong_hash"));
    }

    #[test]
    fn test_detect_changes_added() {
        let old = vec![LockedDependency {
            name: "serde".to_string(),
            version: "1.0.188".to_string(),
            checksum: None,
            source: "crates.io".to_string(),
        }];
        let new = vec![
            LockedDependency {
                name: "serde".to_string(),
                version: "1.0.188".to_string(),
                checksum: None,
                source: "crates.io".to_string(),
            },
            LockedDependency {
                name: "tokio".to_string(),
                version: "1.32.0".to_string(),
                checksum: None,
                source: "crates.io".to_string(),
            },
        ];
        let changes = detect_lockfile_changes(&old, &new);
        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], LockfileChange::Added { name, .. } if name == "tokio"));
    }

    #[test]
    fn test_detect_changes_version_changed() {
        let old = vec![LockedDependency {
            name: "serde".to_string(),
            version: "1.0.188".to_string(),
            checksum: None,
            source: "crates.io".to_string(),
        }];
        let new = vec![LockedDependency {
            name: "serde".to_string(),
            version: "1.0.189".to_string(),
            checksum: None,
            source: "crates.io".to_string(),
        }];
        let changes = detect_lockfile_changes(&old, &new);
        assert_eq!(changes.len(), 1);
        assert!(matches!(&changes[0], LockfileChange::VersionChanged { old_version, new_version, .. }
            if old_version == "1.0.188" && new_version == "1.0.189"));
    }

    #[test]
    fn test_find_missing_checksums() {
        let deps = sample_lockfile();
        let missing = find_missing_checksums(&deps);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0], "ring");
    }

    #[test]
    fn test_is_exact_pin() {
        assert!(is_exact_pin("=1.0.5"));
        assert!(!is_exact_pin("1.0.5"));
        assert!(!is_exact_pin("^1.0.5"));
        assert!(!is_exact_pin("~1.0.5"));
    }

    #[test]
    fn test_recommend_pinning_strategy() {
        let deps = sample_lockfile();
        let recs = recommend_pinning_strategy(&deps);
        assert_eq!(recs.len(), 3);
        assert_eq!(recs[0], ("serde".to_string(), "RANGE".to_string()));
        assert_eq!(recs[2], ("ring".to_string(), "EXACT".to_string()));
    }
}
