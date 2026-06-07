//! # Crate Publishing
//!
//! Publishing a crate to crates.io requires proper metadata, documentation,
//! and adherence to conventions. This module covers the publishing workflow
//! and best practices.
//!
//! ## Key Concepts
//! - **Cargo.toml metadata**: name, version, description, license, etc.
//! - **Categories and keywords**: Helping users discover your crate
//! - **README**: The first thing users see
//! - **Version management**: When and how to publish new versions

/// Represents crate metadata for publishing.
#[derive(Debug, Clone)]
pub struct CrateMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub license: String,
    pub repository: Option<String>,
    pub homepage: Option<String>,
    pub documentation: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub authors: Vec<String>,
    pub edition: String,
    pub rust_version: Option<String>,
}

impl CrateMetadata {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        CrateMetadata {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            license: "MIT".to_string(),
            repository: None,
            homepage: None,
            documentation: None,
            keywords: Vec::new(),
            categories: Vec::new(),
            authors: Vec::new(),
            edition: "2021".to_string(),
            rust_version: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn license(mut self, license: impl Into<String>) -> Self {
        self.license = license.into();
        self
    }

    pub fn repository(mut self, url: impl Into<String>) -> Self {
        self.repository = Some(url.into());
        self
    }

    pub fn keywords(mut self, keywords: Vec<&str>) -> Self {
        self.keywords = keywords.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn categories(mut self, categories: Vec<&str>) -> Self {
        self.categories = categories.into_iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn authors(mut self, authors: Vec<&str>) -> Self {
        self.authors = authors.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// Validates the metadata for crates.io publishing requirements.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.name.is_empty() {
            errors.push("Name is required".into());
        }
        if self.name.len() > 64 {
            errors.push("Name must be 64 characters or less".into());
        }
        if self.description.is_empty() {
            errors.push("Description is required".into());
        }
        if self.description.len() > 256 {
            errors.push("Description must be 256 characters or less".into());
        }
        if self.keywords.len() > 5 {
            errors.push("Maximum 5 keywords allowed".into());
        }
        if self.categories.len() > 5 {
            errors.push("Maximum 5 categories allowed".into());
        }
        if !is_valid_version(&self.version) {
            errors.push(format!("Invalid version format: {}", self.version));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Generates the Cargo.toml [package] section.
    pub fn to_cargo_toml(&self) -> String {
        let mut toml = format!(
            "[package]\nname = \"{}\"\nversion = \"{}\"\nedition = \"{}\"\n",
            self.name, self.version, self.edition
        );

        if !self.description.is_empty() {
            toml.push_str(&format!("description = \"{}\"\n", self.description));
        }
        toml.push_str(&format!("license = \"{}\"\n", self.license));

        if let Some(repo) = &self.repository {
            toml.push_str(&format!("repository = \"{repo}\"\n"));
        }
        if !self.authors.is_empty() {
            let authors: Vec<String> = self.authors.iter().map(|a| format!("\"{a}\"")).collect();
            toml.push_str(&format!("authors = [{}]\n", authors.join(", ")));
        }
        if !self.keywords.is_empty() {
            let keywords: Vec<String> = self.keywords.iter().map(|k| format!("\"{k}\"")).collect();
            toml.push_str(&format!("keywords = [{}]\n", keywords.join(", ")));
        }
        if !self.categories.is_empty() {
            let categories: Vec<String> = self.categories.iter().map(|c| format!("\"{c}\"")).collect();
            toml.push_str(&format!("categories = [{}]\n", categories.join(", ")));
        }

        toml
    }
}

fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u32>().is_ok())
}

/// Validates a crate name follows crates.io naming rules.
pub fn validate_crate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Name cannot be empty".into());
    }
    if name.len() > 64 {
        return Err("Name too long (max 64 characters)".into());
    }
    if name.starts_with('-') || name.ends_with('-') {
        return Err("Name cannot start or end with '-'".into());
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
    {
        return Err(
            "Name can only contain lowercase letters, digits, hyphens, and underscores".into(),
        );
    }
    Ok(())
}

/// A changelog format for release notes.
pub struct Changelog {
    entries: Vec<ChangelogEntry>,
}

pub struct ChangelogEntry {
    pub version: String,
    pub date: String,
    pub changes: Vec<String>,
}

impl Changelog {
    pub fn new() -> Self {
        Changelog { entries: Vec::new() }
    }

    pub fn add_entry(&mut self, entry: ChangelogEntry) {
        self.entries.push(entry);
    }

    pub fn to_markdown(&self) -> String {
        let mut md = String::from("# Changelog\n\n");

        for entry in &self.entries {
            md.push_str(&format!("## {} ({})\n\n", entry.version, entry.date));
            for change in &entry.changes {
                md.push_str(&format!("- {change}\n"));
            }
            md.push('\n');
        }

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_metadata_defaults() {
        let meta = CrateMetadata::new("my-crate", "0.1.0");
        assert_eq!(meta.name, "my-crate");
        assert_eq!(meta.version, "0.1.0");
        assert_eq!(meta.license, "MIT");
        assert_eq!(meta.edition, "2021");
    }

    #[test]
    fn test_crate_metadata_builder() {
        let meta = CrateMetadata::new("my-crate", "1.0.0")
            .description("A great crate")
            .license("Apache-2.0")
            .repository("https://github.com/user/repo")
            .keywords(vec!["cli", "tool"])
            .categories(vec!["command-line-utilities"])
            .authors(vec!["Alice <alice@example.com>"]);

        assert_eq!(meta.description, "A great crate");
        assert_eq!(meta.license, "Apache-2.0");
        assert_eq!(meta.keywords, vec!["cli", "tool"]);
    }

    #[test]
    fn test_crate_metadata_validation_valid() {
        let meta = CrateMetadata::new("my-crate", "1.0.0").description("A crate");
        assert!(meta.validate().is_ok());
    }

    #[test]
    fn test_crate_metadata_validation_missing_name() {
        let meta = CrateMetadata::new("", "1.0.0").description("A crate");
        let err = meta.validate().unwrap_err();
        assert!(err.iter().any(|e| e.contains("Name is required")));
    }

    #[test]
    fn test_crate_metadata_validation_missing_description() {
        let meta = CrateMetadata::new("my-crate", "1.0.0");
        let err = meta.validate().unwrap_err();
        assert!(err.iter().any(|e| e.contains("Description is required")));
    }

    #[test]
    fn test_crate_metadata_validation_too_many_keywords() {
        let meta = CrateMetadata::new("my-crate", "1.0.0")
            .description("A crate")
            .keywords(vec!["a", "b", "c", "d", "e", "f"]);
        let err = meta.validate().unwrap_err();
        assert!(err.iter().any(|e| e.contains("keywords")));
    }

    #[test]
    fn test_crate_metadata_validation_bad_version() {
        let meta = CrateMetadata::new("my-crate", "1.0").description("A crate");
        let err = meta.validate().unwrap_err();
        assert!(err.iter().any(|e| e.contains("version")));
    }

    #[test]
    fn test_cargo_toml_generation() {
        let meta = CrateMetadata::new("my-crate", "1.0.0")
            .description("A great crate")
            .license("MIT")
            .repository("https://github.com/user/my-crate");

        let toml = meta.to_cargo_toml();
        assert!(toml.contains("name = \"my-crate\""));
        assert!(toml.contains("version = \"1.0.0\""));
        assert!(toml.contains("description = \"A great crate\""));
        assert!(toml.contains("license = \"MIT\""));
        assert!(toml.contains("repository = \"https://github.com/user/my-crate\""));
    }

    #[test]
    fn test_validate_crate_name_valid() {
        assert!(validate_crate_name("my-crate").is_ok());
        assert!(validate_crate_name("my_crate").is_ok());
        assert!(validate_crate_name("my-crate-2").is_ok());
    }

    #[test]
    fn test_validate_crate_name_invalid() {
        assert!(validate_crate_name("").is_err());
        assert!(validate_crate_name("-start").is_err());
        assert!(validate_crate_name("end-").is_err());
        assert!(validate_crate_name("Has Caps").is_err());
        assert!(validate_crate_name("has spaces").is_err());
    }

    #[test]
    fn test_changelog() {
        let mut changelog = Changelog::new();
        changelog.add_entry(ChangelogEntry {
            version: "1.0.0".into(),
            date: "2024-01-15".into(),
            changes: vec!["Initial release".into(), "Added core API".into()],
        });
        changelog.add_entry(ChangelogEntry {
            version: "1.1.0".into(),
            date: "2024-02-01".into(),
            changes: vec!["Added new feature".into()],
        });

        let md = changelog.to_markdown();
        assert!(md.contains("# Changelog"));
        assert!(md.contains("## 1.0.0 (2024-01-15)"));
        assert!(md.contains("- Initial release"));
        assert!(md.contains("## 1.1.0 (2024-02-01)"));
    }
}
