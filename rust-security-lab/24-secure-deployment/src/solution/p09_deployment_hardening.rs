//! # Lesson 09: Deployment Hardening (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NetworkRule {
    pub direction: String,
    pub protocol: String,
    pub cidr: String,
    pub port: u16,
    pub allow: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeploymentConfig {
    pub name: String,
    pub run_as_uid: u32,
    pub read_only_fs: bool,
    pub capabilities: Vec<String>,
    pub seccomp_enabled: bool,
    pub seccomp_syscalls: Vec<String>,
    pub memory_limit_mb: u64,
    pub cpu_limit: f64,
    pub pids_limit: u64,
    pub host_network: bool,
    pub privileged: bool,
    pub network_rules: Vec<NetworkRule>,
    pub mount_docker_socket: bool,
}

/// Audit deployment configuration for security issues.
pub fn audit_deployment(config: &DeploymentConfig) -> Vec<String> {
    let mut issues = Vec::new();

    if config.run_as_uid == 0 {
        issues.push("Container runs as root (UID 0) — use non-root user".to_string());
    }
    if !config.read_only_fs {
        issues.push("Filesystem is not read-only — use --read-only".to_string());
    }

    let dangerous_caps = ["SYS_ADMIN", "NET_ADMIN", "SYS_PTRACE", "SYS_RAWIO", "DAC_OVERRIDE", "ALL"];
    for cap in &config.capabilities {
        if dangerous_caps.iter().any(|dc| dc.to_uppercase() == cap.to_uppercase()) {
            issues.push(format!("Dangerous capability: {}", cap));
        }
    }

    if !config.seccomp_enabled {
        issues.push("Seccomp not enabled — apply a seccomp profile".to_string());
    }
    if config.memory_limit_mb == 0 {
        issues.push("No memory limit set".to_string());
    }
    if config.cpu_limit == 0.0 {
        issues.push("No CPU limit set".to_string());
    }
    if config.pids_limit == 0 {
        issues.push("No PID limit set".to_string());
    }
    if config.host_network {
        issues.push("Host network namespace enabled — use isolated network".to_string());
    }
    if config.privileged {
        issues.push("Privileged mode enabled — drop privileges".to_string());
    }
    if config.mount_docker_socket {
        issues.push("Docker socket mounted — high breakout risk".to_string());
    }

    issues
}

/// Score deployment hardening from 0 to 100.
pub fn score_deployment(config: &DeploymentConfig) -> u32 {
    let mut score: i32 = 100;

    if config.run_as_uid == 0 { score -= 20; }
    if !config.read_only_fs { score -= 10; }

    let dangerous_caps = ["SYS_ADMIN", "NET_ADMIN", "SYS_PTRACE", "SYS_RAWIO", "DAC_OVERRIDE", "ALL"];
    let dangerous_count = config.capabilities.iter()
        .filter(|c| dangerous_caps.iter().any(|dc| dc.to_uppercase() == c.to_uppercase()))
        .count();
    score -= (dangerous_count as i32).min(2) * 10;

    if !config.seccomp_enabled { score -= 10; }
    if config.memory_limit_mb == 0 { score -= 5; }
    if config.cpu_limit == 0.0 { score -= 5; }
    if config.pids_limit == 0 { score -= 5; }
    if config.host_network { score -= 10; }
    if config.privileged { score -= 20; }
    if config.mount_docker_socket { score -= 10; }

    score.max(0) as u32
}

/// Create a hardened deployment configuration.
pub fn secure_deployment_defaults(name: &str) -> DeploymentConfig {
    DeploymentConfig {
        name: name.to_string(),
        run_as_uid: 65534,
        read_only_fs: true,
        capabilities: vec![],
        seccomp_enabled: true,
        seccomp_syscalls: vec![
            "read".to_string(), "write".to_string(), "open".to_string(),
            "close".to_string(), "stat".to_string(), "fstat".to_string(),
            "mmap".to_string(), "mprotect".to_string(), "munmap".to_string(),
            "brk".to_string(), "exit_group".to_string(), "futex".to_string(),
            "epoll_wait".to_string(),
        ],
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

/// Validate that network policy is restrictive enough.
pub fn validate_network_policy(rules: &[NetworkRule]) -> Result<(), String> {
    if rules.is_empty() {
        return Err("No network rules defined — policy is too permissive".to_string());
    }

    for rule in rules {
        if rule.allow && rule.cidr == "0.0.0.0/0" {
            if rule.direction == "ingress" {
                return Err("Ingress rule allows traffic from 0.0.0.0/0 (the entire internet)".to_string());
            }
            if rule.direction == "egress" {
                return Err("Egress rule allows traffic to 0.0.0.0/0 (the entire internet)".to_string());
            }
        }

        if rule.allow && rule.direction == "ingress" && rule.port == 0 {
            return Err(format!(
                "Ingress rule for CIDR {} does not specify a port",
                rule.cidr
            ));
        }
    }

    Ok(())
}

/// Generate a human-readable seccomp profile description.
pub fn describe_seccomp_profile(syscalls: &[String]) -> String {
    if syscalls.is_empty() {
        return "Seccomp Profile:\n  (using default profile)".to_string();
    }

    let mut desc = format!(
        "Seccomp Profile:\n  Default Action: SCMP_ACT_ERRNO\n  Allowed Syscalls ({}):\n",
        syscalls.len()
    );
    for syscall in syscalls {
        desc.push_str(&format!("    - {}\n", syscall));
    }
    desc
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
