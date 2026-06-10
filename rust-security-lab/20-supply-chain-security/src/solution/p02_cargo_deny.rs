//! # Lesson 02: cargo-deny — License and Dependency Compliance (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A dependency entry with its license information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub license: String,
    pub source: String,
}

/// Policy configuration for dependency compliance checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub allowed_licenses: Vec<String>,
    pub banned_crates: Vec<String>,
    pub allowed_sources: Vec<String>,
    pub allow_duplicates: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            allowed_licenses: vec![
                "MIT".to_string(),
                "Apache-2.0".to_string(),
                "BSD-2-Clause".to_string(),
                "BSD-3-Clause".to_string(),
                "ISC".to_string(),
                "Zlib".to_string(),
            ],
            banned_crates: vec![],
            allowed_sources: vec![
                "registry+https://github.com/rust-lang/crates.io-index".to_string(),
            ],
            allow_duplicates: false,
        }
    }
}

/// A policy violation found during compliance checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Violation {
    UnapprovedLicense {
        package: String,
        version: String,
        license: String,
    },
    BannedPackage {
        package: String,
        version: String,
        reason: String,
    },
    UnapprovedSource {
        package: String,
        version: String,
        source: String,
    },
    Duplicate {
        package: String,
        versions: Vec<String>,
    },
}

/// Check if a single license identifier is in the allowed list.
fn is_single_license_approved(license: &str, allowed: &[String]) -> bool {
    allowed.iter().any(|a| a == license)
}

/// Check if a license expression is approved by the policy.
///
/// Handles "OR" (any match), "AND" (all must match), and plain expressions.
pub fn is_license_approved(license: &str, allowed: &[String]) -> bool {
    if license.contains(" OR ") {
        // Any one license in an OR expression must be approved
        license
            .split(" OR ")
            .any(|l| is_single_license_approved(l.trim(), allowed))
    } else if license.contains(" AND ") {
        // All licenses in an AND expression must be approved
        license
            .split(" AND ")
            .all(|l| is_single_license_approved(l.trim(), allowed))
    } else {
        is_single_license_approved(license.trim(), allowed)
    }
}

/// Check a single dependency against the full policy.
pub fn check_dependency(dep: &Dependency, config: &PolicyConfig) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Check license
    if !is_license_approved(&dep.license, &config.allowed_licenses) {
        violations.push(Violation::UnapprovedLicense {
            package: dep.name.clone(),
            version: dep.version.clone(),
            license: dep.license.clone(),
        });
    }

    // Check banned crates
    if config.banned_crates.contains(&dep.name) {
        violations.push(Violation::BannedPackage {
            package: dep.name.clone(),
            version: dep.version.clone(),
            reason: "Explicitly banned by policy".to_string(),
        });
    }

    // Check source
    if !config.allowed_sources.is_empty() && !config.allowed_sources.contains(&dep.source) {
        violations.push(Violation::UnapprovedSource {
            package: dep.name.clone(),
            version: dep.version.clone(),
            source: dep.source.clone(),
        });
    }

    violations
}

/// Detect duplicate packages (same name, different versions).
pub fn detect_duplicates(
    dependencies: &[Dependency],
    allow_duplicates: bool,
) -> Vec<Violation> {
    if allow_duplicates {
        return Vec::new();
    }

    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    for dep in dependencies {
        groups
            .entry(dep.name.clone())
            .or_default()
            .push(dep.version.clone());
    }

    groups
        .into_iter()
        .filter_map(|(name, mut versions)| {
            versions.sort();
            versions.dedup();
            if versions.len() > 1 {
                Some(Violation::Duplicate {
                    package: name,
                    versions,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Run all policy checks on a full dependency list.
pub fn audit_dependencies(
    dependencies: &[Dependency],
    config: &PolicyConfig,
) -> Vec<Violation> {
    let mut violations: Vec<Violation> = dependencies
        .iter()
        .flat_map(|dep| check_dependency(dep, config))
        .collect();

    violations.extend(detect_duplicates(dependencies, config.allow_duplicates));
    violations
}

/// Format violations into a human-readable report.
pub fn format_violations(violations: &[Violation]) -> String {
    if violations.is_empty() {
        return "All dependencies pass compliance checks.".to_string();
    }
    violations
        .iter()
        .map(|v| match v {
            Violation::UnapprovedLicense {
                package,
                version,
                license,
            } => format!(
                "DENY: {} {} uses unapproved license {}",
                package, version, license
            ),
            Violation::BannedPackage {
                package,
                version,
                reason,
            } => format!("BAN: {} {} is banned: {}", package, version, reason),
            Violation::UnapprovedSource {
                package,
                version,
                source,
            } => format!(
                "SOURCE: {} {} from unapproved source {}",
                package, version, source
            ),
            Violation::Duplicate { package, versions } => {
                format!("DUP: {} has versions {}", package, versions.join(", "))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Serialize a PolicyConfig to JSON.
pub fn serialize_config(config: &PolicyConfig) -> String {
    serde_json::to_string_pretty(config).unwrap_or_else(|_| "{}".to_string())
}

/// Deserialize a PolicyConfig from JSON.
pub fn deserialize_config(json: &str) -> Option<PolicyConfig> {
    serde_json::from_str(json).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mit_dep(name: &str) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            license: "MIT".to_string(),
            source: "registry+https://github.com/rust-lang/crates.io-index".to_string(),
        }
    }

    fn gpl_dep(name: &str) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: "2.0.0".to_string(),
            license: "GPL-3.0".to_string(),
            source: "registry+https://github.com/rust-lang/crates.io-index".to_string(),
        }
    }

    #[test]
    fn test_license_approved_simple() {
        let config = PolicyConfig::default();
        assert!(is_license_approved("MIT", &config.allowed_licenses));
        assert!(is_license_approved("Apache-2.0", &config.allowed_licenses));
    }

    #[test]
    fn test_license_rejected() {
        let config = PolicyConfig::default();
        assert!(!is_license_approved("GPL-3.0", &config.allowed_licenses));
        assert!(!is_license_approved("AGPL-3.0", &config.allowed_licenses));
    }

    #[test]
    fn test_license_or_expression() {
        let config = PolicyConfig::default();
        assert!(is_license_approved("MIT OR Apache-2.0", &config.allowed_licenses));
        assert!(is_license_approved("GPL-3.0 OR MIT", &config.allowed_licenses));
        assert!(!is_license_approved("GPL-3.0 OR AGPL-3.0", &config.allowed_licenses));
    }

    #[test]
    fn test_license_and_expression() {
        let config = PolicyConfig::default();
        assert!(is_license_approved("MIT AND Apache-2.0", &config.allowed_licenses));
        assert!(!is_license_approved("MIT AND GPL-3.0", &config.allowed_licenses));
    }

    #[test]
    fn test_check_dependency_clean() {
        let config = PolicyConfig::default();
        let dep = mit_dep("serde");
        let violations = check_dependency(&dep, &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_check_dependency_banned() {
        let mut config = PolicyConfig::default();
        config.banned_crates.push("evil-crate".to_string());
        let dep = mit_dep("evil-crate");
        let violations = check_dependency(&dep, &config);
        assert_eq!(violations.len(), 1);
        assert!(matches!(violations[0], Violation::BannedPackage { .. }));
    }

    #[test]
    fn test_check_dependency_unapproved_license() {
        let config = PolicyConfig::default();
        let dep = gpl_dep("some-lib");
        let violations = check_dependency(&dep, &config);
        assert!(violations.iter().any(|v| matches!(v, Violation::UnapprovedLicense { .. })));
    }

    #[test]
    fn test_detect_duplicates_none() {
        let deps = vec![mit_dep("serde"), mit_dep("tokio")];
        let violations = detect_duplicates(&deps, false);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_detect_duplicates_found() {
        let mut dep1 = mit_dep("serde");
        dep1.version = "1.0.0".to_string();
        let mut dep2 = mit_dep("serde");
        dep2.version = "2.0.0".to_string();
        let deps = vec![dep1, dep2];
        let violations = detect_duplicates(&deps, false);
        assert_eq!(violations.len(), 1);
        assert!(matches!(&violations[0], Violation::Duplicate { .. }));
    }

    #[test]
    fn test_detect_duplicates_allowed() {
        let mut dep1 = mit_dep("serde");
        dep1.version = "1.0.0".to_string();
        let mut dep2 = mit_dep("serde");
        dep2.version = "2.0.0".to_string();
        let deps = vec![dep1, dep2];
        let violations = detect_duplicates(&deps, true);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_full_audit_clean() {
        let config = PolicyConfig::default();
        let deps = vec![mit_dep("serde"), mit_dep("tokio")];
        let violations = audit_dependencies(&deps, &config);
        assert!(violations.is_empty());
    }

    #[test]
    fn test_format_violations_empty() {
        let report = format_violations(&[]);
        assert_eq!(report, "All dependencies pass compliance checks.");
    }

    #[test]
    fn test_format_violations_present() {
        let violations = vec![Violation::UnapprovedLicense {
            package: "some-lib".to_string(),
            version: "2.0.0".to_string(),
            license: "GPL-3.0".to_string(),
        }];
        let report = format_violations(&violations);
        assert!(report.contains("DENY"));
        assert!(report.contains("GPL-3.0"));
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = PolicyConfig::default();
        let json = serialize_config(&config);
        let restored = deserialize_config(&json).unwrap();
        assert_eq!(restored.allowed_licenses, config.allowed_licenses);
        assert_eq!(restored.banned_crates, config.banned_crates);
    }
}
