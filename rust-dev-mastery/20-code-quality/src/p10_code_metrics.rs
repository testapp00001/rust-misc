//! # Code Metrics
//!
//! Code metrics help quantify code quality and complexity. This module covers
//! cyclomatic complexity, lines of code, dependency analysis, and tooling.
//!
//! ## Key Metrics:
//!
//! | Metric | Good | Warning | Bad |
//! |--------|------|---------|-----|
//! | Cyclomatic complexity | < 10 | 10-20 | > 20 |
//! | Function length | < 30 | 30-50 | > 50 |
//! | Module length | < 300 | 300-500 | > 500 |
//! | Test coverage | > 80% | 60-80% | < 60% |
//!
//! ## Tools:
//!
//! - `cargo-deny`: Dependency analysis
//! - `cargo-udeps`: Unused dependency detection
//! - `cargo-audit`: Security vulnerability scanning
//! - `cargo-tarpaulin`: Code coverage

use std::collections::HashMap;

/// Cyclomatic complexity calculator.
pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    /// Calculate cyclomatic complexity of a function.
    /// Each decision point adds 1 to complexity.
    pub fn cyclomatic_complexity(code: &str) -> usize {
        let mut complexity = 1; // Base complexity

        for line in code.lines() {
            let trimmed = line.trim();

            // Decision points
            if trimmed.starts_with("if ") || trimmed.starts_with("if(") {
                complexity += 1;
            }
            if trimmed.starts_with("else if") {
                complexity += 1;
            }
            if trimmed.starts_with("match ") {
                complexity += 1;
            }
            // Match arms (count additional arms)
            if trimmed.starts_with('|') || trimmed.starts_with(',') {
                complexity += 1;
            }
            // Loops
            if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
                complexity += 1;
            }
            // Pattern matching with |
            if trimmed.contains(" | ") && trimmed.contains("=>") {
                complexity += 1;
            }
            // Boolean operators
            complexity += trimmed.matches(" && ").count();
            complexity += trimmed.matches(" || ").count();
            // ? operator (error handling)
            complexity += trimmed.matches('?').count();
        }

        complexity
    }

    /// Get a rating for the complexity.
    pub fn complexity_rating(complexity: usize) -> ComplexityRating {
        match complexity {
            0..=5 => ComplexityRating::Simple,
            6..=10 => ComplexityRating::Moderate,
            11..=20 => ComplexityRating::Complex,
            _ => ComplexityRating::VeryComplex,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ComplexityRating {
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Lines of code metrics.
pub struct LocMetrics {
    pub total_lines: usize,
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
}

impl LocMetrics {
    pub fn analyze(code: &str) -> Self {
        let mut total = 0;
        let mut code_count = 0;
        let mut comment_count = 0;
        let mut blank_count = 0;

        for line in code.lines() {
            total += 1;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                blank_count += 1;
            } else if trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("//!") {
                comment_count += 1;
            } else {
                code_count += 1;
            }
        }

        Self {
            total_lines: total,
            code_lines: code_count,
            comment_lines: comment_count,
            blank_lines: blank_count,
        }
    }

    pub fn comment_ratio(&self) -> f64 {
        if self.code_lines == 0 {
            return 0.0;
        }
        self.comment_lines as f64 / self.code_lines as f64
    }
}

/// Dependency analysis.
pub struct DependencyAnalyzer {
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub is_direct: bool,
    pub license: Option<String>,
    pub has_vulnerabilities: bool,
    pub is_unused: bool,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
        }
    }

    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }

    pub fn vulnerable_count(&self) -> usize {
        self.dependencies.iter().filter(|d| d.has_vulnerabilities).count()
    }

    pub fn unused_count(&self) -> usize {
        self.dependencies.iter().filter(|d| d.is_unused).count()
    }

    pub fn direct_count(&self) -> usize {
        self.dependencies.iter().filter(|d| d.is_direct).count()
    }

    pub fn report(&self) -> String {
        format!(
            "Dependencies: {} total, {} direct, {} vulnerable, {} unused",
            self.dependencies.len(),
            self.direct_count(),
            self.vulnerable_count(),
            self.unused_count(),
        )
    }
}

/// MSRV (Minimum Supported Rust Version) tracker.
pub struct MsrvTracker {
    pub current_msrv: String,
    pub features_used: Vec<MsrvFeature>,
}

#[derive(Debug, Clone)]
pub struct MsrvFeature {
    pub name: String,
    pub since_version: String,
}

impl MsrvTracker {
    pub fn new(msrv: &str) -> Self {
        Self {
            current_msrv: msrv.into(),
            features_used: Vec::new(),
        }
    }

    pub fn add_feature(&mut self, name: &str, since: &str) {
        self.features_used.push(MsrvFeature {
            name: name.into(),
            since_version: since.into(),
        });
    }

    /// Check if all features are available at the MSRV.
    pub fn is_compatible(&self) -> bool {
        self.features_used.iter().all(|f| {
            self.version_gte(&self.current_msrv, &f.since_version)
        })
    }

    fn version_gte(&self, a: &str, b: &str) -> bool {
        let parse = |v: &str| -> (u32, u32, u32) {
            let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();
            (
                parts.get(0).copied().unwrap_or(0),
                parts.get(1).copied().unwrap_or(0),
                parts.get(2).copied().unwrap_or(0),
            )
        };
        parse(a) >= parse(b)
    }

    pub fn report(&self) -> String {
        let compatible = if self.is_compatible() {
            "Compatible"
        } else {
            "Incompatible"
        };
        format!(
            "MSRV: {} ({}) - {} features tracked",
            self.current_msrv,
            compatible,
            self.features_used.len()
        )
    }
}

/// Code quality score calculator.
pub struct QualityScore {
    pub complexity_score: f64,
    pub documentation_score: f64,
    pub test_coverage_score: f64,
    pub dependency_score: f64,
}

impl QualityScore {
    pub fn overall(&self) -> f64 {
        (self.complexity_score * 0.3)
            + (self.documentation_score * 0.25)
            + (self.test_coverage_score * 0.25)
            + (self.dependency_score * 0.2)
    }

    pub fn grade(&self) -> char {
        let score = self.overall();
        if score >= 90.0 {
            'A'
        } else if score >= 80.0 {
            'B'
        } else if score >= 70.0 {
            'C'
        } else if score >= 60.0 {
            'D'
        } else {
            'F'
        }
    }
}

/// cargo-deny configuration generator.
pub struct DenyConfig;

impl DenyConfig {
    pub fn generate() -> String {
        r#"[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "warn"

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
]

[bans]
multiple-versions = "warn"
wildcards = "deny"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyclomatic_complexity_simple() {
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let complexity = ComplexityAnalyzer::cyclomatic_complexity(code);
        assert_eq!(complexity, 1); // Base only
    }

    #[test]
    fn test_cyclomatic_complexity_with_if() {
        let code = r#"
fn check(x: i32) -> bool {
    if x > 0 {
        true
    } else {
        false
    }
}"#;
        let complexity = ComplexityAnalyzer::cyclomatic_complexity(code);
        assert!(complexity >= 2);
    }

    #[test]
    fn test_cyclomatic_complexity_with_match() {
        let code = r#"
fn classify(x: i32) -> &'static str {
    match x {
        0 => "zero",
        1..=10 => "small",
        _ => "large",
    }
}"#;
        let complexity = ComplexityAnalyzer::cyclomatic_complexity(code);
        assert!(complexity >= 2);
    }

    #[test]
    fn test_complexity_rating() {
        assert_eq!(ComplexityAnalyzer::complexity_rating(3), ComplexityRating::Simple);
        assert_eq!(ComplexityAnalyzer::complexity_rating(8), ComplexityRating::Moderate);
        assert_eq!(ComplexityAnalyzer::complexity_rating(15), ComplexityRating::Complex);
        assert_eq!(ComplexityAnalyzer::complexity_rating(25), ComplexityRating::VeryComplex);
    }

    #[test]
    fn test_loc_metrics() {
        let code = "// comment\nfn main() {\n\n    println!(\"hello\");\n}\n";
        let metrics = LocMetrics::analyze(code);
        assert_eq!(metrics.total_lines, 5);
        assert_eq!(metrics.comment_lines, 1);
        assert_eq!(metrics.blank_lines, 1);
        assert_eq!(metrics.code_lines, 3);
    }

    #[test]
    fn test_loc_comment_ratio() {
        let code = "// comment 1\n// comment 2\nfn main() {}";
        let metrics = LocMetrics::analyze(code);
        assert!((metrics.comment_ratio() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dependency_analyzer() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_dependency(Dependency {
            name: "serde".into(),
            version: "1.0".into(),
            is_direct: true,
            license: Some("MIT".into()),
            has_vulnerabilities: false,
            is_unused: false,
        });
        analyzer.add_dependency(Dependency {
            name: "vulnerable-crate".into(),
            version: "0.1".into(),
            is_direct: true,
            license: None,
            has_vulnerabilities: true,
            is_unused: false,
        });

        assert_eq!(analyzer.direct_count(), 2);
        assert_eq!(analyzer.vulnerable_count(), 1);
    }

    #[test]
    fn test_dependency_report() {
        let mut analyzer = DependencyAnalyzer::new();
        analyzer.add_dependency(Dependency {
            name: "test".into(),
            version: "1.0".into(),
            is_direct: true,
            license: None,
            has_vulnerabilities: false,
            is_unused: true,
        });

        let report = analyzer.report();
        assert!(report.contains("unused"));
    }

    #[test]
    fn test_msrv_tracker() {
        let mut tracker = MsrvTracker::new("1.70.0");
        tracker.add_feature("let-else", "1.65.0");
        tracker.add_feature("async-fn-in-trait", "1.75.0");

        assert!(!tracker.is_compatible()); // 1.70 < 1.75
    }

    #[test]
    fn test_msrv_tracker_compatible() {
        let mut tracker = MsrvTracker::new("1.80.0");
        tracker.add_feature("let-else", "1.65.0");
        assert!(tracker.is_compatible());
    }

    #[test]
    fn test_msrv_report() {
        let tracker = MsrvTracker::new("1.70.0");
        let report = tracker.report();
        assert!(report.contains("1.70.0"));
        assert!(report.contains("Compatible"));
    }

    #[test]
    fn test_quality_score() {
        let score = QualityScore {
            complexity_score: 90.0,
            documentation_score: 80.0,
            test_coverage_score: 85.0,
            dependency_score: 95.0,
        };
        assert!((score.overall() - 87.0).abs() < 1.0);
        assert_eq!(score.grade(), 'B');
    }

    #[test]
    fn test_quality_score_grade_a() {
        let score = QualityScore {
            complexity_score: 95.0,
            documentation_score: 95.0,
            test_coverage_score: 95.0,
            dependency_score: 95.0,
        };
        assert_eq!(score.grade(), 'A');
    }

    #[test]
    fn test_deny_config() {
        let config = DenyConfig::generate();
        assert!(config.contains("[advisories]"));
        assert!(config.contains("[licenses]"));
        assert!(config.contains("MIT"));
    }

    #[test]
    fn test_loc_empty() {
        let metrics = LocMetrics::analyze("");
        assert_eq!(metrics.total_lines, 0);
        assert_eq!(metrics.comment_ratio(), 0.0);
    }
}
