//! # Lesson 06: Secure Boot Chain
//!
//! ## What is Secure Boot?
//!
//! Secure boot verifies the integrity of each software stage before executing it.
//! Each stage verifies the next before handing off control.
//!
//! ```
//! Stage 0: ROM (immutable, burned into silicon)
//!   └── verifies signature of Stage 1
//! Stage 1: Bootloader (e.g., UEFI)
//!   └── verifies signature of Stage 2
//! Stage 2: OS Kernel
//!   └── verifies signature of Stage 3
//! Stage 3: Drivers / Applications
//! ```
//!
//! ## Measured Boot vs Verified Boot
//!
//! | Property | Measured Boot | Verified Boot |
//! |----------|--------------|---------------|
//! | Action on failure | Logs failure, continues | Stops boot |
//! | Example | TPM + attestation | UEFI Secure Boot, Android Verified Boot |
//! | Trust model | Detection (audit) | Prevention (block) |
//!
//! ## UEFI Secure Boot
//!
//! - Platform has a database of trusted signing keys (db) and revoked keys (dbx)
//! - Each EFI binary is checked: is it signed by a key in db (not in dbx)?
//! - If signature check fails → boot is blocked
//! - Microsoft's key is enrolled by default on most PCs
//!
//! ## Attack: Bootkit
//!
//! A bootkit replaces the bootloader with a malicious one that loads a rootkit
//! before the OS. Defense: Secure Boot verifies the bootloader's signature.
//!
//! ## Attack: Key Enrollment Attack
//!
//! An attacker enrolls their own signing key in the UEFI database.
//! Defense: Physical presence required for key changes, or use custom keys.

use ring::digest;
use ring::hmac;
use serde::{Deserialize, Serialize};

/// A signing key pair (in simulation, we use HMAC as a symmetric proxy).
#[derive(Debug, Clone)]
pub struct SigningKey {
    pub id: String,
    pub key_material: Vec<u8>,
}

impl SigningKey {
    /// Create a new signing key with random material.
    pub fn new(id: &str) -> Self {
        use ring::rand::SecureRandom;
        let rng = ring::rand::SystemRandom::new();
        let mut material = vec![0u8; 32];
        rng.fill(&mut material).unwrap();
        Self {
            id: id.to_string(),
            key_material: material,
        }
    }

    /// Sign data with this key.
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let key = hmac::Key::new(hmac::HMAC_SHA256, &self.key_material);
        hmac::sign(&key, data).as_ref().to_vec()
    }
}

/// A stage in the boot chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootStage {
    /// Name of this stage (e.g., "BIOS", "Bootloader", "Kernel").
    pub name: String,
    /// The code content of this stage.
    pub code: Vec<u8>,
    /// Signature over the code (created by the previous stage's key).
    pub signature: Vec<u8>,
}

/// The platform's secure boot configuration.
#[derive(Debug)]
pub struct SecureBootPlatform {
    /// Trusted signing keys (allowed signers).
    pub trusted_keys: Vec<SigningKey>,
    /// Revoked signing keys (denied signers).
    pub revoked_keys: Vec<SigningKey>,
    /// The boot chain stages in order.
    pub stages: Vec<BootStage>,
    /// Whether boot succeeded (all stages verified).
    pub boot_success: bool,
    /// Log of verification results per stage.
    pub boot_log: Vec<(String, bool)>,
}

impl SecureBootPlatform {
    /// Exercise 1: Create a new secure boot platform.
    ///
    /// Initialize with empty trusted/revoked keys, no stages, and boot_success = false.
    pub fn new() -> Self {
        todo!("Create secure boot platform")
    }

    /// Exercise 2: Add a trusted signing key.
    pub fn add_trusted_key(&mut self, key: SigningKey) {
        todo!("Add a key to the trusted key database")
    }

    /// Exercise 3: Add a revoked signing key.
    pub fn add_revoked_key(&mut self, key: SigningKey) {
        todo!("Add a key to the revoked key database (dbx)")
    }

    /// Exercise 4: Sign a boot stage with a given key.
    ///
    /// Compute HMAC-SHA256 over the stage's code and store the signature.
    pub fn sign_stage(&self, stage: &mut BootStage, key: &SigningKey) {
        todo!("Sign a boot stage's code with a signing key")
    }

    /// Exercise 5: Verify a single boot stage.
    ///
    /// Check:
    /// 1. The signature was NOT made by a revoked key
    /// 2. The signature WAS made by a trusted key
    /// 3. The signature is valid (HMAC matches)
    ///
    /// Returns true if the stage passes all checks.
    pub fn verify_stage(&self, stage: &BootStage) -> bool {
        todo!("Verify a boot stage's signature against trusted/revoked keys")
    }

    /// Exercise 6: Perform a full boot verification.
    ///
    /// Verify each stage in order. If any stage fails:
    /// - In "verified boot" mode: stop and fail
    /// - Log the result for each stage
    ///
    /// Sets boot_success to true only if all stages pass.
    pub fn verify_boot(&mut self) -> bool {
        todo!("Verify entire boot chain, stop on first failure")
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
        // Tamper with the code after signing
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
        platform.sign_stage(&mut bootloader, &bad_key); // signed with revoked key

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
