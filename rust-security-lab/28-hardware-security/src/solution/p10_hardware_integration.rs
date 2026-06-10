//! # Lesson 10: Hardware Integration — Platform Abstraction Layer (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::hmac;
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};

/// Hardware security capability levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Hardware,
    Software,
    None,
}

impl SecurityLevel {
    /// Numeric strength for comparison (higher = more secure).
    fn strength(&self) -> u8 {
        match self {
            SecurityLevel::Hardware => 3,
            SecurityLevel::Software => 2,
            SecurityLevel::None => 1,
        }
    }
}

/// Configuration for the platform abstraction layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub preferred_level: SecurityLevel,
    pub allow_fallback: bool,
    pub audit_logging: bool,
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            preferred_level: SecurityLevel::Hardware,
            allow_fallback: true,
            audit_logging: true,
        }
    }
}

/// A security operation audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub operation: String,
    pub level_used: SecurityLevel,
    pub timestamp: String,
    pub success: bool,
}

/// The hardware security provider trait.
pub trait HardwareSecurityProvider {
    fn generate_key(&self, key_len: usize) -> Vec<u8>;
    fn sign(&self, key: &[u8], data: &[u8]) -> Vec<u8>;
    fn verify(&self, key: &[u8], data: &[u8], signature: &[u8]) -> bool;
    fn security_level(&self) -> SecurityLevel;
}

/// Software-based security provider (fallback).
pub struct SoftwareProvider;

impl HardwareSecurityProvider for SoftwareProvider {
    fn generate_key(&self, key_len: usize) -> Vec<u8> {
        let rng = ring::rand::SystemRandom::new();
        let mut key = vec![0u8; key_len];
        rng.fill(&mut key).unwrap();
        key
    }

    fn sign(&self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key);
        hmac::sign(&hmac_key, data).as_ref().to_vec()
    }

    fn verify(&self, key: &[u8], data: &[u8], signature: &[u8]) -> bool {
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, key);
        hmac::verify(&hmac_key, data, signature).is_ok()
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Software
    }
}

/// The platform abstraction layer.
#[derive(Debug)]
pub struct PlatformSecurity {
    provider_level: SecurityLevel,
    pub config: PlatformConfig,
    pub audit_log: Vec<AuditEntry>,
}

impl PlatformSecurity {
    /// Create a new platform security instance with provider selection.
    pub fn new(config: PlatformConfig) -> Self {
        let mut audit_log = Vec::new();

        // In this simulation, hardware is never directly available — always fall back
        let provider_level = if config.preferred_level == SecurityLevel::Hardware {
            if config.allow_fallback {
                audit_log.push(AuditEntry {
                    operation: "provider_selection".into(),
                    level_used: SecurityLevel::Software,
                    timestamp: "init".into(),
                    success: true,
                });
                SecurityLevel::Software
            } else {
                audit_log.push(AuditEntry {
                    operation: "provider_selection".into(),
                    level_used: SecurityLevel::None,
                    timestamp: "init".into(),
                    success: false,
                });
                SecurityLevel::None
            }
        } else {
            audit_log.push(AuditEntry {
                operation: "provider_selection".into(),
                level_used: config.preferred_level.clone(),
                timestamp: "init".into(),
                success: true,
            });
            config.preferred_level.clone()
        };

        Self {
            provider_level,
            config,
            audit_log,
        }
    }

    /// Get the active security provider.
    pub fn get_provider(&self) -> Box<dyn HardwareSecurityProvider> {
        match self.provider_level {
            SecurityLevel::Software | SecurityLevel::Hardware => Box::new(SoftwareProvider),
            SecurityLevel::None => Box::new(SoftwareProvider), // Fallback even for None in sim
        }
    }

    /// Generate a key with audit logging.
    pub fn generate_key(&mut self, key_len: usize) -> Vec<u8> {
        let provider = self.get_provider();
        let key = provider.generate_key(key_len);
        self.audit_log.push(AuditEntry {
            operation: format!("generate_key({})", key_len),
            level_used: self.provider_level.clone(),
            timestamp: "now".into(),
            success: true,
        });
        key
    }

    /// Sign with audit logging.
    pub fn sign(&mut self, key: &[u8], data: &[u8]) -> Vec<u8> {
        let provider = self.get_provider();
        let sig = provider.sign(key, data);
        self.audit_log.push(AuditEntry {
            operation: "sign".into(),
            level_used: self.provider_level.clone(),
            timestamp: "now".into(),
            success: true,
        });
        sig
    }

    /// Verify with audit logging.
    pub fn verify(&mut self, key: &[u8], data: &[u8], signature: &[u8]) -> bool {
        let provider = self.get_provider();
        let result = provider.verify(key, data, signature);
        self.audit_log.push(AuditEntry {
            operation: "verify".into(),
            level_used: self.provider_level.clone(),
            timestamp: "now".into(),
            success: result,
        });
        result
    }

    /// Check if current level meets a requirement.
    pub fn meets_requirement(&self, required: &SecurityLevel) -> bool {
        self.provider_level.strength() >= required.strength()
    }

    /// Compute audit summary: (total, successful, failed, by_level_counts).
    pub fn audit_summary(&self) -> (usize, usize, usize, Vec<(SecurityLevel, usize)>) {
        let total = self.audit_log.len();
        let success = self.audit_log.iter().filter(|e| e.success).count();
        let failed = total - success;

        let mut by_level = Vec::new();
        for level in &[SecurityLevel::Hardware, SecurityLevel::Software, SecurityLevel::None] {
            let count = self.audit_log.iter().filter(|e| &e.level_used == level).count();
            if count > 0 {
                by_level.push((level.clone(), count));
            }
        }

        (total, success, failed, by_level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn software_config() -> PlatformConfig {
        PlatformConfig {
            preferred_level: SecurityLevel::Software,
            allow_fallback: true,
            audit_logging: true,
        }
    }

    #[test]
    fn test_platform_creation_logs_decision() {
        let config = software_config();
        let platform = PlatformSecurity::new(config);
        assert!(!platform.audit_log.is_empty(), "Should log provider selection");
    }

    #[test]
    fn test_key_generation() {
        let mut platform = PlatformSecurity::new(software_config());
        let key = platform.generate_key(32);
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let mut platform = PlatformSecurity::new(software_config());
        let key = platform.generate_key(32);
        let data = b"test data";
        let sig = platform.sign(&key, data);
        assert!(platform.verify(&key, data, &sig));
    }

    #[test]
    fn test_verify_wrong_data() {
        let mut platform = PlatformSecurity::new(software_config());
        let key = platform.generate_key(32);
        let sig = platform.sign(&key, b"original");
        assert!(!platform.verify(&key, b"tampered", &sig));
    }

    #[test]
    fn test_meets_requirement() {
        let config = software_config();
        let platform = PlatformSecurity::new(config);

        assert!(platform.meets_requirement(&SecurityLevel::Software));
        assert!(platform.meets_requirement(&SecurityLevel::None));
        assert!(!platform.meets_requirement(&SecurityLevel::Hardware));
    }

    #[test]
    fn test_audit_log_growth() {
        let mut platform = PlatformSecurity::new(software_config());
        let initial = platform.audit_log.len();
        platform.generate_key(32);
        assert!(platform.audit_log.len() > initial);
    }

    #[test]
    fn test_audit_summary() {
        let mut platform = PlatformSecurity::new(software_config());
        let key = platform.generate_key(32);
        let sig = platform.sign(&key, b"data1");
        platform.verify(&key, b"data1", &sig);

        let (total, _success, _fail, _by_level) = platform.audit_summary();
        assert!(total >= 3, "Should have at least 3 audit entries");
    }

    #[test]
    fn test_hardware_fallback_to_software() {
        let config = PlatformConfig {
            preferred_level: SecurityLevel::Hardware,
            allow_fallback: true,
            audit_logging: true,
        };
        let mut platform = PlatformSecurity::new(config);
        let key = platform.generate_key(32);
        assert_eq!(key.len(), 32);
        let has_fallback = platform.audit_log.iter().any(|e| e.level_used == SecurityLevel::Software);
        assert!(has_fallback, "Should log fallback to software");
    }
}
