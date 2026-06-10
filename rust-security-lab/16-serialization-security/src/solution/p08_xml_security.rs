//! # Lesson 08: XML Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Detect XML External Entity (XXE) declarations.
pub fn has_xxe_indicators(xml: &str) -> Result<bool, String> {
    let upper = xml.to_uppercase();
    Ok(upper.contains("<!DOCTYPE")
        || upper.contains("<!ENTITY")
        || upper.contains("SYSTEM")
        || upper.contains("PUBLIC"))
}

/// Detect XML bomb (billion laughs) patterns.
pub fn is_xml_bomb(xml: &str, max_entities: usize) -> Result<bool, String> {
    let upper = xml.to_uppercase();
    let entity_count = upper.matches("<!ENTITY").count();
    Ok(entity_count > max_entities)
}

/// Detect entity expansion depth by counting nesting in entity definitions.
pub fn entity_expansion_depth(xml: &str) -> usize {
    // Count how many entity references appear within entity definitions
    // Simplified: look for patterns like &...; inside entity values
    let mut max_depth = 0usize;
    let mut current_depth = 0usize;

    // Scan for entity reference patterns (&name;) and track nesting
    let chars: Vec<char> = xml.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '&' {
            // Find the matching semicolon
            let mut j = i + 1;
            while j < chars.len() && chars[j] != ';' && j - i < 100 {
                j += 1;
            }
            if j < chars.len() && chars[j] == ';' {
                current_depth += 1;
                max_depth = max_depth.max(current_depth);
                i = j + 1;
                continue;
            }
        } else if chars[i] == ';' {
            if current_depth > 0 {
                current_depth -= 1;
            }
        }
        i += 1;
    }

    max_depth
}

/// Sanitize XML input by escaping special characters.
pub fn escape_xml(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '&' => result.push_str("&amp;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            _ => result.push(c),
        }
    }
    result
}

/// Detect XPath injection attempts.
pub fn has_xpath_injection(input: &str) -> Result<bool, String> {
    let lower = input.to_lowercase();

    // Check for quote characters
    if input.contains('\'') || input.contains('"') {
        return Ok(true);
    }

    // Check for XPath-specific patterns
    if input.contains("//") || input.contains("::") {
        return Ok(true);
    }

    // Check for comparison operators
    if input.contains('=') || input.contains('<') || input.contains('>') {
        return Ok(true);
    }

    // Check for XPath boolean operators as whole words
    for word in lower.split_whitespace() {
        match word {
            "or" | "and" | "not" | "div" | "mod" => return Ok(true),
            _ => {}
        }
    }

    Ok(false)
}

/// Build a safe XPath string literal.
pub fn escape_xpath_string(input: &str) -> String {
    if !input.contains('\'') {
        return format!("'{}'", input);
    }

    // Use XPath concat() to handle strings with single quotes
    // concat('before', "'", 'after')
    let parts: Vec<&str> = input.split('\'').collect();
    let mut result = String::from("concat(");
    for (i, part) in parts.iter().enumerate() {
        if i > 0 {
            result.push_str(", \"'\", ");
        }
        result.push('\'');
        result.push_str(part);
        result.push('\'');
    }
    result.push(')');
    result
}

/// Validate that an XML document has a safe structure.
pub fn validate_xml_safety(xml: &str, max_bytes: usize) -> Result<(), String> {
    if xml.len() > max_bytes {
        return Err(format!(
            "XML size {} exceeds maximum {}",
            xml.len(),
            max_bytes
        ));
    }

    let upper = xml.to_uppercase();

    if upper.contains("<!DOCTYPE") {
        return Err("XML must not contain DOCTYPE declarations".to_string());
    }

    if upper.contains("<!ENTITY") {
        return Err("XML must not contain ENTITY declarations".to_string());
    }

    if upper.contains("SYSTEM") || upper.contains("PUBLIC") {
        return Err("XML must not contain SYSTEM or PUBLIC references".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xxe_in_doctype() {
        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE foo [<!ENTITY xxe SYSTEM "file:///etc/passwd">]>
<foo>&xxe;</foo>"#;
        assert!(has_xxe_indicators(xml).unwrap());
    }

    #[test]
    fn test_no_xxe() {
        let xml = r#"<?xml version="1.0"?><root><child>data</child></root>"#;
        assert!(!has_xxe_indicators(xml).unwrap());
    }

    #[test]
    fn test_xml_billion_laughs() {
        let xml = r#"<!DOCTYPE lolz [
<!ENTITY lol "lol">
<!ENTITY lol2 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
<!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
]>"#;
        assert!(is_xml_bomb(xml, 2).unwrap());
    }

    #[test]
    fn test_not_xml_bomb() {
        let xml = r#"<!DOCTYPE foo [<!ENTITY bar "baz">]><root>&bar;</root>"#;
        assert!(!is_xml_bomb(xml, 5).unwrap());
    }

    #[test]
    fn test_escape_special_chars() {
        assert_eq!(
            escape_xml("<b>\"hello\" & 'world'</b>"),
            "&lt;b&gt;&quot;hello&quot; &amp; &#x27;world&#x27;&lt;/b&gt;"
        );
    }

    #[test]
    fn test_escape_no_special_chars() {
        assert_eq!(escape_xml("hello world"), "hello world");
    }

    #[test]
    fn test_xpath_injection_quotes() {
        assert!(has_xpath_injection("' or '1'='1").unwrap());
    }

    #[test]
    fn test_xpath_injection_slashes() {
        assert!(has_xpath_injection("//user").unwrap());
    }

    #[test]
    fn test_xpath_clean_input() {
        assert!(!has_xpath_injection("alice").unwrap());
    }

    #[test]
    fn test_escape_xpath_string_basic() {
        assert_eq!(escape_xpath_string("alice"), "'alice'");
    }

    #[test]
    fn test_escape_xpath_string_with_quotes() {
        let result = escape_xpath_string("O'Brien");
        assert!(result.contains("concat"));
        assert!(result.contains("O"));
        assert!(result.contains("Brien"));
    }

    #[test]
    fn test_xml_safety_clean() {
        let xml = r#"<root><child>data</child></root>"#;
        assert!(validate_xml_safety(xml, 1024).is_ok());
    }

    #[test]
    fn test_xml_safety_has_dtd() {
        let xml = r#"<!DOCTYPE foo><root>data</root>"#;
        assert!(validate_xml_safety(xml, 1024).is_err());
    }

    #[test]
    fn test_xml_safety_too_large() {
        let xml = "<root>data</root>";
        assert!(validate_xml_safety(xml, 5).is_err());
    }
}
