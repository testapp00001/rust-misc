//! # Lesson 10: Release Engineering
//!
//! Release engineering covers versioning strategies, changelogs, publishing
//! to crates.io, tagging releases, and automating the release process.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A release version with metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Release {
    pub version: String,
    pub date: String,
    pub changes: Vec<Change>,
    pub is_prerelease: bool,
    pub git_tag: String,
}

/// A single change in a release, following the Keep a Changelog format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Change {
    pub category: ChangeCategory,
    pub description: String,
    pub breaking: bool,
    pub issue_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeCategory {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

impl ChangeCategory {
    pub fn heading(&self) -> &str {
        match self {
            ChangeCategory::Added => "Added",
            ChangeCategory::Changed => "Changed",
            ChangeCategory::Deprecated => "Deprecated",
            ChangeCategory::Removed => "Removed",
            ChangeCategory::Fixed => "Fixed",
            ChangeCategory::Security => "Security",
        }
    }
}

/// A changelog that tracks all releases.
#[derive(Debug, Serialize, Deserialize)]
pub struct Changelog {
    pub header: String,
    pub releases: Vec<Release>,
}

impl Changelog {
    pub fn new(header: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            releases: Vec::new(),
        }
    }

    pub fn add_release(&mut self, release: Release) {
        self.releases.push(release);
    }

    /// Generate the changelog in Keep a Changelog markdown format.
    pub fn to_markdown(&self) -> String {
        let mut md = format!("{}\n\n", self.header);

        for release in &self.releases {
            let version_header = if release.is_prerelease {
                format!("## [{}] - {} (Pre-release)", release.version, release.date)
            } else {
                format!("## [{}] - {}", release.version, release.date)
            };
            md.push_str(&version_header);
            md.push('\n');

            // Group changes by category
            let mut categories: BTreeMap<String, Vec<&Change>> = BTreeMap::new();
            for change in &release.changes {
                let heading = change.category.heading().to_string();
                categories.entry(heading).or_default().push(change);
            }

            for (category, changes) in &categories {
                md.push_str(&format!("\n### {}\n", category));
                for change in changes {
                    let breaking = if change.breaking { " **BREAKING**" } else { "" };
                    let issue = change
                        .issue_ref
                        .as_ref()
                        .map(|r| format!(" ([{}])", r))
                        .unwrap_or_default();
                    md.push_str(&format!(
                        "- {}{}{}\n",
                        change.description, breaking, issue
                    ));
                }
            }
            md.push('\n');
        }

        md
    }

    /// Get the latest release version.
    pub fn latest_version(&self) -> Option<&str> {
        self.releases.first().map(|r| r.version.as_str())
    }

    /// Check if there are any breaking changes since a given version.
    pub fn has_breaking_since(&self, version: &str) -> bool {
        let mut found = false;
        for release in &self.releases {
            if release.version == version {
                break;
            }
            if release.changes.iter().any(|c| c.breaking) {
                found = true;
            }
        }
        found
    }
}

/// Versioning strategy for a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VersioningStrategy {
    /// Semantic versioning (semver.org).
    SemVer,
    /// Calendar versioning (e.g., 2024.01).
    CalVer {
        year: u32,
        month: u32,
        micro: u32,
    },
    /// Zero-based versioning for pre-1.0 projects.
    ZeroVer,
}

impl VersioningStrategy {
    /// Compute the next version given the current version and change type.
    pub fn next_version(&self, current: &str, bump: VersionBump) -> Result<String, String> {
        match self {
            VersioningStrategy::SemVer | VersioningStrategy::ZeroVer => {
                let parts: Vec<&str> = current.split('.').collect();
                if parts.len() != 3 {
                    return Err(format!("Invalid version: {}", current));
                }
                let major: u32 = parts[0].parse().map_err(|_| "invalid major")?;
                let minor: u32 = parts[1].parse().map_err(|_| "invalid minor")?;
                let patch: u32 = parts[2].parse().map_err(|_| "invalid patch")?;

                match bump {
                    VersionBump::Major => Ok(format!("{}.0.0", major + 1)),
                    VersionBump::Minor => Ok(format!("{}.{}.0", major, minor + 1)),
                    VersionBump::Patch => Ok(format!("{}.{}.{}", major, minor, patch + 1)),
                }
            }
            VersioningStrategy::CalVer { year, month, micro } => match bump {
                VersionBump::Major => Ok(format!("{}.0.0", year + 1)),
                VersionBump::Minor => Ok(format!("{}.{}.0", year, month + 1)),
                VersionBump::Patch => Ok(format!("{}.{}.{}", year, month, micro + 1)),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VersionBump {
    Major,
    Minor,
    Patch,
}

/// Represents a release checklist item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub description: String,
    pub completed: bool,
    pub automated: bool,
}

/// A release checklist for a project.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseChecklist {
    pub items: Vec<ChecklistItem>,
}

impl ReleaseChecklist {
    pub fn standard() -> Self {
        Self {
            items: vec![
                ChecklistItem {
                    description: "Run full test suite".to_string(),
                    completed: false,
                    automated: true,
                },
                ChecklistItem {
                    description: "Run clippy with no warnings".to_string(),
                    completed: false,
                    automated: true,
                },
                ChecklistItem {
                    description: "Update CHANGELOG.md".to_string(),
                    completed: false,
                    automated: false,
                },
                ChecklistItem {
                    description: "Bump version in Cargo.toml".to_string(),
                    completed: false,
                    automated: true,
                },
                ChecklistItem {
                    description: "Run cargo publish --dry-run".to_string(),
                    completed: false,
                    automated: true,
                },
                ChecklistItem {
                    description: "Create git tag".to_string(),
                    completed: false,
                    automated: true,
                },
                ChecklistItem {
                    description: "Publish to crates.io".to_string(),
                    completed: false,
                    automated: true,
                },
                ChecklistItem {
                    description: "Push git tag".to_string(),
                    completed: false,
                    automated: true,
                },
                ChecklistItem {
                    description: "Create GitHub release".to_string(),
                    completed: false,
                    automated: true,
                },
            ],
        }
    }

    pub fn all_completed(&self) -> bool {
        self.items.iter().all(|i| i.completed)
    }

    pub fn pending_items(&self) -> Vec<&ChecklistItem> {
        self.items.iter().filter(|i| !i.completed).collect()
    }

    pub fn automated_items(&self) -> Vec<&ChecklistItem> {
        self.items.iter().filter(|i| i.automated).collect()
    }

    pub fn manual_items(&self) -> Vec<&ChecklistItem> {
        self.items.iter().filter(|i| !i.automated).collect()
    }

    pub fn complete_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items[index].completed = true;
        }
    }
}

/// Generate a git tag string for a version.
pub fn git_tag(version: &str, prefix: &str) -> String {
    if prefix.is_empty() {
        version.to_string()
    } else {
        format!("{}{}", prefix, version)
    }
}

/// Validate that a version string is valid semver.
pub fn validate_version(version: &str) -> Result<(), String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return Err(format!(
            "Version must have exactly 3 parts (major.minor.patch), got '{}'",
            version
        ));
    }
    for (i, part) in parts.iter().enumerate() {
        if part.parse::<u32>().is_err() {
            return Err(format!(
                "Part {} of version '{}' is not a valid number",
                i + 1,
                version
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_release() -> Release {
        Release {
            version: "1.0.0".to_string(),
            date: "2024-01-15".to_string(),
            changes: vec![
                Change {
                    category: ChangeCategory::Added,
                    description: "New API endpoint".to_string(),
                    breaking: false,
                    issue_ref: Some("#42".to_string()),
                },
                Change {
                    category: ChangeCategory::Changed,
                    description: "Renamed config field".to_string(),
                    breaking: true,
                    issue_ref: None,
                },
                Change {
                    category: ChangeCategory::Fixed,
                    description: "Memory leak in cache".to_string(),
                    breaking: false,
                    issue_ref: Some("#38".to_string()),
                },
            ],
            is_prerelease: false,
            git_tag: "v1.0.0".to_string(),
        }
    }

    #[test]
    fn test_changelog_markdown() {
        let mut changelog = Changelog::new("# Changelog");
        changelog.add_release(sample_release());
        let md = changelog.to_markdown();

        assert!(md.contains("# Changelog"));
        assert!(md.contains("## [1.0.0] - 2024-01-15"));
        assert!(md.contains("### Added"));
        assert!(md.contains("New API endpoint"));
        assert!(md.contains("### Changed"));
        assert!(md.contains("**BREAKING**"));
        assert!(md.contains("### Fixed"));
        assert!(md.contains("Memory leak in cache"));
        assert!(md.contains("([#42])"));
    }

    #[test]
    fn test_prerelease_markdown() {
        let mut changelog = Changelog::new("# Changelog");
        let mut release = sample_release();
        release.version = "2.0.0-beta.1".to_string();
        release.is_prerelease = true;
        changelog.add_release(release);

        let md = changelog.to_markdown();
        assert!(md.contains("Pre-release"));
    }

    #[test]
    fn test_latest_version() {
        let mut changelog = Changelog::new("# Changelog");
        changelog.add_release(sample_release());
        assert_eq!(changelog.latest_version(), Some("1.0.0"));

        let empty = Changelog::new("# Changelog");
        assert!(empty.latest_version().is_none());
    }

    #[test]
    fn test_breaking_changes_since() {
        let mut changelog = Changelog::new("# Changelog");
        let mut old = sample_release();
        old.version = "0.9.0".to_string();
        old.changes[1].breaking = false;
        changelog.add_release(sample_release());
        changelog.add_release(old);

        assert!(changelog.has_breaking_since("0.9.0"));
    }

    #[test]
    fn test_no_breaking_changes() {
        let mut changelog = Changelog::new("# Changelog");
        let mut release = sample_release();
        release.changes[1].breaking = false;
        changelog.add_release(release);

        assert!(!changelog.has_breaking_since("0.9.0"));
    }

    #[test]
    fn test_semver_next_version() {
        let strat = VersioningStrategy::SemVer;
        assert_eq!(strat.next_version("1.2.3", VersionBump::Major).unwrap(), "2.0.0");
        assert_eq!(strat.next_version("1.2.3", VersionBump::Minor).unwrap(), "1.3.0");
        assert_eq!(strat.next_version("1.2.3", VersionBump::Patch).unwrap(), "1.2.4");
    }

    #[test]
    fn test_calver_next_version() {
        let strat = VersioningStrategy::CalVer {
            year: 2024,
            month: 1,
            micro: 0,
        };
        assert_eq!(strat.next_version("2024.1.0", VersionBump::Major).unwrap(), "2025.0.0");
        assert_eq!(strat.next_version("2024.1.0", VersionBump::Minor).unwrap(), "2024.2.0");
        assert_eq!(strat.next_version("2024.1.0", VersionBump::Patch).unwrap(), "2024.1.1");
    }

    #[test]
    fn test_release_checklist() {
        let mut checklist = ReleaseChecklist::standard();
        assert!(!checklist.all_completed());
        assert_eq!(checklist.pending_items().len(), checklist.items.len());
        assert!(!checklist.automated_items().is_empty());
        assert!(!checklist.manual_items().is_empty());

        for i in 0..checklist.items.len() {
            checklist.complete_item(i);
        }
        assert!(checklist.all_completed());
        assert!(checklist.pending_items().is_empty());
    }

    #[test]
    fn test_git_tag() {
        assert_eq!(git_tag("1.0.0", "v"), "v1.0.0");
        assert_eq!(git_tag("1.0.0", ""), "1.0.0");
        assert_eq!(git_tag("2.0.0", "release-"), "release-2.0.0");
    }

    #[test]
    fn test_validate_version() {
        assert!(validate_version("1.0.0").is_ok());
        assert!(validate_version("0.1.0").is_ok());
        assert!(validate_version("10.20.30").is_ok());
        assert!(validate_version("1.0").is_err());
        assert!(validate_version("1.0.0.0").is_err());
        assert!(validate_version("a.b.c").is_err());
        assert!(validate_version("").is_err());
    }

    #[test]
    fn test_change_category_headings() {
        assert_eq!(ChangeCategory::Added.heading(), "Added");
        assert_eq!(ChangeCategory::Changed.heading(), "Changed");
        assert_eq!(ChangeCategory::Deprecated.heading(), "Deprecated");
        assert_eq!(ChangeCategory::Removed.heading(), "Removed");
        assert_eq!(ChangeCategory::Fixed.heading(), "Fixed");
        assert_eq!(ChangeCategory::Security.heading(), "Security");
    }

    #[test]
    fn test_multiple_releases() {
        let mut changelog = Changelog::new("# Changelog");
        changelog.add_release(sample_release());
        let mut v0 = sample_release();
        v0.version = "0.9.0".to_string();
        v0.date = "2023-12-01".to_string();
        changelog.add_release(v0);

        let md = changelog.to_markdown();
        assert!(md.contains("1.0.0"));
        assert!(md.contains("0.9.0"));
    }
}
