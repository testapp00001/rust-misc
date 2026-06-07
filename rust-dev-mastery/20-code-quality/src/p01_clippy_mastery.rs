//! # Clippy Mastery
//!
//! Clippy is Rust's linter that catches common mistakes, suggests improvements,
//! and enforces idiomatic patterns. This module covers Clippy lint groups,
//! configuration, and when to suppress vs. fix.
//!
//! ## Lint Groups:
//!
//! | Group | Description |
//! |-------|-------------|
//! | `clippy::all` | All enabled-by-default lints |
//! | `clippy::pedantic` | Extra strict lints |
//! | `clippy::nursery` | New/experimental lints |
//! | `clippy::restriction` | Very restrictive (opt-in) |
//! | `clippy::cargo` | Cargo.toml issues |
//!
//! ## Configuration (clippy.toml):
//!
//! ```toml
//! too-many-arguments-threshold = 8
//! type-complexity-threshold = 500
//! cognitive-complexity-threshold = 25
//! ```

use std::collections::HashMap;

/// Clippy lint configuration representation.
#[derive(Debug, Clone)]
pub struct ClippyConfig {
    pub too_many_arguments_threshold: usize,
    pub type_complexity_threshold: usize,
    pub cognitive_complexity_threshold: usize,
    pub too_many_lines_threshold: usize,
    pub single_component_path_imports: bool,
    pub allowed_scripts: Vec<char>,
}

impl Default for ClippyConfig {
    fn default() -> Self {
        Self {
            too_many_arguments_threshold: 7,
            type_complexity_threshold: 250,
            cognitive_complexity_threshold: 25,
            too_many_lines_threshold: 100,
            single_component_path_imports: true,
            allowed_scripts: vec![')'],
        }
    }
}

impl ClippyConfig {
    /// Generate clippy.toml content.
    pub fn to_toml(&self) -> String {
        format!(
            r#"too-many-arguments-threshold = {}
type-complexity-threshold = {}
cognitive-complexity-threshold = {}
too-many-lines-threshold = {}
"#,
            self.too_many_arguments_threshold,
            self.type_complexity_threshold,
            self.cognitive_complexity_threshold,
            self.too_many_lines_threshold,
        )
    }

    /// Generate lib.rs allow/deny attributes.
    pub fn generate_lint_attributes(&self, level: LintLevel) -> String {
        match level {
            LintLevel::Strict => {
                r#"#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
"#
            }
            LintLevel::Normal => {
                r#"#![warn(clippy::all)]
#![allow(clippy::needless_return)]
"#
            }
            LintLevel::Relaxed => {
                r#"#![allow(clippy::all)]
"#
            }
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub enum LintLevel {
    Strict,
    Normal,
    Relaxed,
}

/// Common Clippy lint categories and their fixes.
pub struct ClippyGuide;

impl ClippyGuide {
    /// Get the fix for a common Clippy lint.
    pub fn lint_fix(lint_name: &str) -> Option<&'static str> {
        match lint_name {
            "clippy::needless_return" => Some("Remove the explicit `return` keyword at the end of a function"),
            "clippy::redundant_closure" => Some("Use the function directly instead of wrapping it in a closure"),
            "clippy::manual_map" => Some("Use `.map()` instead of `match` with `Option`"),
            "clippy::single_match" => Some("Use `if let` instead of `match` with a single arm"),
            "clippy::unnecessary_unwrap" => Some("Use `if let` or `match` instead of unwrap after is_some check"),
            "clippy::let_and_return" => Some("Return the expression directly instead of binding to a variable"),
            "clippy::needless_borrow" => Some("Remove unnecessary `&` borrow"),
            "clippy::clone_on_copy" => Some("Use `.clone()` explicitly or dereference for Copy types"),
            "clippy::map_entry" => Some("Use `entry()` API instead of `contains_key` + `insert`"),
            "clippy::manual_flatten" => Some("Use `.flatten()` instead of nested `if let`"),
            _ => None,
        }
    }

    /// Get all documented lints.
    pub fn all_lints() -> Vec<(&'static str, &'static str)> {
        vec![
            ("clippy::needless_return", "Remove explicit return"),
            ("clippy::redundant_closure", "Simplify closure"),
            ("clippy::manual_map", "Use Option::map"),
            ("clippy::single_match", "Use if-let"),
            ("clippy::unnecessary_unwrap", "Avoid unwrap after check"),
            ("clippy::let_and_return", "Direct return"),
            ("clippy::needless_borrow", "Remove unnecessary borrow"),
            ("clippy::clone_on_copy", "Explicit clone or deref"),
            ("clippy::map_entry", "Use entry API"),
            ("clippy::manual_flatten", "Use flatten"),
        ]
    }
}

/// Code quality check result.
#[derive(Debug, Clone)]
pub struct LintViolation {
    pub lint: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub suggestion: Option<String>,
    pub is_fixable: bool,
}

impl LintViolation {
    pub fn new(lint: &str, line: usize, message: &str) -> Self {
        Self {
            lint: lint.into(),
            line,
            column: 0,
            message: message.into(),
            suggestion: None,
            is_fixable: false,
        }
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.into());
        self.is_fixable = true;
        self
    }
}

/// Lint report generator.
pub struct LintReport {
    violations: Vec<LintViolation>,
    warnings: usize,
    errors: usize,
}

impl LintReport {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            warnings: 0,
            errors: 0,
        }
    }

    pub fn add_warning(&mut self, violation: LintViolation) {
        self.warnings += 1;
        self.violations.push(violation);
    }

    pub fn add_error(&mut self, violation: LintViolation) {
        self.errors += 1;
        self.violations.push(violation);
    }

    pub fn total(&self) -> usize {
        self.warnings + self.errors
    }

    pub fn is_clean(&self) -> bool {
        self.total() == 0
    }

    pub fn report(&self) -> String {
        let mut output = format!(
            "Lint Report: {} warnings, {} errors\n\n",
            self.warnings, self.errors
        );
        for violation in &self.violations {
            output.push_str(&format!(
                "  L{}: [{}] {}\n",
                violation.line, violation.lint, violation.message
            ));
            if let Some(ref suggestion) = violation.suggestion {
                output.push_str(&format!("    Suggestion: {}\n", suggestion));
            }
        }
        output
    }

    pub fn fixable_count(&self) -> usize {
        self.violations.iter().filter(|v| v.is_fixable).count()
    }
}

/// CI lint configuration for different stages.
#[derive(Debug, Clone)]
pub struct CiLintConfig {
    pub check_format: bool,
    pub run_clippy: bool,
    pub clippy_args: Vec<String>,
    pub deny_warnings: bool,
    pub check_docs: bool,
}

impl CiLintConfig {
    /// PR check configuration (strict).
    pub fn pr_check() -> Self {
        Self {
            check_format: true,
            run_clippy: true,
            clippy_args: vec![
                "--all-targets".into(),
                "--all-features".into(),
                "--".into(),
                "-D".into(),
                "warnings".into(),
            ],
            deny_warnings: true,
            check_docs: true,
        }
    }

    /// Nightly check configuration (pedantic).
    pub fn nightly_check() -> Self {
        Self {
            check_format: true,
            run_clippy: true,
            clippy_args: vec![
                "--all-targets".into(),
                "--all-features".into(),
                "--".into(),
                "-W".into(),
                "clippy::pedantic".into(),
            ],
            deny_warnings: false,
            check_docs: true,
        }
    }

    pub fn commands(&self) -> Vec<String> {
        let mut cmds = Vec::new();
        if self.check_format {
            cmds.push("cargo fmt --all -- --check".into());
        }
        if self.run_clippy {
            cmds.push(format!("cargo clippy {}", self.clippy_args.join(" ")));
        }
        if self.check_docs {
            cmds.push("cargo doc --no-deps --all-features".into());
        }
        cmds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clippy_config_default() {
        let config = ClippyConfig::default();
        assert_eq!(config.too_many_arguments_threshold, 7);
        assert_eq!(config.cognitive_complexity_threshold, 25);
    }

    #[test]
    fn test_clippy_config_toml() {
        let config = ClippyConfig::default();
        let toml = config.to_toml();
        assert!(toml.contains("too-many-arguments-threshold = 7"));
        assert!(toml.contains("cognitive-complexity-threshold = 25"));
    }

    #[test]
    fn test_clippy_config_lint_attributes() {
        let config = ClippyConfig::default();

        let strict = config.generate_lint_attributes(LintLevel::Strict);
        assert!(strict.contains("deny(clippy::all)"));
        assert!(strict.contains("warn(clippy::pedantic)"));

        let normal = config.generate_lint_attributes(LintLevel::Normal);
        assert!(normal.contains("warn(clippy::all)"));

        let relaxed = config.generate_lint_attributes(LintLevel::Relaxed);
        assert!(relaxed.contains("allow(clippy::all)"));
    }

    #[test]
    fn test_clippy_guide_lint_fix() {
        assert!(ClippyGuide::lint_fix("clippy::needless_return").is_some());
        assert!(ClippyGuide::lint_fix("clippy::redundant_closure").is_some());
        assert!(ClippyGuide::lint_fix("nonexistent_lint").is_none());
    }

    #[test]
    fn test_clippy_guide_all_lints() {
        let lints = ClippyGuide::all_lints();
        assert!(!lints.is_empty());
        assert!(lints.iter().any(|(name, _)| *name == "clippy::needless_return"));
    }

    #[test]
    fn test_lint_violation() {
        let violation = LintViolation::new("clippy::needless_return", 42, "unnecessary return")
            .with_suggestion("remove 'return' keyword");

        assert!(violation.is_fixable);
        assert!(violation.suggestion.is_some());
    }

    #[test]
    fn test_lint_report() {
        let mut report = LintReport::new();
        report.add_warning(LintViolation::new("clippy::style", 10, "style issue"));
        report.add_error(LintViolation::new("clippy::correctness", 20, "bug"));

        assert_eq!(report.total(), 2);
        assert_eq!(report.warnings, 1);
        assert_eq!(report.errors, 1);
        assert!(!report.is_clean());
    }

    #[test]
    fn test_lint_report_clean() {
        let report = LintReport::new();
        assert!(report.is_clean());
        assert_eq!(report.total(), 0);
    }

    #[test]
    fn test_lint_report_fixable_count() {
        let mut report = LintReport::new();
        report.add_warning(
            LintViolation::new("clippy::style", 1, "fixable").with_suggestion("fix"),
        );
        report.add_warning(LintViolation::new("clippy::complex", 2, "not fixable"));

        assert_eq!(report.fixable_count(), 1);
    }

    #[test]
    fn test_lint_report_display() {
        let mut report = LintReport::new();
        report.add_warning(LintViolation::new("test_lint", 42, "test message"));

        let text = report.report();
        assert!(text.contains("warnings"));
        assert!(text.contains("L42"));
        assert!(text.contains("test_lint"));
    }

    #[test]
    fn test_ci_lint_config_pr_check() {
        let config = CiLintConfig::pr_check();
        assert!(config.check_format);
        assert!(config.run_clippy);
        assert!(config.deny_warnings);
    }

    #[test]
    fn test_ci_lint_config_nightly() {
        let config = CiLintConfig::nightly_check();
        assert!(config.run_clippy);
        assert!(!config.deny_warnings);
    }

    #[test]
    fn test_ci_lint_config_commands() {
        let config = CiLintConfig::pr_check();
        let cmds = config.commands();
        assert!(cmds.iter().any(|c| c.contains("cargo fmt")));
        assert!(cmds.iter().any(|c| c.contains("cargo clippy")));
        assert!(cmds.iter().any(|c| c.contains("cargo doc")));
    }
}
