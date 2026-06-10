//! # Lesson 09: Minimum Version Resolution Testing
//!
//! ## What is Minimum Version Resolution?
//!
//! When you specify `serde = "1.0"` in Cargo.toml, Cargo by default resolves to the
//! **newest** compatible version. But what if your code accidentally depends on a
//! feature only available in `1.0.150` while you claim compatibility with `1.0.0`?
//!
//! Minimum version resolution (`-Z minimal-versions`) forces Cargo to resolve to the
//! **oldest** compatible version. This exposes:
//!
//! - Claims of compatibility with version ranges you don't actually support
//! - Missing feature gates that depend on newer versions
//! - Transitive dependency issues when old versions are used
//!
//! ## MSRV (Minimum Supported Rust Version)
//!
//! MSRV specifies the oldest Rust compiler version your crate supports.
//! Testing with MSRV ensures your code works on older toolchains.
//!
//! ## Attack: Version Floor Attacks
//!
//! An attacker can:
//! 1. Publish a crate with a high minimum version requirement
//! 2. Force users to pull newer (potentially compromised) versions
//! 3. Or exploit code that assumes features from newer versions
//!
//! ## Defense: Minimum Version Testing
//!
//! In this lesson, you will implement version comparison, MSRV validation,
//! and minimum version compatibility checking.

use serde::{Deserialize, Serialize};

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

/// Exercise 1: Parse a semver string "major.minor.patch" into a SemVer struct.
///
/// Return None if the format is invalid.
///
/// Hints:
/// - Split on '.'
/// - Parse each part as u32
/// - Return None if not exactly 3 parts or parsing fails
pub fn parse_semver(s: &str) -> Option<SemVer> {
    todo!("Parse semver string")
}

/// Exercise 2: Format a SemVer as "major.minor.patch" string.
///
/// Hints:
/// - Use `format!("{}.{}.{}", ...)`
pub fn format_semver(v: &SemVer) -> String {
    todo!("Format SemVer to string")
}

/// Exercise 3: Check if a version satisfies a dependency spec.
///
/// A version satisfies the spec if:
/// - version >= min_version
/// - version <= max_version (if specified)
///
/// Use the Ord implementation for SemVer comparison.
///
/// Hints:
/// - Compare with >= for min_version
/// - Compare with <= for max_version if Some
pub fn satisfies_spec(version: &SemVer, spec: &DependencySpec) -> bool {
    todo!("Check if version satisfies dependency spec")
}

/// Exercise 4: Find the minimum version that satisfies a spec.
///
/// The minimum version is simply the min_version from the spec.
/// But verify it's valid (min <= max if max is specified).
///
/// Return the min_version if valid, None if the spec is invalid
/// (min > max).
///
/// Hints:
/// - Check if max is Some and min > max -> invalid
/// - Otherwise return Some(min_version)
pub fn minimum_satisfying_version(spec: &DependencySpec) -> Option<SemVer> {
    todo!("Find minimum satisfying version")
}

/// Exercise 5: Check if a crate's MSRV is compatible with the project's toolchain.
///
/// A crate's MSRV is compatible if the project's Rust version >= the crate's MSRV.
///
/// Return true if compatible.
///
/// Hints:
/// - Parse both versions with `parse_semver`
/// - Compare using SemVer ordering
pub fn is_msrv_compatible(
    project_rust_version: &str,
    crate_msrv: &str,
) -> bool {
    todo!("Check MSRV compatibility")
}

/// Exercise 6: Find all dependencies that require a newer minimum version
/// than what's currently locked.
///
/// Given a list of dependency specs and a list of (name, locked_version) pairs,
/// return names where the locked version is below the minimum.
///
/// Hints:
/// - Build a HashMap from locked versions
/// - For each spec, check if locked version < min_version
pub fn find_below_minimum(
    specs: &[DependencySpec],
    locked_versions: &[(String, SemVer)],
) -> Vec<String> {
    todo!("Find dependencies below minimum version")
}

/// Exercise 7: Recommend version bumps for dependencies below their minimum.
///
/// For each dependency found by `find_below_minimum`, recommend upgrading
/// to the min_version from the spec.
///
/// Return Vec of (name, current_version, recommended_version).
///
/// Hints:
/// - Use `find_below_minimum` results
/// - Look up current locked version and spec min_version
pub fn recommend_upgrades(
    specs: &[DependencySpec],
    locked_versions: &[(String, SemVer)],
) -> Vec<(String, SemVer, SemVer)> {
    todo!("Recommend version upgrades")
}

/// Exercise 8: Generate a minimum version compatibility report.
///
/// For each dependency, output:
/// `{name}: locked={locked}, min={min}, compatible={yes/no}`
///
/// End with a summary: `{n} of {total} dependencies need upgrades.`
pub fn compatibility_report(
    specs: &[DependencySpec],
    locked_versions: &[(String, SemVer)],
) -> String {
    todo!("Generate minimum version compatibility report")
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
