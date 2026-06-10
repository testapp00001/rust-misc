//! # Lesson 01: PII Redaction in Logs (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// Redact an email address: `john.doe@example.com` -> `j***@example.com`
pub fn redact_email(email: &str) -> String {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return "***@***".to_string();
    }
    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() {
        return format!("***@{}", domain);
    }

    let first_char = local.chars().next().unwrap();
    if local.len() == 1 {
        return format!("{}@{}", first_char, domain);
    }

    format!("{}***@{}", first_char, domain)
}

/// Redact an SSN: `123-45-6789` or `123456789` -> `*** **-**89`
pub fn redact_ssn(ssn: &str) -> String {
    let digits: String = ssn.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 9 {
        return "*** **-**ERR".to_string();
    }
    format!("*** **-**{}", &digits[7..])
}

/// Redact a credit card: `4111 1111 1111 1111` -> `**** **** **** 1111`
pub fn redact_credit_card(card: &str) -> String {
    let digits: Vec<char> = card.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 4 {
        return "*".repeat(card.len());
    }

    let mut result = String::new();
    let mut digit_index = 0;
    let total_digits = digits.len();

    for ch in card.chars() {
        if ch.is_ascii_digit() {
            if digit_index < total_digits - 4 {
                result.push('*');
            } else {
                result.push(ch);
            }
            digit_index += 1;
        } else {
            // Keep separators (spaces, dashes)
            result.push(ch);
        }
    }

    // Handle case with no separators
    if !card.chars().any(|c| !c.is_ascii_digit() && !c.is_whitespace()) && !card.contains(' ') {
        // Pure digits, no separators
        let masked_count = total_digits - 4;
        let masked: String = "*".repeat(masked_count);
        let last_four: String = digits[total_digits - 4..].iter().collect();
        format!("{}{}", masked, last_four)
    } else {
        result
    }
}

/// Redact a phone number: `(555) 123-4567` -> `(***) ***-4567`
pub fn redact_phone(phone: &str) -> String {
    let digits: Vec<char> = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() < 4 {
        return "*".repeat(phone.len());
    }

    let mut result = String::new();
    let mut digit_index = 0;
    let total_digits = digits.len();

    for ch in phone.chars() {
        if ch.is_ascii_digit() {
            if digit_index < total_digits - 4 {
                result.push('*');
            } else {
                result.push(ch);
            }
            digit_index += 1;
        } else {
            // Keep separators
            result.push(ch);
        }
    }

    result
}

/// Detect PII type and redact accordingly.
pub fn redact_pii(value: &str) -> String {
    let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
    let digit_count = digits.len();

    // Check for email (contains @ and .)
    if value.contains('@') && value.contains('.') {
        return redact_email(value);
    }

    // Check for SSN: 9 digits, possibly with dashes
    if digit_count == 9 && (value.contains('-') || value.len() == 9) {
        return redact_ssn(value);
    }

    // Check for credit card: 13-19 digits
    if digit_count >= 13 && digit_count <= 19 {
        return redact_credit_card(value);
    }

    // Check for phone: 10 digits
    if digit_count == 10 {
        return redact_phone(value);
    }

    // Not recognized PII -- return unchanged
    value.to_string()
}

/// Sanitize a log message by redacting all PII tokens.
pub fn sanitize_log_message(message: &str) -> String {
    message
        .split_whitespace()
        .map(|token| redact_pii(token))
        .collect::<Vec<_>>()
        .join(" ")
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
