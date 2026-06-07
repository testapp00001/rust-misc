//! # Artifact Management for Rust Projects
//!
//! Managing build artifacts, binary distribution, and package publishing is
//! critical for Rust projects. This module covers GitHub releases, crates.io
//! publishing, cargo install, and binary distribution patterns.
//!
//! ## Distribution Channels:
//!
//! | Channel | Use Case | Audience |
//! |---------|----------|----------|
//! | crates.io | Libraries | Rust developers |
//! | GitHub Releases | Binaries | End users |
//! | cargo install | Binaries | Rust developers |
//! | Package managers | Binaries | System administrators |
//! | Docker Hub | Containers | DevOps |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a release artifact with all metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseArtifact {
    pub name: String,
    pub version: String,
    pub target: String,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub sha256_checksum: String,
    pub artifact_type: ArtifactType,
    pub content_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    Binary,
    Archive,
    Library,
    DebugSymbols,
    Checksum,
    Source,
}

impl ReleaseArtifact {
    /// Create a binary artifact for a specific target.
    pub fn binary(name: &str, version: &str, target: &str, size: u64) -> Self {
        let ext = if target.contains("windows") { ".exe" } else { "" };
        Self {
            name: format!("{}-{}-{}{}", name, version, target, ext),
            version: version.into(),
            target: target.into(),
            file_path: format!("target/{}/release/{}{}", target, name, ext),
            file_size_bytes: size,
            sha256_checksum: format!("{:064x}", size), // simplified
            artifact_type: ArtifactType::Binary,
            content_type: "application/octet-stream".into(),
        }
    }

    /// Create a tar.gz archive artifact.
    pub fn archive(name: &str, version: &str, target: &str, size: u64) -> Self {
        Self {
            name: format!("{}-{}-{}.tar.gz", name, version, target),
            version: version.into(),
            target: target.into(),
            file_path: format!("dist/{}-{}-{}.tar.gz", name, version, target),
            file_size_bytes: size,
            sha256_checksum: format!("{:064x}", size),
            artifact_type: ArtifactType::Archive,
            content_type: "application/gzip".into(),
        }
    }

    pub fn human_size(&self) -> String {
        if self.file_size_bytes >= 1_048_576 {
            format!("{:.2} MB", self.file_size_bytes as f64 / 1_048_576.0)
        } else if self.file_size_bytes >= 1024 {
            format!("{:.2} KB", self.file_size_bytes as f64 / 1024.0)
        } else {
            format!("{} B", self.file_size_bytes)
        }
    }
}

/// GitHub Release configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub body: String,
    pub draft: bool,
    pub prerelease: bool,
    pub artifacts: Vec<ReleaseArtifact>,
    pub generate_checksums: bool,
}

impl GitHubRelease {
    pub fn new(tag: &str, name: &str) -> Self {
        Self {
            tag_name: tag.into(),
            name: name.into(),
            body: String::new(),
            draft: false,
            prerelease: false,
            artifacts: Vec::new(),
            generate_checksums: true,
        }
    }

    pub fn with_body(mut self, body: &str) -> Self {
        self.body = body.into();
        self
    }

    pub fn as_draft(mut self) -> Self {
        self.draft = true;
        self
    }

    pub fn as_prerelease(mut self) -> Self {
        self.prerelease = true;
        self
    }

    pub fn add_artifact(mut self, artifact: ReleaseArtifact) -> Self {
        self.artifacts.push(artifact);
        self
    }

    /// Generate the checksums file content.
    pub fn checksums_file(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("# Checksums for {}\n", self.tag_name));
        for artifact in &self.artifacts {
            output.push_str(&format!("{}  {}\n", artifact.sha256_checksum, artifact.name));
        }
        output
    }

    /// Generate the release body with artifact download table.
    pub fn render_body(&self) -> String {
        let mut body = self.body.clone();
        body.push_str("\n\n## Downloads\n\n");
        body.push_str("| File | Target | Size |\n");
        body.push_str("|------|--------|------|\n");
        for artifact in &self.artifacts {
            body.push_str(&format!(
                "| {} | {} | {} |\n",
                artifact.name, artifact.target, artifact.human_size()
            ));
        }
        if self.generate_checksums {
            body.push_str("\nSee `checksums.txt` for SHA-256 verification.\n");
        }
        body
    }
}

/// Configuration for crates.io publishing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CratesIoConfig {
    pub crate_name: String,
    pub version: String,
    pub description: String,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub readme: Option<String>,
    pub rust_version: Option<String>,
    pub exclude_patterns: Vec<String>,
}

impl CratesIoConfig {
    /// Generate the [package] section of Cargo.toml.
    pub fn to_cargo_toml_package_section(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("[package]\n"));
        output.push_str(&format!("name = \"{}\"\n", self.crate_name));
        output.push_str(&format!("version = \"{}\"\n", self.version));
        output.push_str(&format!("edition = \"2021\"\n"));
        output.push_str(&format!("description = \"{}\"\n", self.description));
        output.push_str(&format!("license = \"{}\"\n", self.license));

        if let Some(ref repo) = self.repository {
            output.push_str(&format!("repository = \"{}\"\n", repo));
        }
        if let Some(ref homepage) = self.homepage {
            output.push_str(&format!("homepage = \"{}\"\n", homepage));
        }
        if let Some(ref docs) = self.documentation {
            output.push_str(&format!("documentation = \"{}\"\n", docs));
        }
        if !self.keywords.is_empty() {
            let kws: Vec<String> = self.keywords.iter().map(|k| format!("\"{}\"", k)).collect();
            output.push_str(&format!("keywords = [{}]\n", kws.join(", ")));
        }
        if !self.categories.is_empty() {
            let cats: Vec<String> = self.categories.iter().map(|c| format!("\"{}\"", c)).collect();
            output.push_str(&format!("categories = [{}]\n", cats.join(", ")));
        }
        if let Some(ref rv) = self.rust_version {
            output.push_str(&format!("rust-version = \"{}\"\n", rv));
        }
        if !self.exclude_patterns.is_empty() {
            let pats: Vec<String> = self.exclude_patterns.iter().map(|p| format!("\"{}\"", p)).collect();
            output.push_str(&format!("exclude = [{}]\n", pats.join(", ")));
        }

        output
    }

    /// Generate the .cargo-ok file content (for publish dry-run).
    pub fn publish_checklist(&self) -> Vec<String> {
        vec![
            "Run cargo test --all-features".into(),
            "Run cargo clippy -- -D warnings".into(),
            "Run cargo doc --no-deps".into(),
            "Verify CHANGELOG.md is updated".into(),
            "Verify version in Cargo.toml is correct".into(),
            "Run cargo publish --dry-run".into(),
            "Run cargo publish".into(),
            format!("Verify https://crates.io/crates/{}", self.crate_name),
        ]
    }
}

/// Binary installation configuration for cargo install.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoInstallConfig {
    pub crate_name: String,
    pub binary_name: Option<String>,
    pub features: Vec<String>,
    pub default_features: bool,
    pub profile: String,
    pub target: Option<String>,
}

impl CargoInstallConfig {
    pub fn new(crate_name: &str) -> Self {
        Self {
            crate_name: crate_name.into(),
            binary_name: None,
            features: vec![],
            default_features: true,
            profile: "release".into(),
            target: None,
        }
    }

    pub fn with_binary(mut self, name: &str) -> Self {
        self.binary_name = Some(name.into());
        self
    }

    pub fn with_features(mut self, features: &[&str]) -> Self {
        self.features = features.iter().map(|f| f.to_string()).collect();
        self
    }

    /// Generate the cargo install command.
    pub fn install_command(&self) -> String {
        let mut cmd = format!("cargo install {}", self.crate_name);

        if let Some(ref binary) = self.binary_name {
            cmd.push_str(&format!(" --bin {}", binary));
        }

        if !self.features.is_empty() {
            cmd.push_str(&format!(" --features {}", self.features.join(",")));
        }

        if !self.default_features {
            cmd.push_str(" --no-default-features");
        }

        cmd.push_str(&format!(" --profile {}", self.profile));

        if let Some(ref target) = self.target {
            cmd.push_str(&format!(" --target {}", target));
        }

        cmd
    }
}

/// Release manifest that aggregates all artifacts and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub project_name: String,
    pub version: String,
    pub release_date: String,
    pub artifacts: Vec<ReleaseArtifact>,
    pub install_instructions: HashMap<String, String>,
    pub changelog_summary: String,
    pub breaking_changes: Vec<String>,
    pub contributors: Vec<String>,
}

impl ReleaseManifest {
    pub fn new(project: &str, version: &str, date: &str) -> Self {
        Self {
            project_name: project.into(),
            version: version.into(),
            release_date: date.into(),
            artifacts: Vec::new(),
            install_instructions: HashMap::new(),
            changelog_summary: String::new(),
            breaking_changes: Vec::new(),
            contributors: Vec::new(),
        }
    }

    /// Add standard installation instructions.
    pub fn with_standard_install_instructions(&mut self) {
        self.install_instructions.insert(
            "cargo".into(),
            format!("cargo install {} --version {}", self.project_name, self.version),
        );
        self.install_instructions.insert(
            "cargo-from-git".into(),
            format!(
                "cargo install --git https://github.com/user/{} --tag v{}",
                self.project_name, self.version
            ),
        );
        self.install_instructions.insert(
            "docker".into(),
            format!(
                "docker pull ghcr.io/user/{}:{}",
                self.project_name, self.version
            ),
        );
    }

    /// Generate the release notes markdown.
    pub fn render_release_notes(&self) -> String {
        let mut notes = String::new();
        notes.push_str(&format!(
            "# {} v{}\n\n",
            self.project_name, self.version
        ));
        notes.push_str(&format!("Released: {}\n\n", self.release_date));

        if !self.changelog_summary.is_empty() {
            notes.push_str("## Changes\n\n");
            notes.push_str(&self.changelog_summary);
            notes.push('\n');
        }

        if !self.breaking_changes.is_empty() {
            notes.push_str("## Breaking Changes\n\n");
            for change in &self.breaking_changes {
                notes.push_str(&format!("- {}\n", change));
            }
            notes.push('\n');
        }

        if !self.install_instructions.is_empty() {
            notes.push_str("## Installation\n\n");
            for (method, instruction) in &self.install_instructions {
                notes.push_str(&format!("**{}**:\n```bash\n{}\n```\n\n", method, instruction));
            }
        }

        if !self.artifacts.is_empty() {
            notes.push_str("## Artifacts\n\n");
            notes.push_str("| File | Size |\n|------|------|\n");
            for artifact in &self.artifacts {
                notes.push_str(&format!("| {} | {} |\n", artifact.name, artifact.human_size()));
            }
        }

        notes
    }
}

/// Generate a Makefile for common release operations.
pub fn generate_release_makefile(project_name: &str) -> String {
    format!(
        r#".PHONY: build test lint release publish clean

build:
	cargo build --release

test:
	cargo test --all-features
	cargo test --doc

lint:
	cargo fmt --all -- --check
	cargo clippy --all-targets --all-features -- -D warnings

doc:
	cargo doc --no-deps --all-features
	@echo "Documentation generated at target/doc/{project_name}/index.html"

release: lint test build
	@echo "Release build complete"

publish-dry-run:
	cargo publish --dry-run

publish:
	cargo publish

clean:
	cargo clean

install-local:
	cargo install --path .

docker-build:
	docker build -t {project_name}:latest .

docker-push:
	docker push ghcr.io/user/{project_name}:latest
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_artifact_binary() {
        let artifact = ReleaseArtifact::binary("myapp", "1.0.0", "x86_64-unknown-linux-gnu", 5_000_000);
        assert!(artifact.name.contains("myapp"));
        assert!(artifact.name.contains("1.0.0"));
        assert!(artifact.name.contains("x86_64"));
        assert!(!artifact.name.contains(".exe"));
        assert_eq!(artifact.artifact_type, ArtifactType::Binary);
    }

    #[test]
    fn test_release_artifact_windows_binary() {
        let artifact = ReleaseArtifact::binary("myapp", "1.0.0", "x86_64-pc-windows-msvc", 4_000_000);
        assert!(artifact.name.contains(".exe"));
    }

    #[test]
    fn test_release_artifact_archive() {
        let artifact = ReleaseArtifact::archive("myapp", "1.0.0", "x86_64-unknown-linux-gnu", 2_000_000);
        assert!(artifact.name.contains(".tar.gz"));
        assert_eq!(artifact.artifact_type, ArtifactType::Archive);
    }

    #[test]
    fn test_release_artifact_human_size() {
        let small = ReleaseArtifact::binary("app", "1.0", "linux", 500);
        assert_eq!(small.human_size(), "500 B");

        let medium = ReleaseArtifact::binary("app", "1.0", "linux", 15_000);
        assert_eq!(medium.human_size(), "14.65 KB");

        let large = ReleaseArtifact::binary("app", "1.0", "linux", 10_000_000);
        assert_eq!(large.human_size(), "9.54 MB");
    }

    #[test]
    fn test_github_release_creation() {
        let release = GitHubRelease::new("v1.0.0", "Release 1.0.0")
            .with_body("Initial stable release")
            .add_artifact(ReleaseArtifact::binary("myapp", "1.0.0", "linux", 5_000_000))
            .add_artifact(ReleaseArtifact::binary("myapp", "1.0.0", "macos", 4_500_000));

        assert_eq!(release.tag_name, "v1.0.0");
        assert_eq!(release.artifacts.len(), 2);
        assert!(!release.draft);
        assert!(!release.prerelease);
    }

    #[test]
    fn test_github_release_checksums() {
        let release = GitHubRelease::new("v1.0.0", "Release 1.0.0")
            .add_artifact(ReleaseArtifact::binary("myapp", "1.0.0", "linux", 1000))
            .add_artifact(ReleaseArtifact::binary("myapp", "1.0.0", "macos", 2000));

        let checksums = release.checksums_file();
        assert!(checksums.contains("v1.0.0"));
        assert!(checksums.contains("myapp-1.0.0"));
    }

    #[test]
    fn test_github_release_render_body() {
        let release = GitHubRelease::new("v1.0.0", "Release 1.0.0")
            .with_body("Bug fixes and improvements")
            .add_artifact(ReleaseArtifact::binary("myapp", "1.0.0", "linux", 5_000_000));

        let body = release.render_body();
        assert!(body.contains("Bug fixes"));
        assert!(body.contains("Downloads"));
        assert!(body.contains("checksums.txt"));
    }

    #[test]
    fn test_github_release_draft_prerelease() {
        let draft = GitHubRelease::new("v1.0.0-beta.1", "Beta 1").as_draft();
        assert!(draft.draft);

        let pre = GitHubRelease::new("v1.0.0-rc.1", "RC 1").as_prerelease();
        assert!(pre.prerelease);
    }

    #[test]
    fn test_crates_io_config_cargo_toml() {
        let config = CratesIoConfig {
            crate_name: "my-crate".into(),
            version: "1.0.0".into(),
            description: "A great crate".into(),
            license: "MIT".into(),
            repository: Some("https://github.com/user/my-crate".into()),
            homepage: None,
            documentation: None,
            keywords: vec!["cli".into(), "tool".into()],
            categories: vec!["command-line-utilities".into()],
            readme: Some("README.md".into()),
            rust_version: Some("1.70".into()),
            exclude_patterns: vec!["tests/".into()],
        };

        let toml = config.to_cargo_toml_package_section();
        assert!(toml.contains("name = \"my-crate\""));
        assert!(toml.contains("version = \"1.0.0\""));
        assert!(toml.contains("license = \"MIT\""));
        assert!(toml.contains("keywords = [\"cli\", \"tool\"]"));
        assert!(toml.contains("rust-version = \"1.70\""));
    }

    #[test]
    fn test_crates_io_checklist() {
        let config = CratesIoConfig {
            crate_name: "my-crate".into(),
            version: "1.0.0".into(),
            description: "test".into(),
            license: "MIT".into(),
            repository: None,
            homepage: None,
            documentation: None,
            keywords: vec![],
            categories: vec![],
            readme: None,
            rust_version: None,
            exclude_patterns: vec![],
        };

        let checklist = config.publish_checklist();
        assert!(!checklist.is_empty());
        assert!(checklist.iter().any(|c| c.contains("cargo test")));
        assert!(checklist.iter().any(|c| c.contains("cargo publish")));
    }

    #[test]
    fn test_cargo_install_config() {
        let config = CargoInstallConfig::new("my-crate")
            .with_binary("my-bin")
            .with_features(&["feat1", "feat2"]);

        let cmd = config.install_command();
        assert!(cmd.contains("cargo install my-crate"));
        assert!(cmd.contains("--bin my-bin"));
        assert!(cmd.contains("--features feat1,feat2"));
        assert!(cmd.contains("--profile release"));
    }

    #[test]
    fn test_cargo_install_minimal() {
        let config = CargoInstallConfig::new("ripgrep");
        let cmd = config.install_command();
        assert_eq!(cmd, "cargo install ripgrep --profile release");
    }

    #[test]
    fn test_release_manifest() {
        let mut manifest = ReleaseManifest::new("myapp", "2.0.0", "2024-06-01");
        manifest.changelog_summary = "New features and bug fixes".into();
        manifest.breaking_changes = vec!["Changed API response format".into()];
        manifest.with_standard_install_instructions();
        manifest.artifacts.push(ReleaseArtifact::binary(
            "myapp", "2.0.0", "linux", 5_000_000,
        ));

        let notes = manifest.render_release_notes();
        assert!(notes.contains("myapp v2.0.0"));
        assert!(notes.contains("2024-06-01"));
        assert!(notes.contains("Breaking Changes"));
        assert!(notes.contains("Installation"));
        assert!(notes.contains("cargo install myapp --version 2.0.0"));
    }

    #[test]
    fn test_release_makefile() {
        let makefile = generate_release_makefile("myapp");
        assert!(makefile.contains("cargo build --release"));
        assert!(makefile.contains("cargo test"));
        assert!(makefile.contains("cargo clippy"));
        assert!(makefile.contains("cargo publish"));
        assert!(makefile.contains("docker build"));
    }

    #[test]
    fn test_crates_io_config_minimal() {
        let config = CratesIoConfig {
            crate_name: "minimal".into(),
            version: "0.1.0".into(),
            description: "minimal crate".into(),
            license: "MIT OR Apache-2.0".into(),
            repository: None,
            homepage: None,
            documentation: None,
            keywords: vec![],
            categories: vec![],
            readme: None,
            rust_version: None,
            exclude_patterns: vec![],
        };

        let toml = config.to_cargo_toml_package_section();
        assert!(toml.contains("edition = \"2021\""));
        assert!(!toml.contains("repository"));
        assert!(!toml.contains("keywords"));
    }
}
