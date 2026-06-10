//! # Lesson 02: Secret Scanning in Code
//!
//! ## The Problem
//!
//! Developers accidentally commit secrets to version control every day:
//! - API keys in configuration files
//! - Database passwords in connection strings
//! - Private keys in test fixtures
//! - Tokens in CI/CD pipeline scripts
//!
//! Once a secret is in a git repository, it's in the history forever — even if you
//! delete it in a later commit. Anyone with access to the repo can find it.
//!
//! ## Attack Demo: Finding Secrets in Git History
//!
//! ```bash
//! # Search entire git history for potential secrets
//! git log -p --all | grep -i "api_key\|password\|secret\|token"
//!
//! # Even "deleted" secrets are recoverable
//! git log --diff-filter=D --summary | grep secret_file
//! git show <commit>:path/to/deleted/file
//! ```
//!
//! ## Defense: Secret Scanning
//!
//! Scan code for patterns that look like secrets before they're committed:
//! - **Pre-commit hooks**: Block commits containing secrets
//! - **CI/CD scanning**: Detect secrets in pull requests
//! - **Repository scanning**: Audit existing history
//!
//! Common patterns to detect:
//! - AWS keys: `AKIA[0-9A-Z]{16}`
//! - Generic API keys: long alphanumeric strings assigned to "key" variables
//! - Private keys: `-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----`
//! - Passwords: `password=`, `passwd=`, `pwd=` followed by a value
//!
//! ## Tools
//!
//! - `trufflehog` — scans git history with high accuracy
//! - `gitleaks` — fast, TOML-configurable
//! - `detect-secrets` — Yelp's pre-commit focused scanner
//! - GitHub secret scanning — built into GitHub

/// A detected secret finding.
#[derive(Debug, Clone, PartialEq)]
pub struct SecretFinding {
    /// The line number where the secret was found (1-indexed).
    pub line: usize,
    /// The type of secret detected (e.g., "api_key", "password", "private_key").
    pub secret_type: String,
    /// The matched text (should be masked in output).
    pub matched_text: String,
    /// Confidence level: "high", "medium", or "low".
    pub confidence: String,
}

/// Exercise 1: Scan a line of code for potential API keys.
///
/// Look for patterns like:
/// - `api_key = "..."` or `apikey = "..."`
/// - `API_KEY=...` (environment variable style)
/// - Assignment to variables containing "key" with a long string value (>=20 chars)
///
/// Return `Some(SecretFinding)` if a potential API key is found, `None` otherwise.
///
/// Hints:
/// - Convert line to lowercase for pattern matching
/// - Check if the line contains "api_key" or "apikey" or "api-key"
/// - Look for `= "..."` or `= '...'` patterns
/// - The value should be at least 20 characters to reduce false positives
pub fn scan_for_api_key(line: &str, line_number: usize) -> Option<SecretFinding> {
    todo!("Scan a line for potential API key patterns")
}

/// Exercise 2: Scan a line for potential passwords.
///
/// Look for patterns like:
/// - `password = "..."` or `passwd = "..."`
/// - `PASSWORD=...` (environment variable style)
/// - `password:` in YAML/JSON
///
/// Return `Some(SecretFinding)` if a potential password is found, `None` otherwise.
///
/// Hints:
/// - Check for "password", "passwd", "pwd" (case-insensitive)
/// - Look for assignment patterns: `= "..."`, `: "..."`, `= '...'`
/// - Exclude obvious non-secrets: "password" in comments about "change your password"
pub fn scan_for_password(line: &str, line_number: usize) -> Option<SecretFinding> {
    todo!("Scan a line for potential password patterns")
}

/// Exercise 3: Scan a line for private key headers.
///
/// Look for PEM-encoded private key headers:
/// - `-----BEGIN RSA PRIVATE KEY-----`
/// - `-----BEGIN EC PRIVATE KEY-----`
/// - `-----BEGIN PRIVATE KEY-----`
/// - `-----BEGIN OPENSSH PRIVATE KEY-----`
/// - `-----BEGIN DSA PRIVATE KEY-----`
///
/// Return `Some(SecretFinding)` if a private key header is found, `None` otherwise.
///
/// Hints:
/// - Look for "-----BEGIN" and "PRIVATE KEY" on the same line
/// - This is a high-confidence finding — private key headers are never legitimate in source code
pub fn scan_for_private_key(line: &str, line_number: usize) -> Option<SecretFinding> {
    todo!("Scan for private key PEM headers")
}

/// Exercise 4: Scan a line for generic tokens and bearer credentials.
///
/// Look for patterns like:
/// - `token = "..."` or `TOKEN=...`
/// - `bearer <token>` or `Bearer <token>`
/// - `authorization: Bearer ...`
///
/// Return `Some(SecretFinding)` if a potential token is found, `None` otherwise.
///
/// Hints:
/// - Check for "token" or "bearer" (case-insensitive)
/// - For bearer tokens, look for "bearer " followed by a non-empty value
/// - For token assignments, look for `= "..."` with a value >= 16 characters
pub fn scan_for_token(line: &str, line_number: usize) -> Option<SecretFinding> {
    todo!("Scan for tokens and bearer credentials")
}

/// Exercise 5: Scan a full source file (multiple lines) for secrets.
///
/// Run all the individual scanners on each line and collect all findings.
///
/// Hints:
/// - Split the input by newlines
/// - For each line, run all four scanners (api_key, password, private_key, token)
/// - Collect all `Some(findings)` into a Vec
/// - Return the Vec of all findings
pub fn scan_source_file(contents: &str) -> Vec<SecretFinding> {
    todo!("Scan an entire source file for secrets")
}

/// Exercise 6: Generate a scan report from findings.
///
/// Given a list of findings, produce a human-readable report string.
///
/// Format:
/// ```
/// Secret Scan Report
/// ==================
/// Found N potential secret(s):
///
/// [HIGH] Line L: PRIVATE_KEY found
///   Match: -----BEGIN RSA PRIV...
///
/// [MEDIUM] Line L: API_KEY found
///   Match: sk-1234...efgh
/// ```
///
/// The matched text in the report should be masked: show first 8 characters + "..."
/// if longer than 12 characters, otherwise show the full text.
///
/// If no findings, return "No secrets detected."
///
/// Hints:
/// - Iterate over findings
/// - Format each one with its confidence level, line number, type
/// - Mask the matched text for the report
pub fn generate_report(findings: &[SecretFinding]) -> String {
    todo!("Generate a human-readable scan report")
}

/// Exercise 7: Extract a quoted value from a line of code.
///
/// Given a line like `api_key = "sk-1234567890"`, extract the value between quotes.
/// Support both single and double quotes.
///
/// Return `Some(value)` if a quoted value is found, `None` otherwise.
///
/// Hints:
/// - Find the first occurrence of `"` or `'`
/// - Find the matching closing quote
/// - Extract the text between them
pub fn extract_quoted_value(line: &str) -> Option<String> {
    todo!("Extract a quoted value from a line of code")
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
        // Short values should not trigger (too many false positives)
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
