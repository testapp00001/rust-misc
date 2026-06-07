//! # Lesson 6: Cargo.toml Deep Dive
//!
//! Cargo.toml is the manifest file that controls everything about a Rust project.
//! This lesson covers every major section: package metadata, dependencies,
//! dev-dependencies, build dependencies, targets, patches, and replacements.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents the complete structure of a Cargo.toml file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoManifest {
    pub package: PackageMetadata,
    pub dependencies: BTreeMap<String, DepSpec>,
    #[serde(rename = "dev-dependencies")]
    pub dev_dependencies: BTreeMap<String, DepSpec>,
    #[serde(rename = "build-dependencies")]
    pub build_dependencies: BTreeMap<String, DepSpec>,
    pub features: BTreeMap<String, Vec<String>>,
    pub targets: Vec<Target>,
    pub patches: BTreeMap<String, BTreeMap<String, DepSpec>>,
}

/// Package metadata from the [package] section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub authors: Vec<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub readme: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub rust_version: Option<String>,
    pub exclude: Vec<String>,
    pub include: Vec<String>,
}

impl PackageMetadata {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            edition: "2021".to_string(),
            authors: Vec::new(),
            description: None,
            license: None,
            repository: None,
            homepage: None,
            documentation: None,
            readme: None,
            keywords: Vec::new(),
            categories: Vec::new(),
            rust_version: None,
            exclude: Vec::new(),
            include: Vec::new(),
        }
    }

    pub fn with_authors(mut self, authors: Vec<impl Into<String>>) -> Self {
        self.authors = authors.into_iter().map(|a| a.into()).collect();
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    pub fn with_repository(mut self, repo: impl Into<String>) -> Self {
        self.repository = Some(repo.into());
        self
    }

    pub fn with_keywords(mut self, keywords: Vec<impl Into<String>>) -> Self {
        self.keywords = keywords.into_iter().map(|k| k.into()).collect();
        self
    }

    pub fn with_categories(mut self, categories: Vec<impl Into<String>>) -> Self {
        self.categories = categories.into_iter().map(|c| c.into()).collect();
        self
    }

    pub fn with_rust_version(mut self, msrv: impl Into<String>) -> Self {
        self.rust_version = Some(msrv.into());
        self
    }
}

/// A dependency specification, covering all Cargo.toml formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DepSpec {
    /// Simple version string: `serde = "1.0"`
    Version(String),
    /// Detailed spec: `serde = { version = "1.0", features = ["derive"] }`
    Detailed {
        version: Option<String>,
        path: Option<String>,
        git: Option<String>,
        branch: Option<String>,
        tag: Option<String>,
        rev: Option<String>,
        features: Vec<String>,
        default_features: bool,
        optional: bool,
    },
}

impl DepSpec {
    pub fn version(v: impl Into<String>) -> Self {
        DepSpec::Version(v.into())
    }

    pub fn with_path(path: impl Into<String>) -> Self {
        DepSpec::Detailed {
            version: None,
            path: Some(path.into()),
            git: None,
            branch: None,
            tag: None,
            rev: None,
            features: Vec::new(),
            default_features: true,
            optional: false,
        }
    }

    pub fn with_git(url: impl Into<String>) -> Self {
        DepSpec::Detailed {
            version: None,
            path: None,
            git: Some(url.into()),
            branch: None,
            tag: None,
            rev: None,
            features: Vec::new(),
            default_features: true,
            optional: false,
        }
    }

    pub fn with_features(mut self, features: Vec<impl Into<String>>) -> Self {
        match &mut self {
            DepSpec::Detailed { features: f, .. } => {
                *f = features.into_iter().map(|f| f.into()).collect();
            }
            DepSpec::Version(_) => {
                self = DepSpec::Detailed {
                    version: Some(match self {
                        DepSpec::Version(v) => v,
                        _ => unreachable!(),
                    }),
                    path: None,
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    features: features.into_iter().map(|f| f.into()).collect(),
                    default_features: true,
                    optional: false,
                };
            }
        }
        self
    }

    pub fn optional(mut self) -> Self {
        match &mut self {
            DepSpec::Detailed { optional, .. } => {
                *optional = true;
            }
            DepSpec::Version(_) => {
                self = DepSpec::Detailed {
                    version: Some(match self {
                        DepSpec::Version(v) => v,
                        _ => unreachable!(),
                    }),
                    path: None,
                    git: None,
                    branch: None,
                    tag: None,
                    rev: None,
                    features: Vec::new(),
                    default_features: true,
                    optional: true,
                };
            }
        }
        self
    }
}

/// A target override (e.g., for WASM or platform-specific deps).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    pub target_triple: String,
    pub dependencies: BTreeMap<String, DepSpec>,
}

impl Target {
    pub fn new(triple: impl Into<String>) -> Self {
        Self {
            target_triple: triple.into(),
            dependencies: BTreeMap::new(),
        }
    }

    pub fn add_dep(&mut self, name: impl Into<String>, spec: DepSpec) {
        self.dependencies.insert(name.into(), spec);
    }
}

/// Generate a complete Cargo.toml from a manifest structure.
pub fn generate_cargo_toml(manifest: &CargoManifest) -> String {
    let mut toml = String::new();

    // [package] section
    toml.push_str("[package]\n");
    toml.push_str(&format!("name = \"{}\"\n", manifest.package.name));
    toml.push_str(&format!("version = \"{}\"\n", manifest.package.version));
    toml.push_str(&format!("edition = \"{}\"\n", manifest.package.edition));

    if !manifest.package.authors.is_empty() {
        let authors: Vec<String> = manifest
            .package
            .authors
            .iter()
            .map(|a| format!("\"{}\"", a))
            .collect();
        toml.push_str(&format!("authors = [{}]\n", authors.join(", ")));
    }

    if let Some(ref desc) = manifest.package.description {
        toml.push_str(&format!("description = \"{}\"\n", desc));
    }

    if let Some(ref license) = manifest.package.license {
        toml.push_str(&format!("license = \"{}\"\n", license));
    }

    if let Some(ref repo) = manifest.package.repository {
        toml.push_str(&format!("repository = \"{}\"\n", repo));
    }

    if let Some(ref msrv) = manifest.package.rust_version {
        toml.push_str(&format!("rust-version = \"{}\"\n", msrv));
    }

    // [dependencies]
    if !manifest.dependencies.is_empty() {
        toml.push_str("\n[dependencies]\n");
        for (name, spec) in &manifest.dependencies {
            toml.push_str(&format!("{} = {}\n", name, dep_spec_to_string(spec)));
        }
    }

    // [dev-dependencies]
    if !manifest.dev_dependencies.is_empty() {
        toml.push_str("\n[dev-dependencies]\n");
        for (name, spec) in &manifest.dev_dependencies {
            toml.push_str(&format!("{} = {}\n", name, dep_spec_to_string(spec)));
        }
    }

    toml
}

fn dep_spec_to_string(spec: &DepSpec) -> String {
    match spec {
        DepSpec::Version(v) => format!("\"{}\"", v),
        DepSpec::Detailed {
            version,
            path,
            git,
            branch,
            tag,
            rev,
            features,
            default_features,
            optional,
        } => {
            let mut parts = Vec::new();
            if let Some(v) = version {
                parts.push(format!("version = \"{}\"", v));
            }
            if let Some(p) = path {
                parts.push(format!("path = \"{}\"", p));
            }
            if let Some(g) = git {
                parts.push(format!("git = \"{}\"", g));
            }
            if let Some(b) = branch {
                parts.push(format!("branch = \"{}\"", b));
            }
            if let Some(t) = tag {
                parts.push(format!("tag = \"{}\"", t));
            }
            if let Some(r) = rev {
                parts.push(format!("rev = \"{}\"", r));
            }
            if !features.is_empty() {
                let f: Vec<String> = features.iter().map(|f| format!("\"{}\"", f)).collect();
                parts.push(format!("features = [{}]", f.join(", ")));
            }
            if !default_features {
                parts.push("default-features = false".to_string());
            }
            if *optional {
                parts.push("optional = true".to_string());
            }
            format!("{{ {} }}", parts.join(", "))
        }
    }
}

/// Validate a Cargo.toml manifest for common issues.
pub fn validate_manifest(manifest: &CargoManifest) -> Vec<String> {
    let mut warnings = Vec::new();

    if manifest.package.name.is_empty() {
        warnings.push("Package name is empty".to_string());
    }

    if manifest.package.name.contains('-') {
        // This is actually fine, but we note it for awareness
        warnings.push(format!(
            "Package name '{}' uses hyphens; Cargo normalizes to underscores in code",
            manifest.package.name
        ));
    }

    if manifest.package.authors.is_empty() {
        warnings.push("No authors specified".to_string());
    }

    if manifest.package.license.is_none() {
        warnings.push("No license specified; required for crates.io publishing".to_string());
    }

    if manifest.package.description.is_none() {
        warnings.push("No description; required for crates.io publishing".to_string());
    }

    if manifest.package.categories.len() > 5 {
        warnings.push("crates.io allows at most 5 categories".to_string());
    }

    if manifest.package.keywords.len() > 5 {
        warnings.push("crates.io allows at most 5 keywords".to_string());
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest() -> CargoManifest {
        let mut deps = BTreeMap::new();
        deps.insert("serde".to_string(), DepSpec::version("1.0"));
        deps.insert(
            "tokio".to_string(),
            DepSpec::version("1.0").with_features(vec!["full"]),
        );

        let mut dev_deps = BTreeMap::new();
        dev_deps.insert("criterion".to_string(), DepSpec::version("0.5"));

        CargoManifest {
            package: PackageMetadata::new("my-app", "0.1.0")
                .with_authors(vec!["Alice <alice@example.com>"])
                .with_description("A sample application")
                .with_license("MIT")
                .with_repository("https://github.com/alice/my-app")
                .with_keywords(vec!["web", "api"])
                .with_rust_version("1.70"),
            dependencies: deps,
            dev_dependencies: dev_deps,
            build_dependencies: BTreeMap::new(),
            features: BTreeMap::new(),
            targets: Vec::new(),
            patches: BTreeMap::new(),
        }
    }

    #[test]
    fn test_generate_cargo_toml() {
        let manifest = sample_manifest();
        let toml = generate_cargo_toml(&manifest);

        assert!(toml.contains("name = \"my-app\""));
        assert!(toml.contains("version = \"0.1.0\""));
        assert!(toml.contains("edition = \"2021\""));
        assert!(toml.contains("license = \"MIT\""));
        assert!(toml.contains("description = \"A sample application\""));
        assert!(toml.contains("rust-version = \"1.70\""));
        assert!(toml.contains("[dependencies]"));
        assert!(toml.contains("serde = \"1.0\""));
        assert!(toml.contains("[dev-dependencies]"));
        assert!(toml.contains("criterion = \"0.5\""));
    }

    #[test]
    fn test_dep_spec_features() {
        let spec = DepSpec::version("1.0").with_features(vec!["derive", "rc"]);
        match &spec {
            DepSpec::Detailed { features, .. } => {
                assert_eq!(features.len(), 2);
                assert!(features.contains(&"derive".to_string()));
            }
            _ => panic!("Expected Detailed"),
        }
    }

    #[test]
    fn test_dep_spec_optional() {
        let spec = DepSpec::version("1.0").optional();
        match &spec {
            DepSpec::Detailed { optional, .. } => {
                assert!(*optional);
            }
            _ => panic!("Expected Detailed"),
        }
    }

    #[test]
    fn test_dep_spec_path() {
        let spec = DepSpec::with_path("../shared");
        match &spec {
            DepSpec::Detailed { path, .. } => {
                assert_eq!(path.as_deref(), Some("../shared"));
            }
            _ => panic!("Expected Detailed"),
        }
    }

    #[test]
    fn test_dep_spec_git() {
        let spec = DepSpec::with_git("https://github.com/user/repo.git");
        match &spec {
            DepSpec::Detailed { git, .. } => {
                assert!(git.as_ref().unwrap().contains("github.com"));
            }
            _ => panic!("Expected Detailed"),
        }
    }

    #[test]
    fn test_target() {
        let mut target = Target::new("wasm32-unknown-unknown");
        target.add_dep("wasm-bindgen", DepSpec::version("0.2"));
        assert_eq!(target.dependencies.len(), 1);
        assert!(target.dependencies.contains_key("wasm-bindgen"));
    }

    #[test]
    fn test_validate_manifest_warnings() {
        let manifest = sample_manifest();
        let warnings = validate_manifest(&manifest);
        // Should have at least the hyphen warning
        assert!(warnings.iter().any(|w| w.contains("hyphens")));
    }

    #[test]
    fn test_validate_empty_name() {
        let mut manifest = sample_manifest();
        manifest.package.name = String::new();
        let warnings = validate_manifest(&manifest);
        assert!(warnings.iter().any(|w| w.contains("name is empty")));
    }

    #[test]
    fn test_validate_no_license() {
        let mut manifest = sample_manifest();
        manifest.package.license = None;
        let warnings = validate_manifest(&manifest);
        assert!(warnings.iter().any(|w| w.contains("license")));
    }

    #[test]
    fn test_validate_no_description() {
        let mut manifest = sample_manifest();
        manifest.package.description = None;
        let warnings = validate_manifest(&manifest);
        assert!(warnings.iter().any(|w| w.contains("description")));
    }

    #[test]
    fn test_validate_too_many_keywords() {
        let mut manifest = sample_manifest();
        manifest.package.keywords = (0..6).map(|i| format!("kw{}", i)).collect();
        let warnings = validate_manifest(&manifest);
        assert!(warnings.iter().any(|w| w.contains("keywords")));
    }

    #[test]
    fn test_dep_spec_to_string() {
        assert_eq!(dep_spec_to_string(&DepSpec::version("1.0")), "\"1.0\"");

        let detailed = DepSpec::Detailed {
            version: Some("1.0".to_string()),
            path: None,
            git: None,
            branch: None,
            tag: None,
            rev: None,
            features: vec!["derive".to_string()],
            default_features: true,
            optional: false,
        };
        let s = dep_spec_to_string(&detailed);
        assert!(s.contains("version = \"1.0\""));
        assert!(s.contains("features = [\"derive\"]"));
    }

    #[test]
    fn test_package_metadata_builder() {
        let pkg = PackageMetadata::new("test", "1.0.0")
            .with_authors(vec!["Test Author"])
            .with_description("Test description")
            .with_license("Apache-2.0")
            .with_keywords(vec!["test"])
            .with_categories(vec!["development-tools"]);

        assert_eq!(pkg.name, "test");
        assert_eq!(pkg.version, "1.0.0");
        assert_eq!(pkg.edition, "2021");
        assert_eq!(pkg.authors, vec!["Test Author"]);
        assert_eq!(pkg.description, Some("Test description".to_string()));
        assert_eq!(pkg.license, Some("Apache-2.0".to_string()));
    }
}
