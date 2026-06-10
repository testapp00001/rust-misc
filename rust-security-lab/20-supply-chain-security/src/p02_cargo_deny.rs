//! # Lesson 02: cargo-deny — License and Dependency Compliance
//!
//! ## What is cargo-deny?
//!
//! `cargo-deny` is a cargo plugin that lints your dependency graph for multiple policy
//! violations. Unlike `cargo-audit` which only checks for security advisories, `cargo-deny`
//! enforces organizational policies across several dimensions.
//!
//! ## Checks Performed
//!
//! - **Licenses**: Ensure all dependencies use only approved licenses
//! - **Bans**: Ban specific crates or specific versions
//! - **Sources**: Restrict where dependencies can come from (e.g., only crates.io)
//! - **Duplicates**: Detect when multiple versions of the same crate are pulled in
//!
//! ## Why License Compliance Matters
//!
//! Using a dependency with a copyleft license (like GPL) in your proprietary software
//! can create legal obligations to open-source your entire codebase. `cargo-deny`
//! prevents this by blocking builds that include unapproved licenses.
//!
//! ## Attack: License Laundering
//!
//! An attacker publishes a crate with a permissive license, but includes a dependency
//! with a restrictive license deep in the dependency tree. Without automated checks,
//! this goes unnoticed until legal review (which may never happen).
//!
//! ## Defense: License Policy Engine
//!
//! In this lesson, you will implement a license policy engine that evaluates
//! dependency licenses against configurable allowlists and denylists.

use serde::{Deserialize, Serialize};

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
    /// SPDX license expressions that are allowed (e.g., "MIT", "Apache-2.0")
    pub allowed_licenses: Vec<String>,
    /// Crate names that are explicitly banned
    pub banned_crates: Vec<String>,
    /// Allowed dependency sources (e.g., "registry+https://github.com/rust-lang/crates.io-index")
    pub allowed_sources: Vec<String>,
    /// Whether to allow duplicate versions of the same crate
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
    /// License not in the allowed list.
    UnapprovedLicense {
        package: String,
        version: String,
        license: String,
    },
    /// Package is on the ban list.
    BannedPackage {
        package: String,
        version: String,
        reason: String,
    },
    /// Source is not in the allowed sources list.
    UnapprovedSource {
        package: String,
        version: String,
        source: String,
    },
    /// Multiple versions of the same crate detected.
    Duplicate {
        package: String,
        versions: Vec<String>,
    },
}

/// Exercise 1: Check if a license is approved by the policy.
///
/// SPDX license expressions can contain:
/// - Simple: "MIT", "Apache-2.0"
/// - Compound: "MIT OR Apache-2.0" (any one is sufficient)
/// - With exceptions: "MIT AND Apache-2.0" (all must be approved)
///
/// For this exercise, handle "OR" expressions: split on " OR " and check
/// if ANY of the licenses is in the allowed list.
///
/// For "AND" expressions: split on " AND " and check if ALL are approved.
/// For plain licenses: check directly against the allowed list.
///
/// Hints:
/// - Check for " OR " first, then " AND ", then plain match
/// - Use `Vec::contains` or `.any()` for OR
/// - Use `.all()` for AND
pub fn is_license_approved(license: &str, allowed: &[String]) -> bool {
    todo!("Check if a license expression is approved")
}

/// Exercise 2: Check a single dependency against the full policy.
///
/// Returns a Vec of violations found. A dependency can have multiple violations
/// (e.g., unapproved license AND banned source).
///
/// Hints:
/// - Check license with `is_license_approved`
/// - Check if name is in `banned_crates`
/// - Check if source is in `allowed_sources`
/// - Push each violation found
pub fn check_dependency(dep: &Dependency, config: &PolicyConfig) -> Vec<Violation> {
    todo!("Check a dependency against the full policy")
}

/// Exercise 3: Detect duplicate packages (same name, different versions).
///
/// Group dependencies by name. For each group with more than one distinct version,
/// create a `Duplicate` violation (unless `allow_duplicates` is true).
///
/// Hints:
/// - Use a `HashMap<String, Vec<String>>` to group by name
/// - Filter groups with > 1 distinct version
/// - Sort versions for deterministic output
pub fn detect_duplicates(
    dependencies: &[Dependency],
    allow_duplicates: bool,
) -> Vec<Violation> {
    todo!("Detect duplicate package versions")
}

/// Exercise 4: Run all policy checks on a full dependency list.
///
/// Returns all violations found across license checks, ban checks,
/// source checks, and duplicate detection.
///
/// Hints:
/// - Iterate dependencies, call `check_dependency` for each
/// - Call `detect_duplicates` once for the full list
/// - Combine all violations into one Vec
pub fn audit_dependencies(
    dependencies: &[Dependency],
    config: &PolicyConfig,
) -> Vec<Violation> {
    todo!("Run full compliance audit on dependencies")
}

/// Exercise 5: Format violations into a human-readable report.
///
/// Format each violation on its own line:
/// - "DENY: {package} {version} uses unapproved license {license}"
/// - "BAN: {package} {version} is banned: {reason}"
/// - "SOURCE: {package} {version} from unapproved source {source}"
/// - "DUP: {package} has versions {v1}, {v2}, ..."
///
/// If no violations, return "All dependencies pass compliance checks."
pub fn format_violations(violations: &[Violation]) -> String {
    todo!("Format violations into a human-readable report")
}

/// Exercise 6: Serialize a PolicyConfig to JSON.
///
/// Hints:
/// - Use `serde_json::to_string_pretty` for readable output
pub fn serialize_config(config: &PolicyConfig) -> String {
    todo!("Serialize PolicyConfig to JSON")
}

/// Exercise 7: Deserialize a PolicyConfig from JSON.
///
/// Hints:
/// - Use `serde_json::from_str`
pub fn deserialize_config(json: &str) -> Option<PolicyConfig> {
    todo!("Deserialize JSON into PolicyConfig")
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
