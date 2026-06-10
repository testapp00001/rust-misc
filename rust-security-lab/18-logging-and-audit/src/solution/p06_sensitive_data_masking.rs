//! # Lesson 06: Sensitive Data Masking (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

#[derive(Debug, Clone)]
pub enum MaskingRule {
    LastN(usize),
    FirstN(usize),
    FirstLastN(usize, usize),
    Full,
    Custom(Vec<usize>),
}

pub fn apply_mask(input: &str, rule: &MaskingRule) -> String {
    if input.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();

    match rule {
        MaskingRule::LastN(n) => {
            if *n >= len {
                return input.to_string();
            }
            let masked = "*".repeat(len - n);
            let visible: String = chars[len - n..].iter().collect();
            format!("{}{}", masked, visible)
        }
        MaskingRule::FirstN(n) => {
            if *n >= len {
                return input.to_string();
            }
            let visible: String = chars[..*n].iter().collect();
            let masked = "*".repeat(len - n);
            format!("{}{}", visible, masked)
        }
        MaskingRule::FirstLastN(first, last) => {
            if first + last >= len {
                return input.to_string();
            }
            let first_part: String = chars[..first].iter().collect();
            let masked = "*".repeat(len - first - last);
            let last_part: String = chars[len - last..].iter().collect();
            format!("{}{}{}", first_part, masked, last_part)
        }
        MaskingRule::Full => "*".repeat(len),
        MaskingRule::Custom(positions) => {
            let pos_set: std::collections::HashSet<usize> = positions.iter().copied().collect();
            chars
                .iter()
                .enumerate()
                .map(|(i, c)| if pos_set.contains(&i) { *c } else { '*' })
                .collect()
        }
    }
}

pub fn mask_email(email: &str) -> String {
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
    format!("{}***@{}", first_char, domain)
}

pub fn mask_credit_card(card: &str) -> String {
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
            result.push(ch);
        }
    }

    result
}

pub fn mask_ssn(ssn: &str) -> String {
    let digits: String = ssn.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() != 9 {
        return "*** **-**ERR".to_string();
    }
    format!("*** **-**{}", &digits[7..])
}

pub fn mask_phone(phone: &str) -> String {
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
            result.push(ch);
        }
    }

    result
}

pub fn mask_ip(ip: &str) -> String {
    let octets: Vec<&str> = ip.split('.').collect();
    if octets.len() != 4 {
        return ip.to_string();
    }
    format!("{}.{}.xxx.xxx", octets[0], octets[1])
}

pub fn mask_name(name: &str) -> String {
    name.split_whitespace()
        .map(|word| {
            if word.is_empty() {
                return String::new();
            }
            let first = word.chars().next().unwrap();
            let stars = "*".repeat(word.len() - 1);
            format!("{}{}", first, stars)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn auto_mask(input: &str) -> String {
    let digits: String = input.chars().filter(|c| c.is_ascii_digit()).collect();
    let digit_count = digits.len();

    // Email
    if input.contains('@') && input.contains('.') {
        return mask_email(input);
    }

    // IP address: 4 dot-separated numbers
    let parts: Vec<&str> = input.split('.').collect();
    if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
        return mask_ip(input);
    }

    // SSN: 9 digits
    if digit_count == 9 {
        return mask_ssn(input);
    }

    // Credit card: 13-19 digits
    if digit_count >= 13 && digit_count <= 19 {
        return mask_credit_card(input);
    }

    // Phone: 10 digits
    if digit_count == 10 {
        return mask_phone(input);
    }

    // Default: show last 4
    apply_mask(input, &MaskingRule::LastN(4))
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
