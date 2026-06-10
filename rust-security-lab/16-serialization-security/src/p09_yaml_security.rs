//! # Lesson 09: YAML Security
//!
//! ## The Threat
//!
//! YAML is a popular configuration format but is notoriously dangerous when
//! parsing untrusted input. The key attacks are:
//!
//! 1. **Arbitrary code execution** -- In many languages (Python's `yaml.load`,
//!    Ruby's `YAML.load`), YAML can instantiate arbitrary objects. While Rust's
//!    `serde_yaml` does NOT execute code during deserialization, the YAML spec
//!    itself supports tags that indicate types, and misconfigured parsers can
//!    be exploited.
//!
//! 2. **YAML bombs** -- Anchors and aliases (`&` and `*`) can create exponential
//!    expansion, similar to XML billion laughs. A small YAML document can
//!    expand to consume gigabytes of memory.
//!
//! 3. **Type confusion** -- YAML's flexible typing means `yes` is `true`,
//!    `1.0` might be a float or string depending on the schema, and `null`
//!    appears in many forms (`null`, `~`, empty value).
//!
//! 4. **Anchor abuse** -- Repeated anchors can create deeply nested or
//!    extremely large structures that exhaust memory or stack.
//!
//! ## Defense
//!
//! - Limit input size before parsing.
//! - Detect and limit anchor/alias usage.
//! - Validate types strictly after parsing.
//! - Use `serde_yaml` (safe) instead of custom YAML executors.
//! - Consider using JSON instead of YAML for untrusted input.

/// Exercise 1: Detect YAML anchor bomb patterns.
///
/// Count the number of anchor definitions (`&name`) in the YAML string.
/// If the count exceeds `max_anchors`, return Ok(true) (potential bomb).
///
/// An anchor is defined with `&` followed by a name, typically as `&anchor_name`.
pub fn has_yaml_bomb_indicators(yaml: &str, max_anchors: usize) -> Result<bool, String> {
    todo!("Detect YAML anchor bomb patterns")
}

/// Exercise 2: Count YAML aliases (references to anchors).
///
/// An alias is a reference like `*anchor_name`. Count how many alias
/// references exist in the YAML string.
///
/// Return the count.
pub fn count_yaml_aliases(yaml: &str) -> usize {
    todo!("Count YAML alias references")
}

/// Exercise 3: Detect YAML type confusion.
///
/// YAML interprets certain strings as non-string types:
/// - `true`, `True`, `TRUE`, `yes`, `Yes`, `YES`, `on`, `On`, `ON` -> boolean true
/// - `false`, `False`, `FALSE`, `no`, `No`, `NO`, `off`, `Off`, `OFF` -> boolean false
/// - `null`, `Null`, `NULL`, `~` -> null
///
/// Given a string value, return Ok(true) if YAML would interpret it as a
/// non-string type (boolean or null), Ok(false) if it stays a string.
pub fn is_yaml_type_confusion(value: &str) -> Result<bool, String> {
    todo!("Detect YAML type confusion")
}

/// Exercise 4: Sanitize YAML input by checking for dangerous patterns.
///
/// Return Ok(()) if the YAML is safe, Err(message) if it contains:
/// - `!!` tags (YAML type tags, e.g., `!!python/object`)
/// - Excessive nesting (more than `max_depth` levels of indentation)
/// - More than `max_anchors` anchor definitions
pub fn validate_yaml_safety(yaml: &str, max_depth: usize, max_anchors: usize) -> Result<(), String> {
    todo!("Validate YAML safety")
}

/// Exercise 5: Parse YAML with strict type validation.
///
/// A configuration value that should be a string but YAML might interpret
/// as something else. Since serde_yaml is not available, implement a simple
/// key-value parser that demonstrates type confusion awareness.
#[derive(Debug, PartialEq)]
pub struct YamlConfig {
    pub name: String,
    pub enabled: bool,
    pub count: u64,
}

/// Parse the YAML string and validate that:
/// - `name` is actually a non-empty string (not null, not a number disguised as string)
/// - `enabled` is a boolean (watch for YAML type confusion: yes/no/on/off)
/// - `count` is a positive integer
///
/// Parse the simple `key: value` format line by line.
/// Return Ok(config) or Err(message).
pub fn parse_yaml_config(yaml: &str) -> Result<YamlConfig, String> {
    todo!("Parse YAML with strict validation")
}

/// Exercise 6: Detect YAML merge keys and anchors that could cause expansion.
///
/// Merge keys (`<<:`) allow inheriting keys from an anchored mapping.
/// While useful, they can be abused to create unexpected structures.
///
/// Return Ok(true) if the YAML contains merge keys, Ok(false) otherwise.
pub fn has_yaml_merge_keys(yaml: &str) -> Result<bool, String> {
    todo!("Detect YAML merge keys")
}

/// Exercise 7: Estimate the expansion ratio of a YAML document.
///
/// Given the raw YAML string and the number of aliases found, estimate
/// the expansion ratio as: (alias_count + 1) / (anchor_count + 1).
///
/// A ratio above 10 suggests potential bomb.
/// Return the ratio as f64.
pub fn yaml_expansion_ratio(yaml: &str) -> Result<f64, String> {
    todo!("Estimate YAML expansion ratio")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_bomb_many_anchors() {
        let yaml = r#"
a: &a1 "x"
b: &a2 "x"
c: &a3 "x"
d: &a4 "x"
"#;
        assert!(has_yaml_bomb_indicators(yaml, 3).unwrap());
    }

    #[test]
    fn test_yaml_no_bomb() {
        let yaml = r#"
a: &anchor "value"
b: *anchor
"#;
        assert!(!has_yaml_bomb_indicators(yaml, 5).unwrap());
    }

    #[test]
    fn test_count_aliases() {
        let yaml = r#"
a: &ref1 "hello"
b: *ref1
c: *ref1
"#;
        assert_eq!(count_yaml_aliases(yaml), 2);
    }

    #[test]
    fn test_type_confusion_yes() {
        assert!(is_yaml_type_confusion("yes").unwrap());
        assert!(is_yaml_type_confusion("True").unwrap());
        assert!(is_yaml_type_confusion("on").unwrap());
    }

    #[test]
    fn test_type_confusion_null() {
        assert!(is_yaml_type_confusion("null").unwrap());
        assert!(is_yaml_type_confusion("~").unwrap());
    }

    #[test]
    fn test_type_confusion_not() {
        assert!(!is_yaml_type_confusion("hello").unwrap());
        assert!(!is_yaml_type_confusion("42").unwrap());
    }

    #[test]
    fn test_yaml_safety_clean() {
        let yaml = "name: test\nenabled: true\n";
        assert!(validate_yaml_safety(yaml, 10, 5).is_ok());
    }

    #[test]
    fn test_yaml_safety_dangerous_tag() {
        let yaml = "data: !!python/object/apply:os.system ['rm -rf /']\n";
        assert!(validate_yaml_safety(yaml, 10, 5).is_err());
    }

    #[test]
    fn test_parse_yaml_config_valid() {
        let yaml = "name: myapp\nenabled: true\ncount: 42\n";
        let config = parse_yaml_config(yaml).unwrap();
        assert_eq!(config.name, "myapp");
        assert!(config.enabled);
        assert_eq!(config.count, 42);
    }

    #[test]
    fn test_parse_yaml_config_empty_name() {
        let yaml = "name: ''\nenabled: true\ncount: 1\n";
        assert!(parse_yaml_config(yaml).is_err());
    }

    #[test]
    fn test_yaml_merge_keys() {
        let yaml = r#"
defaults: &defaults
  timeout: 30
service:
  <<: *defaults
  name: myservice
"#;
        assert!(has_yaml_merge_keys(yaml).unwrap());
    }

    #[test]
    fn test_yaml_no_merge_keys() {
        let yaml = "name: test\ntimeout: 30\n";
        assert!(!has_yaml_merge_keys(yaml).unwrap());
    }

    #[test]
    fn test_expansion_ratio_low() {
        let yaml = r#"
a: &ref "value"
b: *ref
"#;
        let ratio = yaml_expansion_ratio(yaml).unwrap();
        assert!(ratio <= 10.0);
    }
}
