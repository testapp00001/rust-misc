//! # Lesson 08: XML Security
//!
//! ## The Threat
//!
//! XML has been around since 1998 and carries decades of security baggage.
//! The most critical XML-specific attacks are:
//!
//! 1. **XML External Entity (XXE)** -- An attacker injects `<!DOCTYPE>` with
//!    entity definitions that read local files, make network requests, or
//!    cause denial of service via recursive entity expansion ("billion laughs").
//!
//! 2. **XML Bomb (Billion Laughs)** -- A small XML document that expands to
//!    gigabytes in memory through nested entity references.
//!
//! 3. **XPath Injection** -- User input concatenated into XPath queries,
//!    analogous to SQL injection.
//!
//! 4. **Schema Poisoning** -- An attacker provides a malicious XSD schema
//!    that relaxes validation rules.
//!
//! ## Defense
//!
//! - Disable DTD processing entirely when parsing untrusted XML.
//! - Disable external entity resolution.
//! - Set maximum expansion limits for entities.
//! - Use parameterized XPath queries.
//! - Never fetch schemas from untrusted URLs.
//!
//! ## Note
//!
//! This lesson uses string inspection and manual checks to simulate XML
//! security patterns. In production, use a hardened XML parser with DTD
//! processing disabled.

/// Exercise 1: Detect XML External Entity (XXE) declarations.
///
/// Scan the XML string for any of these patterns:
/// - `<!DOCTYPE` declarations
/// - `<!ENTITY` declarations
/// - `SYSTEM` keyword (used in external entity references)
/// - `PUBLIC` keyword (used in external entity references)
///
/// Return Ok(true) if any XXE indicators are found, Ok(false) otherwise.
///
/// This is a simplified check -- real XXE detection requires a full parser.
pub fn has_xxe_indicators(xml: &str) -> Result<bool, String> {
    todo!("Detect XXE indicators in XML")
}

/// Exercise 2: Detect XML bomb (billion laughs) patterns.
///
/// Check for nested entity definitions that could cause exponential expansion.
/// A simple heuristic: count the number of `<!ENTITY` declarations.
/// If there are more than `max_entities`, return Ok(true) (potential bomb).
///
/// Return Ok(false) if within limits.
pub fn is_xml_bomb(xml: &str, max_entities: usize) -> Result<bool, String> {
    todo!("Detect XML bomb patterns")
}

/// Exercise 3: Detect entity expansion depth.
///
/// Count how many levels of `&entityName;` references appear nested.
/// For example, `&a;` where `a` contains `&b;` where `b` contains `&c;`
/// has depth 3.
///
/// Simplified: count consecutive `&...;` patterns in entity definitions.
/// Return the maximum nesting depth found.
pub fn entity_expansion_depth(xml: &str) -> usize {
    todo!("Measure entity expansion depth")
}

/// Exercise 4: Sanitize XML input by escaping special characters.
///
/// Replace these characters with their XML entity equivalents:
/// - `<` becomes `&lt;`
/// - `>` becomes `&gt;`
/// - `&` becomes `&amp;`
/// - `"` becomes `&quot;`
/// - `'` becomes `&#x27;`
///
/// Return the sanitized string.
pub fn escape_xml(input: &str) -> String {
    todo!("Escape XML special characters")
}

/// Exercise 5: Detect XPath injection attempts.
///
/// Check if the input contains characters or patterns commonly used in
/// XPath injection:
/// - Single quotes (`'`)
/// - Double quotes (`"`)
/// - XPath operators: `or`, `and`, `not`
/// - XPath axes: `//`, `/`
/// - Comparison: `=`, `<`, `>`
///
/// Return Ok(true) if suspicious patterns found, Ok(false) otherwise.
pub fn has_xpath_injection(input: &str) -> Result<bool, String> {
    todo!("Detect XPath injection")
}

/// Exercise 6: Build a safe XPath query using string escaping.
///
/// Given a user-supplied value, escape it so it can be safely embedded
/// in an XPath string literal. In XPath, the main concern is quotes.
///
/// Strategy: if the value contains single quotes, use concat() to build
/// the string safely. Otherwise, wrap in single quotes.
///
/// For simplicity, just escape double quotes and wrap in single quotes.
/// Return the safe string literal.
pub fn escape_xpath_string(input: &str) -> String {
    todo!("Escape XPath string literal")
}

/// Exercise 7: Validate that an XML document has a safe structure.
///
/// Check these rules:
/// 1. Must not contain `<!DOCTYPE` (no DTD)
/// 2. Must not contain `<!ENTITY` (no entity definitions)
/// 3. Must not contain `SYSTEM` or `PUBLIC` (no external references)
/// 4. Must not exceed `max_bytes` in size
///
/// Return Ok(()) if safe, Err(message) if any rule is violated.
pub fn validate_xml_safety(xml: &str, max_bytes: usize) -> Result<(), String> {
    todo!("Validate XML document safety")
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
        assert_eq!(escape_xml("<b>\"hello\" & 'world'</b>"),
            "&lt;b&gt;&quot;hello&quot; &amp; &#x27;world&#x27;&lt;/b&gt;");
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
        // Should handle quotes safely
        let result = escape_xpath_string("O'Brien");
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
