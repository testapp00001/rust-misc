//! # Comment Philosophy
//!
//! Good comments explain *why*, not *what*. This module covers when and how
//! to write effective comments in Rust code.
//!
//! ## Principles:
//!
//! 1. **Self-documenting code first**: Clear naming reduces need for comments
//! 2. **Why, not what**: Explain intent, not mechanics
//! 3. **Doc comments for public API**: `///` for rustdoc
//! 4. **Inline comments for complexity**: Explain non-obvious logic
//! 5. **No commented-out code**: Use version control instead

/// Comment quality analyzer.
pub struct CommentAnalyzer;

impl CommentAnalyzer {
    /// Classify a comment by its quality.
    pub fn classify(comment: &str) -> CommentQuality {
        let trimmed = comment.trim();

        if trimmed.is_empty() {
            return CommentQuality::Empty;
        }

        // Strip leading comment markers (//, ///, //!, /*) for pattern matching.
        let content = trimmed
            .trim_start_matches('/')
            .trim_start_matches('!')
            .trim_start_matches('*')
            .trim();

        if content.is_empty() {
            return CommentQuality::Empty;
        }

        // Good patterns: explains why
        if content.starts_with("Because")
            || content.starts_with("We need")
            || content.starts_with("This is needed")
            || content.starts_with("Note:")
            || content.starts_with("Safety:")
            || content.starts_with("Invariant:")
        {
            return CommentQuality::Good;
        }

        // Bad patterns: explains what (obvious from code)
        if content.starts_with("Increment")
            || content.starts_with("Set")
            || content.starts_with("Get")
            || content.starts_with("Return")
            || content.starts_with("Loop")
        {
            return CommentQuality::Redundant;
        }

        // Commented-out code
        if content.starts_with("fn ")
            || content.starts_with("let ")
            || content.starts_with("if ")
            || content.starts_with("return ")
        {
            return CommentQuality::CommentedCode;
        }

        CommentQuality::Acceptable
    }

    /// Check if a line needs a comment.
    pub fn needs_comment(code: &str) -> bool {
        let trimmed = code.trim();

        // Complex expressions that benefit from explanation
        if trimmed.contains("unsafe") {
            return true;
        }
        if trimmed.len() > 100 {
            return true;
        }
        // Bitwise operations
        if trimmed.contains("<<") || trimmed.contains(">>") || trimmed.contains('^') {
            return true;
        }
        false
    }
}

#[derive(Debug, PartialEq)]
pub enum CommentQuality {
    Good,
    Acceptable,
    Redundant,
    CommentedCode,
    Empty,
}

/// Comment conventions for different contexts.
pub struct CommentConventions;

impl CommentConventions {
    /// Generate a safety comment for unsafe code.
    pub fn safety_comment(reason: &str) -> String {
        format!("// Safety: {}", reason)
    }

    /// Generate an invariant comment.
    pub fn invariant_comment(description: &str) -> String {
        format!("// Invariant: {}", description)
    }

    /// Generate a performance note.
    pub fn performance_note(note: &str) -> String {
        format!("// Performance: {}", note)
    }

    /// Generate a TODO comment with tracking.
    pub fn todo_comment(issue: &str, description: &str) -> String {
        format!("// TODO({}): {}", issue, description)
    }

    /// Generate a FIXME comment.
    pub fn fixme_comment(description: &str) -> String {
        format!("// FIXME: {}", description)
    }
}

/// API documentation conventions.
pub struct ApiDocConventions;

impl ApiDocConventions {
    /// Check if a doc comment follows conventions.
    pub fn validate_doc(doc: &str) -> Vec<DocIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = doc.lines().collect();

        if lines.is_empty() {
            issues.push(DocIssue::EmptyDoc);
            return issues;
        }

        // First line should be a summary (short)
        if let Some(first) = lines.first() {
            if first.len() > 80 {
                issues.push(DocIssue::SummaryTooLong);
            }
            if !first.ends_with('.') {
                issues.push(DocIssue::MissingPeriod);
            }
        }

        // Should have a blank line after summary
        if lines.len() > 1 && !lines[1].trim().is_empty() {
            issues.push(DocIssue::MissingBlankLine);
        }

        issues
    }
}

#[derive(Debug, PartialEq)]
pub enum DocIssue {
    EmptyDoc,
    SummaryTooLong,
    MissingPeriod,
    MissingBlankLine,
}

/// Examples should be testable.
pub fn add_with_docs(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_quality_good() {
        assert_eq!(
            CommentAnalyzer::classify("// Because this is critical for safety"),
            CommentQuality::Good
        );
        assert_eq!(
            CommentAnalyzer::classify("// Safety: pointer is valid"),
            CommentQuality::Good
        );
    }

    #[test]
    fn test_comment_quality_redundant() {
        assert_eq!(
            CommentAnalyzer::classify("// Increment counter"),
            CommentQuality::Redundant
        );
        assert_eq!(
            CommentAnalyzer::classify("// Return the value"),
            CommentQuality::Redundant
        );
    }

    #[test]
    fn test_comment_quality_commented_code() {
        assert_eq!(
            CommentAnalyzer::classify("// fn old_function() {}"),
            CommentQuality::CommentedCode
        );
    }

    #[test]
    fn test_comment_quality_acceptable() {
        assert_eq!(
            CommentAnalyzer::classify("// This handles the edge case"),
            CommentQuality::Acceptable
        );
    }

    #[test]
    fn test_comment_quality_empty() {
        assert_eq!(CommentAnalyzer::classify("//"), CommentQuality::Empty);
    }

    #[test]
    fn test_needs_comment() {
        assert!(CommentAnalyzer::needs_comment("unsafe { ptr.read() }"));
        assert!(CommentAnalyzer::needs_comment("let x = a << 3 | b >> 1;"));
        assert!(!CommentAnalyzer::needs_comment("let x = 1;"));
    }

    #[test]
    fn test_safety_comment() {
        let comment = CommentConventions::safety_comment("pointer is aligned");
        assert!(comment.contains("Safety:"));
        assert!(comment.contains("aligned"));
    }

    #[test]
    fn test_invariant_comment() {
        let comment = CommentConventions::invariant_comment("len <= capacity");
        assert!(comment.contains("Invariant:"));
    }

    #[test]
    fn test_todo_comment() {
        let comment = CommentConventions::todo_comment("ISSUE-123", "optimize this");
        assert!(comment.contains("TODO(ISSUE-123)"));
        assert!(comment.contains("optimize this"));
    }

    #[test]
    fn test_fixme_comment() {
        let comment = CommentConventions::fixme_comment("potential overflow");
        assert!(comment.contains("FIXME:"));
        assert!(comment.contains("overflow"));
    }

    #[test]
    fn test_api_doc_validate_good() {
        let doc = "Adds two numbers.\n\nReturns the sum.";
        let issues = ApiDocConventions::validate_doc(doc);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_api_doc_validate_empty() {
        let issues = ApiDocConventions::validate_doc("");
        assert!(issues.contains(&DocIssue::EmptyDoc));
    }

    #[test]
    fn test_api_doc_validate_long_summary() {
        let doc = "A".repeat(100);
        let issues = ApiDocConventions::validate_doc(&doc);
        assert!(issues.contains(&DocIssue::SummaryTooLong));
    }

    #[test]
    fn test_api_doc_validate_missing_period() {
        let doc = "Adds two numbers\n\nReturns the sum.";
        let issues = ApiDocConventions::validate_doc(doc);
        assert!(issues.contains(&DocIssue::MissingPeriod));
    }

    #[test]
    fn test_performance_note() {
        let note = CommentConventions::performance_note("O(n log n) complexity");
        assert!(note.contains("Performance:"));
    }

    #[test]
    fn test_add_with_docs() {
        assert_eq!(add_with_docs(2, 3), 5);
    }
}
