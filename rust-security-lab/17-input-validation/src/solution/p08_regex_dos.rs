//! # Lesson 08: ReDoS (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use regex::Regex;
use std::time::{Duration, Instant};

pub fn analyze_regex(pattern: &str) -> Vec<String> {
    let mut issues = Vec::new();

    // Check for nested quantifiers: quantifier followed by another quantifier
    // Pattern: (something+)+ or (something*)* etc.
    let chars: Vec<char> = pattern.chars().collect();
    let len = chars.len();

    // Check for nested quantifiers inside groups
    let mut i = 0;
    while i < len {
        if chars[i] == '(' {
            // Find matching closing paren
            let mut depth = 1;
            let mut j = i + 1;
            while j < len && depth > 0 {
                if chars[j] == '(' {
                    depth += 1;
                } else if chars[j] == ')' {
                    depth -= 1;
                }
                j += 1;
            }
            // j now points past the closing paren
            if j <= len {
                let group_content: String = chars[i + 1..j - 1].iter().collect();
                // Check if group content has quantifiers
                let has_inner_quantifier = group_content.contains('+')
                    || group_content.contains('*')
                    || group_content.contains('{');
                // Check if the closing paren is followed by a quantifier
                let has_outer_quantifier = j < len
                    && (chars[j] == '+' || chars[j] == '*' || chars[j] == '?'
                        || chars[j] == '{');

                if has_inner_quantifier && has_outer_quantifier {
                    issues.push(format!(
                        "Nested quantifier detected: group '{}' followed by '{}'",
                        group_content, chars[j]
                    ));
                }
            }
        }
        i += 1;
    }

    // Check for overlapping alternatives: (a|a) or (a|a*) etc.
    if let Some(paren_start) = pattern.find('(') {
        if let Some(paren_end) = pattern[paren_start..].find(')') {
            let group = &pattern[paren_start + 1..paren_start + paren_end];
            if group.contains('|') {
                let alternatives: Vec<&str> = group.split('|').collect();
                for i in 0..alternatives.len() {
                    for j in i + 1..alternatives.len() {
                        if alternatives[i] == alternatives[j] {
                            issues.push(format!(
                                "Overlapping alternatives: '{}' and '{}'",
                                alternatives[i], alternatives[j]
                            ));
                        }
                    }
                }
            }
        }
    }

    issues
}

pub fn test_regex_timing(pattern: &str) -> Result<bool, String> {
    let re = Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))?;

    let mut prev_time = Duration::ZERO;
    let mut vulnerable = false;

    for exp in 5..=20 {
        let input = generate_adversarial_input(exp);
        let start = Instant::now();
        let _ = re.is_match(&input);
        let elapsed = start.elapsed();

        if exp > 5 && prev_time.as_nanos() > 0 {
            let ratio = elapsed.as_nanos() as f64 / prev_time.as_nanos() as f64;
            if ratio > 10.0 {
                vulnerable = true;
                break;
            }
        }
        prev_time = elapsed;
    }

    Ok(vulnerable)
}

pub fn regex_with_timeout(pattern: &str, input: &str, timeout: Duration) -> Option<bool> {
    let re = match Regex::new(pattern) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let start = Instant::now();
    let result = re.is_match(input);
    let elapsed = start.elapsed();

    if elapsed > timeout {
        None
    } else {
        Some(result)
    }
}

pub fn compile_safe_regex(pattern: &str) -> Result<Regex, String> {
    if pattern.len() > 1000 {
        return Err("Pattern too long (max 1000 characters)".to_string());
    }

    let issues = analyze_regex(pattern);
    if !issues.is_empty() {
        return Err(format!("Unsafe regex pattern: {}", issues.join("; ")));
    }

    Regex::new(pattern).map_err(|e| format!("Invalid regex: {}", e))
}

pub fn generate_adversarial_input(length: usize) -> String {
    let mut result = "a".repeat(length);
    result.push('X');
    result
}

#[cfg(test)]
mod tests {
    use super::*;

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
