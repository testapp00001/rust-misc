//! # API Versioning
//!
//! Semantic versioning (SemVer) in Rust requires understanding what constitutes
//! a breaking change. This module covers SemVer rules, deprecation patterns,
//! and migration strategies.
//!
//! ## Key Concepts
//! - **SemVer**: MAJOR.MINOR.PATCH — breaking, feature, fix
//! - **Breaking changes**: Removing public items, changing signatures
//! - **Non-breaking additions**: New methods, new trait impls, new enum variants
//! - **Deprecation**: Marking items for future removal

/// Represents a semantic version.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }

    pub fn with_pre_release(mut self, pre: impl Into<String>) -> Self {
        self.pre_release = Some(pre.into());
        self
    }

    /// Determines if a change from `other` to `self` is breaking.
    pub fn is_breaking_change_from(&self, other: &Version) -> bool {
        self.major != other.major && self.major > 0
    }

    /// Determines if a change is a new feature (minor bump).
    pub fn is_feature_change_from(&self, other: &Version) -> bool {
        self.major == other.major && self.minor > other.minor
    }

    /// Bumps the major version (breaking change).
    pub fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    /// Bumps the minor version (new feature).
    pub fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    /// Bumps the patch version (bug fix).
    pub fn bump_patch(&mut self) {
        self.patch += 1;
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre_release {
            write!(f, "-{pre}")?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (version_part, pre_release) = if let Some(idx) = s.find('-') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionParseError::InvalidFormat(s.to_string()));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| VersionParseError::InvalidNumber(parts[0].to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| VersionParseError::InvalidNumber(parts[1].to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| VersionParseError::InvalidNumber(parts[2].to_string()))?;

        Ok(Version {
            major,
            minor,
            patch,
            pre_release,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VersionParseError {
    InvalidFormat(String),
    InvalidNumber(String),
}

impl std::fmt::Display for VersionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionParseError::InvalidFormat(s) => write!(f, "Invalid version format: {s}"),
            VersionParseError::InvalidNumber(s) => write!(f, "Invalid number: {s}"),
        }
    }
}

/// A changelog entry documenting API changes.
#[derive(Debug, Clone)]
pub struct ChangelogEntry {
    pub version: Version,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone)]
pub struct Change {
    pub kind: ChangeKind,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeKind {
    Breaking,
    Feature,
    Fix,
    Deprecation,
}

impl std::fmt::Display for ChangeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeKind::Breaking => write!(f, "BREAKING"),
            ChangeKind::Feature => write!(f, "FEATURE"),
            ChangeKind::Fix => write!(f, "FIX"),
            ChangeKind::Deprecation => write!(f, "DEPRECATED"),
        }
    }
}

/// A version compatibility checker.
pub struct CompatibilityChecker {
    current: Version,
    changelog: Vec<ChangelogEntry>,
}

impl CompatibilityChecker {
    pub fn new(version: Version) -> Self {
        CompatibilityChecker {
            current: version,
            changelog: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: ChangelogEntry) {
        self.changelog.push(entry);
    }

    /// Checks if upgrading from `from` to the current version requires
    /// breaking changes.
    pub fn requires_migration(&self, from: &Version) -> bool {
        self.changelog.iter().any(|entry| {
            entry.version > *from
                && entry
                    .changes
                    .iter()
                    .any(|c| c.kind == ChangeKind::Breaking)
        })
    }

    /// Lists all breaking changes between two versions.
    pub fn breaking_changes(&self, from: &Version, to: &Version) -> Vec<&Change> {
        self.changelog
            .iter()
            .filter(|entry| entry.version > *from && entry.version <= *to)
            .flat_map(|entry| &entry.changes)
            .filter(|c| c.kind == ChangeKind::Breaking)
            .collect()
    }

    /// Lists deprecations between two versions.
    pub fn deprecations(&self, from: &Version, to: &Version) -> Vec<&Change> {
        self.changelog
            .iter()
            .filter(|entry| entry.version > *from && entry.version <= *to)
            .flat_map(|entry| &entry.changes)
            .filter(|c| c.kind == ChangeKind::Deprecation)
            .collect()
    }
}

/// Demonstrates the deprecation pattern in Rust API design.
pub mod api_v1 {
    /// The original function.
    #[deprecated(since = "2.0.0", note = "Use `process_data` instead")]
    pub fn old_process(data: &str) -> String {
        data.to_uppercase()
    }

    /// The replacement function with a better API.
    pub fn process_data(data: &str, options: ProcessOptions) -> String {
        let result = data.to_uppercase();
        if options.trim {
            result.trim().to_string()
        } else {
            result
        }
    }

    #[derive(Debug, Clone)]
    pub struct ProcessOptions {
        pub trim: bool,
    }

    impl Default for ProcessOptions {
        fn default() -> Self {
            ProcessOptions { trim: false }
        }
    }
}

/// A versioned API with explicit version parameter.
pub enum ApiVersion {
    V1,
    V2,
}

pub struct VersionedApi;

impl VersionedApi {
    pub fn process(version: ApiVersion, data: &str) -> String {
        match version {
            ApiVersion::V1 => data.to_uppercase(),
            ApiVersion::V2 => {
                let mut result = data.to_uppercase();
                result.push_str(" [v2]");
                result
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_display() {
        let v = Version::new(1, 2, 3);
        assert_eq!(format!("{v}"), "1.2.3");
    }

    #[test]
    fn test_version_with_pre_release() {
        let v = Version::new(1, 0, 0).with_pre_release("beta.1");
        assert_eq!(format!("{v}"), "1.0.0-beta.1");
    }

    #[test]
    fn test_version_parse() {
        let v: Version = "1.2.3".parse().unwrap();
        assert_eq!(v, Version::new(1, 2, 3));
    }

    #[test]
    fn test_version_parse_with_pre_release() {
        let v: Version = "2.0.0-alpha".parse().unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.pre_release, Some("alpha".to_string()));
    }

    #[test]
    fn test_version_parse_invalid() {
        assert!("1.2".parse::<Version>().is_err());
        assert!("abc".parse::<Version>().is_err());
        assert!("1.2.x".parse::<Version>().is_err());
    }

    #[test]
    fn test_version_ordering() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        let v3 = Version::new(2, 0, 0);

        assert!(v1 < v2);
        assert!(v2 < v3);
    }

    #[test]
    fn test_breaking_change() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);
        assert!(v2.is_breaking_change_from(&v1));
    }

    #[test]
    fn test_feature_change() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        assert!(v2.is_feature_change_from(&v1));
        assert!(!v2.is_breaking_change_from(&v1));
    }

    #[test]
    fn test_version_bumps() {
        let mut v = Version::new(1, 2, 3);

        v.bump_patch();
        assert_eq!(v, Version::new(1, 2, 4));

        v.bump_minor();
        assert_eq!(v, Version::new(1, 3, 0));

        v.bump_major();
        assert_eq!(v, Version::new(2, 0, 0));
    }

    #[test]
    fn test_compatibility_checker() {
        let mut checker = CompatibilityChecker::new(Version::new(3, 0, 0));
        checker.add_entry(ChangelogEntry {
            version: Version::new(2, 0, 0),
            changes: vec![
                Change {
                    kind: ChangeKind::Breaking,
                    description: "Removed old_process".into(),
                },
                Change {
                    kind: ChangeKind::Feature,
                    description: "Added process_data".into(),
                },
            ],
        });
        checker.add_entry(ChangelogEntry {
            version: Version::new(3, 0, 0),
            changes: vec![Change {
                kind: ChangeKind::Breaking,
                description: "Changed return type".into(),
            }],
        });

        let v1 = Version::new(1, 0, 0);
        assert!(checker.requires_migration(&v1));

        let breaking = checker.breaking_changes(&v1, &Version::new(3, 0, 0));
        assert_eq!(breaking.len(), 2);
    }

    #[test]
    fn test_deprecations() {
        let mut checker = CompatibilityChecker::new(Version::new(2, 0, 0));
        checker.add_entry(ChangelogEntry {
            version: Version::new(1, 5, 0),
            changes: vec![Change {
                kind: ChangeKind::Deprecation,
                description: "old_function deprecated".into(),
            }],
        });

        let deps = checker.deprecations(&Version::new(1, 0, 0), &Version::new(2, 0, 0));
        assert_eq!(deps.len(), 1);
        assert!(deps[0].description.contains("old_function"));
    }

    #[test]
    fn test_versioned_api() {
        let v1 = VersionedApi::process(ApiVersion::V1, "hello");
        let v2 = VersionedApi::process(ApiVersion::V2, "hello");

        assert_eq!(v1, "HELLO");
        assert_eq!(v2, "HELLO [v2]");
    }

    #[test]
    fn test_change_kind_display() {
        assert_eq!(format!("{}", ChangeKind::Breaking), "BREAKING");
        assert_eq!(format!("{}", ChangeKind::Fix), "FIX");
    }
}
