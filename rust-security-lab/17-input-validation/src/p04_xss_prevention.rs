//! # Lesson 04: XSS Prevention
//!
//! ## The Problem
//!
//! Cross-Site Scripting (XSS) occurs when user-controlled data is rendered
//! in a web page without proper encoding. The browser treats the data as
//! HTML/JavaScript, allowing attackers to:
//!
//! - Steal session cookies (`document.cookie`)
//! - Redirect users to phishing sites
//! - Deface page content
//! - Log keystrokes
//!
//! ## Types of XSS
//!
//! 1. **Reflected XSS**: Input is immediately reflected in the response
//!    (e.g., search queries, error messages)
//! 2. **Stored XSS**: Input is saved and later rendered to other users
//!    (e.g., comments, forum posts)
//! 3. **DOM-based XSS**: Client-side JavaScript processes input unsafely
//!
//! ## The Golden Rule
//!
//! **Encode output for its context.** Different contexts require different
//! encoding:
//!
//! | Context         | Encoding                                              |
//! |-----------------|-------------------------------------------------------|
//! | HTML body       | Encode `< > " ' &` as HTML entities                   |
//! | HTML attribute  | Encode all non-alphanumeric chars as `&#xNN;`         |
//! | JavaScript      | Encode for JS string literal (escape `' " \ \n`)      |
//! | URL             | Percent-encode special characters                     |
//! | CSS             | Encode using CSS escape sequences                     |
//!
//! ## Defense Layers
//!
//! 1. **Output encoding** (primary defense)
//! 2. **Content Security Policy** (CSP) headers
//! 3. **Input validation** (defense in depth)
//! 4. **HTTPOnly cookies** (limit cookie theft impact)

/// Encode a string for safe inclusion in an HTML body context.
///
/// Must encode these characters as HTML entities:
/// - `&` -> `&amp;`
/// - `<` -> `&lt;`
/// - `>` -> `&gt;`
/// - `"` -> `&quot;`
/// - `'` -> `&#x27;`
///
/// This is the most common XSS encoding -- it prevents injected HTML tags
/// from being interpreted by the browser.
pub fn encode_html_body(input: &str) -> String {
    todo!("Encode HTML special characters")
}

/// Encode a string for safe inclusion in an HTML attribute value.
///
/// HTML attributes are more dangerous than body content because they
/// can be broken out of using various characters. Encode ALL characters
/// that are not alphanumeric or a hyphen using `&#xNN;` format.
///
/// Example: `hello world` -> `&#x68;&#x65;&#x6C;&#x6C;&#x6F;&#x20;...`
/// (or a simpler approach: encode only the dangerous characters)
pub fn encode_html_attribute(input: &str) -> String {
    todo!("Encode for HTML attribute context")
}

/// Encode a string for safe inclusion inside a JavaScript string literal.
///
/// Must escape characters that could break out of a JS string:
/// - `\` -> `\\`
/// - `'` -> `\'`
/// - `"` -> `\"`
/// - `\n` -> `\\n`
/// - `\r` -> `\\r`
/// - `\t` -> `\\t`
/// - `<` -> `\\u003C` (prevents `</script>` injection)
/// - `>` -> `\\u003E`
///
/// The input should also be wrapped in quotes by the caller.
pub fn encode_javascript(input: &str) -> String {
    todo!("Encode for JavaScript string context")
}

/// Encode a string for safe use as a URL parameter value.
///
/// Must percent-encode all characters except unreserved characters
/// (A-Z, a-z, 0-9, `-`, `.`, `_`, `~`).
///
/// Example: `hello world` -> `hello%20world`
/// Example: `<script>` -> `%3Cscript%3E`
pub fn encode_url(input: &str) -> String {
    todo!("Percent-encode for URL context")
}

/// Detect potential XSS payloads in user input.
///
/// Checks for:
/// - HTML tags: `<script`, `<img`, `<iframe`, `<object`, `<embed`, `<svg`,
///   `<body`, `<link`, `<style`, `<form`, `<input`, `<details`, `<marquee`
/// - Event handlers: `onerror`, `onload`, `onclick`, `onmouseover`, `onfocus`,
///   `onblur`, `onsubmit`, `onchange`
/// - JavaScript protocol: `javascript:`, `data:`, `vbscript:`
/// - HTML entities used for evasion: `&#`, `&#x`
/// - Common evasion patterns: null bytes, backticks
///
/// Returns true if suspicious patterns are found.
pub fn detect_xss(input: &str) -> bool {
    todo!("Detect XSS attack patterns")
}

/// Build a Content-Security-Policy header value for a basic application.
///
/// Takes a nonce (a random string) for inline script allowlisting.
///
/// The policy should:
/// - Restrict scripts to nonce-based inline and same-origin
/// - Restrict styles to same-origin and unsafe-inline (minimal)
/// - Restrict images to same-origin and data: URIs
/// - Block all plugins (object-src 'none')
/// - Restrict connections to same-origin
pub fn build_csp_header(nonce: &str) -> String {
    todo!("Build a Content-Security-Policy header string")
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
        // At minimum, quotes and angle brackets should be encoded
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
