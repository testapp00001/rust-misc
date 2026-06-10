//! # Lesson 06: Secure Boot Chain (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::hmac;
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};

/// A signing key pair.
#[derive(Debug, Clone)]
pub struct SigningKey {
    pub id: String,
    pub key_material: Vec<u8>,
}

impl SigningKey {
    pub fn new(id: &str) -> Self {
        let rng = ring::rand::SystemRandom::new();
        let mut material = vec![0u8; 32];
        rng.fill(&mut material).unwrap();
        Self {
            id: id.to_string(),
            key_material: material,
        }
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, &self.key_material);
        hmac::sign(&key, data).as_ref().to_vec()
    }
}

/// A stage in the boot chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootStage {
    pub name: String,
    pub code: Vec<u8>,
    pub signature: Vec<u8>,
}

/// The platform's secure boot configuration.
#[derive(Debug)]
pub struct SecureBootPlatform {
    pub trusted_keys: Vec<SigningKey>,
    pub revoked_keys: Vec<SigningKey>,
    pub stages: Vec<BootStage>,
    pub boot_success: bool,
    pub boot_log: Vec<(String, bool)>,
}

impl SecureBootPlatform {
    /// Create a new secure boot platform.
    pub fn new() -> Self {
        Self {
            trusted_keys: Vec::new(),
            revoked_keys: Vec::new(),
            stages: Vec::new(),
            boot_success: false,
            boot_log: Vec::new(),
        }
    }

    /// Add a trusted signing key.
    pub fn add_trusted_key(&mut self, key: SigningKey) {
        self.trusted_keys.push(key);
    }

    /// Add a revoked signing key.
    pub fn add_revoked_key(&mut self, key: SigningKey) {
        self.revoked_keys.push(key);
    }

    /// Sign a boot stage with a given key.
    pub fn sign_stage(&self, stage: &mut BootStage, key: &SigningKey) {
        stage.signature = key.sign(&stage.code);
    }

    /// Verify a single boot stage.
    pub fn verify_stage(&self, stage: &BootStage) -> bool {
        if stage.signature.is_empty() {
            return false;
        }

        // Check if signed by a revoked key
        for key in &self.revoked_keys {
            let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &key.key_material);
            if hmac::verify(&hmac_key, &stage.code, &stage.signature).is_ok() {
                return false; // Signed by revoked key
            }
        }

        // Check if signed by a trusted key
        for key in &self.trusted_keys {
            let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &key.key_material);
            if hmac::verify(&hmac_key, &stage.code, &stage.signature).is_ok() {
                return true; // Signed by trusted key
            }
        }

        false // Not signed by any trusted key
    }

    /// Verify entire boot chain. Stop on first failure.
    pub fn verify_boot(&mut self) -> bool {
        self.boot_log.clear();
        self.boot_success = true;

        for stage in &self.stages {
            let result = self.verify_stage(stage);
            self.boot_log.push((stage.name.clone(), result));
            if !result {
                self.boot_success = false;
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_platform_with_keys() -> (SecureBootPlatform, SigningKey, SigningKey) {
        let mut platform = SecureBootPlatform::new();
        let good_key = SigningKey::new("good-key");
        let bad_key = SigningKey::new("bad-key");
        platform.add_trusted_key(good_key.clone());
        platform.add_revoked_key(bad_key.clone());
        (platform, good_key, bad_key)
    }

    #[test]
    fn test_platform_creation() {
        let platform = SecureBootPlatform::new();
        assert!(platform.trusted_keys.is_empty());
        assert!(!platform.boot_success);
    }

    #[test]
    fn test_sign_and_verify_stage() {
        let (mut platform, good_key, _) = make_platform_with_keys();
        let mut stage = BootStage {
            name: "Bootloader".into(),
            code: b"bootloader code here".to_vec(),
            signature: vec![],
        };
        platform.sign_stage(&mut stage, &good_key);
        assert!(platform.verify_stage(&stage));
    }

    #[test]
    fn test_revoked_key_rejected() {
        let (mut platform, _, bad_key) = make_platform_with_keys();
        let mut stage = BootStage {
            name: "Malicious".into(),
            code: b"rootkit".to_vec(),
            signature: vec![],
        };
        platform.sign_stage(&mut stage, &bad_key);
        assert!(!platform.verify_stage(&stage), "Revoked key should be rejected");
    }

    #[test]
    fn test_unsigned_stage_rejected() {
        let (platform, _, _) = make_platform_with_keys();
        let stage = BootStage {
            name: "Unsigned".into(),
            code: b"unsigned code".to_vec(),
            signature: vec![],
        };
        assert!(!platform.verify_stage(&stage), "Unsigned stage should fail");
    }

    #[test]
    fn test_tampered_code_rejected() {
        let (mut platform, good_key, _) = make_platform_with_keys();
        let mut stage = BootStage {
            name: "Kernel".into(),
            code: b"original kernel".to_vec(),
            signature: vec![],
        };
        platform.sign_stage(&mut stage, &good_key);
        stage.code = b"tampered kernel".to_vec();
        assert!(!platform.verify_stage(&stage), "Tampered code should fail verification");
    }

    #[test]
    fn test_full_boot_success() {
        let (mut platform, good_key, _) = make_platform_with_keys();

        let mut bios = BootStage { name: "BIOS".into(), code: b"bios code".to_vec(), signature: vec![] };
        let mut bootloader = BootStage { name: "Bootloader".into(), code: b"grub code".to_vec(), signature: vec![] };
        let mut kernel = BootStage { name: "Kernel".into(), code: b"linux code".to_vec(), signature: vec![] };

        platform.sign_stage(&mut bios, &good_key);
        platform.sign_stage(&mut bootloader, &good_key);
        platform.sign_stage(&mut kernel, &good_key);

        platform.stages = vec![bios, bootloader, kernel];
        assert!(platform.verify_boot());
        assert!(platform.boot_success);
    }

    #[test]
    fn test_boot_fails_on_tampered_stage() {
        let (mut platform, good_key, bad_key) = make_platform_with_keys();

        let mut bios = BootStage { name: "BIOS".into(), code: b"bios code".to_vec(), signature: vec![] };
        let mut bootloader = BootStage { name: "Bootloader".into(), code: b"grub code".to_vec(), signature: vec![] };

        platform.sign_stage(&mut bios, &good_key);
        platform.sign_stage(&mut bootloader, &bad_key);

        platform.stages = vec![bios, bootloader];
        assert!(!platform.verify_boot());
        assert!(!platform.boot_success);
    }

    #[test]
    fn test_boot_log_recorded() {
        let (mut platform, good_key, _) = make_platform_with_keys();

        let mut stage = BootStage { name: "Stage1".into(), code: b"code".to_vec(), signature: vec![] };
        platform.sign_stage(&mut stage, &good_key);
        platform.stages = vec![stage];
        platform.verify_boot();

        assert_eq!(platform.boot_log.len(), 1);
        assert_eq!(platform.boot_log[0].0, "Stage1");
        assert!(platform.boot_log[0].1);
    }
}
