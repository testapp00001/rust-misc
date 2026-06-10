//! # Lesson 01: cargo-audit — Vulnerability Checking (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

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

/// Parse a severity string into a `Severity` enum.
///
/// Accepts case-insensitive input.
pub fn parse_severity(s: &str) -> Option<Severity> {
    match s.to_lowercase().as_str() {
        "critical" => Some(Severity::Critical),
        "high" => Some(Severity::High),
        "medium" => Some(Severity::Medium),
        "low" => Some(Severity::Low),
        "informational" => Some(Severity::Informational),
        _ => None,
    }
}

/// Parse a single version string "major.minor.patch" into (u32, u32, u32).
fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some((major, minor, patch))
}

/// Compare two version tuples. Returns Ordering.
fn cmp_versions(a: &(u32, u32, u32), b: &(u32, u32, u32)) -> std::cmp::Ordering {
    a.cmp(b)
}

/// Check if a version satisfies a single constraint (e.g., ">=1.0.0" or "<2.0.0").
fn check_single_constraint(version: &(u32, u32, u32), constraint: &str) -> bool {
    let constraint = constraint.trim();
    if constraint.starts_with(">=") {
        if let Some(min) = parse_version(&constraint[2..]) {
            return cmp_versions(version, &min) != std::cmp::Ordering::Less;
        }
    } else if constraint.starts_with("<=") {
        if let Some(max) = parse_version(&constraint[2..]) {
            return cmp_versions(version, &max) != std::cmp::Ordering::Greater;
        }
    } else if constraint.starts_with('<') {
        if let Some(max) = parse_version(&constraint[1..]) {
            return cmp_versions(version, &max) == std::cmp::Ordering::Less;
        }
    } else if constraint.starts_with('>') {
        if let Some(min) = parse_version(&constraint[1..]) {
            return cmp_versions(version, &min) == std::cmp::Ordering::Greater;
        }
    }
    false
}

/// Check if a version string satisfies a version range specifier.
///
/// Supports comma-separated constraints like ">=1.0.0,<2.0.0".
pub fn version_in_range(version: &str, range: &str) -> bool {
    let ver = match parse_version(version) {
        Some(v) => v,
        None => return false,
    };
    range
        .split(',')
        .all(|constraint| check_single_constraint(&ver, constraint))
}

/// Scan dependencies against a list of advisories.
pub fn scan_dependencies<'a>(
    dependencies: &[(String, String)],
    advisories: &'a [Advisory],
) -> Vec<&'a Advisory> {
    advisories
        .iter()
        .filter(|advisory| {
            dependencies.iter().any(|(name, version)| {
                // Must match package name
                if name != &advisory.package {
                    return false;
                }
                // Must be in affected range
                let in_affected = advisory
                    .affected_versions
                    .iter()
                    .any(|range| version_in_range(version, range));
                if !in_affected {
                    return false;
                }
                // Must NOT be in patched versions
                let is_patched = advisory.patched_versions.iter().any(|pv| pv == version);
                !is_patched
            })
        })
        .collect()
}

/// Calculate a risk score from advisories.
pub fn calculate_risk_score(advisories: &[Advisory]) -> u32 {
    advisories.iter().map(|a| a.severity.score() as u32).sum()
}

/// Filter advisories by minimum severity threshold.
pub fn filter_by_severity<'a>(
    advisories: &'a [Advisory],
    min_severity: &Severity,
) -> Vec<&'a Advisory> {
    let threshold = min_severity.score();
    advisories
        .iter()
        .filter(|a| a.severity.score() >= threshold)
        .collect()
}

/// Generate a text vulnerability report.
pub fn generate_report(advisories: &[Advisory]) -> String {
    if advisories.is_empty() {
        return "No vulnerabilities found.".to_string();
    }
    advisories
        .iter()
        .map(|a| {
            format!(
                "{} | {} | {:?} | {}",
                a.id, a.package, a.severity, a.title
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Serialize an advisory to JSON string.
pub fn serialize_advisory(advisory: &Advisory) -> String {
    serde_json::to_string(advisory).unwrap_or_else(|_| "{}".to_string())
}

/// Deserialize an advisory from JSON string.
pub fn deserialize_advisory(json: &str) -> Option<Advisory> {
    serde_json::from_str(json).ok()
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
