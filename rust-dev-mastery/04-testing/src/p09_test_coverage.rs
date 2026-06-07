//! # Lesson 9: Test Coverage
//!
//! Test coverage measures how much code is exercised by tests.
//! This lesson covers coverage tools, coverage reports, and CI integration.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Coverage report model
// ---------------------------------------------------------------------------

/// A coverage report for a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCoverage {
    pub path: String,
    pub total_lines: usize,
    pub covered_lines: usize,
    pub total_branches: usize,
    pub covered_branches: usize,
    pub total_functions: usize,
    pub covered_functions: usize,
}

impl FileCoverage {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
            total_lines: 0,
            covered_lines: 0,
            total_branches: 0,
            covered_branches: 0,
            total_functions: 0,
            covered_functions: 0,
        }
    }

    pub fn line_coverage_percent(&self) -> f64 {
        if self.total_lines == 0 {
            return 100.0;
        }
        (self.covered_lines as f64 / self.total_lines as f64) * 100.0
    }

    pub fn branch_coverage_percent(&self) -> f64 {
        if self.total_branches == 0 {
            return 100.0;
        }
        (self.covered_branches as f64 / self.total_branches as f64) * 100.0
    }

    pub fn function_coverage_percent(&self) -> f64 {
        if self.total_functions == 0 {
            return 100.0;
        }
        (self.covered_functions as f64 / self.total_functions as f64) * 100.0
    }
}

/// A complete coverage report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub files: Vec<FileCoverage>,
    pub tool: String,
    pub timestamp: String,
}

impl CoverageReport {
    pub fn new(tool: &str) -> Self {
        Self {
            files: Vec::new(),
            tool: tool.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    pub fn add_file(&mut self, file: FileCoverage) {
        self.files.push(file);
    }

    pub fn total_lines(&self) -> usize {
        self.files.iter().map(|f| f.total_lines).sum()
    }

    pub fn total_covered_lines(&self) -> usize {
        self.files.iter().map(|f| f.covered_lines).sum()
    }

    pub fn overall_line_coverage(&self) -> f64 {
        let total = self.total_lines();
        if total == 0 {
            return 100.0;
        }
        (self.total_covered_lines() as f64 / total as f64) * 100.0
    }

    pub fn overall_branch_coverage(&self) -> f64 {
        let total: usize = self.files.iter().map(|f| f.total_branches).sum();
        if total == 0 {
            return 100.0;
        }
        let covered: usize = self.files.iter().map(|f| f.covered_branches).sum();
        (covered as f64 / total as f64) * 100.0
    }

    pub fn function_coverage_percent(&self) -> f64 {
        let total: usize = self.files.iter().map(|f| f.total_functions).sum();
        if total == 0 {
            return 100.0;
        }
        let covered: usize = self.files.iter().map(|f| f.covered_functions).sum();
        (covered as f64 / total as f64) * 100.0
    }

    /// Get files below a coverage threshold.
    pub fn files_below_threshold(&self, threshold: f64) -> Vec<&FileCoverage> {
        self.files
            .iter()
            .filter(|f| f.line_coverage_percent() < threshold)
            .collect()
    }

    /// Generate a text summary.
    pub fn summary(&self) -> String {
        format!(
            "Coverage Report ({})\n\
             Files: {}\n\
             Line coverage: {:.1}% ({}/{})\n\
             Branch coverage: {:.1}%\n\
             Function coverage: {:.1}%\n\
             Files below 80%: {}",
            self.tool,
            self.files.len(),
            self.overall_line_coverage(),
            self.total_covered_lines(),
            self.total_lines(),
            self.overall_branch_coverage(),
            self.function_coverage_percent(),
            self.files_below_threshold(80.0).len()
        )
    }

    /// Generate a text-based coverage bar.
    pub fn coverage_bar(&self) -> String {
        let percent = self.overall_line_coverage();
        let filled = (percent / 2.5) as usize;
        let empty = 40 - filled;
        format!(
            "[{}{}] {:.1}%",
            "#".repeat(filled),
            ".".repeat(empty),
            percent
        )
    }
}

// ---------------------------------------------------------------------------
// Coverage CI integration
// ---------------------------------------------------------------------------

/// Configuration for coverage in CI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageConfig {
    pub tool: CoverageTool,
    pub minimum_line_coverage: f64,
    pub minimum_branch_coverage: f64,
    pub fail_on_regression: bool,
    pub regression_threshold: f64,
    pub exclude_patterns: Vec<String>,
    pub report_format: ReportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoverageTool {
    Tarpaulin,
    LlvmCov,
    Codecov,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Html,
    Lcov,
    Cobertura,
    Json,
}

impl CoverageConfig {
    pub fn default_config() -> Self {
        Self {
            tool: CoverageTool::Tarpaulin,
            minimum_line_coverage: 80.0,
            minimum_branch_coverage: 70.0,
            fail_on_regression: true,
            regression_threshold: 1.0,
            exclude_patterns: vec![
                "tests/*".into(),
                "benches/*".into(),
                "examples/*".into(),
            ],
            report_format: ReportFormat::Lcov,
        }
    }

    /// Check if a coverage report passes the configured thresholds.
    pub fn check(&self, report: &CoverageReport) -> CoverageCheckResult {
        let mut issues = Vec::new();

        let line_cov = report.overall_line_coverage();
        if line_cov < self.minimum_line_coverage {
            issues.push(format!(
                "Line coverage {:.1}% below minimum {:.1}%",
                line_cov, self.minimum_line_coverage
            ));
        }

        let branch_cov = report.overall_branch_coverage();
        if branch_cov < self.minimum_branch_coverage {
            issues.push(format!(
                "Branch coverage {:.1}% below minimum {:.1}%",
                branch_cov, self.minimum_branch_coverage
            ));
        }

        CoverageCheckResult {
            passed: issues.is_empty(),
            issues,
            line_coverage: line_cov,
            branch_coverage: branch_cov,
        }
    }
}

#[derive(Debug)]
pub struct CoverageCheckResult {
    pub passed: bool,
    pub issues: Vec<String>,
    pub line_coverage: f64,
    pub branch_coverage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_coverage_percentages() {
        let mut file = FileCoverage::new("src/lib.rs");
        file.total_lines = 100;
        file.covered_lines = 80;
        file.total_branches = 20;
        file.covered_branches = 15;
        file.total_functions = 10;
        file.covered_functions = 9;

        assert!((file.line_coverage_percent() - 80.0).abs() < 0.01);
        assert!((file.branch_coverage_percent() - 75.0).abs() < 0.01);
        assert!((file.function_coverage_percent() - 90.0).abs() < 0.01);
    }

    #[test]
    fn test_file_coverage_zero_lines() {
        let file = FileCoverage::new("empty.rs");
        assert_eq!(file.line_coverage_percent(), 100.0);
    }

    #[test]
    fn test_coverage_report() {
        let mut report = CoverageReport::new("tarpaulin");

        let mut file1 = FileCoverage::new("src/lib.rs");
        file1.total_lines = 100;
        file1.covered_lines = 90;
        report.add_file(file1);

        let mut file2 = FileCoverage::new("src/main.rs");
        file2.total_lines = 50;
        file2.covered_lines = 30;
        report.add_file(file2);

        assert_eq!(report.total_lines(), 150);
        assert_eq!(report.total_covered_lines(), 120);
        assert!((report.overall_line_coverage() - 80.0).abs() < 0.01);
    }

    #[test]
    fn test_coverage_report_empty() {
        let report = CoverageReport::new("tarpaulin");
        assert_eq!(report.overall_line_coverage(), 100.0);
    }

    #[test]
    fn test_files_below_threshold() {
        let mut report = CoverageReport::new("tarpaulin");

        let mut good = FileCoverage::new("good.rs");
        good.total_lines = 100;
        good.covered_lines = 90;
        report.add_file(good);

        let mut bad = FileCoverage::new("bad.rs");
        bad.total_lines = 100;
        bad.covered_lines = 50;
        report.add_file(bad);

        let below = report.files_below_threshold(80.0);
        assert_eq!(below.len(), 1);
        assert_eq!(below[0].path, "bad.rs");
    }

    #[test]
    fn test_coverage_summary() {
        let mut report = CoverageReport::new("tarpaulin");
        let mut file = FileCoverage::new("src/lib.rs");
        file.total_lines = 100;
        file.covered_lines = 85;
        file.total_branches = 20;
        file.covered_branches = 16;
        file.total_functions = 10;
        file.covered_functions = 10;
        report.add_file(file);

        let summary = report.summary();
        assert!(summary.contains("tarpaulin"));
        assert!(summary.contains("85.0%"));
    }

    #[test]
    fn test_coverage_bar() {
        let mut report = CoverageReport::new("test");
        let mut file = FileCoverage::new("test.rs");
        file.total_lines = 100;
        file.covered_lines = 80;
        report.add_file(file);

        let bar = report.coverage_bar();
        assert!(bar.contains("["));
        assert!(bar.contains("]"));
        assert!(bar.contains("80.0%"));
    }

    #[test]
    fn test_coverage_config_check_pass() {
        let config = CoverageConfig {
            minimum_line_coverage: 80.0,
            minimum_branch_coverage: 70.0,
            ..CoverageConfig::default_config()
        };

        let mut report = CoverageReport::new("test");
        let mut file = FileCoverage::new("test.rs");
        file.total_lines = 100;
        file.covered_lines = 85;
        file.total_branches = 20;
        file.covered_branches = 16;
        report.add_file(file);

        let result = config.check(&report);
        assert!(result.passed);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_coverage_config_check_fail() {
        let config = CoverageConfig {
            minimum_line_coverage: 90.0,
            minimum_branch_coverage: 80.0,
            ..CoverageConfig::default_config()
        };

        let mut report = CoverageReport::new("test");
        let mut file = FileCoverage::new("test.rs");
        file.total_lines = 100;
        file.covered_lines = 85;
        file.total_branches = 20;
        file.covered_branches = 14;
        report.add_file(file);

        let result = config.check(&report);
        assert!(!result.passed);
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_coverage_config_default() {
        let config = CoverageConfig::default_config();
        assert_eq!(config.minimum_line_coverage, 80.0);
        assert_eq!(config.minimum_branch_coverage, 70.0);
        assert!(config.fail_on_regression);
        assert!(!config.exclude_patterns.is_empty());
    }

    #[test]
    fn test_overall_branch_coverage() {
        let mut report = CoverageReport::new("test");

        let mut f1 = FileCoverage::new("a.rs");
        f1.total_branches = 10;
        f1.covered_branches = 8;
        report.add_file(f1);

        let mut f2 = FileCoverage::new("b.rs");
        f2.total_branches = 10;
        f2.covered_branches = 6;
        report.add_file(f2);

        assert!((report.overall_branch_coverage() - 70.0).abs() < 0.01);
    }

    #[test]
    fn test_overall_branch_coverage_empty() {
        let report = CoverageReport::new("test");
        assert_eq!(report.overall_branch_coverage(), 100.0);
    }
}
