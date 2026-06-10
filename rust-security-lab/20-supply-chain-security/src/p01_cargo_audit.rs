//! # Lesson 01: cargo-audit — Vulnerability Checking
//!
//! ## What is cargo-audit?
//!
//! `cargo-audit` checks your `Cargo.lock` against the [RustSec Advisory Database](https://rustsec.org/)
//! to find dependencies with known security vulnerabilities. It is the first line of defense
//! in supply chain security.
//!
//! ## How It Works
//!
//! 1. Reads `Cargo.lock` to get exact dependency versions
//! 2. Fetches the RustSec advisory database (OSV-compatible)
//! 3. Matches each dependency version against known advisories
//! 4. Reports vulnerabilities with severity, affected versions, and patches
//!
//! ## Advisory Structure
//!
//! Each advisory contains:
//! - **ID**: e.g., `RUSTSEC-2023-0001`
//! - **Package**: The affected crate name
//! - **Version**: Affected version ranges
//! - **Severity**: CVSS-based scoring (Critical/High/Medium/Low)
//! - **Patched versions**: Versions that fix the issue
//!
//! ## Attack: Ignoring Advisories
//!
//! A common failure mode: teams run `cargo-audit` but ignore warnings or never update
//! dependencies. Over time, known vulnerabilities accumulate, creating a large attack surface.
//!
//! ## Defense: Advisory Parsing and Triage
//!
//! In this lesson, you will implement advisory data structures, severity scoring,
//! and a vulnerability scanner that checks package versions against known advisories.

use serde::{Deserialize, Serialize};

/// Severity levels for security advisories, based on CVSS scoring.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl Severity {
    /// Return a numeric score for comparison (higher = more severe).
    pub fn score(&self) -> u8 {
        match self {
            Severity::Critical => 4,
            Severity::High => 3,
            Severity::Medium => 2,
            Severity::Low => 1,
            Severity::Informational => 0,
        }
    }
}

/// A security advisory entry, similar to RustSec format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    pub package: String,
    pub title: String,
    pub severity: Severity,
    pub affected_versions: Vec<String>,
    pub patched_versions: Vec<String>,
    pub description: String,
}

/// Exercise 1: Parse a severity string into a `Severity` enum.
///
/// Accept case-insensitive input: "critical", "HIGH", "Medium", etc.
/// Return `None` for unrecognized strings.
///
/// Hints:
/// - Use `.to_lowercase()` or `.eq_ignore_ascii_case()`
/// - Match on the lowercase string
pub fn parse_severity(s: &str) -> Option<Severity> {
    todo!("Parse a severity string into Severity enum")
}

/// Exercise 2: Check if a given version string satisfies a version range specifier.
///
/// Version range specifiers use a simple format:
/// - ">=1.0.0" means version >= 1.0.0
/// - "<2.0.0" means version < 2.0.0
/// - ">=1.0.0,<2.0.0" means version >= 1.0.0 AND < 2.0.0 (comma-separated)
///
/// Compare using simple string-based semver comparison (major.minor.patch).
///
/// Hints:
/// - Split on ',' for multiple constraints
/// - Parse each constraint: strip the operator prefix (>=, <=, <, >)
/// - Split version on '.' and compare major, minor, patch as integers
/// - Return `true` only if ALL constraints are satisfied
pub fn version_in_range(version: &str, range: &str) -> bool {
    todo!("Check if version satisfies range specifier")
}

/// Exercise 3: Scan a list of dependencies against a list of advisories.
///
/// For each advisory, check if any dependency matches the package name
/// AND has a version that falls within the affected range AND does NOT
/// match any patched version.
///
/// Return a vector of references to matching advisories.
///
/// Hints:
/// - Iterate over advisories
/// - For each advisory, find matching dependencies by name
/// - Check version_in_range for affected_versions
/// - Check if version matches any patched version (exact match)
/// - If matched and not patched, include in results
pub fn scan_dependencies<'a>(
    dependencies: &[(String, String)],
    advisories: &'a [Advisory],
) -> Vec<&'a Advisory> {
    todo!("Scan dependencies against advisory database")
}

/// Exercise 4: Calculate a risk score for a set of advisories.
///
/// Risk score = sum of (severity.score() * number_of_affected_packages).
/// Each advisory counts once for its own severity.
///
/// For example: 2 Critical (4 each) + 1 Medium (2) = 2*4 + 1*2 = 10
///
/// Hints:
/// - Iterate over advisories, sum `severity.score()`
pub fn calculate_risk_score(advisories: &[Advisory]) -> u32 {
    todo!("Calculate aggregate risk score from advisories")
}

/// Exercise 5: Filter advisories by minimum severity threshold.
///
/// Return only advisories whose severity score >= the threshold severity score.
///
/// Hints:
/// - Use the `score()` method on both the advisory severity and threshold
pub fn filter_by_severity<'a>(
    advisories: &'a [Advisory],
    min_severity: &Severity,
) -> Vec<&'a Advisory> {
    todo!("Filter advisories by minimum severity")
}

/// Exercise 6: Generate a text report from a list of advisories.
///
/// Format: One advisory per line as "ID | PACKAGE | SEVERITY | TITLE"
/// Lines are separated by `\n`. If no advisories, return "No vulnerabilities found."
///
/// Hints:
/// - Use `format!()` with the advisory fields
/// - Join lines with `\n` or collect into a String
pub fn generate_report(advisories: &[Advisory]) -> String {
    todo!("Generate a text vulnerability report")
}

/// Exercise 7: Serialize an advisory to JSON string.
///
/// Hints:
/// - Use `serde_json::to_string(&advisory)`
/// - Handle the Result by unwrapping or returning a default
pub fn serialize_advisory(advisory: &Advisory) -> String {
    todo!("Serialize an Advisory to JSON")
}

/// Exercise 8: Deserialize an advisory from JSON string.
///
/// Hints:
/// - Use `serde_json::from_str::<Advisory>(json)`
/// - Handle the Result
pub fn deserialize_advisory(json: &str) -> Option<Advisory> {
    todo!("Deserialize JSON into an Advisory")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_advisory() -> Advisory {
        Advisory {
            id: "RUSTSEC-2023-0001".to_string(),
            package: "smallvec".to_string(),
            title: "Buffer overflow in SmallVec".to_string(),
            severity: Severity::High,
            affected_versions: vec![">=0.0.0,<1.11.0".to_string()],
            patched_versions: vec!["1.11.0".to_string()],
            description: "Buffer overflow when growing the vector".to_string(),
        }
    }

    fn sample_advisory_critical() -> Advisory {
        Advisory {
            id: "RUSTSEC-2023-0002".to_string(),
            package: "hyper".to_string(),
            title: "HTTP request smuggling".to_string(),
            severity: Severity::Critical,
            affected_versions: vec![">=0.14.0,<0.14.28".to_string()],
            patched_versions: vec!["0.14.28".to_string()],
            description: "HTTP request smuggling vulnerability".to_string(),
        }
    }

    #[test]
    fn test_parse_severity_critical() {
        assert_eq!(parse_severity("critical"), Some(Severity::Critical));
        assert_eq!(parse_severity("CRITICAL"), Some(Severity::Critical));
    }

    #[test]
    fn test_parse_severity_all_levels() {
        assert_eq!(parse_severity("high"), Some(Severity::High));
        assert_eq!(parse_severity("medium"), Some(Severity::Medium));
        assert_eq!(parse_severity("low"), Some(Severity::Low));
        assert_eq!(parse_severity("informational"), Some(Severity::Informational));
    }

    #[test]
    fn test_parse_severity_invalid() {
        assert_eq!(parse_severity("unknown"), None);
        assert_eq!(parse_severity(""), None);
    }

    #[test]
    fn test_version_in_range_gte() {
        assert!(version_in_range("1.5.0", ">=1.0.0"));
        assert!(version_in_range("1.0.0", ">=1.0.0"));
        assert!(!version_in_range("0.9.9", ">=1.0.0"));
    }

    #[test]
    fn test_version_in_range_lt() {
        assert!(version_in_range("1.9.9", "<2.0.0"));
        assert!(!version_in_range("2.0.0", "<2.0.0"));
    }

    #[test]
    fn test_version_in_range_composite() {
        assert!(version_in_range("1.5.0", ">=1.0.0,<2.0.0"));
        assert!(!version_in_range("2.0.0", ">=1.0.0,<2.0.0"));
        assert!(!version_in_range("0.9.0", ">=1.0.0,<2.0.0"));
    }

    #[test]
    fn test_scan_finds_vulnerable() {
        let deps = vec![
            ("smallvec".to_string(), "1.10.0".to_string()),
            ("serde".to_string(), "1.0.100".to_string()),
        ];
        let advisories = vec![sample_advisory()];
        let results = scan_dependencies(&deps, &advisories);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "RUSTSEC-2023-0001");
    }

    #[test]
    fn test_scan_ignores_patched() {
        let deps = vec![
            ("smallvec".to_string(), "1.11.0".to_string()),
        ];
        let advisories = vec![sample_advisory()];
        let results = scan_dependencies(&deps, &advisories);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_scan_ignores_unaffected_package() {
        let deps = vec![
            ("serde".to_string(), "1.0.100".to_string()),
        ];
        let advisories = vec![sample_advisory()];
        let results = scan_dependencies(&deps, &advisories);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_risk_score_single() {
        let advisories = vec![sample_advisory()];
        assert_eq!(calculate_risk_score(&advisories), 3);
    }

    #[test]
    fn test_risk_score_multiple() {
        let advisories = vec![sample_advisory(), sample_advisory_critical()];
        assert_eq!(calculate_risk_score(&advisories), 7);
    }

    #[test]
    fn test_filter_by_severity() {
        let advisories = vec![sample_advisory(), sample_advisory_critical()];
        let critical_only = filter_by_severity(&advisories, &Severity::Critical);
        assert_eq!(critical_only.len(), 1);
        assert_eq!(critical_only[0].id, "RUSTSEC-2023-0002");
    }

    #[test]
    fn test_report_format() {
        let advisories = vec![sample_advisory()];
        let report = generate_report(&advisories);
        assert!(report.contains("RUSTSEC-2023-0001"));
        assert!(report.contains("smallvec"));
        assert!(report.contains("High"));
    }

    #[test]
    fn test_report_empty() {
        let report = generate_report(&[]);
        assert_eq!(report, "No vulnerabilities found.");
    }

    #[test]
    fn test_serialize_roundtrip() {
        let advisory = sample_advisory();
        let json = serialize_advisory(&advisory);
        let deserialized = deserialize_advisory(&json).unwrap();
        assert_eq!(deserialized.id, advisory.id);
        assert_eq!(deserialized.package, advisory.package);
        assert_eq!(deserialized.severity, advisory.severity);
    }
}
