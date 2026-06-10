//! # Lesson 09: Minimum Version Resolution Testing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A semantic version with major.minor.patch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

/// A dependency specification with a version range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencySpec {
    pub name: String,
    pub min_version: SemVer,
    pub max_version: Option<SemVer>,
    pub msrv: Option<SemVer>,
}

/// Parse a semver string "major.minor.patch" into a SemVer struct.
pub fn parse_semver(s: &str) -> Option<SemVer> {
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u32>().ok()?;
    let minor = parts[1].parse::<u32>().ok()?;
    let patch = parts[2].parse::<u32>().ok()?;
    Some(SemVer::new(major, minor, patch))
}

/// Format a SemVer as "major.minor.patch" string.
pub fn format_semver(v: &SemVer) -> String {
    format!("{}.{}.{}", v.major, v.minor, v.patch)
}

/// Check if a version satisfies a dependency spec.
pub fn satisfies_spec(version: &SemVer, spec: &DependencySpec) -> bool {
    if version < &spec.min_version {
        return false;
    }
    if let Some(ref max) = spec.max_version {
        if version > max {
            return false;
        }
    }
    true
}

/// Find the minimum version that satisfies a spec.
pub fn minimum_satisfying_version(spec: &DependencySpec) -> Option<SemVer> {
    // Validate: min <= max (if max specified)
    if let Some(ref max) = spec.max_version {
        if spec.min_version > *max {
            return None;
        }
    }
    Some(spec.min_version.clone())
}

/// Check if a crate's MSRV is compatible with the project's toolchain.
pub fn is_msrv_compatible(
    project_rust_version: &str,
    crate_msrv: &str,
) -> bool {
    let project = match parse_semver(project_rust_version) {
        Some(v) => v,
        None => return false,
    };
    let msrv = match parse_semver(crate_msrv) {
        Some(v) => v,
        None => return false,
    };
    project >= msrv
}

/// Find all dependencies that require a newer minimum version than what's locked.
pub fn find_below_minimum(
    specs: &[DependencySpec],
    locked_versions: &[(String, SemVer)],
) -> Vec<String> {
    let locked_map: HashMap<&str, &SemVer> = locked_versions
        .iter()
        .map(|(name, ver)| (name.as_str(), ver))
        .collect();

    specs
        .iter()
        .filter_map(|spec| {
            if let Some(locked) = locked_map.get(spec.name.as_str()) {
                if *locked < &spec.min_version {
                    return Some(spec.name.clone());
                }
            }
            None
        })
        .collect()
}

/// Recommend version upgrades for dependencies below their minimum.
pub fn recommend_upgrades(
    specs: &[DependencySpec],
    locked_versions: &[(String, SemVer)],
) -> Vec<(String, SemVer, SemVer)> {
    let locked_map: HashMap<&str, &SemVer> = locked_versions
        .iter()
        .map(|(name, ver)| (name.as_str(), ver))
        .collect();

    specs
        .iter()
        .filter_map(|spec| {
            if let Some(locked) = locked_map.get(spec.name.as_str()) {
                if *locked < &spec.min_version {
                    return Some((
                        spec.name.clone(),
                        (*locked).clone(),
                        spec.min_version.clone(),
                    ));
                }
            }
            None
        })
        .collect()
}

/// Generate a minimum version compatibility report.
pub fn compatibility_report(
    specs: &[DependencySpec],
    locked_versions: &[(String, SemVer)],
) -> String {
    let locked_map: HashMap<&str, &SemVer> = locked_versions
        .iter()
        .map(|(name, ver)| (name.as_str(), ver))
        .collect();

    let mut lines: Vec<String> = specs
        .iter()
        .map(|spec| {
            let locked_str = locked_map
                .get(spec.name.as_str())
                .map(|v| format_semver(v))
                .unwrap_or_else(|| "not locked".to_string());
            let min_str = format_semver(&spec.min_version);
            let compatible = locked_map
                .get(spec.name.as_str())
                .map(|locked| *locked >= &spec.min_version)
                .unwrap_or(false);
            format!(
                "{}: locked={}, min={}, compatible={}",
                spec.name,
                locked_str,
                min_str,
                if compatible { "yes" } else { "no" }
            )
        })
        .collect();

    let needs_upgrade = find_below_minimum(specs, locked_versions).len();
    lines.push(format!(
        "{} of {} dependencies need upgrades.",
        needs_upgrade,
        specs.len()
    ));

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(name: &str, min: (u32, u32, u32), max: Option<(u32, u32, u32)>) -> DependencySpec {
        DependencySpec {
            name: name.to_string(),
            min_version: SemVer::new(min.0, min.1, min.2),
            max_version: max.map(|(a, b, c)| SemVer::new(a, b, c)),
            msrv: None,
        }
    }

    #[test]
    fn test_parse_semver_valid() {
        let v = parse_semver("1.2.3").unwrap();
        assert_eq!(v, SemVer::new(1, 2, 3));
    }

    #[test]
    fn test_parse_semver_invalid() {
        assert!(parse_semver("1.2").is_none());
        assert!(parse_semver("abc.def.ghi").is_none());
        assert!(parse_semver("").is_none());
    }

    #[test]
    fn test_format_semver() {
        assert_eq!(format_semver(&SemVer::new(1, 2, 3)), "1.2.3");
    }

    #[test]
    fn test_semver_ordering() {
        assert!(SemVer::new(1, 0, 0) < SemVer::new(2, 0, 0));
        assert!(SemVer::new(1, 0, 0) < SemVer::new(1, 1, 0));
        assert!(SemVer::new(1, 0, 0) < SemVer::new(1, 0, 1));
        assert!(SemVer::new(1, 0, 0) == SemVer::new(1, 0, 0));
    }

    #[test]
    fn test_satisfies_spec_in_range() {
        let s = spec("serde", (1, 0, 0), Some((2, 0, 0)));
        assert!(satisfies_spec(&SemVer::new(1, 5, 0), &s));
        assert!(satisfies_spec(&SemVer::new(1, 0, 0), &s));
        assert!(satisfies_spec(&SemVer::new(2, 0, 0), &s));
    }

    #[test]
    fn test_satisfies_spec_out_of_range() {
        let s = spec("serde", (1, 0, 0), Some((2, 0, 0)));
        assert!(!satisfies_spec(&SemVer::new(0, 9, 0), &s));
        assert!(!satisfies_spec(&SemVer::new(2, 0, 1), &s));
    }

    #[test]
    fn test_satisfies_spec_no_max() {
        let s = spec("serde", (1, 0, 0), None);
        assert!(satisfies_spec(&SemVer::new(1, 0, 0), &s));
        assert!(satisfies_spec(&SemVer::new(99, 0, 0), &s));
        assert!(!satisfies_spec(&SemVer::new(0, 9, 0), &s));
    }

    #[test]
    fn test_minimum_satisfying_version() {
        let s = spec("serde", (1, 0, 5), Some((2, 0, 0)));
        let min = minimum_satisfying_version(&s).unwrap();
        assert_eq!(min, SemVer::new(1, 0, 5));
    }

    #[test]
    fn test_minimum_satisfying_version_invalid() {
        let s = DependencySpec {
            name: "bad".to_string(),
            min_version: SemVer::new(2, 0, 0),
            max_version: Some(SemVer::new(1, 0, 0)),
            msrv: None,
        };
        assert!(minimum_satisfying_version(&s).is_none());
    }

    #[test]
    fn test_msrv_compatible() {
        assert!(is_msrv_compatible("1.72.0", "1.65.0"));
        assert!(is_msrv_compatible("1.65.0", "1.65.0"));
        assert!(!is_msrv_compatible("1.60.0", "1.65.0"));
    }

    #[test]
    fn test_find_below_minimum() {
        let specs = vec![
            spec("serde", (1, 0, 150), None),
            spec("tokio", (1, 30, 0), None),
        ];
        let locked = vec![
            ("serde".to_string(), SemVer::new(1, 0, 100)),
            ("tokio".to_string(), SemVer::new(1, 35, 0)),
        ];
        let below = find_below_minimum(&specs, &locked);
        assert_eq!(below, vec!["serde"]);
    }

    #[test]
    fn test_recommend_upgrades() {
        let specs = vec![
            spec("serde", (1, 0, 150), None),
        ];
        let locked = vec![
            ("serde".to_string(), SemVer::new(1, 0, 100)),
        ];
        let recs = recommend_upgrades(&specs, &locked);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].0, "serde");
        assert_eq!(recs[0].1, SemVer::new(1, 0, 100));
        assert_eq!(recs[0].2, SemVer::new(1, 0, 150));
    }

    #[test]
    fn test_compatibility_report() {
        let specs = vec![spec("serde", (1, 0, 150), None)];
        let locked = vec![("serde".to_string(), SemVer::new(1, 0, 100))];
        let report = compatibility_report(&specs, &locked);
        assert!(report.contains("serde"));
        assert!(report.contains("need upgrades"));
    }
}
