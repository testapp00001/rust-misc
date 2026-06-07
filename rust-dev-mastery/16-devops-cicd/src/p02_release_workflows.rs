//! # Release Workflows for Rust Projects
//!
//! Automating releases ensures consistency and reduces human error. This module
//! covers semantic versioning, changelog generation, cargo-release integration,
//! and GitHub release automation.
//!
//! ## Release Workflow (YAML reference):
//!
//! ```yaml
//! name: Release
//! on:
//!   push:
//!     tags: ['v*']
//!
//! jobs:
//!   release:
//!     runs-on: ubuntu-latest
//!     steps:
//!       - uses: actions/checkout@v4
//!         with:
//!           fetch-depth: 0  # Full history for changelog
//!       - uses: dtolnay/rust-toolchain@stable
//!       - run: cargo test --all-features
//!       - run: cargo build --release
//!       - uses: softprops/action-gh-release@v1
//!         with:
//!           files: target/release/*
//!           generate_release_notes: true
//! ```

use serde::{Deserialize, Serialize};

/// Semantic version following semver.org specification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre_release: Option<String>,
    pub build_metadata: Option<String>,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
            build_metadata: None,
        }
    }

    pub fn with_pre_release(mut self, pre: impl Into<String>) -> Self {
        self.pre_release = Some(pre.into());
        self
    }

    /// Parse a semver string like "1.2.3" or "1.2.3-beta.1+build.42".
    pub fn parse(version: &str) -> Result<Self, String> {
        let (version_part, build) = if let Some(idx) = version.find('+') {
            (&version[..idx], Some(version[idx + 1..].to_string()))
        } else {
            (version, None)
        };

        let (version_part, pre) = if let Some(idx) = version_part.find('-') {
            (
                &version_part[..idx],
                Some(version_part[idx + 1..].to_string()),
            )
        } else {
            (version_part, None)
        };

        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid semver: expected 3 parts, got {}", parts.len()));
        }

        let major: u32 = parts[0]
            .parse()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
        let minor: u32 = parts[1]
            .parse()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
        let patch: u32 = parts[2]
            .parse()
            .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;

        Ok(Self {
            major,
            minor,
            patch,
            pre_release: pre,
            build_metadata: build,
        })
    }

    /// Bump the major version (breaking changes).
    pub fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
        self.pre_release = None;
        self.build_metadata = None;
    }

    /// Bump the minor version (new features, backwards compatible).
    pub fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
        self.pre_release = None;
        self.build_metadata = None;
    }

    /// Bump the patch version (bug fixes).
    pub fn bump_patch(&mut self) {
        self.patch += 1;
        self.pre_release = None;
        self.build_metadata = None;
    }

    pub fn to_tag(&self) -> String {
        format!("v{}", self)
    }
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre_release {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build_metadata {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

/// Represents a conventional commit for changelog generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub commit_type: CommitType,
    pub scope: Option<String>,
    pub description: String,
    pub body: Option<String>,
    pub breaking: bool,
    pub footer: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitType {
    Feat,
    Fix,
    Docs,
    Style,
    Refactor,
    Perf,
    Test,
    Build,
    Ci,
    Chore,
    Revert,
}

impl CommitType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "feat" => Some(Self::Feat),
            "fix" => Some(Self::Fix),
            "docs" => Some(Self::Docs),
            "style" => Some(Self::Style),
            "refactor" => Some(Self::Refactor),
            "perf" => Some(Self::Perf),
            "test" => Some(Self::Test),
            "build" => Some(Self::Build),
            "ci" => Some(Self::Ci),
            "chore" => Some(Self::Chore),
            "revert" => Some(Self::Revert),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Feat => "feat",
            Self::Fix => "fix",
            Self::Docs => "docs",
            Self::Style => "style",
            Self::Refactor => "refactor",
            Self::Perf => "perf",
            Self::Test => "test",
            Self::Build => "build",
            Self::Ci => "ci",
            Self::Chore => "chore",
            Self::Revert => "revert",
        }
    }
}

/// Parse a conventional commit message.
pub fn parse_conventional_commit(message: &str) -> Option<Commit> {
    let lines: Vec<&str> = message.lines().collect();
    if lines.is_empty() {
        return None;
    }

    let first_line = lines[0];

    // Parse: type(scope)!: description or type!: description
    // Find the colon to split type/scope from description
    let colon_idx = first_line.find(':')?;
    let type_part = &first_line[..colon_idx];
    let rest = first_line[colon_idx + 1..].trim();

    // Check for breaking change marker (!)
    let breaking = type_part.ends_with('!');
    let type_part = if breaking {
        &type_part[..type_part.len() - 1]
    } else {
        type_part
    };

    let line = first_line; // Keep for backward compat

    let (commit_type, scope) = if let Some(idx) = type_part.find('(') {
        let ct = CommitType::from_str(&type_part[..idx])?;
        let scope_end = type_part.find(')')?;
        (ct, Some(type_part[idx + 1..scope_end].to_string()))
    } else {
        (CommitType::from_str(type_part.trim())?, None)
    };

    let body = if lines.len() > 1 {
        let body_lines: Vec<&str> = lines[1..]
            .iter()
            .take_while(|l| !l.starts_with("BREAKING CHANGE") && !l.is_empty())
            .copied()
            .collect();
        if body_lines.is_empty() {
            None
        } else {
            Some(body_lines.join("\n"))
        }
    } else {
        None
    };

    let footer = lines.last().and_then(|l| {
        if l.starts_with("BREAKING CHANGE") || l.starts_with("Closes") || l.starts_with("Refs") {
            Some(l.to_string())
        } else {
            None
        }
    });

    Some(Commit {
        hash: String::new(), // Would be filled from git
        commit_type,
        scope,
        description: rest.to_string(),
        body,
        breaking,
        footer,
    })
}

/// Changelog generator following Keep a Changelog format.
#[derive(Debug)]
pub struct ChangelogGenerator {
    pub version: SemVer,
    pub date: String,
    pub entries: Vec<ChangelogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub category: ChangelogCategory,
    pub description: String,
    pub scope: Option<String>,
    pub commit_hash: Option<String>,
    pub breaking_change: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangelogCategory {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

impl ChangelogCategory {
    /// Map conventional commit types to changelog categories.
    pub fn from_commit_type(ct: &CommitType, breaking: bool) -> Self {
        if breaking {
            return Self::Changed;
        }
        match ct {
            CommitType::Feat => Self::Added,
            CommitType::Fix => Self::Fixed,
            CommitType::Docs => Self::Added,
            CommitType::Perf => Self::Changed,
            CommitType::Refactor => Self::Changed,
            CommitType::Revert => Self::Changed,
            _ => Self::Changed,
        }
    }
}

impl ChangelogGenerator {
    pub fn new(version: SemVer, date: String) -> Self {
        Self {
            version,
            date,
            entries: Vec::new(),
        }
    }

    /// Add an entry from a conventional commit.
    pub fn add_from_commit(&mut self, commit: &Commit) {
        let category =
            ChangelogCategory::from_commit_type(&commit.commit_type, commit.breaking);
        self.entries.push(ChangelogEntry {
            category,
            description: commit.description.clone(),
            scope: commit.scope.clone(),
            commit_hash: Some(commit.hash.clone()),
            breaking_change: commit.breaking,
        });
    }

    /// Generate the changelog markdown for this version.
    pub fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!(
            "## [{}] - {}\n\n",
            self.version, self.date
        ));

        let categories = [
            ChangelogCategory::Security,
            ChangelogCategory::Added,
            ChangelogCategory::Changed,
            ChangelogCategory::Deprecated,
            ChangelogCategory::Removed,
            ChangelogCategory::Fixed,
        ];

        for category in &categories {
            let entries: Vec<&ChangelogEntry> = self
                .entries
                .iter()
                .filter(|e| &e.category == category)
                .collect();

            if entries.is_empty() {
                continue;
            }

            output.push_str(&format!("### {:?}\n\n", category));
            for entry in &entries {
                let scope_prefix = entry
                    .scope
                    .as_ref()
                    .map(|s| format!("**{}**: ", s))
                    .unwrap_or_default();
                let hash_suffix = entry
                    .commit_hash
                    .as_ref()
                    .map(|h| format!(" ({})", &h[..8.min(h.len())]))
                    .unwrap_or_default();
                output.push_str(&format!(
                    "- {}{}{}\n",
                    scope_prefix, entry.description, hash_suffix
                ));
            }
            output.push('\n');
        }

        output
    }
}

/// Determine the next version based on commits since the last release.
pub fn determine_next_version(current: &SemVer, commits: &[Commit]) -> SemVer {
    let mut next = current.clone();
    let has_breaking = commits.iter().any(|c| c.breaking);
    let has_feat = commits.iter().any(|c| c.commit_type == CommitType::Feat);

    if has_breaking {
        next.bump_major();
    } else if has_feat {
        next.bump_minor();
    } else {
        next.bump_patch();
    }
    next
}

/// Release configuration that would typically come from release.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfig {
    /// The crate name
    pub crate_name: String,
    /// Pre-release commit message template
    pub pre_release_commit_message: String,
    /// Tag message template
    pub tag_message: String,
    /// Tag prefix (e.g., "v")
    pub tag_prefix: String,
    /// Whether to push tags
    pub push_tags: bool,
    /// Whether to publish to crates.io
    pub publish: bool,
    /// Sign tags with GPG
    pub sign_tags: bool,
}

impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            crate_name: String::new(),
            pre_release_commit_message: "(cargo-release) version {{version}}".to_string(),
            tag_message: "(cargo-release) {{crate_name}} version {{version}}".to_string(),
            tag_prefix: "v".to_string(),
            push_tags: true,
            publish: true,
            sign_tags: false,
        }
    }
}

impl ReleaseConfig {
    /// Render the tag name for a given version.
    pub fn render_tag(&self, version: &SemVer) -> String {
        format!("{}{}", self.tag_prefix, version)
    }

    /// Render the commit message for a release.
    pub fn render_commit_message(&self, version: &SemVer) -> String {
        self.pre_release_commit_message
            .replace("{{version}}", &version.to_string())
            .replace("{{crate_name}}", &self.crate_name)
    }

    /// Render the tag message for a release.
    pub fn render_tag_message(&self, version: &SemVer) -> String {
        self.tag_message
            .replace("{{version}}", &version.to_string())
            .replace("{{crate_name}}", &self.crate_name)
    }
}

/// Represents a release plan with all the steps needed.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReleasePlan {
    pub current_version: SemVer,
    pub next_version: SemVer,
    pub changelog: String,
    pub tag: String,
    pub commit_message: String,
    pub publish: bool,
    pub steps: Vec<ReleaseStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseStep {
    pub name: String,
    pub command: String,
    pub description: String,
}

impl ReleasePlan {
    /// Create a release plan from commits and configuration.
    pub fn create(
        current_version: SemVer,
        commits: &[Commit],
        config: &ReleaseConfig,
        date: &str,
    ) -> Self {
        let next_version = determine_next_version(&current_version, commits);

        let mut changelog = ChangelogGenerator::new(next_version.clone(), date.to_string());
        for commit in commits {
            changelog.add_from_commit(commit);
        }

        let steps = vec![
            ReleaseStep {
                name: "Verify".to_string(),
                command: "cargo test --all-features && cargo clippy -- -D warnings".to_string(),
                description: "Run all tests and lints".to_string(),
            },
            ReleaseStep {
                name: "Bump version".to_string(),
                command: format!(
                    "sed -i 's/version = \"{}\"/version = \"{}\"/g' Cargo.toml",
                    current_version, next_version
                ),
                description: "Update version in Cargo.toml".to_string(),
            },
            ReleaseStep {
                name: "Update changelog".to_string(),
                command: "prepend CHANGELOG.md with release notes".to_string(),
                description: "Add release notes to changelog".to_string(),
            },
            ReleaseStep {
                name: "Commit".to_string(),
                command: format!(
                    "git add -A && git commit -m '{}'",
                    config.render_commit_message(&next_version)
                ),
                description: "Commit version bump".to_string(),
            },
            ReleaseStep {
                name: "Tag".to_string(),
                command: format!(
                    "git tag {} -m '{}'",
                    config.render_tag(&next_version),
                    config.render_tag_message(&next_version)
                ),
                description: "Create release tag".to_string(),
            },
            ReleaseStep {
                name: "Push".to_string(),
                command: "git push origin main --tags".to_string(),
                description: "Push commits and tags".to_string(),
            },
        ];

        let tag = config.render_tag(&next_version);
        let commit_message = config.render_commit_message(&next_version);
        Self {
            current_version,
            next_version,
            changelog: changelog.render(),
            tag,
            commit_message,
            publish: config.publish,
            steps,
        }
    }
}

/// Generate the GitHub Actions release workflow YAML.
pub fn generate_release_workflow(targets: &[&str]) -> String {
    let matrix_targets: String = targets
        .iter()
        .map(|t| format!("          - {}", t))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"name: Release
on:
  push:
    tags: ['v*']

permissions:
  contents: write

jobs:
  build:
    runs-on: ${{{{ matrix.os }}}}
    strategy:
      matrix:
        include:
{matrix_targets}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{{{ matrix.target }}}}
      - name: Build
        run: cargo build --release --target ${{{{ matrix.target }}}}
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{{{ matrix.target }}}}
          path: target/${{{{ matrix.target }}}}/release/*

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Download artifacts
        uses: actions/download-artifact@v4
      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          generate_release_notes: true
          files: |
            */*
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_parse() {
        let v = SemVer::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre_release.is_none());
    }

    #[test]
    fn test_semver_parse_with_pre_release() {
        let v = SemVer::parse("1.2.3-beta.1").unwrap();
        assert_eq!(v.pre_release, Some("beta.1".to_string()));
    }

    #[test]
    fn test_semver_parse_with_build_metadata() {
        let v = SemVer::parse("1.2.3+build.42").unwrap();
        assert_eq!(v.build_metadata, Some("build.42".to_string()));
    }

    #[test]
    fn test_semver_parse_full() {
        let v = SemVer::parse("2.0.0-rc.1+build.123").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.pre_release, Some("rc.1".to_string()));
        assert_eq!(v.build_metadata, Some("build.123".to_string()));
    }

    #[test]
    fn test_semver_parse_invalid() {
        assert!(SemVer::parse("1.2").is_err());
        assert!(SemVer::parse("not.a.version").is_err());
        assert!(SemVer::parse("").is_err());
    }

    #[test]
    fn test_semver_bump_major() {
        let mut v = SemVer::new(1, 2, 3);
        v.bump_major();
        assert_eq!(v, SemVer::new(2, 0, 0));
    }

    #[test]
    fn test_semver_bump_minor() {
        let mut v = SemVer::new(1, 2, 3);
        v.bump_minor();
        assert_eq!(v, SemVer::new(1, 3, 0));
    }

    #[test]
    fn test_semver_bump_patch() {
        let mut v = SemVer::new(1, 2, 3);
        v.bump_patch();
        assert_eq!(v, SemVer::new(1, 2, 4));
    }

    #[test]
    fn test_semver_display() {
        let v = SemVer::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");

        let v = SemVer::new(1, 2, 3).with_pre_release("beta.1");
        assert_eq!(v.to_string(), "1.2.3-beta.1");
    }

    #[test]
    fn test_semver_ordering() {
        let v1 = SemVer::new(1, 0, 0);
        let v2 = SemVer::new(2, 0, 0);
        let v3 = SemVer::new(1, 1, 0);
        assert!(v1 < v2);
        assert!(v1 < v3);
        assert!(v3 < v2);
    }

    #[test]
    fn test_parse_conventional_commit() {
        let commit = parse_conventional_commit("feat(auth): add OAuth2 support").unwrap();
        assert_eq!(commit.commit_type, CommitType::Feat);
        assert_eq!(commit.scope, Some("auth".to_string()));
        assert_eq!(commit.description, "add OAuth2 support");
        assert!(!commit.breaking);
    }

    #[test]
    fn test_parse_breaking_commit() {
        let commit =
            parse_conventional_commit("feat(api)!: change response format").unwrap();
        assert!(commit.breaking);
    }

    #[test]
    fn test_parse_commit_without_scope() {
        let commit = parse_conventional_commit("fix: resolve memory leak").unwrap();
        assert_eq!(commit.commit_type, CommitType::Fix);
        assert!(commit.scope.is_none());
    }

    #[test]
    fn test_determine_next_version() {
        let current = SemVer::new(1, 2, 3);

        // Only fixes -> patch bump
        let commits = vec![Commit {
            hash: "abc".into(),
            commit_type: CommitType::Fix,
            scope: None,
            description: "fix bug".into(),
            body: None,
            breaking: false,
            footer: None,
        }];
        let next = determine_next_version(&current, &commits);
        assert_eq!(next, SemVer::new(1, 2, 4));

        // Feature -> minor bump
        let commits = vec![Commit {
            hash: "def".into(),
            commit_type: CommitType::Feat,
            scope: None,
            description: "add feature".into(),
            body: None,
            breaking: false,
            footer: None,
        }];
        let next = determine_next_version(&current, &commits);
        assert_eq!(next, SemVer::new(1, 3, 0));

        // Breaking -> major bump
        let commits = vec![Commit {
            hash: "ghi".into(),
            commit_type: CommitType::Feat,
            scope: None,
            description: "breaking change".into(),
            body: None,
            breaking: true,
            footer: None,
        }];
        let next = determine_next_version(&current, &commits);
        assert_eq!(next, SemVer::new(2, 0, 0));
    }

    #[test]
    fn test_changelog_generation() {
        let mut gen = ChangelogGenerator::new(SemVer::new(1, 1, 0), "2024-01-15".to_string());

        gen.add_from_commit(&Commit {
            hash: "abc1234567890".into(),
            commit_type: CommitType::Feat,
            scope: Some("auth".into()),
            description: "add OAuth2 support".into(),
            body: None,
            breaking: false,
            footer: None,
        });

        gen.add_from_commit(&Commit {
            hash: "def4567890123".into(),
            commit_type: CommitType::Fix,
            scope: None,
            description: "fix memory leak in parser".into(),
            body: None,
            breaking: false,
            footer: None,
        });

        let rendered = gen.render();
        assert!(rendered.contains("## [1.1.0]"));
        assert!(rendered.contains("2024-01-15"));
        assert!(rendered.contains("OAuth2"));
        assert!(rendered.contains("memory leak"));
    }

    #[test]
    fn test_release_config_rendering() {
        let config = ReleaseConfig {
            crate_name: "my-crate".into(),
            ..Default::default()
        };

        let version = SemVer::new(1, 2, 3);
        assert_eq!(config.render_tag(&version), "v1.2.3");
        assert_eq!(
            config.render_commit_message(&version),
            "(cargo-release) version 1.2.3"
        );
    }

    #[test]
    fn test_release_plan_creation() {
        let current = SemVer::new(1, 0, 0);
        let commits = vec![
            Commit {
                hash: "abc".into(),
                commit_type: CommitType::Feat,
                scope: Some("api".into()),
                description: "new endpoint".into(),
                body: None,
                breaking: false,
                footer: None,
            },
            Commit {
                hash: "def".into(),
                commit_type: CommitType::Fix,
                scope: None,
                description: "bug fix".into(),
                body: None,
                breaking: false,
                footer: None,
            },
        ];

        let config = ReleaseConfig::default();
        let plan = ReleasePlan::create(current, &commits, &config, "2024-06-01");

        assert_eq!(plan.next_version, SemVer::new(1, 1, 0));
        assert!(!plan.changelog.is_empty());
        assert_eq!(plan.tag, "v1.1.0");
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_generate_release_workflow() {
        let targets = vec![
            "          - os: ubuntu-latest\n            target: x86_64-unknown-linux-gnu",
            "          - os: macos-latest\n            target: x86_64-apple-darwin",
        ];
        let yaml = generate_release_workflow(&targets);
        assert!(yaml.contains("Release"));
        assert!(yaml.contains("x86_64-unknown-linux-gnu"));
    }

    #[test]
    fn test_changelog_category_mapping() {
        assert_eq!(
            ChangelogCategory::from_commit_type(&CommitType::Feat, false),
            ChangelogCategory::Added
        );
        assert_eq!(
            ChangelogCategory::from_commit_type(&CommitType::Fix, false),
            ChangelogCategory::Fixed
        );
        assert_eq!(
            ChangelogCategory::from_commit_type(&CommitType::Feat, true),
            ChangelogCategory::Changed
        );
    }
}
