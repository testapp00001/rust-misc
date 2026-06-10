//! # Lesson 10: Hardware Integration — Platform Abstraction Layer
//!
//! ## Why Abstract Hardware?
//!
//! Different platforms have different hardware security capabilities:
//! - Desktop/server: TPM 2.0, HSM (PKCS#11), SGX
//! - Mobile: TrustZone, Secure Enclave (iOS), StrongBox (Android)
//! - Embedded: Secure elements (ATECC608), hardware crypto accelerators
//! - Cloud: vTPM, Nitro Enclaves (AWS), Confidential VMs (Azure)
//!
//! A platform abstraction layer lets application code work regardless of the
//! underlying hardware, while still using hardware security when available.
//!
//! ## Trait-Based Abstraction in Rust
//!
//! Rust traits are perfect for hardware abstraction:
//! ```rust
//! trait HardwareSecurity {
//!     fn generate_key(&self, params: KeyParams) -> Result<KeyHandle>;
//!     fn sign(&self, key: &KeyHandle, data: &[u8]) -> Result<Vec<u8>>;
//!     fn encrypt(&self, key: &KeyHandle, plaintext: &[u8]) -> Result<Vec<u8>>;
//! }
//! ```
//!
//! The same code works with:
//! - A real TPM (via tss-esapi crate)
//! - A software fallback (for development/testing)
//! - A remote HSM (via PKCS#11)
//!
//! ## Graceful Degradation
//!
//! When hardware security is unavailable, the system should:
//! 1. Log a warning (audit trail)
//! 2. Fall back to software implementation
//! 3. Record that software-only mode is active (for compliance)

use ring::digest;
use ring::hmac;
use serde::{Deserialize, Serialize};

/// Hardware security capability levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Full hardware security (TPM, HSM, secure enclave).
    Hardware,
    /// Software-based security (fallback when hardware is unavailable).
    Software,
    /// No security (development/testing only — should never be used in production).
    None,
}

/// Configuration for the platform abstraction layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// Preferred security level.
    pub preferred_level: SecurityLevel,
    /// Whether to allow fallback to lower security levels.
    pub allow_fallback: bool,
    /// Whether to log security decisions.
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
///
/// Implementations provide different levels of security:
/// - HardwareProvider: uses real hardware (TPM/HSM)
/// - SoftwareProvider: uses software crypto (ring)
/// - MockProvider: no security (testing)
pub trait HardwareSecurityProvider {
    /// Generate a key with the given parameters.
    fn generate_key(&self, key_len: usize) -> Vec<u8>;

    /// Sign data using HMAC-SHA256.
    fn sign(&self, key: &[u8], data: &[u8]) -> Vec<u8>;

    /// Verify a signature.
    fn verify(&self, key: &[u8], data: &[u8], signature: &[u8]) -> bool;

    /// Get the security level of this provider.
    fn security_level(&self) -> SecurityLevel;
}

/// Software-based security provider (fallback).
pub struct SoftwareProvider;

impl HardwareSecurityProvider for SoftwareProvider {
    fn generate_key(&self, key_len: usize) -> Vec<u8> {
        use ring::rand::SecureRandom;
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

/// The platform abstraction layer — manages providers and audit logging.
#[derive(Debug)]
pub struct PlatformSecurity {
    /// The active security provider.
    provider_level: SecurityLevel,
    /// Configuration.
    pub config: PlatformConfig,
    /// Audit log of all security operations.
    pub audit_log: Vec<AuditEntry>,
}

impl PlatformSecurity {
    /// Exercise 1: Create a new platform security instance.
    ///
    /// Try to initialize with the preferred security level.
    /// If hardware is unavailable and fallback is allowed, use software.
    /// Log the decision in the audit log.
    pub fn new(config: PlatformConfig) -> Self {
        todo!("Create platform security with provider selection and audit logging")
    }

    /// Exercise 2: Get the active security provider.
    ///
    /// Returns a boxed trait object for the current provider.
    ///
    /// Hints:
    /// - Match on provider_level
    /// - Return SoftwareProvider for Software level
    /// - For Hardware level (simulated), also return SoftwareProvider with a log
    pub fn get_provider(&self) -> Box<dyn HardwareSecurityProvider> {
        todo!("Return the appropriate security provider based on active level")
    }

    /// Exercise 3: Perform a key generation with audit logging.
    ///
    /// Uses the active provider to generate a key and logs the operation.
    pub fn generate_key(&mut self, key_len: usize) -> Vec<u8> {
        todo!("Generate key with audit logging")
    }

    /// Exercise 4: Perform signing with audit logging.
    pub fn sign(&mut self, key: &[u8], data: &[u8]) -> Vec<u8> {
        todo!("Sign with audit logging")
    }

    /// Exercise 5: Perform verification with audit logging.
    pub fn verify(&mut self, key: &[u8], data: &[u8], signature: &[u8]) -> bool {
        todo!("Verify with audit logging")
    }

    /// Exercise 6: Check if the current security level meets a requirement.
    ///
    /// Hardware > Software > None (in terms of security strength).
    pub fn meets_requirement(&self, required: &SecurityLevel) -> bool {
        todo!("Check if current level meets required security level")
    }

    /// Exercise 7: Get audit summary.
    ///
    /// Returns (total_operations, successful, failed, by_level_counts).
    pub fn audit_summary(&self) -> (usize, usize, usize, Vec<(SecurityLevel, usize)>) {
        todo!("Compute audit log summary statistics")
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
        let mut config = software_config();
        config.preferred_level = SecurityLevel::Software;
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
        // Should still work (falls back to software)
        let key = platform.generate_key(32);
        assert_eq!(key.len(), 32);
        // Should have logged the fallback
        let has_fallback = platform.audit_log.iter().any(|e| e.level_used == SecurityLevel::Software);
        assert!(has_fallback, "Should log fallback to software");
    }
}
