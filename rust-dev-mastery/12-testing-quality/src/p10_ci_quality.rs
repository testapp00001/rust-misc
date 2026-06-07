//! # CI Quality & Code Quality Tools
//!
//! Continuous integration ensures code quality through automated checks.
//! This lesson covers the tools and configurations for maintaining
//! high code quality in Rust projects.

use std::collections::HashMap;

/// Demonstrates a code quality checker that validates various rules.
pub struct QualityChecker {
    rules: Vec<Box<dyn Rule>>,
}

pub trait Rule {
    fn name(&self) -> &str;
    fn check(&self, code: &str) -> QualityResult;
}

pub struct QualityResult {
    pub rule: String,
    pub passed: bool,
    pub message: String,
}

impl QualityChecker {
    pub fn new() -> Self {
        QualityChecker { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: Box<dyn Rule>) {
        self.rules.push(rule);
    }

    pub fn check_all(&self, code: &str) -> Vec<QualityResult> {
        self.rules.iter().map(|rule| rule.check(code)).collect()
    }

    pub fn all_passed(&self, code: &str) -> bool {
        self.check_all(code).iter().all(|r| r.passed)
    }
}

/// Rule: No TODO comments in production code.
pub struct NoTodoRule;

impl Rule for NoTodoRule {
    fn name(&self) -> &str {
        "no_todo"
    }

    fn check(&self, code: &str) -> QualityResult {
        let has_todo = code.lines().any(|line| line.contains("TODO"));
        QualityResult {
            rule: self.name().to_string(),
            passed: !has_todo,
            message: if has_todo {
                "Found TODO comments".to_string()
            } else {
                "No TODO comments found".to_string()
            },
        }
    }
}

/// Rule: Maximum line length.
pub struct MaxLineLengthRule {
    pub max_length: usize,
}

impl Rule for MaxLineLengthRule {
    fn name(&self) -> &str {
        "max_line_length"
    }

    fn check(&self, code: &str) -> QualityResult {
        let violations: Vec<usize> = code
            .lines()
            .enumerate()
            .filter(|(_, line)| line.len() > self.max_length)
            .map(|(i, _)| i + 1)
            .collect();

        QualityResult {
            rule: self.name().to_string(),
            passed: violations.is_empty(),
            message: if violations.is_empty() {
                "All lines within limit".to_string()
            } else {
                format!("Lines exceeding {} chars: {:?}", self.max_length, violations)
            },
        }
    }
}

/// Rule: Functions should have doc comments.
pub struct DocCommentRule;

impl Rule for DocCommentRule {
    fn name(&self) -> &str {
        "doc_comments"
    }

    fn check(&self, code: &str) -> QualityResult {
        let pub_fns: usize = code.lines().filter(|line| line.contains("pub fn ")).count();
        let doc_comments: usize = code
            .lines()
            .filter(|line| line.trim().starts_with("///"))
            .count();

        QualityResult {
            rule: self.name().to_string(),
            passed: doc_comments >= pub_fns,
            message: format!("{pub_fns} pub fns, {doc_comments} doc comments"),
        }
    }
}

/// Rule: No unwrap() in production code.
pub struct NoUnwrapRule;

impl Rule for NoUnwrapRule {
    fn name(&self) -> &str {
        "no_unwrap"
    }

    fn check(&self, code: &str) -> QualityResult {
        let has_unwrap = code
            .lines()
            .any(|line| line.contains(".unwrap()") && !line.trim().starts_with("//"));

        QualityResult {
            rule: self.name().to_string(),
            passed: !has_unwrap,
            message: if has_unwrap {
                "Found .unwrap() calls".to_string()
            } else {
                "No .unwrap() found".to_string()
            },
        }
    }
}

/// Rule: Module-level documentation.
pub struct ModuleDocRule;

impl Rule for ModuleDocRule {
    fn name(&self) -> &str {
        "module_doc"
    }

    fn check(&self, code: &str) -> QualityResult {
        let has_module_doc = code.starts_with("//!") || code.contains("\n//!");

        QualityResult {
            rule: self.name().to_string(),
            passed: has_module_doc,
            message: if has_module_doc {
                "Module has documentation".to_string()
            } else {
                "Missing module documentation".to_string()
            },
        }
    }
}

/// Demonstrates a lint configuration.
#[derive(Debug, Clone)]
pub struct LintConfig {
    pub max_line_length: usize,
    pub max_function_lines: usize,
    pub max_complexity: usize,
    pub require_docs: bool,
    pub deny_unwrap: bool,
}

impl Default for LintConfig {
    fn default() -> Self {
        LintConfig {
            max_line_length: 100,
            max_function_lines: 50,
            max_complexity: 10,
            require_docs: true,
            deny_unwrap: true,
        }
    }
}

/// Demonstrates a coverage report structure.
#[derive(Debug)]
pub struct CoverageReport {
    pub total_lines: usize,
    pub covered_lines: usize,
    pub total_functions: usize,
    pub covered_functions: usize,
    pub total_branches: usize,
    pub covered_branches: usize,
}

impl CoverageReport {
    pub fn line_coverage(&self) -> f64 {
        if self.total_lines == 0 {
            return 100.0;
        }
        (self.covered_lines as f64 / self.total_lines as f64) * 100.0
    }

    pub fn function_coverage(&self) -> f64 {
        if self.total_functions == 0 {
            return 100.0;
        }
        (self.covered_functions as f64 / self.total_functions as f64) * 100.0
    }

    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.line_coverage() >= threshold
    }
}

/// Demonstrates a test report structure.
#[derive(Debug)]
pub struct TestReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub duration_ms: u64,
}

impl TestReport {
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.passed as f64 / self.total as f64) * 100.0
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

/// Demonstrates a CI configuration representation.
#[derive(Debug)]
pub struct CiConfig {
    pub rust_version: String,
    pub run_tests: bool,
    pub run_clippy: bool,
    pub run_fmt_check: bool,
    pub run_doc_tests: bool,
    pub min_coverage: f64,
    pub features: Vec<String>,
}

impl Default for CiConfig {
    fn default() -> Self {
        CiConfig {
            rust_version: "stable".to_string(),
            run_tests: true,
            run_clippy: true,
            run_fmt_check: true,
            run_doc_tests: true,
            min_coverage: 80.0,
            features: Vec::new(),
        }
    }
}

impl CiConfig {
    pub fn to_github_actions(&self) -> String {
        let mut yaml = String::from("name: CI\non: [push, pull_request]\njobs:\n  test:\n");
        yaml.push_str("    runs-on: ubuntu-latest\n");
        yaml.push_str(&format!("    steps:\n      - uses: actions/checkout@v4\n"));
        yaml.push_str(&format!(
            "      - uses: dtolnay/rust-toolchain@{}\n",
            self.rust_version
        ));

        if self.run_fmt_check {
            yaml.push_str("      - run: cargo fmt -- --check\n");
        }
        if self.run_clippy {
            yaml.push_str("      - run: cargo clippy -- -D warnings\n");
        }
        if self.run_tests {
            yaml.push_str("      - run: cargo test\n");
        }
        if self.run_doc_tests {
            yaml.push_str("      - run: cargo test --doc\n");
        }

        yaml
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_checker_no_rules() {
        let checker = QualityChecker::new();
        assert!(checker.all_passed("any code"));
    }

    #[test]
    fn test_no_todo_rule() {
        let rule = NoTodoRule;
        assert!(rule.check("fn foo() {}").passed);
        assert!(!rule.check("// TODO: fix this").passed);
    }

    #[test]
    fn test_max_line_length() {
        let rule = MaxLineLengthRule { max_length: 80 };
        assert!(rule.check("short line").passed);
        assert!(!rule.check(&"x".repeat(100)).passed);
    }

    #[test]
    fn test_doc_comment_rule() {
        let rule = DocCommentRule;
        assert!(rule.check("/// Doc\npub fn foo() {}").passed);
        assert!(!rule.check("pub fn foo() {}").passed);
    }

    #[test]
    fn test_no_unwrap_rule() {
        let rule = NoUnwrapRule;
        assert!(rule.check("let x = foo().unwrap_or(0);").passed);
        assert!(!rule.check("let x = foo().unwrap();").passed);
    }

    #[test]
    fn test_module_doc_rule() {
        let rule = ModuleDocRule;
        assert!(rule.check("//! Module doc\npub fn foo() {}").passed);
        assert!(!rule.check("pub fn foo() {}").passed);
    }

    #[test]
    fn test_coverage_report() {
        let report = CoverageReport {
            total_lines: 100,
            covered_lines: 85,
            total_functions: 20,
            covered_functions: 18,
            total_branches: 50,
            covered_branches: 40,
        };
        assert!((report.line_coverage() - 85.0).abs() < 0.01);
        assert!((report.function_coverage() - 90.0).abs() < 0.01);
        assert!(report.meets_threshold(80.0));
        assert!(!report.meets_threshold(90.0));
    }

    #[test]
    fn test_test_report() {
        let report = TestReport {
            total: 100,
            passed: 95,
            failed: 3,
            ignored: 2,
            duration_ms: 5000,
        };
        assert!((report.pass_rate() - 95.0).abs() < 0.01);
        assert!(!report.all_passed());
    }

    #[test]
    fn test_test_report_all_passed() {
        let report = TestReport {
            total: 50,
            passed: 48,
            failed: 0,
            ignored: 2,
            duration_ms: 1000,
        };
        assert!(report.all_passed());
    }

    #[test]
    fn test_ci_config_default() {
        let config = CiConfig::default();
        assert!(config.run_tests);
        assert!(config.run_clippy);
        assert_eq!(config.rust_version, "stable");
    }

    #[test]
    fn test_ci_config_github_actions() {
        let config = CiConfig::default();
        let yaml = config.to_github_actions();
        assert!(yaml.contains("cargo test"));
        assert!(yaml.contains("cargo clippy"));
        assert!(yaml.contains("cargo fmt"));
    }

    #[test]
    fn test_lint_config_default() {
        let config = LintConfig::default();
        assert_eq!(config.max_line_length, 100);
        assert!(config.require_docs);
        assert!(config.deny_unwrap);
    }
}
