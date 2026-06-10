//! # Lesson 01: PII Redaction in Logs
//!
//! ## The Problem
//!
//! Developers frequently log user data for debugging: email addresses, social
//! security numbers, credit card numbers, phone numbers. This data flows into
//! log aggregation systems (Splunk, ELK, Datadog) that are often less secure
//! than the primary database. A breach of your logging system should NOT expose
//! your users' PII.
//!
//! ## Attack Scenario
//!
//! A developer adds `log::info!("User login: {}", user.email)` for debugging.
//! Six months later, the ELK cluster is compromised. The attacker now has every
//! email address that ever logged in -- a goldmine for phishing campaigns.
//!
//! ## Defense
//!
//! PII must be redacted BEFORE it enters the logging pipeline. Use pattern
//! matching to detect and mask sensitive data:
//! - Email: `j***@***.com` (show first char and domain)
//! - SSN: `*** **-**34` (show last 2 digits)
//! - Credit card: `**** **** **** 1234` (show last 4)
//! - Phone: `***-***-7890` (show last 4)
//!
//! ## What You'll Implement
//!
//! 1. Email redaction -- mask everything except first char and domain
//! 2. SSN redaction -- mask everything except last 2 digits
//! 3. Credit card redaction -- mask everything except last 4 digits
//! 4. Phone number redaction -- mask everything except last 4 digits
//! 5. A general-purpose PII redactor that handles all types
//! 6. A log message sanitizer that redacts all PII in a free-form string

/// Exercise 1: Redact an email address.
///
/// Rules:
/// - Show the first character of the local part
/// - Mask the rest of the local part with `*`
/// - Show the domain (including TLD)
/// - Example: `john.doe@example.com` -> `j***@example.com`
///
/// Edge cases:
/// - Single char local: `a@b.com` -> `a@b.com` (nothing to mask)
/// - Empty local: return `"***@domain"` format
///
/// Hints:
/// - Split on `@` to separate local and domain
/// - Use `chars().next()` to get the first character
/// - Build the result with `format!`
pub fn redact_email(email: &str) -> String {
    todo!("Implement email redaction")
}

/// Exercise 2: Redact a Social Security Number.
///
/// Rules:
/// - SSN format: `XXX-XX-XXXX` or `XXXXXXXXX` (no dashes)
/// - Mask everything except the last 2 digits
/// - Always output with dashes: `*** **-**34`
///
/// Edge cases:
/// - Input with dashes: `123-45-6789` -> `*** **-**89`
/// - Input without dashes: `123456789` -> `*** **-**89`
/// - Invalid length: return `"*** **-**ERR"`
///
/// Hints:
/// - Strip dashes first, then check length == 9
/// - Use `&digits[7..]` to get last 2 digits
pub fn redact_ssn(ssn: &str) -> String {
    todo!("Implement SSN redaction")
}

/// Exercise 3: Redact a credit card number.
///
/// Rules:
/// - Show only the last 4 digits
/// - Mask everything else with `*`
/// - Preserve grouping (spaces or dashes) in the output
/// - Example: `4111 1111 1111 1111` -> `**** **** **** 1111`
/// - Example: `4111-1111-1111-1111` -> `****-****-****-1111`
///
/// Edge cases:
/// - No separators: `4111111111111111` -> `************1111`
/// - Too short (< 4 digits): return all `*`
///
/// Hints:
/// - Extract only digits with `filter(|c| c.is_ascii_digit())`
/// - If < 4 digits, return all asterisks of same length
/// - Replace each digit except the last 4 with `*`, keep separators
pub fn redact_credit_card(card: &str) -> String {
    todo!("Implement credit card redaction")
}

/// Exercise 4: Redact a phone number.
///
/// Rules:
/// - Show only the last 4 digits
/// - Mask everything else with `*`
/// - Preserve separators (dashes, dots, spaces, parentheses)
/// - Example: `(555) 123-4567` -> `(***) ***-4567`
/// - Example: `555-123-4567` -> `***-***-4567`
///
/// Hints:
/// - Same approach as credit card: track digit positions
/// - Replace non-last-4 digits with `*`, keep separators
pub fn redact_phone(phone: &str) -> String {
    todo!("Implement phone number redaction")
}

/// Exercise 5: General-purpose PII redactor.
///
/// Detect the type of PII and redact accordingly:
/// - If it contains `@` and `.` -> email
/// - If it's 9 digits (with or without dashes) -> SSN
/// - If it's 13-19 digits (with or without separators) -> credit card
/// - If it's 10 digits (with or without separators) -> phone
/// - Otherwise -> return unchanged
///
/// Hints:
/// - Count digits with `chars().filter(|c| c.is_ascii_digit()).count()`
/// - Check for `@` to detect email
/// - Delegate to the specific redaction functions
pub fn redact_pii(value: &str) -> String {
    todo!("Implement general PII redaction")
}

/// Exercise 6: Sanitize a log message by redacting all PII.
///
/// Scan the message for PII patterns and redact each one:
/// - Email pattern: word containing `@` and `.`
/// - SSN pattern: `\d{3}-\d{2}-\d{4}`
/// - Credit card pattern: 13-19 digits, possibly separated by spaces/dashes
/// - Phone pattern: 10 digits, possibly with separators
///
/// This is the function you'd call before every `log::info!()` call.
///
/// Hints:
/// - Split the message into tokens (by whitespace)
/// - Apply `redact_pii` to each token
/// - Rejoin with spaces
/// - For production, use regex; for this exercise, token-based is fine
pub fn sanitize_log_message(message: &str) -> String {
    todo!("Implement log message sanitization")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_email_standard() {
        assert_eq!(redact_email("john.doe@example.com"), "j***@example.com");
    }

    #[test]
    fn test_redact_email_short_local() {
        assert_eq!(redact_email("a@b.com"), "a@b.com");
    }

    #[test]
    fn test_redact_ssn_with_dashes() {
        assert_eq!(redact_ssn("123-45-6789"), "*** **-**89");
    }

    #[test]
    fn test_redact_ssn_no_dashes() {
        assert_eq!(redact_ssn("123456789"), "*** **-**89");
    }

    #[test]
    fn test_redact_credit_card_with_spaces() {
        assert_eq!(redact_credit_card("4111 1111 1111 1111"), "**** **** **** 1111");
    }

    #[test]
    fn test_redact_credit_card_no_separators() {
        assert_eq!(redact_credit_card("4111111111111111"), "************1111");
    }

    #[test]
    fn test_redact_phone_with_parens() {
        assert_eq!(redact_phone("(555) 123-4567"), "(***) ***-4567");
    }

    #[test]
    fn test_redact_phone_dashes() {
        assert_eq!(redact_phone("555-123-4567"), "***-***-4567");
    }

    #[test]
    fn test_redact_pii_auto_detect_email() {
        let result = redact_pii("john@example.com");
        assert!(result.contains("***"), "Should redact email");
        assert!(result.contains("@example.com"), "Should preserve domain");
    }

    #[test]
    fn test_redact_pii_auto_detect_ssn() {
        let result = redact_pii("123-45-6789");
        assert_eq!(result, "*** **-**89");
    }

    #[test]
    fn test_sanitize_log_message() {
        let msg = "User john@example.com logged in with SSN 123-45-6789";
        let sanitized = sanitize_log_message(msg);
        assert!(!sanitized.contains("john@example.com"), "Email should be redacted");
        assert!(!sanitized.contains("6789") || sanitized.contains("**89"),
            "Full SSN should not appear");
    }

    #[test]
    fn test_sanitize_no_pii() {
        let msg = "Server started on port 8080";
        assert_eq!(sanitize_log_message(msg), msg);
    }
}
