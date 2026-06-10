//! # Lesson 06: Dependency Auditing in CI
//!
//! ## The Problem
//!
//! Your code might be secure, but what about your dependencies? The average Rust
//! project has 50-200 transitive dependencies. Any one of them could have a
//! known vulnerability.
//!
//! ```text
//! your-app
//!   ├── ring 0.17        (1 direct dep)
//!   ├── sha2 0.10        (1 direct dep)
//!   └── ...              (50+ transitive deps)
//!       ├── libc 0.2     (CVE-2023-XXXXX?)
//!       └── nix 0.26     (CVE-2023-YYYYY?)
//! ```
//!
//! ## Tools
//!
//! ### cargo-audit
//! Checks dependencies against the RustSec Advisory Database:
//! ```bash
//! cargo audit
//! ```
//!
//! ### cargo-deny
//! Policy enforcement: bans specific crates, checks licenses, detects duplicates:
//! ```bash
//! cargo deny check advisories
//! cargo deny check licenses
//! cargo deny check bans
//! ```
//!
//! ## Attack: Supply Chain Vulnerability
//!
//! 1. A dependency you use (e.g., `hyper 0.14.18`) has a published CVE
//! 2. Attacker scans for apps using vulnerable versions
//! 3. Attacker exploits the vulnerability in your production environment
//!
//! Defense: automated auditing on every PR blocks vulnerable dependencies.
//!
//! ## Exercise
//!
//! Implement dependency policy checking logic (the core of what cargo-deny does).

/// Exercise 1: Check if a crate version is in a ban list.
///
/// cargo-deny's `[bans]` section lets you block specific crate versions.
///
/// Requirements:
/// - Return `true` if the crate name is in the ban list
/// - Return `false` otherwise
/// - Case-sensitive matching on crate name
pub fn is_crate_banned(crate_name: &str, ban_list: &[&str]) -> bool {
    todo!("Check if crate is in the ban list")
}

/// Exercise 2: Validate a crate version against known vulnerable versions.
///
/// Requirements:
/// - Check if `(crate_name, version)` matches any entry in `advisories`
/// - Each advisory is a tuple of `(name, vulnerable_version)`
/// - Return `Some(advisory_index)` if vulnerable, `None` if clean
pub fn check_vulnerability(
    crate_name: &str,
    version: &str,
    advisories: &[(&str, &str)],
) -> Option<usize> {
    todo!("Check crate version against vulnerability advisories")
}

/// Exercise 3: Validate a license against an allowed list.
///
/// Requirements:
/// - Return `true` if `license` is in the `allowed_licenses` list
/// - Return `false` otherwise
/// - Handle SPDX expressions: if license contains "OR", split and check each part
pub fn is_license_allowed(license: &str, allowed_licenses: &[&str]) -> bool {
    todo!("Check if license is in the allowed list")
}

/// Exercise 4: Detect duplicate dependencies (same crate, different versions).
///
/// Requirements:
/// - Given a list of `(crate_name, version)` pairs, find crates that appear
///   more than once with DIFFERENT versions
/// - Return a list of crate names that have version conflicts
/// - A crate appearing multiple times with the SAME version is not a conflict
pub fn detect_duplicate_versions(dependencies: &[(&str, &str)]) -> Vec<String> {
    todo!("Detect duplicate dependency versions")
}

/// Exercise 5: Check minimum dependency age policy.
///
/// Some organizations require dependencies to be published for a minimum
/// number of days before use (to avoid zero-day supply chain attacks).
///
/// Requirements:
/// - A dependency is "too new" if its `days_since_publish` is less than `min_age_days`
/// - Return `true` if the dependency is old enough, `false` if too new
pub fn is_dependency_mature(days_since_publish: u64, min_age_days: u64) -> bool {
    todo!("Check if dependency meets minimum age requirement")
}

/// Exercise 6: Generate a dependency audit summary.
///
/// Requirements:
/// - Given counts of `total`, `vulnerable`, `banned`, and `license_violations`,
///   return a summary string
/// - Format: "Audited X deps: Y vulnerable, Z banned, W license issues"
/// - If all counts are 0 for issues, append " [PASS]"
/// - If any count > 0, append " [FAIL]"
pub fn audit_summary(total: usize, vulnerable: usize, banned: usize, license_violations: usize) -> String {
    todo!("Generate audit summary string")
}

/// Exercise 7: Check if a crate's source is trusted.
///
/// Requirements:
/// - Trusted sources: "crates.io", "github.com"
/// - Return `true` if `source` contains any trusted source string (case-insensitive)
/// - Return `false` otherwise (e.g., "unknown-registry", "local path")
pub fn is_trusted_source(source: &str) -> bool {
    todo!("Check if dependency source is trusted")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_crate_banned_found() {
        assert!(is_crate_banned("openssl", &["openssl", "native-tls"]));
    }

    #[test]
    fn test_is_crate_banned_not_found() {
        assert!(!is_crate_banned("ring", &["openssl", "native-tls"]));
    }

    #[test]
    fn test_check_vulnerability_found() {
        let advisories = [("hyper", "0.14.18"), ("tokio", "1.0.0")];
        assert_eq!(check_vulnerability("hyper", "0.14.18", &advisories), Some(0));
    }

    #[test]
    fn test_check_vulnerability_clean() {
        let advisories = [("hyper", "0.14.18")];
        assert_eq!(check_vulnerability("hyper", "0.14.25", &advisories), None);
    }

    #[test]
    fn test_check_vulnerability_unknown_crate() {
        let advisories = [("hyper", "0.14.18")];
        assert_eq!(check_vulnerability("rand", "0.8.0", &advisories), None);
    }

    #[test]
    fn test_is_license_allowed_simple() {
        assert!(is_license_allowed("MIT", &["MIT", "Apache-2.0"]));
    }

    #[test]
    fn test_is_license_allowed_spdx_or() {
        assert!(is_license_allowed("MIT OR Apache-2.0", &["MIT", "Apache-2.0", "BSD-3-Clause"]));
    }

    #[test]
    fn test_is_license_allowed_blocked() {
        assert!(!is_license_allowed("GPL-3.0", &["MIT", "Apache-2.0"]));
    }

    #[test]
    fn test_detect_duplicate_versions_conflict() {
        let deps = vec![
            ("serde", "1.0.0"),
            ("tokio", "1.0.0"),
            ("serde", "1.1.0"),
        ];
        let conflicts = detect_duplicate_versions(&deps);
        assert!(conflicts.contains(&"serde".to_string()));
        assert!(!conflicts.contains(&"tokio".to_string()));
    }

    #[test]
    fn test_detect_duplicate_versions_same_version() {
        let deps = vec![
            ("serde", "1.0.0"),
            ("serde", "1.0.0"),
        ];
        let conflicts = detect_duplicate_versions(&deps);
        assert!(conflicts.is_empty(), "Same version is not a conflict");
    }

    #[test]
    fn test_detect_duplicate_versions_no_duplicates() {
        let deps = vec![("serde", "1.0.0"), ("tokio", "1.0.0")];
        assert!(detect_duplicate_versions(&deps).is_empty());
    }

    #[test]
    fn test_is_dependency_mature_enough() {
        assert!(is_dependency_mature(90, 30));
    }

    #[test]
    fn test_is_dependency_mature_too_new() {
        assert!(!is_dependency_mature(5, 30));
    }

    #[test]
    fn test_is_dependency_mature_exact() {
        assert!(is_dependency_mature(30, 30));
    }

    #[test]
    fn test_audit_summary_pass() {
        let summary = audit_summary(50, 0, 0, 0);
        assert!(summary.contains("[PASS]"));
        assert!(summary.contains("50"));
    }

    #[test]
    fn test_audit_summary_fail() {
        let summary = audit_summary(50, 2, 1, 0);
        assert!(summary.contains("[FAIL]"));
        assert!(summary.contains("2 vulnerable"));
    }

    #[test]
    fn test_audit_summary_multiple_issues() {
        let summary = audit_summary(100, 3, 1, 2);
        assert!(summary.contains("[FAIL]"));
        assert!(summary.contains("3 vulnerable"));
        assert!(summary.contains("1 banned"));
        assert!(summary.contains("2 license"));
    }

    #[test]
    fn test_is_trusted_source_crates_io() {
        assert!(is_trusted_source("registry+https://github.com/rust-lang/crates.io-index"));
    }

    #[test]
    fn test_is_trusted_source_github() {
        assert!(is_trusted_source("git+https://github.com/user/repo"));
    }

    #[test]
    fn test_is_trusted_source_unknown() {
        assert!(!is_trusted_source("unknown-registry"));
    }
}
