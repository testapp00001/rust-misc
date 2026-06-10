//! # Lesson 02: Secret Scanning in Code (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

/// A detected secret finding.
#[derive(Debug, Clone, PartialEq)]
pub struct SecretFinding {
    pub line: usize,
    pub secret_type: String,
    pub matched_text: String,
    pub confidence: String,
}

pub fn scan_for_api_key(line: &str, line_number: usize) -> Option<SecretFinding> {
    let lower = line.to_lowercase();
    if !(lower.contains("api_key") || lower.contains("apikey") || lower.contains("api-key")) {
        return None;
    }
    if let Some(value) = extract_quoted_value(line) {
        if value.len() >= 20 {
            return Some(SecretFinding {
                line: line_number,
                secret_type: "api_key".to_string(),
                matched_text: value,
                confidence: "medium".to_string(),
            });
        }
    }
    // Env-style: API_KEY=value
    if let Some(eq_pos) = line.find('=') {
        let value = line[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'');
        if value.len() >= 20 {
            return Some(SecretFinding {
                line: line_number,
                secret_type: "api_key".to_string(),
                matched_text: value.to_string(),
                confidence: "medium".to_string(),
            });
        }
    }
    None
}

pub fn scan_for_password(line: &str, line_number: usize) -> Option<SecretFinding> {
    let lower = line.to_lowercase();
    if !(lower.contains("password") || lower.contains("passwd") || lower.contains("pwd=")) {
        return None;
    }
    if let Some(value) = extract_quoted_value(line) {
        if !value.is_empty() {
            return Some(SecretFinding {
                line: line_number,
                secret_type: "password".to_string(),
                matched_text: value,
                confidence: "medium".to_string(),
            });
        }
    }
    // Env-style: PASSWORD=value
    for prefix in &["password", "passwd", "pwd"] {
        if let Some(pos) = lower.find(prefix) {
            let rest = &line[pos + prefix.len()..].trim_start();
            if let Some(eq_pos) = rest.find('=') {
                let value = rest[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'');
                if !value.is_empty() {
                    return Some(SecretFinding {
                        line: line_number,
                        secret_type: "password".to_string(),
                        matched_text: value.to_string(),
                        confidence: "medium".to_string(),
                    });
                }
            }
        }
    }
    None
}

pub fn scan_for_private_key(line: &str, line_number: usize) -> Option<SecretFinding> {
    let lower = line.to_lowercase();
    if lower.contains("-----begin") && lower.contains("private key") {
        Some(SecretFinding {
            line: line_number,
            secret_type: "private_key".to_string(),
            matched_text: line.trim().to_string(),
            confidence: "high".to_string(),
        })
    } else {
        None
    }
}

pub fn scan_for_token(line: &str, line_number: usize) -> Option<SecretFinding> {
    let lower = line.to_lowercase();

    // Bearer token
    if let Some(pos) = lower.find("bearer ") {
        let value = line[pos + 7..].trim();
        if !value.is_empty() {
            return Some(SecretFinding {
                line: line_number,
                secret_type: "token".to_string(),
                matched_text: value.to_string(),
                confidence: "medium".to_string(),
            });
        }
    }

    // Token assignment
    if lower.contains("token") {
        if let Some(value) = extract_quoted_value(line) {
            if value.len() >= 16 {
                return Some(SecretFinding {
                    line: line_number,
                    secret_type: "token".to_string(),
                    matched_text: value,
                    confidence: "medium".to_string(),
                });
            }
        }
        // Env-style
        if let Some(eq_pos) = line.find('=') {
            let value = line[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'');
            if value.len() >= 16 {
                return Some(SecretFinding {
                    line: line_number,
                    secret_type: "token".to_string(),
                    matched_text: value.to_string(),
                    confidence: "medium".to_string(),
                });
            }
        }
    }
    None
}

pub fn scan_source_file(contents: &str) -> Vec<SecretFinding> {
    let mut findings = Vec::new();
    for (i, line) in contents.lines().enumerate() {
        let line_num = i + 1;
        for finding_fn in &[
            scan_for_api_key as fn(&str, usize) -> Option<SecretFinding>,
            scan_for_password,
            scan_for_private_key,
            scan_for_token,
        ] {
            if let Some(finding) = finding_fn(line, line_num) {
                findings.push(finding);
            }
        }
    }
    findings
}

pub fn generate_report(findings: &[SecretFinding]) -> String {
    if findings.is_empty() {
        return "No secrets detected.".to_string();
    }

    let mut report = format!("Secret Scan Report\n==================\nFound {} potential secret(s):\n", findings.len());
    for f in findings {
        let masked = if f.matched_text.len() > 12 {
            format!("{}...", &f.matched_text[..8])
        } else {
            f.matched_text.clone()
        };
        report.push_str(&format!(
            "\n[{}] Line {}: {} found\n  Match: {}\n",
            f.confidence.to_uppercase(),
            f.line,
            f.secret_type.to_uppercase(),
            masked
        ));
    }
    report
}

pub fn extract_quoted_value(line: &str) -> Option<String> {
    // Try double quotes first
    if let Some(start) = line.find('"') {
        if let Some(end) = line[start + 1..].find('"') {
            return Some(line[start + 1..start + 1 + end].to_string());
        }
    }
    // Try single quotes
    if let Some(start) = line.find('\'') {
        if let Some(end) = line[start + 1..].find('\'') {
            return Some(line[start + 1..start + 1 + end].to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_api_key_assignment() {
        let line = r#"api_key = "sk-1234567890abcdef1234""#;
        let finding = scan_for_api_key(line, 1);
        assert!(finding.is_some());
        let f = finding.unwrap();
        assert_eq!(f.secret_type, "api_key");
        assert_eq!(f.line, 1);
    }

    #[test]
    fn test_scan_api_key_env_style() {
        let line = "API_KEY=sk-1234567890abcdef1234";
        let finding = scan_for_api_key(line, 5);
        assert!(finding.is_some());
        assert_eq!(finding.unwrap().line, 5);
    }

    #[test]
    fn test_scan_api_key_not_too_short() {
        let line = r#"api_key = "short""#;
        let finding = scan_for_api_key(line, 1);
        assert!(finding.is_none());
    }

    #[test]
    fn test_scan_password_assignment() {
        let line = r#"password = "super_secret_password""#;
        let finding = scan_for_password(line, 3);
        assert!(finding.is_some());
        assert_eq!(finding.unwrap().secret_type, "password");
    }

    #[test]
    fn test_scan_password_env_style() {
        let line = "DB_PASSWORD=mysecretpassword123";
        let finding = scan_for_password(line, 7);
        assert!(finding.is_some());
    }

    #[test]
    fn test_scan_private_key_rsa() {
        let line = "-----BEGIN RSA PRIVATE KEY-----";
        let finding = scan_for_private_key(line, 10);
        assert!(finding.is_some());
        let f = finding.unwrap();
        assert_eq!(f.secret_type, "private_key");
        assert_eq!(f.confidence, "high");
    }

    #[test]
    fn test_scan_private_key_generic() {
        let line = "-----BEGIN PRIVATE KEY-----";
        let finding = scan_for_private_key(line, 1);
        assert!(finding.is_some());
    }

    #[test]
    fn test_scan_no_private_key_in_public_cert() {
        let line = "-----BEGIN CERTIFICATE-----";
        let finding = scan_for_private_key(line, 1);
        assert!(finding.is_none());
    }

    #[test]
    fn test_scan_token_bearer() {
        let line = "Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.token";
        let finding = scan_for_token(line, 12);
        assert!(finding.is_some());
        assert_eq!(finding.unwrap().secret_type, "token");
    }

    #[test]
    fn test_scan_source_file_multiple() {
        let contents = r#"let host = "localhost";
let api_key = "sk-1234567890abcdef1234";
let password = "hunter2secret";
-----BEGIN RSA PRIVATE KEY-----"#;
        let findings = scan_source_file(contents);
        assert!(findings.len() >= 3, "Should find api_key, password, and private_key");
    }

    #[test]
    fn test_generate_report_no_findings() {
        let report = generate_report(&[]);
        assert_eq!(report, "No secrets detected.");
    }

    #[test]
    fn test_generate_report_with_findings() {
        let findings = vec![
            SecretFinding {
                line: 5,
                secret_type: "api_key".to_string(),
                matched_text: "sk-1234567890abcdef".to_string(),
                confidence: "medium".to_string(),
            },
        ];
        let report = generate_report(&findings);
        assert!(report.contains("1 potential secret"));
        assert!(report.contains("Line 5"));
        assert!(report.contains("API_KEY") || report.contains("api_key"));
    }

    #[test]
    fn test_extract_quoted_value_double() {
        let line = r#"api_key = "sk-1234567890""#;
        let value = extract_quoted_value(line);
        assert_eq!(value, Some("sk-1234567890".to_string()));
    }

    #[test]
    fn test_extract_quoted_value_single() {
        let line = "api_key = 'sk-1234567890'";
        let value = extract_quoted_value(line);
        assert_eq!(value, Some("sk-1234567890".to_string()));
    }

    #[test]
    fn test_extract_quoted_value_none() {
        let line = "api_key = some_unquoted_value";
        let value = extract_quoted_value(line);
        assert!(value.is_none());
    }
}
