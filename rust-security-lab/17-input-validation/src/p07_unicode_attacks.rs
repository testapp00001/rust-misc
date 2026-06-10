//! # Lesson 07: Unicode Attacks
//!
//! ## The Problem
//!
//! Unicode is enormously complex -- over 149,000 characters across hundreds
//! of scripts. This complexity creates subtle attack vectors that many
//! validation routines miss entirely.
//!
//! ## Attack Types
//!
//! ### 1. Homoglyph Attacks
//! Characters that look identical but have different Unicode code points:
//!
//! | Latin | Cyrillic | Code Points    |
//! |-------|----------|----------------|
//! | `a`   | `а`      | U+0061 vs U+0430 |
//! | `e`   | `е`      | U+0065 vs U+0435 |
//! | `o`   | `о`      | U+006F vs U+043E |
//! | `p`   | `р`      | U+0070 vs U+0440 |
//!
//! An attacker registers `аpple.com` (Cyrillic `а`) which looks identical
//! to `apple.com` in the browser address bar.
//!
//! ### 2. Right-to-Left Override (RLO)
//! Unicode character U+202E reverses the display order of text.
//! A file named `fun[U+202E]gpj.exe` displays as `funex.jpg`.
//!
//! ### 3. Zero-Width Characters
//! Invisible characters (U+200B zero-width space, U+200C zero-width
//! non-joiner, U+FEFF BOM) can be inserted to bypass keyword filters.
//!
//! ### 4. Normalization Attacks
//! The same visual character can have multiple Unicode representations:
//! - Precomposed: `e` + `\u{0301}` (combining acute accent)
//! - Decomposed: `\u{00E9}` (precomposed e-acute)
//!
//! A filter checking for `admin` might miss `a\u{0308}dmin` (a with
//! combining diaeresis, visually similar).
//!
//! ## Defense
//!
//! 1. **Normalize** all input to NFC (Canonical Decomposition + Composition)
//! 2. **Restrict** to allowed character sets (ASCII-only where possible)
//! 3. **Detect** confusable/homoglyph characters
//! 4. **Reject** control characters and zero-width characters

/// Check if a string contains only ASCII characters.
///
/// This is the simplest and most effective defense against Unicode attacks
/// in contexts where only ASCII is expected (usernames, identifiers, etc.).
pub fn is_ascii_only(input: &str) -> bool {
    todo!("Check if string is ASCII-only")
}

/// Check for right-to-left override characters.
///
/// Detects:
/// - U+202E (RIGHT-TO-LEFT OVERRIDE)
/// - U+202D (LEFT-TO-RIGHT OVERRIDE)
/// - U+2066 (LEFT-TO-RIGHT ISOLATE)
/// - U+2067 (RIGHT-TO-LEFT ISOLATE)
/// - U+202B (RIGHT-TO-LEFT EMBEDDING)
/// - U+202A (LEFT-TO-RIGHT EMBEDDING)
pub fn contains_bidi_override(input: &str) -> bool {
    todo!("Detect bidirectional override characters")
}

/// Check for zero-width and invisible characters.
///
/// Detects:
/// - U+200B (ZERO WIDTH SPACE)
/// - U+200C (ZERO WIDTH NON-JOINER)
/// - U+200D (ZERO WIDTH JOINER)
/// - U+FEFF (BOM / ZERO WIDTH NO-BREAK SPACE)
/// - U+2060 (WORD JOINER)
/// - U+00AD (SOFT HYPHEN)
/// - U+034F (COMBINING GRAPHEME JOINER)
/// - U+2800 (BRAILLE PATTERN BLANK)
/// - U+180E (MONGOLIAN VOWEL SEPARATOR)
pub fn contains_zero_width_chars(input: &str) -> bool {
    todo!("Detect zero-width and invisible characters")
}

/// Detect characters that are commonly used in homoglyph attacks.
///
/// Checks if the string contains non-ASCII characters that visually
/// resemble ASCII characters (Cyrillic, Greek, etc.).
///
/// A simple heuristic: if the string is supposed to be ASCII-like but
/// contains code points in these ranges, flag it:
/// - Cyrillic: U+0400..=U+04FF
/// - Greek and Coptic: U+0370..=U+03FF
/// - Armenian: U+0530..=U+058F
/// - Fullwidth Latin: U+FF01..=U+FF5E
/// - Mathematical Alphanumeric: U+1D400..=U+1D7FF
///
/// Returns true if suspicious non-ASCII characters are found.
pub fn detect_homoglyphs(input: &str) -> bool {
    todo!("Detect homoglyph attack characters")
}

/// Normalize Unicode input to NFC form.
///
/// This collapses combining character sequences into their precomposed
/// equivalents where possible.
///
/// Since we can't use external Unicode normalization crates, implement a
/// simplified version that:
/// 1. Strips all combining characters (Unicode category M: U+0300..=U+036F)
/// 2. Removes zero-width characters
/// 3. Removes bidirectional overrides
/// 4. Returns the cleaned string
pub fn normalize_unicode(input: &str) -> String {
    todo!("Normalize Unicode input")
}

/// Validate a username using Unicode-safe rules.
///
/// A valid username:
/// - Contains only ASCII alphanumeric, underscores, and hyphens
/// - Is 3-32 characters long
/// - Starts with an alphabetic character
/// - Contains no homoglyphs, bidi overrides, or zero-width characters
///
/// Returns Ok(username) if valid, Err(message) if invalid.
pub fn validate_unicode_username(input: &str) -> Result<&str, String> {
    todo!("Validate username with Unicode safety checks")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_only_clean() {
        assert!(is_ascii_only("hello_world-42"));
    }

    #[test]
    fn test_ascii_only_cyrillic() {
        // Cyrillic 'а' (U+0430) looks like Latin 'a'
        assert!(!is_ascii_only("аdmin"));
    }

    #[test]
    fn test_bidi_override() {
        let malicious = format!("fun\u{202E}gpj.exe");
        assert!(contains_bidi_override(&malicious));
    }

    #[test]
    fn test_bidi_clean() {
        assert!(!contains_bidi_override("normal_file.txt"));
    }

    #[test]
    fn test_zero_width_space() {
        assert!(contains_zero_width_chars("hello\u{200B}world"));
    }

    #[test]
    fn test_zero_width_clean() {
        assert!(!contains_zero_width_chars("normal text"));
    }

    #[test]
    fn test_homoglyph_cyrillic_a() {
        // Cyrillic а (U+0430) looks like Latin a (U+0061)
        assert!(detect_homoglyphs("аpple"));
    }

    #[test]
    fn test_homoglyph_clean() {
        assert!(!detect_homoglyphs("apple"));
    }

    #[test]
    fn test_normalize_strips_combining() {
        // e + combining acute accent
        let input = "caf\u{0301}e";
        let result = normalize_unicode(input);
        assert!(!result.contains('\u{0301}'));
    }

    #[test]
    fn test_validate_unicode_username_clean() {
        assert!(validate_unicode_username("alice_99").is_ok());
    }

    #[test]
    fn test_validate_unicode_username_cyrillic() {
        assert!(validate_unicode_username("аlice").is_err());
    }

    #[test]
    fn test_validate_unicode_username_bidi() {
        let malicious = format!("user\u{202E}name");
        assert!(validate_unicode_username(&malicious).is_err());
    }
}
