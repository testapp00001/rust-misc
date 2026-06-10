//! # Lesson 10: Input Sanitization Patterns (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use regex::Regex;

pub fn sanitize_input(input: &str, max_length: usize) -> String {
    // Step 1: Trim whitespace
    let trimmed = input.trim();

    // Step 2: Remove null bytes
    let no_nulls: String = trimmed.chars().filter(|c| *c != '\0').collect();

    // Step 3: Normalize whitespace (collapse multiple spaces/tabs to single space)
    let normalized: String = {
        let mut result = String::with_capacity(no_nulls.len());
        let mut prev_was_space = false;
        for c in no_nulls.chars() {
            if c.is_whitespace() && c != '\n' {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(c);
                prev_was_space = false;
            }
        }
        result
    };

    // Step 4: Remove control characters (except newline and tab)
    let cleaned: String = normalized
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect();

    // Step 5: Truncate to max_length
    if cleaned.len() > max_length {
        cleaned[..max_length].to_string()
    } else {
        cleaned
    }
}

pub fn validate_allowlist<'a>(input: &'a str, pattern: &str) -> Result<&'a str, String> {
    let full_pattern = if pattern.starts_with('^') && pattern.ends_with('$') {
        pattern.to_string()
    } else {
        format!("^{}$", pattern)
    };

    let re = Regex::new(&full_pattern)
        .map_err(|e| format!("Invalid allowlist pattern: {}", e))?;

    if re.is_match(input) {
        Ok(input)
    } else {
        Err(format!(
            "Input '{}' does not match allowlist pattern '{}'",
            input, pattern
        ))
    }
}

pub fn validate_denylist<'a>(input: &'a str, patterns: &[&str]) -> Result<&'a str, String> {
    let lower = input.to_lowercase();
    for pattern in patterns {
        if lower.contains(&pattern.to_lowercase()) {
            return Err(format!("Input contains denied pattern: '{}'", pattern));
        }
    }
    Ok(input)
}

pub struct ValidationPipeline<'a> {
    steps: Vec<Box<dyn Fn(&str) -> Result<&str, String> + 'a>>,
}

impl<'a> ValidationPipeline<'a> {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step<F>(mut self, step: F) -> Self
    where
        F: Fn(&str) -> Result<&str, String> + 'a,
    {
        self.steps.push(Box::new(step));
        self
    }

    pub fn validate<'b>(&'b self, input: &'b str) -> Result<&'b str, String> {
        let mut current = input;
        for step in &self.steps {
            current = step(current)?;
        }
        Ok(current)
    }
}

pub struct FieldSanitizer {
    max_length: usize,
    allowlist_pattern: Option<String>,
    strip_html: bool,
}

impl FieldSanitizer {
    pub fn new(max_length: usize) -> Self {
        Self {
            max_length,
            allowlist_pattern: None,
            strip_html: false,
        }
    }

    pub fn with_allowlist(mut self, pattern: &str) -> Self {
        self.allowlist_pattern = Some(pattern.to_string());
        self
    }

    pub fn with_strip_html(mut self) -> Self {
        self.strip_html = true;
        self
    }

    pub fn sanitize(&self, input: &str) -> Result<String, String> {
        // Step 1: Trim whitespace
        let mut result = input.trim().to_string();

        // Step 2: Strip HTML tags if enabled
        if self.strip_html {
            result = strip_html_tags(&result);
        }

        // Step 3: Remove null bytes
        result = result.chars().filter(|c| *c != '\0').collect();

        // Step 4: Truncate to max_length
        if result.len() > self.max_length {
            result.truncate(self.max_length);
        }

        // Step 5: Check allowlist if configured
        if let Some(ref pattern) = self.allowlist_pattern {
            validate_allowlist(&result, pattern)?;
        }

        Ok(result)
    }
}

pub fn strip_html_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;

    for c in input.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {} // Skip characters inside tags
        }
    }

    // Decode common HTML entities
    result = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'");

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_trims_whitespace() {
        assert_eq!(sanitize_input("  hello  ", 100), "hello");
    }

    #[test]
    fn test_sanitize_removes_null_bytes() {
        assert_eq!(sanitize_input("hello\0world", 100), "helloworld");
    }

    #[test]
    fn test_sanitize_truncates() {
        let result = sanitize_input("hello world", 5);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_allowlist_valid() {
        let result = validate_allowlist("alice_99", r"^[a-zA-Z0-9_]{3,32}$");
        assert!(result.is_ok());
    }

    #[test]
    fn test_allowlist_invalid() {
        let result = validate_allowlist("alice@home!", r"^[a-zA-Z0-9_]{3,32}$");
        assert!(result.is_err());
    }

    #[test]
    fn test_denylist_clean() {
        let result = validate_denylist("hello world", &["<script>", "DROP TABLE"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_denylist_match() {
        let result = validate_denylist("hello <script>alert(1)</script>", &["<script>"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_all_pass() {
        let pipeline = ValidationPipeline::new()
            .add_step(|s| if !s.is_empty() { Ok(s) } else { Err("empty".into()) })
            .add_step(|s| if s.is_ascii() { Ok(s) } else { Err("not ascii".into()) });
        let result = pipeline.validate("hello");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pipeline_first_failure() {
        let pipeline = ValidationPipeline::new()
            .add_step(|s| if s.len() >= 10 { Ok(s) } else { Err("too short".into()) })
            .add_step(|s| if s.is_ascii() { Ok(s) } else { Err("not ascii".into()) });
        let result = pipeline.validate("hi");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("short"));
    }

    #[test]
    fn test_field_sanitizer_strips_html() {
        let sanitizer = FieldSanitizer::new(100).with_strip_html();
        let result = sanitizer.sanitize("<b>hello</b>").unwrap();
        assert!(!result.contains('<'));
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_field_sanitizer_enforces_allowlist() {
        let sanitizer = FieldSanitizer::new(100).with_allowlist(r"^[a-z]+$");
        assert!(sanitizer.sanitize("hello").is_ok());
        assert!(sanitizer.sanitize("hello123").is_err());
    }

    #[test]
    fn test_strip_html_entities() {
        let result = strip_html_tags("a &amp; b &lt; c");
        assert!(result.contains("a & b"));
        assert!(result.contains("< c"));
    }
}
