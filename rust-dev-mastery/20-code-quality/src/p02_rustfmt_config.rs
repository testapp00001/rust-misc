//! # rustfmt Configuration
//!
//! rustfmt enforces consistent code formatting across a project. This module
//! covers rustfmt.toml configuration, formatting rules, and best practices.
//!
//! ## Key Settings:
//!
//! ```toml
//! edition = "2021"
//! max_width = 100
//! tab_spaces = 4
//! use_field_init_shorthand = true
//! use_try_shorthand = true
//! reorder_imports = true
//! group_imports = "StdExternalCrate"
//! ```

/// Represents rustfmt configuration options.
#[derive(Debug, Clone)]
pub struct RustfmtConfig {
    pub edition: String,
    pub max_width: usize,
    pub tab_spaces: usize,
    pub use_field_init_shorthand: bool,
    pub use_try_shorthand: bool,
    pub reorder_imports: bool,
    pub reorder_modules: bool,
    pub group_imports: ImportGrouping,
    pub blank_lines_upper_bound: usize,
    pub blank_lines_lower_bound: usize,
    pub normalize_comments: bool,
    pub normalize_doc_attributes: bool,
    pub format_code_in_doc_comments: bool,
    pub wrap_comments: bool,
}

#[derive(Debug, Clone)]
pub enum ImportGrouping {
    Preserve,
    StdExternalCrate,
    One,
}

impl Default for RustfmtConfig {
    fn default() -> Self {
        Self {
            edition: "2021".into(),
            max_width: 100,
            tab_spaces: 4,
            use_field_init_shorthand: true,
            use_try_shorthand: true,
            reorder_imports: true,
            reorder_modules: true,
            group_imports: ImportGrouping::StdExternalCrate,
            blank_lines_upper_bound: 1,
            blank_lines_lower_bound: 0,
            normalize_comments: false,
            normalize_doc_attributes: true,
            format_code_in_doc_comments: true,
            wrap_comments: false,
        }
    }
}

impl RustfmtConfig {
    /// Generate rustfmt.toml content.
    pub fn to_toml(&self) -> String {
        let group_imports = match self.group_imports {
            ImportGrouping::Preserve => "Preserve",
            ImportGrouping::StdExternalCrate => "StdExternalCrate",
            ImportGrouping::One => "One",
        };

        format!(
            r#"edition = "{}"
max_width = {}
tab_spaces = {}
use_field_init_shorthand = {}
use_try_shorthand = {}
reorder_imports = {}
reorder_modules = {}
group_imports = "{}"
blank_lines_upper_bound = {}
blank_lines_lower_bound = {}
normalize_doc_attributes = {}
format_code_in_doc_comments = {}
"#,
            self.edition,
            self.max_width,
            self.tab_spaces,
            self.use_field_init_shorthand,
            self.use_try_shorthand,
            self.reorder_imports,
            self.reorder_modules,
            group_imports,
            self.blank_lines_upper_bound,
            self.blank_lines_lower_bound,
            self.normalize_doc_attributes,
            self.format_code_in_doc_comments,
        )
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.max_width < 40 {
            errors.push("max_width should be at least 40".into());
        }
        if self.tab_spaces == 0 {
            errors.push("tab_spaces cannot be 0".into());
        }
        if self.max_width > 200 {
            errors.push("max_width over 200 is not recommended".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Pre-defined rustfmt profiles.
pub struct RustfmtProfiles;

impl RustfmtProfiles {
    /// Default Rust style.
    pub fn default_style() -> RustfmtConfig {
        RustfmtConfig::default()
    }

    /// Narrow style for better readability.
    pub fn narrow() -> RustfmtConfig {
        RustfmtConfig {
            max_width: 80,
            ..Default::default()
        }
    }

    /// Wide style for less wrapping.
    pub fn wide() -> RustfmtConfig {
        RustfmtConfig {
            max_width: 120,
            ..Default::default()
        }
    }

    /// Strict style with all options.
    pub fn strict() -> RustfmtConfig {
        RustfmtConfig {
            max_width: 100,
            normalize_comments: true,
            normalize_doc_attributes: true,
            format_code_in_doc_comments: true,
            wrap_comments: true,
            reorder_imports: true,
            reorder_modules: true,
            ..Default::default()
        }
    }
}

/// Code formatting check result.
#[derive(Debug, Clone)]
pub struct FormatCheckResult {
    pub file: String,
    pub is_formatted: bool,
    pub violations: Vec<FormatViolation>,
}

#[derive(Debug, Clone)]
pub struct FormatViolation {
    pub line: usize,
    pub expected: String,
    pub actual: String,
}

impl FormatCheckResult {
    pub fn clean(file: &str) -> Self {
        Self {
            file: file.into(),
            is_formatted: true,
            violations: Vec::new(),
        }
    }

    pub fn with_violations(file: &str, violations: Vec<FormatViolation>) -> Self {
        Self {
            file: file.into(),
            is_formatted: violations.is_empty(),
            violations,
        }
    }

    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// Import ordering checker.
pub struct ImportOrderChecker {
    groups: Vec<ImportGroup>,
}

#[derive(Debug, Clone)]
pub struct ImportGroup {
    pub name: String,
    pub imports: Vec<String>,
}

impl ImportOrderChecker {
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }

    pub fn add_group(&mut self, name: &str, imports: Vec<&str>) {
        self.groups.push(ImportGroup {
            name: name.into(),
            imports: imports.into_iter().map(String::from).collect(),
        });
    }

    /// Check if imports are in the correct order.
    pub fn check_order(&self) -> bool {
        for group in &self.groups {
            let mut sorted = group.imports.clone();
            sorted.sort();
            if group.imports != sorted {
                return false;
            }
        }
        true
    }

    /// Get the correctly ordered imports.
    pub fn sorted_imports(&self) -> Vec<ImportGroup> {
        self.groups
            .iter()
            .map(|g| {
                let mut sorted = g.clone();
                sorted.imports.sort();
                sorted
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rustfmt_config_default() {
        let config = RustfmtConfig::default();
        assert_eq!(config.max_width, 100);
        assert_eq!(config.tab_spaces, 4);
        assert!(config.use_field_init_shorthand);
    }

    #[test]
    fn test_rustfmt_config_toml() {
        let config = RustfmtConfig::default();
        let toml = config.to_toml();
        assert!(toml.contains("max_width = 100"));
        assert!(toml.contains("tab_spaces = 4"));
        assert!(toml.contains("edition = \"2021\""));
    }

    #[test]
    fn test_rustfmt_config_validate_valid() {
        let config = RustfmtConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_rustfmt_config_validate_narrow() {
        let config = RustfmtConfig {
            max_width: 30,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_rustfmt_config_validate_wide() {
        let config = RustfmtConfig {
            max_width: 250,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_rustfmt_profiles() {
        let narrow = RustfmtProfiles::narrow();
        assert_eq!(narrow.max_width, 80);

        let wide = RustfmtProfiles::wide();
        assert_eq!(wide.max_width, 120);

        let strict = RustfmtProfiles::strict();
        assert!(strict.normalize_comments);
        assert!(strict.wrap_comments);
    }

    #[test]
    fn test_format_check_result_clean() {
        let result = FormatCheckResult::clean("main.rs");
        assert!(result.is_formatted);
        assert_eq!(result.violation_count(), 0);
    }

    #[test]
    fn test_format_check_result_violations() {
        let violations = vec![FormatViolation {
            line: 10,
            expected: "    let x = 1;".into(),
            actual: "  let x = 1;".into(),
        }];
        let result = FormatCheckResult::with_violations("main.rs", violations);
        assert!(!result.is_formatted);
        assert_eq!(result.violation_count(), 1);
    }

    #[test]
    fn test_import_order_checker() {
        let mut checker = ImportOrderChecker::new();
        checker.add_group("std", vec!["std::collections::HashMap", "std::io"]);
        assert!(checker.check_order());
    }

    #[test]
    fn test_import_order_checker_unordered() {
        let mut checker = ImportOrderChecker::new();
        checker.add_group("std", vec!["std::io", "std::collections::HashMap"]);
        assert!(!checker.check_order());
    }

    #[test]
    fn test_import_order_checker_sorted() {
        let mut checker = ImportOrderChecker::new();
        checker.add_group("std", vec!["std::io", "std::collections::HashMap"]);

        let sorted = checker.sorted_imports();
        assert_eq!(sorted[0].imports[0], "std::collections::HashMap");
        assert_eq!(sorted[0].imports[1], "std::io");
    }

    #[test]
    fn test_import_grouping_variants() {
        let config = RustfmtConfig {
            group_imports: ImportGrouping::One,
            ..Default::default()
        };
        let toml = config.to_toml();
        assert!(toml.contains("group_imports = \"One\""));
    }
}
