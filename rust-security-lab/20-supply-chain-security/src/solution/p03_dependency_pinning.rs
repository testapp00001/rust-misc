//! # Lesson 03: Dependency Pinning and Cargo.lock (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Represents a single resolved dependency from a lock file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedDependency {
    pub name: String,
    pub version: String,
    pub checksum: Option<String>,
    pub source: String,
}

/// Parse a simplified lock file format.
pub fn parse_lockfile(input: &str) -> Vec<LockedDependency> {
    input
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut name = String::new();
            let mut version = String::new();
            let mut source = String::new();
            let mut checksum = None;

            for pair in line.split(';') {
                if let Some((key, value)) = pair.split_once('=') {
                    match key.trim() {
                        "name" => name = value.trim().to_string(),
                        "version" => version = value.trim().to_string(),
                        "source" => source = value.trim().to_string(),
                        "checksum" => checksum = Some(value.trim().to_string()),
                        _ => {}
                    }
                }
            }

            if !name.is_empty() && !version.is_empty() {
                Some(LockedDependency {
                    name,
                    version,
                    checksum,
                    source,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Compute a hash of the lock file contents for integrity verification.
pub fn compute_lockfile_hash(deps: &[LockedDependency]) -> String {
    let mut hasher = Sha256::new();
    for dep in deps {
        let entry = format!("{}@{}@{}", dep.name, dep.version, dep.source);
        hasher.update(entry.as_bytes());
    }
    hex::encode(hasher.finalize())
}

/// Verify lock file integrity against an expected hash.
pub fn verify_lockfile_integrity(
    deps: &[LockedDependency],
    expected_hash: &str,
) -> bool {
    compute_lockfile_hash(deps) == expected_hash
}

/// Detect changes between two lock file snapshots.
pub fn detect_lockfile_changes(
    old: &[LockedDependency],
    new: &[LockedDependency],
) -> Vec<LockfileChange> {
    let old_map: HashMap<&str, &LockedDependency> =
        old.iter().map(|d| (d.name.as_str(), d)).collect();
    let new_map: HashMap<&str, &LockedDependency> =
        new.iter().map(|d| (d.name.as_str(), d)).collect();

    let mut changes = Vec::new();

    // Find additions and version changes
    for (name, new_dep) in &new_map {
        match old_map.get(name) {
            None => {
                changes.push(LockfileChange::Added {
                    name: name.to_string(),
                    version: new_dep.version.clone(),
                });
            }
            Some(old_dep) => {
                if old_dep.version != new_dep.version {
                    changes.push(LockfileChange::VersionChanged {
                        name: name.to_string(),
                        old_version: old_dep.version.clone(),
                        new_version: new_dep.version.clone(),
                    });
                }
            }
        }
    }

    // Find removals
    for (name, old_dep) in &old_map {
        if !new_map.contains_key(name) {
            changes.push(LockfileChange::Removed {
                name: name.to_string(),
                version: old_dep.version.clone(),
            });
        }
    }

    changes
}

/// A change detected between two lock file snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockfileChange {
    Added { name: String, version: String },
    Removed { name: String, version: String },
    VersionChanged { name: String, old_version: String, new_version: String },
}

/// Find dependencies missing checksums.
pub fn find_missing_checksums(deps: &[LockedDependency]) -> Vec<String> {
    deps.iter()
        .filter(|d| d.checksum.is_none())
        .map(|d| d.name.clone())
        .collect()
}

/// Check if a version spec is an exact pin.
pub fn is_exact_pin(version_spec: &str) -> bool {
    version_spec.starts_with('=')
}

/// Recommend pinning strategy for dependencies.
pub fn recommend_pinning_strategy(deps: &[LockedDependency]) -> Vec<(String, String)> {
    deps.iter()
        .map(|d| {
            let strategy = if d.checksum.is_none() {
                "EXACT".to_string()
            } else {
                "RANGE".to_string()
            };
            (d.name.clone(), strategy)
        })
        .collect()
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
