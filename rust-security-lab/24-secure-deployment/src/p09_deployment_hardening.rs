//! # Lesson 09: Deployment Hardening
//!
//! ## Attack: Privilege Escalation in Production
//!
//! An attacker finds an RCE (Remote Code Execution) vulnerability in your app.
//! Because the container runs as root with default capabilities, the attacker can:
//! - Read secrets from mounted volumes
//! - Pivot to other containers on the same network
//! - Exploit kernel vulnerabilities to escape the container
//! - Install persistent backdoors
//!
//! With proper hardening, the same RCE vulnerability would be contained to a
//! non-root process with no capabilities, no network access, and a read-only filesystem.
//!
//! ## Defend: Defense in Depth
//!
//! 1. **Least privilege**: Minimal user permissions and capabilities
//! 2. **Network policy**: Only allow necessary traffic
//! 3. **Seccomp**: Restrict system calls
//! 4. **Resource limits**: Prevent resource exhaustion attacks
//! 5. **Read-only filesystem**: Prevent malware installation
//!
//! ## Audit: Hardening Checklist
//!
//! - [ ] Container runs as non-root
//! - [ ] All unnecessary capabilities dropped
//! [ ] Network policy restricts ingress/egress
//! - [ ] Seccomp profile applied
//! [ ] Resource limits set (CPU, memory, PIDs)
//! - [ ] Read-only root filesystem
//! - [ ] No host network namespace
//! - [ ] No privileged mode

use serde::{Deserialize, Serialize};

/// Network policy rule.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkRule {
    /// Direction: "ingress" or "egress"
    pub direction: String,
    /// Protocol: "tcp", "udp", or "any"
    pub protocol: String,
    /// Allowed source/destination CIDR (e.g., "10.0.0.0/8")
    pub cidr: String,
    /// Allowed port range (0 = any)
    pub port: u16,
    /// Whether this rule allows or denies traffic
    pub allow: bool,
}

/// Deployment configuration to audit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeploymentConfig {
    /// Application name
    pub name: String,
    /// UID the process runs as
    pub run_as_uid: u32,
    /// Whether the filesystem is read-only
    pub read_only_fs: bool,
    /// Linux capabilities to keep
    pub capabilities: Vec<String>,
    /// Whether seccomp is enabled
    pub seccomp_enabled: bool,
    /// Seccomp syscall whitelist (empty = default profile)
    pub seccomp_syscalls: Vec<String>,
    /// Memory limit in MB (0 = unlimited)
    pub memory_limit_mb: u64,
    /// CPU cores limit (0.0 = unlimited)
    pub cpu_limit: f64,
    /// PID limit (0 = unlimited)
    pub pids_limit: u64,
    /// Whether host network namespace is used
    pub host_network: bool,
    /// Whether privileged mode is enabled
    pub privileged: bool,
    /// Network policy rules
    pub network_rules: Vec<NetworkRule>,
    /// Whether the app needs access to the Docker/Kubernetes socket
    pub mount_docker_socket: bool,
}

/// Exercise 1: Audit a deployment configuration.
///
/// Check for these issues and return descriptions:
/// 1. `run_as_uid` is 0 (root)
/// 2. `read_only_fs` is false
/// 3. `capabilities` contains dangerous caps: "SYS_ADMIN", "NET_ADMIN",
///    "SYS_PTRACE", "SYS_RAWIO", "DAC_OVERRIDE", "ALL"
/// 4. `seccomp_enabled` is false
/// 5. `memory_limit_mb` is 0
/// 6. `cpu_limit` is 0.0
/// 7. `pids_limit` is 0
/// 8. `host_network` is true
/// 9. `privileged` is true
/// 10. `mount_docker_socket` is true
///
/// Return list of issue descriptions. Empty = hardened.
pub fn audit_deployment(config: &DeploymentConfig) -> Vec<String> {
    todo!("Audit deployment configuration for security issues")
}

/// Exercise 2: Score a deployment (0-100).
///
/// Scoring:
/// - Start at 100
/// - Subtract 20 if UID is 0
/// - Subtract 10 if read_only_fs is false
/// - Subtract 10 per dangerous capability (max -20)
/// - Subtract 10 if seccomp is disabled
/// - Subtract 5 if memory_limit is 0
/// - Subtract 5 if cpu_limit is 0
/// - Subtract 5 if pids_limit is 0
/// - Subtract 10 if host_network is true
/// - Subtract 20 if privileged is true
/// - Subtract 10 if mount_docker_socket is true
/// - Minimum is 0
pub fn score_deployment(config: &DeploymentConfig) -> u32 {
    todo!("Score deployment hardening from 0 to 100")
}

/// Exercise 3: Generate a secure default deployment configuration.
///
/// Return a `DeploymentConfig` with:
/// - run_as_uid: 65534 (nobody)
/// - read_only_fs: true
/// - capabilities: [] (empty)
/// - seccomp_enabled: true
/// - seccomp_syscalls: ["read", "write", "open", "close", "stat", "fstat",
///   "mmap", "mprotect", "munmap", "brk", "exit_group", "futex", "epoll_wait"]
/// - memory_limit_mb: 512
/// - cpu_limit: 1.0
/// - pids_limit: 256
/// - host_network: false
/// - privileged: false
/// - network_rules: single egress rule allowing TCP to "10.0.0.0/8" on port 443
/// - mount_docker_socket: false
pub fn secure_deployment_defaults(name: &str) -> DeploymentConfig {
    todo!("Create a hardened deployment configuration")
}

/// Exercise 4: Validate network policy.
///
/// A network policy is "restrictive" if:
/// 1. There are no rules allowing ingress from "0.0.0.0/0" (the internet)
/// 2. There are no rules allowing egress to "0.0.0.0/0"
/// 3. All ingress rules specify a port
/// 4. There is at least one rule
///
/// Return Ok(()) if restrictive, Err(description) if not.
pub fn validate_network_policy(rules: &[NetworkRule]) -> Result<(), String> {
    todo!("Validate that network policy is restrictive enough")
}

/// Exercise 5: Generate a seccomp profile description.
///
/// Given a list of allowed syscall names, return a JSON-like string describing
/// the profile. Format:
///
/// ```text
/// Seccomp Profile:
///   Default Action: SCMP_ACT_ERRNO
///   Allowed Syscalls ({count}):
///     - {syscall1}
///     - {syscall2}
///     ...
/// ```
///
/// If the list is empty, show "(using default profile)" instead.
pub fn describe_seccomp_profile(syscalls: &[String]) -> String {
    todo!("Generate a human-readable seccomp profile description")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hardened_config() -> DeploymentConfig {
        DeploymentConfig {
            name: "myapp".to_string(),
            run_as_uid: 65534,
            read_only_fs: true,
            capabilities: vec![],
            seccomp_enabled: true,
            seccomp_syscalls: vec!["read".to_string(), "write".to_string()],
            memory_limit_mb: 512,
            cpu_limit: 1.0,
            pids_limit: 256,
            host_network: false,
            privileged: false,
            network_rules: vec![NetworkRule {
                direction: "egress".to_string(),
                protocol: "tcp".to_string(),
                cidr: "10.0.0.0/8".to_string(),
                port: 443,
                allow: true,
            }],
            mount_docker_socket: false,
        }
    }

    fn insecure_config() -> DeploymentConfig {
        DeploymentConfig {
            name: "insecure".to_string(),
            run_as_uid: 0,
            read_only_fs: false,
            capabilities: vec!["SYS_ADMIN".to_string()],
            seccomp_enabled: false,
            seccomp_syscalls: vec![],
            memory_limit_mb: 0,
            cpu_limit: 0.0,
            pids_limit: 0,
            host_network: true,
            privileged: true,
            network_rules: vec![],
            mount_docker_socket: true,
        }
    }

    #[test]
    fn test_audit_hardened_passes() {
        let config = hardened_config();
        let issues = audit_deployment(&config);
        assert!(issues.is_empty(), "Hardened config should pass, got: {:?}", issues);
    }

    #[test]
    fn test_audit_finds_root() {
        let config = insecure_config();
        let issues = audit_deployment(&config);
        assert!(issues.iter().any(|i| i.contains("root") || i.contains("UID") || i.contains("uid")));
    }

    #[test]
    fn test_audit_finds_privileged() {
        let config = insecure_config();
        let issues = audit_deployment(&config);
        assert!(issues.iter().any(|i| i.contains("Privileged") || i.contains("privileged")));
    }

    #[test]
    fn test_audit_finds_docker_socket() {
        let config = insecure_config();
        let issues = audit_deployment(&config);
        assert!(issues.iter().any(|i| i.contains("docker") || i.contains("socket")));
    }

    #[test]
    fn test_score_hardened_is_100() {
        let config = hardened_config();
        assert_eq!(score_deployment(&config), 100);
    }

    #[test]
    fn test_score_insecure_is_low() {
        let config = insecure_config();
        let score = score_deployment(&config);
        assert!(score == 0, "Maximally insecure config should score 0, got {}", score);
    }

    #[test]
    fn test_secure_defaults() {
        let config = secure_deployment_defaults("myapp");
        assert_eq!(config.run_as_uid, 65534);
        assert!(config.read_only_fs);
        assert!(config.capabilities.is_empty());
        assert!(config.seccomp_enabled);
        assert!(!config.privileged);
        assert!(!config.host_network);
        assert!(!config.mount_docker_socket);
        assert!(!config.network_rules.is_empty());
    }

    #[test]
    fn test_validate_network_policy_restrictive() {
        let rules = vec![NetworkRule {
            direction: "egress".to_string(),
            protocol: "tcp".to_string(),
            cidr: "10.0.0.0/8".to_string(),
            port: 443,
            allow: true,
        }];
        assert!(validate_network_policy(&rules).is_ok());
    }

    #[test]
    fn test_validate_network_policy_too_open() {
        let rules = vec![NetworkRule {
            direction: "egress".to_string(),
            protocol: "any".to_string(),
            cidr: "0.0.0.0/0".to_string(),
            port: 0,
            allow: true,
        }];
        assert!(validate_network_policy(&rules).is_err());
    }

    #[test]
    fn test_validate_network_policy_empty() {
        let rules: Vec<NetworkRule> = vec![];
        assert!(validate_network_policy(&rules).is_err());
    }

    #[test]
    fn test_describe_seccomp_profile() {
        let syscalls = vec!["read".to_string(), "write".to_string()];
        let desc = describe_seccomp_profile(&syscalls);
        assert!(desc.contains("read"));
        assert!(desc.contains("write"));
        assert!(desc.contains("2") || desc.contains("Syscalls"));
    }

    #[test]
    fn test_describe_seccomp_empty() {
        let syscalls: Vec<String> = vec![];
        let desc = describe_seccomp_profile(&syscalls);
        assert!(desc.contains("default") || desc.contains("empty") || desc.contains("no syscalls"));
    }
}
