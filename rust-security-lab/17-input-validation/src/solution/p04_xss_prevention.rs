//! # Lesson 04: XSS Prevention (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

pub fn encode_html_body(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#x27;"),
            _ => result.push(c),
        }
    }
    result
}

pub fn encode_html_attribute(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 4);
    for c in input.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            result.push(c);
        } else {
            result.push_str(&format!("&#x{:02X};", c as u32));
        }
    }
    result
}

pub fn encode_javascript(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '\'' => result.push_str("\\'"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '<' => result.push_str("\\u003C"),
            '>' => result.push_str("\\u003E"),
            _ => result.push(c),
        }
    }
    result
}

pub fn encode_url(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

pub fn detect_xss(input: &str) -> bool {
    let lower = input.to_lowercase();

    let tag_patterns = [
        "<script", "<img", "<iframe", "<object", "<embed",
        "<svg", "<body", "<link", "<style", "<form",
        "<input", "<details", "<marquee", "</script",
    ];

    for pattern in &tag_patterns {
        if lower.contains(pattern) {
            return true;
        }
    }

    let event_handlers = [
        "onerror", "onload", "onclick", "onmouseover",
        "onfocus", "onblur", "onsubmit", "onchange",
    ];

    for handler in &event_handlers {
        if lower.contains(handler) {
            return true;
        }
    }

    let protocols = ["javascript:", "data:", "vbscript:"];
    for protocol in &protocols {
        if lower.contains(protocol) {
            return true;
        }
    }

    if lower.contains("&#") || lower.contains("&#x") {
        return true;
    }

    if input.contains('\0') || input.contains('`') {
        return true;
    }

    false
}

pub fn build_csp_header(nonce: &str) -> String {
    format!(
        "script-src 'nonce-{}' 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; object-src 'none'; connect-src 'self'",
        nonce
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_html_tags() {
        let result = encode_html_body("<script>alert('xss')</script>");
        assert!(!result.contains("<script>"));
        assert!(result.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_encode_html_ampersand() {
        let result = encode_html_body("A & B");
        assert_eq!(result, "A &amp; B");
    }

    #[test]
    fn test_encode_attribute_basic() {
        let result = encode_html_attribute("hello");
        assert!(!result.contains('"'));
    }

    #[test]
    fn test_encode_js_newline() {
        let result = encode_javascript("line1\nline2");
        assert!(result.contains("\\n"));
        assert!(!result.contains('\n'));
    }

    #[test]
    fn test_encode_js_script_close() {
        let result = encode_javascript("</script>");
        assert!(result.contains("\\u003C"));
        assert!(result.contains("\\u003E"));
    }

    #[test]
    fn test_encode_url_special_chars() {
        let result = encode_url("hello world&foo=bar");
        assert!(result.contains("%20"));
        assert!(result.contains("%26"));
        assert!(result.contains("%3D"));
    }

    #[test]
    fn test_detect_xss_script_tag() {
        assert!(detect_xss("<script>alert(1)</script>"));
    }

    #[test]
    fn test_detect_xss_event_handler() {
        assert!(detect_xss("x onerror=alert(1)"));
    }

    #[test]
    fn test_detect_xss_javascript_protocol() {
        assert!(detect_xss("javascript:alert(1)"));
    }

    #[test]
    fn test_detect_xss_clean_input() {
        assert!(!detect_xss("Hello, welcome to our site!"));
    }

    #[test]
    fn test_csp_header_contains_nonce() {
        let csp = build_csp_header("abc123");
        assert!(csp.contains("abc123"));
        assert!(csp.contains("script-src"));
    }
}
