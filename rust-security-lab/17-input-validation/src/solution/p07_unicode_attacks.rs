//! # Lesson 07: Unicode Attacks (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

pub fn is_ascii_only(input: &str) -> bool {
    input.is_ascii()
}

const BIDI_CHARS: &[char] = &[
    '\u{202E}', // RIGHT-TO-LEFT OVERRIDE
    '\u{202D}', // LEFT-TO-RIGHT OVERRIDE
    '\u{2066}', // LEFT-TO-RIGHT ISOLATE
    '\u{2067}', // RIGHT-TO-LEFT ISOLATE
    '\u{202B}', // RIGHT-TO-LEFT EMBEDDING
    '\u{202A}', // LEFT-TO-RIGHT EMBEDDING
];

pub fn contains_bidi_override(input: &str) -> bool {
    input.chars().any(|c| BIDI_CHARS.contains(&c))
}

const ZERO_WIDTH_CHARS: &[char] = &[
    '\u{200B}', // ZERO WIDTH SPACE
    '\u{200C}', // ZERO WIDTH NON-JOINER
    '\u{200D}', // ZERO WIDTH JOINER
    '\u{FEFF}', // BOM / ZERO WIDTH NO-BREAK SPACE
    '\u{2060}', // WORD JOINER
    '\u{00AD}', // SOFT HYPHEN
    '\u{034F}', // COMBINING GRAPHEME JOINER
    '\u{2800}', // BRAILLE PATTERN BLANK
    '\u{180E}', // MONGOLIAN VOWEL SEPARATOR
];

pub fn contains_zero_width_chars(input: &str) -> bool {
    input.chars().any(|c| ZERO_WIDTH_CHARS.contains(&c))
}

pub fn detect_homoglyphs(input: &str) -> bool {
    for c in input.chars() {
        let cp = c as u32;
        // Cyrillic
        if (0x0400..=0x04FF).contains(&cp) {
            return true;
        }
        // Greek and Coptic
        if (0x0370..=0x03FF).contains(&cp) {
            return true;
        }
        // Armenian
        if (0x0530..=0x058F).contains(&cp) {
            return true;
        }
        // Fullwidth Latin
        if (0xFF01..=0xFF5E).contains(&cp) {
            return true;
        }
        // Mathematical Alphanumeric Symbols
        if (0x1D400..=0x1D7FF).contains(&cp) {
            return true;
        }
    }
    false
}

pub fn normalize_unicode(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            let cp = *c as u32;
            // Remove combining characters (U+0300..=U+036F)
            if (0x0300..=0x036F).contains(&cp) {
                return false;
            }
            // Remove zero-width characters
            if ZERO_WIDTH_CHARS.contains(c) {
                return false;
            }
            // Remove bidi overrides
            if BIDI_CHARS.contains(c) {
                return false;
            }
            true
        })
        .collect()
}

pub fn validate_unicode_username(input: &str) -> Result<&str, String> {
    if !is_ascii_only(input) {
        return Err("Username must contain only ASCII characters".to_string());
    }

    if contains_bidi_override(input) {
        return Err("Username contains bidirectional override characters".to_string());
    }

    if contains_zero_width_chars(input) {
        return Err("Username contains zero-width characters".to_string());
    }

    if input.len() < 3 || input.len() > 32 {
        return Err("Username must be 3-32 characters".to_string());
    }

    let first = input.chars().next().unwrap();
    if !first.is_ascii_alphabetic() {
        return Err("Username must start with an alphabetic character".to_string());
    }

    if !input
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("Username must contain only alphanumeric, underscore, or hyphen".to_string());
    }

    Ok(input)
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
