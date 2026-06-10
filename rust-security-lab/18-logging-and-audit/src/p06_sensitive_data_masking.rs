//! # Lesson 06: Sensitive Data Masking
//!
//! ## The Problem
//!
//! Sometimes you NEED to log some identifying information for debugging, but you
//! can't log the full value. A support engineer might need to know which account
//! had an error, but they don't need the full account number. The solution is
//! **partial masking** -- showing just enough to identify without exposing.
//!
//! ## Masking Rules
//!
//! Different data types have different masking conventions:
//! - **Credit card**: `**** **** **** 1234` (last 4 digits)
//! - **SSN**: `*** **-**34` (last 2 digits)
//! - **Email**: `j***@example.com` (first char + domain)
//! - **Phone**: `***-***-7890` (last 4 digits)
//! - **Account number**: `****5678` (last 4 digits)
//! - **IP address**: `192.168.xxx.xxx` (mask last two octets)
//! - **Name**: `J*** D**` (first letter of each word)
//!
//! ## Defense in Depth
//!
//! Masking is a defense-in-depth measure. Even if an attacker gains access to your
//! logs, they get masked values, not raw PII. This is especially important for:
//! - Log aggregation systems with broader access
//! - Support ticket systems that capture log excerpts
//! - Error reporting services (Sentry, Datadog)
//!
//! ## What You'll Implement
//!
//! 1. A `MaskingRule` enum for different masking strategies
//! 2. Apply masking rules to strings
//! 3. A configurable masking engine
//! 4. Context-aware masking (detect data type and apply appropriate rule)
//! 5. Partial reveal for different data types
//! 6. Batch masking for structured data

/// Masking strategies for different data types.
#[derive(Debug, Clone)]
pub enum MaskingRule {
    /// Show only the last N characters, mask the rest with `*`
    LastN(usize),
    /// Show only the first N characters, mask the rest with `*`
    FirstN(usize),
    /// Show first and last N characters, mask the middle
    FirstLastN(usize, usize),
    /// Mask everything
    Full,
    /// Custom: show specific character positions
    Custom(Vec<usize>), // positions to reveal (0-indexed)
}

/// Exercise 1: Apply a masking rule to a string.
///
/// Rules:
/// - `LastN(n)`: Replace all but last n chars with `*`
/// - `FirstN(n)`: Replace all but first n chars with `*`
/// - `FirstLastN(f, l)`: Show first f and last l, mask middle
/// - `Full`: Replace all chars with `*`
/// - `Custom(positions)`: Show chars at given positions, mask rest
///
/// Edge cases:
/// - If n > string length, return the original string
/// - Empty string -> empty string
///
/// Hints:
/// - Convert to char vec for indexing
/// - Build result char by char
pub fn apply_mask(input: &str, rule: &MaskingRule) -> String {
    todo!("Implement masking rule application")
}

/// Exercise 2: Mask an email address.
///
/// Show first character of local part + `***@` + full domain.
/// `john.doe@example.com` -> `j***@example.com`
///
/// Hints:
/// - Split on `@`
/// - Take first char of local part
pub fn mask_email(email: &str) -> String {
    todo!("Implement email masking")
}

/// Exercise 3: Mask a credit card number.
///
/// Show only the last 4 digits, mask everything else.
/// Preserve the original separator style (spaces or dashes).
///
/// `4111 1111 1111 1111` -> `**** **** **** 1111`
///
/// Hints:
/// - Track digit positions
/// - Replace non-last-4 digits with `*`
/// - Keep separators as-is
pub fn mask_credit_card(card: &str) -> String {
    todo!("Implement credit card masking")
}

/// Exercise 4: Mask a Social Security Number.
///
/// Show only last 2 digits: `*** **-**34`
///
/// Hints:
/// - Strip dashes, get last 2 digits
/// - Format as `*** **-**XX`
pub fn mask_ssn(ssn: &str) -> String {
    todo!("Implement SSN masking")
}

/// Exercise 5: Mask a phone number.
///
/// Show only last 4 digits, preserve separators.
/// `(555) 123-4567` -> `(***) ***-4567`
///
/// Hints:
/// - Same approach as credit card
pub fn mask_phone(phone: &str) -> String {
    todo!("Implement phone masking")
}

/// Exercise 6: Mask an IP address.
///
/// Mask the last two octets: `192.168.1.100` -> `192.168.xxx.xxx`
///
/// Hints:
/// - Split on `.`
/// - Keep first two octets, replace last two with `xxx`
pub fn mask_ip(ip: &str) -> String {
    todo!("Implement IP address masking")
}

/// Exercise 7: Mask a person's name.
///
/// Show first letter of each word, mask the rest with `*`.
/// `John Doe` -> `J*** D**`
///
/// Hints:
/// - Split by whitespace
/// - For each word: first char + `*` repeated (len - 1) times
pub fn mask_name(name: &str) -> String {
    todo!("Implement name masking")
}

/// Exercise 8: Auto-detect data type and apply appropriate masking.
///
/// Detection rules:
/// - Contains `@` and `.` -> email
/// - 9 digits (with/without dashes) -> SSN
/// - 13-19 digits -> credit card
/// - 10 digits -> phone
/// - 4 dot-separated numbers -> IP address
/// - Otherwise -> apply LastN(4)
///
/// Hints:
/// - Count digits
/// - Check for `@`, dots
/// - Delegate to specific masking functions
pub fn auto_mask(input: &str) -> String {
    todo!("Implement auto-detection masking")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_last_n() {
        assert_eq!(apply_mask("1234567890", &MaskingRule::LastN(4)), "******7890");
    }

    #[test]
    fn test_mask_first_n() {
        assert_eq!(apply_mask("1234567890", &MaskingRule::FirstN(2)), "12********");
    }

    #[test]
    fn test_mask_full() {
        assert_eq!(apply_mask("secret", &MaskingRule::Full), "******");
    }

    #[test]
    fn test_mask_email() {
        assert_eq!(mask_email("john@example.com"), "j***@example.com");
    }

    #[test]
    fn test_mask_credit_card_spaces() {
        assert_eq!(mask_credit_card("4111 1111 1111 1111"), "**** **** **** 1111");
    }

    #[test]
    fn test_mask_ssn() {
        assert_eq!(mask_ssn("123-45-6789"), "*** **-**89");
    }

    #[test]
    fn test_mask_phone() {
        assert_eq!(mask_phone("(555) 123-4567"), "(***) ***-4567");
    }

    #[test]
    fn test_mask_ip() {
        assert_eq!(mask_ip("192.168.1.100"), "192.168.xxx.xxx");
    }

    #[test]
    fn test_mask_name() {
        assert_eq!(mask_name("John Doe"), "J*** D**");
    }

    #[test]
    fn test_auto_mask_email() {
        let result = auto_mask("john@example.com");
        assert!(result.contains("@example.com"));
        assert!(result.starts_with("j***"));
    }

    #[test]
    fn test_auto_mask_ssn() {
        let result = auto_mask("123-45-6789");
        assert_eq!(result, "*** **-**89");
    }

    #[test]
    fn test_auto_mask_ip() {
        let result = auto_mask("10.0.1.200");
        assert_eq!(result, "10.0.xxx.xxx");
    }
}
