//! # Code Review Practices
//!
//! Effective code review catches bugs, ensures consistency, and shares knowledge.
//! This module covers review checklists, common findings, and automation.
//!
//! ## Review Checklist:
//!
//! - [ ] Error handling is comprehensive
//! - [ ] No unwrap() in library code
//! - [ ] Public API is documented
//! - [ ] Tests cover edge cases
//! - [ ] No unnecessary clones
//! - [ ] Naming follows conventions
//! - [ ] Complexity is reasonable

use std::collections::HashMap;

/// Code review checklist item.
#[derive(Debug, Clone)]
pub struct ChecklistItem {
    pub category: String,
    pub description: String,
    pub severity: Severity,
    pub checked: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Critical,
    Major,
    Minor,
    Suggestion,
}

/// Code review checklist builder.
pub struct ReviewChecklist {
    items: Vec<ChecklistItem>,
}

impl ReviewChecklist {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn with_defaults() -> Self {
        let mut checklist = Self::new();

        checklist.add_item("Error Handling", "No unwrap() in library code", Severity::Critical);
        checklist.add_item("Error Handling", "All Results are handled", Severity::Critical);
        checklist.add_item("Safety", "No unnecessary unsafe blocks", Severity::Critical);
        checklist.add_item("Documentation", "Public API has doc comments", Severity::Major);
        checklist.add_item("Documentation", "Complex logic is commented", Severity::Minor);
        checklist.add_item("Testing", "Edge cases are tested", Severity::Major);
        checklist.add_item("Testing", "Error paths are tested", Severity::Major);
        checklist.add_item("Performance", "No unnecessary allocations", Severity::Minor);
        checklist.add_item("Performance", "No unnecessary clones", Severity::Minor);
        checklist.add_item("Style", "Naming follows conventions", Severity::Minor);
        checklist.add_item("Style", "Functions are short", Severity::Suggestion);
        checklist.add_item("API Design", "Public types are Send + Sync", Severity::Major);
        checklist.add_item("API Design", "Builder pattern used for complex types", Severity::Suggestion);

        checklist
    }

    pub fn add_item(&mut self, category: &str, description: &str, severity: Severity) {
        self.items.push(ChecklistItem {
            category: category.into(),
            description: description.into(),
            severity,
            checked: false,
        });
    }

    pub fn check(&mut self, index: usize) {
        if let Some(item) = self.items.get_mut(index) {
            item.checked = true;
        }
    }

    pub fn unchecked_items(&self) -> Vec<&ChecklistItem> {
        self.items.iter().filter(|i| !i.checked).collect()
    }

    pub fn critical_unchecked(&self) -> Vec<&ChecklistItem> {
        self.items
            .iter()
            .filter(|i| !i.checked && i.severity == Severity::Critical)
            .collect()
    }

    pub fn report(&self) -> String {
        let total = self.items.len();
        let checked = self.items.iter().filter(|i| i.checked).count();
        let mut output = format!("Review Checklist: {}/{} checked\n\n", checked, total);

        for item in &self.items {
            let status = if item.checked { "[x]" } else { "[ ]" };
            output.push_str(&format!(
                "  {} [{:?}] {}: {}\n",
                status, item.severity, item.category, item.description
            ));
        }
        output
    }
}

/// Common code review findings in Rust.
pub struct CommonFindings;

impl CommonFindings {
    /// Get a description of a common finding.
    pub fn describe(finding: &str) -> Option<&'static str> {
        match finding {
            "unwrap" => Some("unwrap() can panic in production. Use ? operator or proper error handling."),
            "clone" => Some("Unnecessary clone(). Consider using references or Cow<T>."),
            "unsafe" => Some("Unsafe code requires careful review. Document safety invariants."),
            "todo" => Some("TODO/FIXME markers should be tracked as issues."),
            "magic_number" => Some("Magic numbers should be named constants."),
            "long_function" => Some("Functions over 50 lines should be split."),
            "deep_nesting" => Some("Deep nesting (>3 levels) reduces readability."),
            "missing_docs" => Some("Public items should have documentation."),
            "missing_tests" => Some("New functionality should have tests."),
            "panicking_code" => Some("Library code should not panic unexpectedly."),
            _ => None,
        }
    }

    pub fn all_findings() -> Vec<(&'static str, &'static str)> {
        vec![
            ("unwrap", "unwrap() in library code"),
            ("clone", "Unnecessary clone"),
            ("unsafe", "Unsafe code without documentation"),
            ("todo", "TODO/FIXME markers"),
            ("magic_number", "Magic numbers"),
            ("long_function", "Functions too long"),
            ("deep_nesting", "Deep nesting"),
            ("missing_docs", "Missing documentation"),
            ("missing_tests", "Missing tests"),
            ("panicking_code", "Panicking library code"),
        ]
    }
}

/// Review finding with location information.
#[derive(Debug, Clone)]
pub struct ReviewFinding {
    pub file: String,
    pub line: usize,
    pub category: String,
    pub description: String,
    pub severity: Severity,
    pub suggestion: Option<String>,
}

impl ReviewFinding {
    pub fn new(file: &str, line: usize, category: &str, description: &str, severity: Severity) -> Self {
        Self {
            file: file.into(),
            line,
            category: category.into(),
            description: description.into(),
            severity,
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Review report aggregator.
pub struct ReviewReport {
    findings: Vec<ReviewFinding>,
}

impl ReviewReport {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
        }
    }

    pub fn add_finding(&mut self, finding: ReviewFinding) {
        self.findings.push(finding);
    }

    pub fn critical_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Critical)
            .count()
    }

    pub fn is_approvable(&self) -> bool {
        self.critical_count() == 0
    }

    pub fn summary(&self) -> String {
        let critical = self.critical_count();
        let major = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Major)
            .count();
        let minor = self
            .findings
            .iter()
            .filter(|f| f.severity == Severity::Minor)
            .count();

        format!(
            "Review: {} critical, {} major, {} minor issues",
            critical, major, minor
        )
    }

    pub fn report(&self) -> String {
        let mut output = format!("{}\n\n", self.summary());
        for finding in &self.findings {
            output.push_str(&format!(
                "  {} L{}: [{}] {}\n",
                finding.file, finding.line, finding.category, finding.description
            ));
            if let Some(ref suggestion) = finding.suggestion {
                output.push_str(&format!("    Suggestion: {}\n", suggestion));
            }
        }
        output
    }

    pub fn findings_by_file(&self) -> HashMap<&str, Vec<&ReviewFinding>> {
        let mut by_file: HashMap<&str, Vec<&ReviewFinding>> = HashMap::new();
        for finding in &self.findings {
            by_file
                .entry(&finding.file)
                .or_default()
                .push(finding);
        }
        by_file
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_checklist_defaults() {
        let checklist = ReviewChecklist::with_defaults();
        assert!(!checklist.items.is_empty());
        assert!(!checklist.critical_unchecked().is_empty());
    }

    #[test]
    fn test_review_checklist_check() {
        let mut checklist = ReviewChecklist::with_defaults();
        checklist.check(0);
        assert_eq!(checklist.unchecked_items().len(), checklist.items.len() - 1);
    }

    #[test]
    fn test_review_checklist_report() {
        let mut checklist = ReviewChecklist::new();
        checklist.add_item("Test", "Test item", Severity::Major);
        checklist.check(0);

        let report = checklist.report();
        assert!(report.contains("[x]"));
        assert!(report.contains("1/1"));
    }

    #[test]
    fn test_common_findings() {
        assert!(CommonFindings::describe("unwrap").is_some());
        assert!(CommonFindings::describe("clone").is_some());
        assert!(CommonFindings::describe("nonexistent").is_none());
    }

    #[test]
    fn test_common_findings_all() {
        let findings = CommonFindings::all_findings();
        assert!(!findings.is_empty());
        assert!(findings.iter().any(|(name, _)| *name == "unwrap"));
    }

    #[test]
    fn test_review_finding() {
        let finding = ReviewFinding::new("main.rs", 42, "unwrap", "unwrap() called", Severity::Critical)
            .with_suggestion("Use ? operator");

        assert!(finding.suggestion.is_some());
        assert_eq!(finding.severity, Severity::Critical);
    }

    #[test]
    fn test_review_report() {
        let mut report = ReviewReport::new();
        report.add_finding(ReviewFinding::new("a.rs", 1, "test", "issue1", Severity::Critical));
        report.add_finding(ReviewFinding::new("a.rs", 2, "test", "issue2", Severity::Major));
        report.add_finding(ReviewFinding::new("b.rs", 1, "test", "issue3", Severity::Minor));

        assert_eq!(report.critical_count(), 1);
        assert!(!report.is_approvable());
        assert!(report.summary().contains("1 critical"));
    }

    #[test]
    fn test_review_report_approvable() {
        let mut report = ReviewReport::new();
        report.add_finding(ReviewFinding::new("a.rs", 1, "style", "minor", Severity::Minor));
        assert!(report.is_approvable());
    }

    #[test]
    fn test_review_report_by_file() {
        let mut report = ReviewReport::new();
        report.add_finding(ReviewFinding::new("a.rs", 1, "test", "i1", Severity::Minor));
        report.add_finding(ReviewFinding::new("b.rs", 1, "test", "i2", Severity::Minor));
        report.add_finding(ReviewFinding::new("a.rs", 2, "test", "i3", Severity::Minor));

        let by_file = report.findings_by_file();
        assert_eq!(by_file.get("a.rs").unwrap().len(), 2);
        assert_eq!(by_file.get("b.rs").unwrap().len(), 1);
    }

    #[test]
    fn test_review_report_display() {
        let mut report = ReviewReport::new();
        report.add_finding(
            ReviewFinding::new("main.rs", 10, "error", "test issue", Severity::Major)
                .with_suggestion("fix it"),
        );

        let text = report.report();
        assert!(text.contains("main.rs"));
        assert!(text.contains("L10"));
        assert!(text.contains("fix it"));
    }
}
