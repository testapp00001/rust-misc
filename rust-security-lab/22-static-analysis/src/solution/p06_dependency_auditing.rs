//! # Lesson 06: Dependency Auditing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Check if crate is in the ban list.
pub fn is_crate_banned(crate_name: &str, ban_list: &[&str]) -> bool {
    ban_list.contains(&crate_name)
}

/// Check crate version against vulnerability advisories.
pub fn check_vulnerability(
    crate_name: &str,
    version: &str,
    advisories: &[(&str, &str)],
) -> Option<usize> {
    advisories
        .iter()
        .position(|(name, vuln_ver)| *name == crate_name && *vuln_ver == version)
}

/// Check if license is in the allowed list, handling SPDX OR expressions.
pub fn is_license_allowed(license: &str, allowed_licenses: &[&str]) -> bool {
    if license.contains(" OR ") {
        // SPDX expression: at least one part must be allowed
        license
            .split(" OR ")
            .any(|part| allowed_licenses.contains(&part.trim()))
    } else {
        allowed_licenses.contains(&license)
    }
}

/// Detect duplicate dependency versions.
pub fn detect_duplicate_versions(dependencies: &[(&str, &str)]) -> Vec<String> {
    use std::collections::{HashMap, HashSet};

    let mut crate_versions: HashMap<&str, HashSet<&str>> = HashMap::new();
    for (name, version) in dependencies {
        crate_versions.entry(name).or_default().insert(version);
    }

    crate_versions
        .iter()
        .filter(|(_, versions)| versions.len() > 1)
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Check if dependency meets minimum age requirement.
pub fn is_dependency_mature(days_since_publish: u64, min_age_days: u64) -> bool {
    days_since_publish >= min_age_days
}

/// Generate audit summary string.
pub fn audit_summary(total: usize, vulnerable: usize, banned: usize, license_violations: usize) -> String {
    let status = if vulnerable == 0 && banned == 0 && license_violations == 0 {
        "[PASS]"
    } else {
        "[FAIL]"
    };
    format!(
        "Audited {} deps: {} vulnerable, {} banned, {} license issues {}",
        total, vulnerable, banned, license_violations, status
    )
}

/// Check if dependency source is trusted.
pub fn is_trusted_source(source: &str) -> bool {
    let lower = source.to_lowercase();
    lower.contains("crates.io") || lower.contains("github.com")
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
