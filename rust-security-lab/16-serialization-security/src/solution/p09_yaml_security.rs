//! # Lesson 09: YAML Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Detect YAML anchor bomb patterns by counting anchor definitions.
pub fn has_yaml_bomb_indicators(yaml: &str, max_anchors: usize) -> Result<bool, String> {
    // Filter out YAML document markers (& is also used in anchors)
    // A YAML anchor is typically `&name ` pattern
    let mut actual_anchors = 0usize;
    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.contains('&') && !trimmed.starts_with('#') {
            // Count anchor definitions (not alias references with *)
            for (i, ch) in trimmed.chars().enumerate() {
                if ch == '&' && (i == 0 || trimmed.as_bytes()[i - 1] != b'*') {
                    actual_anchors += 1;
                }
            }
        }
    }
    Ok(actual_anchors > max_anchors)
}

/// Count YAML aliases (references to anchors).
pub fn count_yaml_aliases(yaml: &str) -> usize {
    yaml.matches('*').count()
}

/// Detect YAML type confusion -- strings YAML interprets as non-string types.
pub fn is_yaml_type_confusion(value: &str) -> Result<bool, String> {
    let lower = value.to_lowercase();
    match lower.as_str() {
        "true" | "yes" | "on" => Ok(true),
        "false" | "no" | "off" => Ok(true),
        "null" | "~" => Ok(true),
        _ => Ok(false),
    }
}

/// Validate YAML safety by checking for dangerous patterns.
pub fn validate_yaml_safety(yaml: &str, max_depth: usize, max_anchors: usize) -> Result<(), String> {
    // Check for dangerous type tags
    if yaml.contains("!!") {
        return Err("YAML must not contain type tags (!!)".to_string());
    }

    // Check nesting depth by measuring indentation
    let mut max_indent = 0usize;
    for line in yaml.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        // Each 2 spaces is approximately one level
        let level = indent / 2;
        max_indent = max_indent.max(level);
    }
    if max_indent > max_depth {
        return Err(format!(
            "YAML nesting depth {} exceeds maximum {}",
            max_indent, max_depth
        ));
    }

    // Check for anchor bombs
    if has_yaml_bomb_indicators(yaml, max_anchors)? {
        return Err("YAML has too many anchor definitions (potential bomb)".to_string());
    }

    Ok(())
}

/// Parse YAML with strict type validation.
///
/// Since serde_yaml is not available, we implement a simple key-value parser
/// that handles the basic YAML key: value format and demonstrates type
/// confusion awareness.
#[derive(Debug, PartialEq)]
pub struct YamlConfig {
    pub name: String,
    pub enabled: bool,
    pub count: u64,
}

pub fn parse_yaml_config(yaml: &str) -> Result<YamlConfig, String> {
    let mut name = None;
    let mut enabled = None;
    let mut count = None;

    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = trimmed.splitn(2, ':').collect();
        if parts.len() != 2 {
            continue;
        }
        let key = parts[0].trim();
        let val = parts[1].trim();

        match key {
            "name" => {
                let v = val.trim_matches('"').trim_matches('\'');
                name = Some(v.to_string());
            }
            "enabled" => {
                // Type confusion awareness: YAML interprets yes/no/on/off/true/false
                let v = match val.to_lowercase().as_str() {
                    "true" | "yes" | "on" => true,
                    "false" | "no" | "off" => false,
                    _ => return Err(format!("cannot parse '{}' as boolean", val)),
                };
                enabled = Some(v);
            }
            "count" => {
                let v: u64 = val
                    .parse()
                    .map_err(|e| format!("cannot parse '{}' as u64: {}", val, e))?;
                count = Some(v);
            }
            _ => {
                return Err(format!("unknown field '{}'", key));
            }
        }
    }

    let name = name.ok_or_else(|| "missing required field 'name'".to_string())?;
    let enabled = enabled.ok_or_else(|| "missing required field 'enabled'".to_string())?;
    let count = count.ok_or_else(|| "missing required field 'count'".to_string())?;

    if name.is_empty() {
        return Err("name must not be empty".to_string());
    }
    if count == 0 {
        return Err("count must be positive".to_string());
    }

    Ok(YamlConfig {
        name,
        enabled,
        count,
    })
}

/// Detect YAML merge keys.
pub fn has_yaml_merge_keys(yaml: &str) -> Result<bool, String> {
    Ok(yaml.contains("<<:"))
}

/// Estimate the expansion ratio of a YAML document.
pub fn yaml_expansion_ratio(yaml: &str) -> Result<f64, String> {
    let alias_count = count_yaml_aliases(yaml) as f64;
    let anchor_count = yaml.matches('&').count() as f64;

    if anchor_count == 0.0 {
        return Ok(1.0);
    }

    Ok((alias_count + 1.0) / (anchor_count + 1.0))
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
