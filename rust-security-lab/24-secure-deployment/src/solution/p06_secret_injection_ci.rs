//! # Lesson 06: Secret Injection in CI (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecretLeak {
    pub location: String,
    pub leak_type: String,
    pub key: String,
    pub severity: String,
}

fn is_secret_key(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    let secret_words = ["password", "secret", "key", "token", "credential", "api_key", "private"];
    secret_words.iter().any(|w| key_lower.contains(w))
}

/// Scan Dockerfile lines for hardcoded secrets.
pub fn scan_dockerfile(dockerfile_lines: &[&str]) -> Vec<SecretLeak> {
    let mut leaks = Vec::new();

    for (i, line) in dockerfile_lines.iter().enumerate() {
        let trimmed = line.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let directive = parts[0].to_uppercase();

        if directive == "ENV" || directive == "ARG" {
            let leak_type = if directive == "ENV" { "env_var" } else { "build_arg" };
            // Handle: ENV KEY=value or ENV KEY1=val1 KEY2=val2
            for part in &parts[1..] {
                if let Some(eq_pos) = part.find('=') {
                    let key = &part[..eq_pos];
                    if is_secret_key(key) {
                        leaks.push(SecretLeak {
                            location: format!("line {}", i + 1),
                            leak_type: leak_type.to_string(),
                            key: key.to_string(),
                            severity: "critical".to_string(),
                        });
                    }
                }
            }
        }
    }

    leaks
}

/// Check that all required secrets are available from runtime sources.
pub fn check_secret_availability(
    required_secrets: &[&str],
    available_sources: &[(String, String)],
) -> Vec<String> {
    let available: Vec<&str> = available_sources.iter().map(|(_, name)| name.as_str()).collect();

    required_secrets.iter()
        .filter(|secret| !available.contains(secret))
        .map(|s| s.to_string())
        .collect()
}

/// Generate a secure Dockerfile snippet.
pub fn generate_secure_dockerfile(app_name: &str) -> String {
    format!(
        "FROM rust:1.75-slim AS builder\n\
         WORKDIR /app\n\
         COPY . .\n\
         RUN cargo build --release --locked\n\
         FROM gcr.io/distroless/static-debian12\n\
         COPY --from=builder /app/target/release/{app_name} /{app_name}\n\
         USER 65534:65534\n\
         HEALTHCHECK CMD [\"/{app_name}\", \"--health\"]\n\
         ENTRYPOINT [\"/{app_name}\"]"
    )
}

/// Heuristically detect if a value is a secret.
pub fn looks_like_secret(value: &str) -> bool {
    // Check known prefixes
    let prefixes = ["sk-", "pk-", "ghp_", "gho_", "AKIA", "xoxb-", "xoxp-"];
    for prefix in &prefixes {
        if value.starts_with(prefix) {
            return true;
        }
    }

    // Check for long hex strings (32+ chars)
    if value.len() >= 32 && value.chars().all(|c| c.is_ascii_hexdigit()) {
        return true;
    }

    // Check for long mixed alphanumeric (16+ chars with both letters and digits)
    if value.len() > 16 {
        let has_alpha = value.chars().any(|c| c.is_ascii_alphabetic());
        let has_digit = value.chars().any(|c| c.is_ascii_digit());
        if has_alpha && has_digit {
            // Check if it looks like base64
            if value.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
                return true;
            }
        }
    }

    // Check for UUID format
    if value.len() == 36 && value.chars().enumerate().all(|(i, c)| {
        c.is_ascii_hexdigit() || (i == 8 || i == 13 || i == 18 || i == 23) && c == '-'
    }) {
        return true;
    }

    false
}

/// Audit CI environment variables for secret leaks.
pub fn audit_ci_env(env_vars: &[(String, String)]) -> Vec<SecretLeak> {
    env_vars.iter()
        .filter(|(key, value)| is_secret_key(key) || looks_like_secret(value))
        .map(|(key, _value)| {
            let key_lower = key.to_lowercase();
            let severity = if key_lower.contains("password") || key_lower.contains("private") {
                "critical"
            } else if key_lower.contains("secret") || key_lower.contains("token")
                || key_lower.contains("key") || key_lower.contains("credential")
            {
                "high"
            } else {
                "medium"
            };
            SecretLeak {
                location: "ci_env".to_string(),
                leak_type: "ci_env_var".to_string(),
                key: key.clone(),
                severity: severity.to_string(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_dockerfile_clean() {
        let lines = vec![
            "FROM rust:1.75-slim",
            "WORKDIR /app",
            "COPY . .",
            "RUN cargo build --release",
        ];
        let leaks = scan_dockerfile(&lines);
        assert!(leaks.is_empty(), "Clean Dockerfile should have no leaks");
    }

    #[test]
    fn test_scan_dockerfile_env_secret() {
        let lines = vec![
            "FROM node:20",
            "ENV DB_PASSWORD=supersecret123",
            "RUN npm install",
        ];
        let leaks = scan_dockerfile(&lines);
        assert!(!leaks.is_empty());
        assert!(leaks.iter().any(|l| l.key.contains("DB_PASSWORD")));
    }

    #[test]
    fn test_scan_dockerfile_arg_secret() {
        let lines = vec![
            "FROM node:20",
            "ARG API_SECRET=sk-1234567890",
        ];
        let leaks = scan_dockerfile(&lines);
        assert!(!leaks.is_empty());
        assert!(leaks.iter().any(|l| l.leak_type == "build_arg"));
    }

    #[test]
    fn test_check_secret_availability_all_present() {
        let required = vec!["DB_PASSWORD", "API_KEY"];
        let sources = vec![
            ("vault".to_string(), "DB_PASSWORD".to_string()),
            ("env".to_string(), "API_KEY".to_string()),
        ];
        let missing = check_secret_availability(&required, &sources);
        assert!(missing.is_empty(), "All secrets should be available");
    }

    #[test]
    fn test_check_secret_availability_missing() {
        let required = vec!["DB_PASSWORD", "API_KEY", "SIGNING_KEY"];
        let sources = vec![
            ("vault".to_string(), "DB_PASSWORD".to_string()),
        ];
        let missing = check_secret_availability(&required, &sources);
        assert!(missing.contains(&"API_KEY".to_string()));
        assert!(missing.contains(&"SIGNING_KEY".to_string()));
    }

    #[test]
    fn test_generate_secure_dockerfile() {
        let dockerfile = generate_secure_dockerfile("myapp");
        assert!(dockerfile.contains("65534"), "Should set non-root user");
        assert!(dockerfile.contains("HEALTHCHECK"), "Should have healthcheck");
        assert!(!dockerfile.to_uppercase().contains("ENV ") || !dockerfile.contains("password"),
            "Should not contain secrets in ENV");
    }

    #[test]
    fn test_looks_like_secret_prefix() {
        assert!(looks_like_secret("sk-1234567890abcdef"));
        assert!(looks_like_secret("ghp_abcdefghijklmnopqrstuvwxyz"));
        assert!(looks_like_secret("AKIAIOSFODNN7EXAMPLE"));
    }

    #[test]
    fn test_looks_like_secret_hex() {
        assert!(looks_like_secret(&"a".repeat(64)));
        assert!(looks_like_secret(&"0123456789abcdef".repeat(2)));
    }

    #[test]
    fn test_looks_like_secret_not_secret() {
        assert!(!looks_like_secret("hello"));
        assert!(!looks_like_secret("8080"));
        assert!(!looks_like_secret("localhost"));
        assert!(!looks_like_secret("true"));
    }

    #[test]
    fn test_audit_ci_env_finds_leaks() {
        let env_vars = vec![
            ("PORT".to_string(), "8080".to_string()),
            ("DB_PASSWORD".to_string(), "supersecretvalue123".to_string()),
            ("APP_NAME".to_string(), "myapp".to_string()),
        ];
        let leaks = audit_ci_env(&env_vars);
        assert!(leaks.iter().any(|l| l.key == "DB_PASSWORD"));
        assert!(!leaks.iter().any(|l| l.key == "PORT"));
    }

    #[test]
    fn test_audit_ci_env_severity() {
        let env_vars = vec![
            ("DB_PASSWORD".to_string(), "secret123".to_string()),
            ("API_TOKEN".to_string(), "token123".to_string()),
        ];
        let leaks = audit_ci_env(&env_vars);
        let password_leak = leaks.iter().find(|l| l.key == "DB_PASSWORD").unwrap();
        assert_eq!(password_leak.severity, "critical");
        let token_leak = leaks.iter().find(|l| l.key == "API_TOKEN").unwrap();
        assert_eq!(token_leak.severity, "high");
    }
}
