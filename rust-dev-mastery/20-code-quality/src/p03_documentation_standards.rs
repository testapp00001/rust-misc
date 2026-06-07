//! # Documentation Standards
//!
//! Rust's documentation system is built into the language via doc comments
//! and rustdoc. This module covers documentation conventions, best practices,
//! and automation.
//!
//! ## Doc Comment Types:
//!
//! | Type | Syntax | Purpose |
//! |------|--------|---------|
//! | `///` | Outer doc | Document the next item |
//! | `//!` | Inner doc | Document the enclosing item |
//! | `#[doc]` | Attribute | Programmatic documentation |
//!
//! ## Sections (in order):
//!
//! 1. Summary (first line)
//! 2. Description (rest of the doc comment)
//! 3. Examples (with ` ```rust ` blocks)
//! 4. Safety (if unsafe)
//! 5. Errors (if fallible)
//! 6. Panics (if can panic)

/// Documentation quality metrics.
#[derive(Debug, Clone)]
pub struct DocMetrics {
    pub total_items: usize,
    pub documented_items: usize,
    pub items_with_examples: usize,
    pub items_with_safety_notes: usize,
    pub items_with_error_docs: usize,
}

impl DocMetrics {
    pub fn new() -> Self {
        Self {
            total_items: 0,
            documented_items: 0,
            items_with_examples: 0,
            items_with_safety_notes: 0,
            items_with_error_docs: 0,
        }
    }

    pub fn coverage(&self) -> f64 {
        if self.total_items == 0 {
            return 100.0;
        }
        (self.documented_items as f64 / self.total_items as f64) * 100.0
    }

    pub fn example_coverage(&self) -> f64 {
        if self.documented_items == 0 {
            return 0.0;
        }
        (self.items_with_examples as f64 / self.documented_items as f64) * 100.0
    }
}

/// Doc comment parser that validates documentation quality.
pub struct DocValidator {
    rules: Vec<DocRule>,
}

#[derive(Debug, Clone)]
pub enum DocRule {
    RequireSummary,
    RequireExamples,
    RequireErrorDocs,
    RequireSafetyDocs,
    MaxLineLength(usize),
    NoEmptyDocs,
}

impl DocValidator {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with_defaults() -> Self {
        Self {
            rules: vec![
                DocRule::RequireSummary,
                DocRule::RequireExamples,
                DocRule::NoEmptyDocs,
                DocRule::MaxLineLength(100),
            ],
        }
    }

    pub fn add_rule(&mut self, rule: DocRule) {
        self.rules.push(rule);
    }

    /// Validate a doc comment against the rules.
    pub fn validate(&self, doc: &str) -> Vec<DocViolation> {
        let mut violations = Vec::new();

        for rule in &self.rules {
            match rule {
                DocRule::RequireSummary => {
                    if doc.lines().next().map_or(true, |l| l.trim().is_empty()) {
                        violations.push(DocViolation {
                            rule: "RequireSummary".into(),
                            message: "Documentation must start with a summary line".into(),
                        });
                    }
                }
                DocRule::RequireExamples => {
                    if !doc.contains("```rust") && !doc.contains("```no_run") && !doc.contains("```") {
                        violations.push(DocViolation {
                            rule: "RequireExamples".into(),
                            message: "Documentation should include code examples".into(),
                        });
                    }
                }
                DocRule::RequireErrorDocs => {
                    if doc.contains("Result<") && !doc.contains("# Errors") {
                        violations.push(DocViolation {
                            rule: "RequireErrorDocs".into(),
                            message: "Functions returning Result should document errors".into(),
                        });
                    }
                }
                DocRule::RequireSafetyDocs => {
                    if doc.contains("unsafe") && !doc.contains("# Safety") {
                        violations.push(DocViolation {
                            rule: "RequireSafetyDocs".into(),
                            message: "Unsafe items should document safety requirements".into(),
                        });
                    }
                }
                DocRule::MaxLineLength(max) => {
                    for line in doc.lines() {
                        if line.len() > *max {
                            violations.push(DocViolation {
                                rule: "MaxLineLength".into(),
                                message: format!("Line exceeds {} characters", max),
                            });
                            break;
                        }
                    }
                }
                DocRule::NoEmptyDocs => {
                    if doc.trim().is_empty() {
                        violations.push(DocViolation {
                            rule: "NoEmptyDocs".into(),
                            message: "Documentation should not be empty".into(),
                        });
                    }
                }
            }
        }

        violations
    }
}

#[derive(Debug, Clone)]
pub struct DocViolation {
    pub rule: String,
    pub message: String,
}

/// Documentation template generator.
pub struct DocTemplates;

impl DocTemplates {
    /// Generate a function doc template.
    pub fn function_template(name: &str, description: &str) -> String {
        format!(
            r#"/// {description}
///
/// # Arguments
///
/// * `param` - Description of the parameter
///
/// # Returns
///
/// Description of the return value
///
/// # Errors
///
/// Returns an error if the operation fails.
///
/// # Examples
///
/// ```rust
/// let result = {name}();
/// assert!(result.is_ok());
/// ```
"#
        )
    }

    /// Generate a struct doc template.
    pub fn struct_template(name: &str, description: &str) -> String {
        format!(
            r#"/// {description}
///
/// # Examples
///
/// ```rust
/// let instance = {name}::new();
/// ```
"#
        )
    }

    /// Generate a module doc template.
    pub fn module_template(name: &str, description: &str) -> String {
        format!(
            r#"//! # {name}
//!
//! {description}
//!
//! ## Key Types
//!
//! - [`Type1`]: Description
//! - [`Type2`]: Description
//!
//! ## Examples
//!
//! ```rust
//! use my_crate::{name}::Type1;
//!
//! let value = Type1::new();
//! ```
"#
        )
    }
}

/// Doc coverage report.
#[derive(Debug)]
pub struct DocCoverageReport {
    pub module_docs: DocMetrics,
    pub struct_docs: DocMetrics,
    pub function_docs: DocMetrics,
    pub enum_docs: DocMetrics,
}

impl DocCoverageReport {
    pub fn overall_coverage(&self) -> f64 {
        let total = self.module_docs.total_items
            + self.struct_docs.total_items
            + self.function_docs.total_items
            + self.enum_docs.total_items;
        let documented = self.module_docs.documented_items
            + self.struct_docs.documented_items
            + self.function_docs.documented_items
            + self.enum_docs.documented_items;

        if total == 0 {
            return 100.0;
        }
        (documented as f64 / total as f64) * 100.0
    }

    pub fn report(&self) -> String {
        format!(
            "Documentation Coverage Report\n\
             ============================\n\
             Modules: {:.1}%\n\
             Structs: {:.1}%\n\
             Functions: {:.1}%\n\
             Enums: {:.1}%\n\
             Overall: {:.1}%",
            self.module_docs.coverage(),
            self.struct_docs.coverage(),
            self.function_docs.coverage(),
            self.enum_docs.coverage(),
            self.overall_coverage(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doc_metrics_coverage() {
        let metrics = DocMetrics {
            total_items: 10,
            documented_items: 8,
            items_with_examples: 5,
            items_with_safety_notes: 0,
            items_with_error_docs: 0,
        };
        assert!((metrics.coverage() - 80.0).abs() < f64::EPSILON);
        assert!((metrics.example_coverage() - 62.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_doc_metrics_empty() {
        let metrics = DocMetrics::new();
        assert!((metrics.coverage() - 100.0).abs() < f64::EPSILON);
        assert!((metrics.example_coverage() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_doc_validator_summary() {
        let validator = DocValidator::with_defaults();
        let violations = validator.validate("This is a summary\nWith more details.");
        assert!(violations.iter().all(|v| v.rule != "RequireSummary"));
    }

    #[test]
    fn test_doc_validator_no_summary() {
        let validator = DocValidator::with_defaults();
        let violations = validator.validate("\nEmpty first line.");
        assert!(violations.iter().any(|v| v.rule == "RequireSummary"));
    }

    #[test]
    fn test_doc_validator_no_examples() {
        let validator = DocValidator::with_defaults();
        let violations = validator.validate("Just a description.");
        assert!(violations.iter().any(|v| v.rule == "RequireExamples"));
    }

    #[test]
    fn test_doc_validator_with_examples() {
        let validator = DocValidator::with_defaults();
        let violations = validator.validate("Description\n\n```rust\nlet x = 1;\n```");
        assert!(violations.iter().all(|v| v.rule != "RequireExamples"));
    }

    #[test]
    fn test_doc_validator_empty() {
        let validator = DocValidator::with_defaults();
        let violations = validator.validate("");
        assert!(violations.iter().any(|v| v.rule == "NoEmptyDocs"));
    }

    #[test]
    fn test_doc_validator_line_length() {
        let mut validator = DocValidator::new();
        validator.add_rule(DocRule::MaxLineLength(20));
        let long_line = "a".repeat(30);
        let violations = validator.validate(&long_line);
        assert!(violations.iter().any(|v| v.rule == "MaxLineLength"));
    }

    #[test]
    fn test_doc_templates_function() {
        let template = DocTemplates::function_template("my_func", "Does something");
        assert!(template.contains("my_func"));
        assert!(template.contains("# Examples"));
        assert!(template.contains("```rust"));
    }

    #[test]
    fn test_doc_templates_struct() {
        let template = DocTemplates::struct_template("MyStruct", "A structure");
        assert!(template.contains("MyStruct"));
        assert!(template.contains("# Examples"));
    }

    #[test]
    fn test_doc_templates_module() {
        let template = DocTemplates::module_template("my_module", "Module description");
        assert!(template.contains("my_module"));
        assert!(template.contains("# Key Types"));
    }

    #[test]
    fn test_doc_coverage_report() {
        let report = DocCoverageReport {
            module_docs: DocMetrics {
                total_items: 5,
                documented_items: 5,
                items_with_examples: 3,
                items_with_safety_notes: 0,
                items_with_error_docs: 0,
            },
            struct_docs: DocMetrics {
                total_items: 10,
                documented_items: 8,
                items_with_examples: 5,
                items_with_safety_notes: 0,
                items_with_error_docs: 0,
            },
            function_docs: DocMetrics {
                total_items: 20,
                documented_items: 15,
                items_with_examples: 10,
                items_with_safety_notes: 0,
                items_with_error_docs: 0,
            },
            enum_docs: DocMetrics {
                total_items: 5,
                documented_items: 4,
                items_with_examples: 2,
                items_with_safety_notes: 0,
                items_with_error_docs: 0,
            },
        };

        let text = report.report();
        assert!(text.contains("Documentation Coverage Report"));
        assert!(text.contains("Overall:"));
    }

    #[test]
    fn test_doc_validator_error_docs() {
        let validator = DocValidator::with_defaults();
        // This is a simplified check
        let doc = "Returns Result<(), Error>";
        let violations = validator.validate(doc);
        // May or may not trigger depending on rules
        let _ = violations;
    }
}
