//! # Lesson 06: Secret Injection in CI
//!
//! ## Attack: Baked Secrets
//!
//! A developer accidentally commits a Dockerfile with `ENV API_KEY=sk-12345`.
//! Even after removing the line, the secret persists in:
//! - Docker image layers (accessible via `docker history`)
//! - Git history (accessible via `git log --all`)
//! - CI build logs
//! - Container registries
//!
//! The attacker scans public Docker registries for leaked secrets and gains
//! access to production databases, payment processors, and cloud accounts.
//!
//! ## Defend: Runtime Secret Injection
//!
//! Never embed secrets in images. Instead:
//! - Use Docker BuildKit secrets (`--secret`) during build
//! - Use environment variables from a secret manager at runtime
//! - Use volume-mounted secret files
//! - Use Kubernetes Secrets with RBAC
//!
//! ## Audit: Secret Detection Checklist
//!
//! - [ ] No secrets in Dockerfile ENV or ARG directives
//! - [ ] No secrets in docker-compose.yml environment section
//! - [ ] No secrets in CI pipeline YAML files
//! - [ ] .dockerignore excludes credential files
//! - [ ] Git history is clean of secrets (use git-secrets, truffleHog)
//! - [ ] Image layers don't contain secrets (use dive, docker history)

use serde::{Deserialize, Serialize};

/// Represents a potential secret leak in a deployment configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecretLeak {
    /// Where the secret was found
    pub location: String,
    /// Type of leak (e.g., "env_var", "file_content", "build_arg")
    pub leak_type: String,
    /// Name of the leaked secret
    pub key: String,
    /// Severity: "critical", "high", "medium", "low"
    pub severity: String,
}

/// Exercise 1: Scan a Dockerfile for hardcoded secrets.
///
/// Parse lines of a Dockerfile and detect these patterns:
/// - `ENV KEY=value` where KEY contains secret-related words
///   ("password", "secret", "key", "token", "credential", "api_key", "private")
/// - `ARG KEY=value` where KEY contains the same words
///
/// Return a `SecretLeak` for each finding. The `location` should be the line number
/// as a string (e.g., "line 5"). The `leak_type` should be "env_var" or "build_arg".
///
/// Hints:
/// - Split each line by whitespace
/// - Check if the first token is "ENV" or "ARG"
/// - For ENV, the format is `ENV KEY=value` (single pair) or `ENV KEY1=val1 KEY2=val2`
/// - Split the key=value part on '=' to get the key
pub fn scan_dockerfile(dockerfile_lines: &[&str]) -> Vec<SecretLeak> {
    todo!("Scan Dockerfile lines for hardcoded secrets")
}

/// Exercise 2: Validate that secrets are properly injected.
///
/// Given a list of required secret names and a list of available secret sources,
/// verify all required secrets are available at runtime (not baked in).
///
/// `required_secrets`: names of secrets the app needs (e.g., ["DB_PASSWORD", "API_KEY"])
/// `available_sources`: tuples of (source_type, source_name) where source_type is
///   "env", "file", "vault", "k8s_secret"
///
/// Return the names of secrets that are NOT available from any source.
pub fn check_secret_availability(
    required_secrets: &[&str],
    available_sources: &[(String, String)],
) -> Vec<String> {
    todo!("Check that all required secrets are available from runtime sources")
}

/// Exercise 3: Generate a secure Dockerfile snippet.
///
/// Given an app name, return a multi-line Dockerfile string that:
/// 1. Uses multi-stage build
/// 2. Runs as non-root user (65534)
/// 3. Has a HEALTHCHECK
/// 4. Does NOT contain any ENV with secrets
///
/// The format should be:
/// ```text
/// FROM rust:1.75-slim AS builder
/// WORKDIR /app
/// COPY . .
/// RUN cargo build --release --locked
/// FROM gcr.io/distroless/static-debian12
/// COPY --from=builder /app/target/release/{app_name} /{app_name}
/// USER 65534:65534
/// HEALTHCHECK CMD ["/{app_name}", "--health"]
/// ENTRYPOINT ["/{app_name}"]
/// ```
pub fn generate_secure_dockerfile(app_name: &str) -> String {
    todo!("Generate a secure Dockerfile snippet")
}

/// Exercise 4: Check if a value looks like a secret.
///
/// A value is likely a secret if ANY of these are true:
/// - It's longer than 16 characters AND contains both letters and digits
/// - It starts with common secret prefixes: "sk-", "pk-", "ghp_", "gho_",
///   "AKIA", "xoxb-", "xoxp-"
/// - It's a valid base64 string longer than 32 characters
/// - It matches common patterns: hex string of 32+ chars, UUID format
pub fn looks_like_secret(value: &str) -> bool {
    todo!("Heuristically detect if a value is a secret")
}

/// Exercise 5: Audit CI environment variables for leaks.
///
/// Given a list of (variable_name, variable_value) pairs from a CI pipeline,
/// find any that look like they contain secrets.
///
/// Return a list of `SecretLeak` entries. The `location` should be "ci_env".
/// The `leak_type` should be "ci_env_var".
/// Severity: "critical" if the key contains "password" or "private",
/// "high" for "secret", "token", "key", "credential",
/// "medium" if the value looks like a secret but the key is benign.
pub fn audit_ci_env(env_vars: &[(String, String)]) -> Vec<SecretLeak> {
    todo!("Audit CI environment variables for secret leaks")
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
        assert!(looks_like_secret(&"a".repeat(64))); // 64-char hex string
        assert!(looks_like_secret(&"0123456789abcdef".repeat(2))); // 32-char hex
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
