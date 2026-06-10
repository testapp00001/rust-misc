//! # Lesson 02: Container Security
//!
//! ## Attack: Container Breakout via Root
//!
//! A container running as root (UID 0) with default capabilities can exploit kernel
//! vulnerabilities to escape the container and access the host. Even without a kernel
//! exploit, a root container can:
//! - Read sensitive files mounted from the host
//! - Modify network configuration
//! - Install malware that persists in volumes
//! - Access the Docker socket to control other containers
//!
//! ## Defend: Least-Privilege Containers
//!
//! 1. Run as non-root user
//! 2. Use read-only filesystem
//! 3. Drop all Linux capabilities
//! 4. Set resource limits
//! 5. Use minimal base images (scratch, distroless)
//!
//! ## Audit: Container Security Checklist
//!
//! - [ ] USER directive set in Dockerfile
//! [ ] --read-only flag passed at runtime
//! [ ] --cap-drop=ALL with only needed capabilities added back
//! [ ] --no-new-privileges prevents privilege escalation
//! [ ] Resource limits (--memory, --cpus, --pids-limit) set
//! [ ] Health check defined
//! [ ] No secrets in image layers

use serde::{Deserialize, Serialize};

/// Represents the security configuration of a container.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContainerConfig {
    /// Image name and tag (e.g., "myapp:1.0")
    pub image: String,
    /// Whether the container runs as root (UID 0)
    pub runs_as_root: bool,
    /// Whether the filesystem is read-only
    pub read_only_fs: bool,
    /// Linux capabilities that are dropped
    pub dropped_caps: Vec<String>,
    /// Linux capabilities that are added
    pub added_caps: Vec<String>,
    /// Memory limit in MB (0 = unlimited)
    pub memory_limit_mb: u64,
    /// CPU limit (0.0 = unlimited)
    pub cpu_limit: f64,
    /// Whether --no-new-privileges is set
    pub no_new_privileges: bool,
    /// Maximum number of processes (0 = unlimited)
    pub pids_limit: u64,
    /// Whether a health check is defined
    pub has_health_check: bool,
    /// Environment variables (check for secrets)
    pub env_vars: Vec<(String, String)>,
}

/// Exercise 1: Audit a container configuration for security issues.
///
/// Check for these problems and return a list of issue descriptions:
///
/// 1. `runs_as_root` is true
/// 2. `read_only_fs` is false
/// 3. `dropped_caps` does not contain "ALL"
/// 4. `added_caps` contains dangerous capabilities: "SYS_ADMIN", "NET_ADMIN",
///    "SYS_PTRACE", "SYS_RAWIO", "DAC_OVERRIDE"
/// 5. `memory_limit_mb` is 0 (unlimited)
/// 6. `no_new_privileges` is false
/// 7. `has_health_check` is false
/// 8. Any env var value looks like a secret (contains "password", "secret", "key",
///    "token", "credential" — case-insensitive)
///
/// Return problems as strings. Empty list = secure config.
pub fn audit_container(config: &ContainerConfig) -> Vec<String> {
    todo!("Audit container configuration for security issues")
}

/// Exercise 2: Score a container configuration (0-100).
///
/// Scoring rules:
/// - Start at 100
/// - Subtract 20 if running as root
/// - Subtract 15 if filesystem is not read-only
/// - Subtract 15 if "ALL" is not in dropped_caps
/// - Subtract 10 per dangerous added capability (max -20)
/// - Subtract 10 if memory_limit_mb is 0
/// - Subtract 10 if no_new_privileges is false
/// - Subtract 5 if has_health_check is false
/// - Subtract 5 per env var that looks like a secret (max -10)
/// - Minimum score is 0
pub fn score_container(config: &ContainerConfig) -> u32 {
    todo!("Score container security from 0 to 100")
}

/// Exercise 3: Generate a secure container configuration.
///
/// Given an image name, return a `ContainerConfig` with secure defaults:
/// - runs_as_root: false
/// - read_only_fs: true
/// - dropped_caps: ["ALL"]
/// - added_caps: [] (empty)
/// - memory_limit_mb: 512
/// - cpu_limit: 1.0
/// - no_new_privileges: true
/// - pids_limit: 256
/// - has_health_check: true
/// - env_vars: [] (empty)
pub fn secure_defaults(image: &str) -> ContainerConfig {
    todo!("Create a container config with secure defaults")
}

/// Exercise 4: Check if a container image tag is safe.
///
/// These tags are considered UNSAFE:
/// - "latest" (mutable, non-deterministic)
/// - Tags shorter than 5 characters (likely not a specific version)
/// - Tags containing "edge", "canary", "nightly" (unstable)
///
/// Return Ok(()) if safe, Err with description if unsafe.
pub fn validate_image_tag(image: &str) -> Result<(), String> {
    todo!("Validate that an image tag is safe and deterministic")
}

/// Exercise 5: Detect secrets in environment variables.
///
/// An env var value is considered a potential secret if:
/// - The key (case-insensitive) contains: "password", "secret", "key", "token",
///   "credential", "auth", "api_key", "private"
/// - OR the value is longer than 32 characters and looks like base64 or hex
///
/// Return the keys of suspicious env vars.
pub fn detect_secrets_in_env(env_vars: &[(String, String)]) -> Vec<String> {
    todo!("Detect potential secrets in environment variables")
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
