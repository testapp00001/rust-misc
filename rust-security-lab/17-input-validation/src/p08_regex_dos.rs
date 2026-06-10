//! # Lesson 08: ReDoS (Regular Expression Denial of Service)
//!
//! ## The Problem
//!
//! Regular expressions with certain patterns can cause catastrophic
//! backtracking, where the regex engine's execution time grows
//! exponentially with the input length. An attacker can exploit this
//! by crafting input that triggers worst-case behavior, causing the
//! server to hang.
//!
//! ## Vulnerable Patterns
//!
//! ### 1. Overlapping Alternatives
//! ```ignore
//! // VULNERABLE: (a+)+ matches "aaaa..." with exponential backtracking
//! let re = Regex::new(r"^(a+)+$").unwrap();
//! re.is_match("aaaaaaaaaaaaaaaaX"); // Takes ~2^17 steps!
//! ```
//!
//! The `+` inside the group and the `+` outside create overlapping
//! quantifiers. For "aaa", the engine tries every possible way to
//! split the input between the inner and outer quantifiers.
//!
//! ### 2. Overlapping Character Classes
//! ```ignore
//! // VULNERABLE: (a|a)+ has the same problem
//! let re = Regex::new(r"^(a|a)+$").unwrap();
//! ```
//!
//! ### 3. Star-height > 1
//! ```ignore
//! // VULNERABLE: Nested quantifiers with overlapping matches
//! let re = Regex::new(r"(a+)*b").unwrap();
//! ```
//!
//! ## Safe Patterns
//!
//! Regexes are safe when:
//! - Quantifiers don't overlap (each position can only match one way)
//! - No nested quantifiers on groups that can match the same characters
//! - The regex engine's work is linear in the input length
//!
//! ## Defense
//!
//! 1. **Audit regexes** for nested/overlapping quantifiers
//! 2. **Set timeouts** on regex execution
//! 3. **Use bounded repetition** instead of `*` or `+`
//! 4. **Test with adversarial input** before deploying

use regex::Regex;
use std::time::{Duration, Instant};

/// Analyze a regex pattern for potential ReDoS vulnerability.
///
/// Checks for:
/// - Nested quantifiers: `(a+)+`, `(a*)*`, `(a+)*`, `(a*)+`
/// - Overlapping alternatives: `(a|a)+`, `(a|a*)+`
/// - Star-height > 1 (quantifier applied to a group containing a quantifier)
///
/// Returns a list of vulnerability descriptions found.
/// Returns an empty Vec if the pattern appears safe.
///
/// Note: This is a heuristic analysis -- it checks for known dangerous
/// patterns but cannot guarantee safety for all possible regexes.
pub fn analyze_regex(pattern: &str) -> Vec<String> {
    todo!("Analyze regex pattern for ReDoS vulnerabilities")
}

/// Test if a regex is vulnerable to ReDoS by timing its execution
/// against adversarial input.
///
/// Generates increasingly long adversarial strings and measures execution
/// time. If time grows exponentially (more than 10x for each doubling of
/// input length), the regex is likely vulnerable.
///
/// Returns:
/// - Ok(true) if the regex appears vulnerable
/// - Ok(false) if execution time grows linearly
/// - Err(message) if the regex pattern is invalid
pub fn test_regex_timing(pattern: &str) -> Result<bool, String> {
    todo!("Test regex timing against adversarial input")
}

/// Execute a regex with a timeout.
///
/// Returns:
/// - Some(matches) if the regex completed within the timeout
/// - None if the regex took longer than `timeout`
///
/// Note: Rust's regex crate doesn't support timeouts natively, so this
/// runs the regex in a thread and checks timing after. For this exercise,
/// implement a simple wrapper that checks elapsed time.
///
/// In production, consider using the `regex-automata` crate which supports
/// bounded execution, or the `fancy-regex` crate with backtracking limits.
pub fn regex_with_timeout(pattern: &str, input: &str, timeout: Duration) -> Option<bool> {
    todo!("Execute regex with a timeout guard")
}

/// Validate that a user-provided regex pattern is safe to compile and use.
///
/// Rejects patterns that:
/// - Are invalid regex syntax
/// - Contain nested quantifiers
/// - Contain overlapping alternatives
/// - Are excessively long (> 1000 characters)
///
/// Returns Ok(Regex) if safe, Err(message) if dangerous or invalid.
pub fn compile_safe_regex(pattern: &str) -> Result<Regex, String> {
    todo!("Validate and compile a safe regex")
}

/// Generate an adversarial input string for a given vulnerable pattern.
///
/// For nested quantifier patterns like `(a+)+`, returns a string of
/// 'a's followed by a character that forces the engine to backtrack
/// through all combinations.
///
/// The length parameter controls how many 'a' characters to generate.
/// Even small values (20-25) can cause multi-second backtracking.
pub fn generate_adversarial_input(length: usize) -> String {
    todo!("Generate adversarial input for ReDoS testing")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_analyze_nested_quantifier() {
        let issues = analyze_regex(r"^(a+)+$");
        assert!(!issues.is_empty(), "Should detect nested quantifier");
    }

    #[test]
    fn test_analyze_safe_pattern() {
        let issues = analyze_regex(r"^a+$");
        assert!(issues.is_empty(), "Simple pattern should be safe");
    }

    #[test]
    fn test_analyze_overlapping_alternatives() {
        let issues = analyze_regex(r"^(a|a)+$");
        assert!(!issues.is_empty(), "Should detect overlapping alternatives");
    }

    #[test]
    fn test_compile_safe_valid() {
        let result = compile_safe_regex(r"^\d{3}-\d{4}$");
        assert!(result.is_ok());
    }

    #[test]
    fn test_compile_safe_rejects_nested_quantifiers() {
        let result = compile_safe_regex(r"^(a+)+$");
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_safe_rejects_invalid() {
        let result = compile_safe_regex(r"^(unclosed");
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_safe_rejects_too_long() {
        let long_pattern = "a".repeat(1001);
        let result = compile_safe_regex(&long_pattern);
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_adversarial() {
        let input = generate_adversarial_input(20);
        assert_eq!(input.len(), 21); // 20 'a's + 'X'
        assert!(input.ends_with('X'));
    }
}
