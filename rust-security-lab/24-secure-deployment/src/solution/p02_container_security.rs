//! # Lesson 02: Container Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContainerConfig {
    pub image: String,
    pub runs_as_root: bool,
    pub read_only_fs: bool,
    pub dropped_caps: Vec<String>,
    pub added_caps: Vec<String>,
    pub memory_limit_mb: u64,
    pub cpu_limit: f64,
    pub no_new_privileges: bool,
    pub pids_limit: u64,
    pub has_health_check: bool,
    pub env_vars: Vec<(String, String)>,
}

/// Audit a container configuration for security issues.
pub fn audit_container(config: &ContainerConfig) -> Vec<String> {
    let mut issues = Vec::new();

    if config.runs_as_root {
        issues.push("Container runs as root (UID 0) — use USER directive".to_string());
    }

    if !config.read_only_fs {
        issues.push("Filesystem is not read-only — use --read-only flag".to_string());
    }

    if !config.dropped_caps.iter().any(|c| c.to_uppercase() == "ALL") {
        issues.push("Not all capabilities dropped — add --cap-drop=ALL".to_string());
    }

    let dangerous_caps = ["SYS_ADMIN", "NET_ADMIN", "SYS_PTRACE", "SYS_RAWIO", "DAC_OVERRIDE"];
    for cap in &config.added_caps {
        if dangerous_caps.iter().any(|dc| dc.to_uppercase() == cap.to_uppercase()) {
            issues.push(format!("Dangerous capability added: {}", cap));
        }
    }

    if config.memory_limit_mb == 0 {
        issues.push("No memory limit set — use --memory flag".to_string());
    }

    if !config.no_new_privileges {
        issues.push("no_new_privileges not set — use --security-opt=no-new-privileges".to_string());
    }

    if !config.has_health_check {
        issues.push("No health check defined — add HEALTHCHECK directive".to_string());
    }

    for (key, _value) in &config.env_vars {
        if looks_like_secret_key(key) {
            issues.push(format!("Potential secret in env var: {}", key));
        }
    }

    issues
}

fn looks_like_secret_key(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    let secret_words = ["password", "secret", "key", "token", "credential", "auth", "api_key", "private"];
    secret_words.iter().any(|w| key_lower.contains(w))
}

fn looks_like_secret_value(value: &str) -> bool {
    if value.len() > 32 {
        let has_alpha = value.chars().any(|c| c.is_ascii_alphabetic());
        let has_digit = value.chars().any(|c| c.is_ascii_digit());
        if has_alpha && has_digit {
            return true;
        }
    }
    false
}

/// Score a container configuration (0-100).
pub fn score_container(config: &ContainerConfig) -> u32 {
    let mut score: i32 = 100;

    if config.runs_as_root { score -= 20; }
    if !config.read_only_fs { score -= 15; }
    if !config.dropped_caps.iter().any(|c| c.to_uppercase() == "ALL") { score -= 15; }

    let dangerous_caps = ["SYS_ADMIN", "NET_ADMIN", "SYS_PTRACE", "SYS_RAWIO", "DAC_OVERRIDE"];
    let dangerous_count = config.added_caps.iter()
        .filter(|c| dangerous_caps.iter().any(|dc| dc.to_uppercase() == c.to_uppercase()))
        .count();
    score -= (dangerous_count as i32).min(2) * 10;

    if config.memory_limit_mb == 0 { score -= 10; }
    if !config.no_new_privileges { score -= 10; }
    if !config.has_health_check { score -= 5; }

    let secret_count = config.env_vars.iter()
        .filter(|(k, _)| looks_like_secret_key(k))
        .count();
    score -= (secret_count as i32).min(2) * 5;

    score.max(0) as u32
}

/// Create a container config with secure defaults.
pub fn secure_defaults(image: &str) -> ContainerConfig {
    ContainerConfig {
        image: image.to_string(),
        runs_as_root: false,
        read_only_fs: true,
        dropped_caps: vec!["ALL".to_string()],
        added_caps: vec![],
        memory_limit_mb: 512,
        cpu_limit: 1.0,
        no_new_privileges: true,
        pids_limit: 256,
        has_health_check: true,
        env_vars: vec![],
    }
}

/// Validate that an image tag is safe and deterministic.
pub fn validate_image_tag(image: &str) -> Result<(), String> {
    let tag = image.split(':').nth(1).unwrap_or("");

    if tag == "latest" {
        return Err("Tag 'latest' is mutable and non-deterministic — use a specific version".to_string());
    }

    if tag.len() < 5 {
        return Err(format!("Tag '{}' is too short — likely not a specific version", tag));
    }

    let unstable = ["edge", "canary", "nightly"];
    let tag_lower = tag.to_lowercase();
    for u in &unstable {
        if tag_lower.contains(u) {
            return Err(format!("Tag '{}' contains '{}' which is an unstable release channel", tag, u));
        }
    }

    Ok(())
}

/// Detect potential secrets in environment variables.
pub fn detect_secrets_in_env(env_vars: &[(String, String)]) -> Vec<String> {
    env_vars.iter()
        .filter(|(key, value)| {
            looks_like_secret_key(key) || looks_like_secret_value(value)
        })
        .map(|(key, _)| key.clone())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secure_config() -> ContainerConfig {
        ContainerConfig {
            image: "myapp:1.2.3".to_string(),
            runs_as_root: false,
            read_only_fs: true,
            dropped_caps: vec!["ALL".to_string()],
            added_caps: vec![],
            memory_limit_mb: 512,
            cpu_limit: 1.0,
            no_new_privileges: true,
            pids_limit: 256,
            has_health_check: true,
            env_vars: vec![],
        }
    }

    fn insecure_config() -> ContainerConfig {
        ContainerConfig {
            image: "myapp:latest".to_string(),
            runs_as_root: true,
            read_only_fs: false,
            dropped_caps: vec![],
            added_caps: vec!["SYS_ADMIN".to_string()],
            memory_limit_mb: 0,
            cpu_limit: 0.0,
            no_new_privileges: false,
            pids_limit: 0,
            has_health_check: false,
            env_vars: vec![
                ("DB_PASSWORD".to_string(), "hunter2".to_string()),
            ],
        }
    }

    #[test]
    fn test_audit_secure_config() {
        let config = secure_config();
        let issues = audit_container(&config);
        assert!(issues.is_empty(), "Secure config should have no issues, got: {:?}", issues);
    }

    #[test]
    fn test_audit_root_user() {
        let mut config = secure_config();
        config.runs_as_root = true;
        let issues = audit_container(&config);
        assert!(issues.iter().any(|i| i.contains("root") || i.contains("UID")));
    }

    #[test]
    fn test_audit_readonly_fs() {
        let mut config = secure_config();
        config.read_only_fs = false;
        let issues = audit_container(&config);
        assert!(issues.iter().any(|i| i.contains("read-only") || i.contains("filesystem")));
    }

    #[test]
    fn test_audit_dangerous_caps() {
        let mut config = secure_config();
        config.added_caps = vec!["SYS_ADMIN".to_string()];
        let issues = audit_container(&config);
        assert!(issues.iter().any(|i| i.contains("SYS_ADMIN") || i.contains("capabilit")));
    }

    #[test]
    fn test_audit_secret_in_env() {
        let mut config = secure_config();
        config.env_vars = vec![("API_SECRET".to_string(), "sk-12345".to_string())];
        let issues = audit_container(&config);
        assert!(issues.iter().any(|i| i.contains("secret") || i.contains("API_SECRET")));
    }

    #[test]
    fn test_score_secure_container() {
        let config = secure_config();
        let score = score_container(&config);
        assert_eq!(score, 100, "Fully secure container should score 100");
    }

    #[test]
    fn test_score_insecure_container() {
        let config = insecure_config();
        let score = score_container(&config);
        assert!(score < 50, "Insecure container should score below 50, got {}", score);
    }

    #[test]
    fn test_secure_defaults() {
        let config = secure_defaults("nginx:1.25.3");
        assert!(!config.runs_as_root);
        assert!(config.read_only_fs);
        assert!(config.dropped_caps.contains(&"ALL".to_string()));
        assert!(config.added_caps.is_empty());
        assert!(config.no_new_privileges);
        assert!(config.has_health_check);
        assert_eq!(config.memory_limit_mb, 512);
    }

    #[test]
    fn test_validate_tag_latest_unsafe() {
        assert!(validate_image_tag("myapp:latest").is_err());
    }

    #[test]
    fn test_validate_tag_nightly_unsafe() {
        assert!(validate_image_tag("rust:nightly").is_err());
    }

    #[test]
    fn test_validate_tag_specific_version_safe() {
        assert!(validate_image_tag("myapp:1.2.3").is_ok());
    }

    #[test]
    fn test_detect_secrets_in_env() {
        let env_vars = vec![
            ("PORT".to_string(), "8080".to_string()),
            ("DB_PASSWORD".to_string(), "hunter2".to_string()),
            ("API_TOKEN".to_string(), "sk-abcdef1234567890".to_string()),
            ("APP_NAME".to_string(), "myapp".to_string()),
        ];
        let suspicious = detect_secrets_in_env(&env_vars);
        assert!(suspicious.contains(&"DB_PASSWORD".to_string()));
        assert!(suspicious.contains(&"API_TOKEN".to_string()));
        assert!(!suspicious.contains(&"PORT".to_string()));
        assert!(!suspicious.contains(&"APP_NAME".to_string()));
    }
}
