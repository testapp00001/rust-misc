//! # Lesson 03: Certificate Pinning
//!
//! ## What is Certificate Pinning?
//!
//! Standard TLS validates certificates against the system's CA trust store.
//! This means you trust hundreds of CAs — any one of them can issue a certificate
//! for any domain. Certificate pinning restricts trust to specific certificates
//! or public keys you expect for a given server.
//!
//! ## How Pinning Works
//!
//! Instead of trusting the entire CA system, you hardcode (pin) the expected:
//! - **Certificate pin**: Hash of the certificate (SPKI - Subject Public Key Info)
//! - **Public key pin**: Hash of just the public key
//!
//! During TLS handshake, if the presented certificate doesn't match the pin,
//! the connection is rejected — even if the cert is otherwise valid.
//!
//! ## Attack Scenario: Compromised CA
//!
//! In 2011, the DigiNotar CA was compromised. Attackers issued valid certificates
//! for google.com and used them to MITM Iranian Gmail users. Certificate pinning
//! would have detected this because the pinned public key wouldn't match.
//!
//! ## Pinning Strategies
//!
//! 1. **Pin the leaf certificate** — most restrictive, breaks on cert rotation
//! 2. **Pin the intermediate CA** — allows rotation under the same CA
//! 3. **Pin the public key** — survives cert renewal (same key pair)
//! 4. **Backup pins** — always include a backup pin for key rotation
//!
//! ## Why This Matters
//!
//! Mobile apps and embedded devices should pin certificates. Without pinning,
//! anyone who compromises a CA (or compels one legally) can MITM your traffic.

use ring::digest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A pin is a SHA-256 hash of a certificate's Subject Public Key Info (SPKI).
///
/// In real implementations, this is the hash of the DER-encoded SPKI.
/// We simulate it by hashing the public key bytes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pin {
    /// SHA-256 hash of the SPKI (32 bytes)
    pub spki_hash: Vec<u8>,
    /// Optional: hostname this pin applies to
    pub hostname: String,
    /// Optional: pin expiration
    pub expires_at: u64,
}

/// A pin set for a hostname — contains primary and backup pins.
#[derive(Debug, Clone)]
pub struct PinSet {
    pub hostname: String,
    /// Primary pins (must match at least one)
    pub pins: Vec<Pin>,
    /// Backup pins (for key rotation, not required to match now)
    pub backup_pins: Vec<Pin>,
    /// Whether to also allow standard CA validation as fallback
    pub include_subdomains: bool,
}

/// Simplified certificate for pinning exercises.
#[derive(Debug, Clone)]
pub struct PinnedCertificate {
    pub subject_cn: String,
    pub public_key: Vec<u8>,
    pub issuer_cn: String,
}

impl PinnedCertificate {
    /// Compute the SPKI pin hash for this certificate.
    ///
    /// In real TLS, this is SHA-256(DER-encoded SPKI).
    /// We simulate by hashing the public key bytes.
    pub fn compute_pin(&self) -> Vec<u8> {
        digest::digest(&digest::SHA256, &self.public_key)
            .as_ref()
            .to_vec()
    }
}

/// Exercise: Create a Pin from a certificate's public key.
///
/// Hints:
/// - Hash the public key with SHA-256
/// - Return a Pin with the hash, hostname, and a default expiration
pub fn create_pin(cert: &PinnedCertificate, hostname: &str, expires_at: u64) -> Pin {
    todo!("Implement pin creation")
}

/// Exercise: Verify that a certificate matches at least one pin in the pin set.
///
/// Hints:
/// - Compute the pin hash of the presented certificate
/// - Compare against all pins in the pin set (both primary and backup)
/// - Return Ok(()) if any primary pin matches
/// - Return Err if no primary pin matches
pub fn verify_pin(presented_cert: &PinnedCertificate, pin_set: &PinSet) -> Result<(), String> {
    todo!("Implement pin verification")
}

/// Exercise: Build a pin manager that stores pin sets for multiple hostnames.
///
/// The pin manager should:
/// - Add pin sets for hostnames
/// - Verify certificates against stored pins
/// - Support pin rotation (adding new pins, removing old ones)
pub struct PinManager {
    pin_sets: HashMap<String, PinSet>,
}

impl PinManager {
    pub fn new() -> Self {
        todo!("Create a new PinManager")
    }

    /// Add a pin set for a hostname.
    pub fn add_pin_set(&mut self, pin_set: PinSet) {
        todo!("Implement add_pin_set")
    }

    /// Verify a certificate against the stored pins for a hostname.
    ///
    /// If no pin set exists for the hostname, return an error (strict mode).
    pub fn verify(&self, cert: &PinnedCertificate, hostname: &str) -> Result<(), String> {
        todo!("Implement verification against stored pins")
    }

    /// Rotate pins: replace old pins with new ones for a hostname.
    ///
    /// This simulates what happens when a server gets a new certificate.
    /// The old pin should become a backup, and the new cert's pin becomes primary.
    pub fn rotate_pins(&mut self, hostname: &str, new_cert: &PinnedCertificate) -> Result<(), String> {
        todo!("Implement pin rotation")
    }
}

/// Exercise: Simulate a MITM attack against a pinned connection.
///
/// Show that even a valid certificate from a trusted CA is rejected
/// if it doesn't match the expected pin.
///
/// Hints:
/// - Create a pin set for "api.example.com" with a specific pin
/// - Present a valid cert with a different public key
/// - Verification should fail even though the cert is "valid"
pub fn simulate_pinning_attack(
    expected_pin: &PinSet,
    attacker_cert: &PinnedCertificate,
) -> Result<(), String> {
    todo!("Implement pinning attack simulation")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cert(cn: &str, key_bytes: Vec<u8>) -> PinnedCertificate {
        PinnedCertificate {
            subject_cn: cn.to_string(),
            public_key: key_bytes,
            issuer_cn: "Test CA".to_string(),
        }
    }

    #[test]
    fn test_compute_pin() {
        let cert = make_cert("example.com", vec![1u8; 32]);
        let pin = cert.compute_pin();
        assert_eq!(pin.len(), 32, "Pin should be 32 bytes (SHA-256)");
    }

    #[test]
    fn test_same_key_same_pin() {
        let cert1 = make_cert("example.com", vec![1u8; 32]);
        let cert2 = make_cert("example.com", vec![1u8; 32]);
        assert_eq!(cert1.compute_pin(), cert2.compute_pin());
    }

    #[test]
    fn test_different_key_different_pin() {
        let cert1 = make_cert("example.com", vec![1u8; 32]);
        let cert2 = make_cert("example.com", vec![2u8; 32]);
        assert_ne!(cert1.compute_pin(), cert2.compute_pin());
    }

    #[test]
    fn test_pin_verification_success() {
        let cert = make_cert("api.example.com", vec![42u8; 32]);
        let pin = Pin {
            spki_hash: cert.compute_pin(),
            hostname: "api.example.com".to_string(),
            expires_at: u64::MAX,
        };
        let pin_set = PinSet {
            hostname: "api.example.com".to_string(),
            pins: vec![pin],
            backup_pins: vec![],
            include_subdomains: false,
        };
        assert!(verify_pin(&cert, &pin_set).is_ok());
    }

    #[test]
    fn test_pin_verification_failure() {
        let cert = make_cert("api.example.com", vec![42u8; 32]);
        let wrong_pin = Pin {
            spki_hash: vec![0u8; 32],
            hostname: "api.example.com".to_string(),
            expires_at: u64::MAX,
        };
        let pin_set = PinSet {
            hostname: "api.example.com".to_string(),
            pins: vec![wrong_pin],
            backup_pins: vec![],
            include_subdomains: false,
        };
        assert!(verify_pin(&cert, &pin_set).is_err());
    }

    #[test]
    fn test_pin_manager_basic() {
        let mut manager = PinManager::new();
        let cert = make_cert("api.example.com", vec![42u8; 32]);
        let pin = Pin {
            spki_hash: cert.compute_pin(),
            hostname: "api.example.com".to_string(),
            expires_at: u64::MAX,
        };
        let pin_set = PinSet {
            hostname: "api.example.com".to_string(),
            pins: vec![pin],
            backup_pins: vec![],
            include_subdomains: false,
        };
        manager.add_pin_set(pin_set);
        assert!(manager.verify(&cert, "api.example.com").is_ok());
    }

    #[test]
    fn test_pin_manager_unknown_hostname() {
        let manager = PinManager::new();
        let cert = make_cert("unknown.com", vec![1u8; 32]);
        assert!(manager.verify(&cert, "unknown.com").is_err());
    }

    #[test]
    fn test_mitm_attack_rejected() {
        let real_cert = make_cert("api.example.com", vec![42u8; 32]);
        let pin = Pin {
            spki_hash: real_cert.compute_pin(),
            hostname: "api.example.com".to_string(),
            expires_at: u64::MAX,
        };
        let pin_set = PinSet {
            hostname: "api.example.com".to_string(),
            pins: vec![pin],
            backup_pins: vec![],
            include_subdomains: false,
        };
        let attacker_cert = make_cert("api.example.com", vec![99u8; 32]);
        let result = simulate_pinning_attack(&pin_set, &attacker_cert);
        assert!(result.is_err(), "MITM attack should be detected");
    }

    #[test]
    fn test_pin_rotation() {
        let mut manager = PinManager::new();
        let old_cert = make_cert("api.example.com", vec![42u8; 32]);
        let pin = Pin {
            spki_hash: old_cert.compute_pin(),
            hostname: "api.example.com".to_string(),
            expires_at: u64::MAX,
        };
        let pin_set = PinSet {
            hostname: "api.example.com".to_string(),
            pins: vec![pin],
            backup_pins: vec![],
            include_subdomains: false,
        };
        manager.add_pin_set(pin_set);

        let new_cert = make_cert("api.example.com", vec![99u8; 32]);
        manager.rotate_pins("api.example.com", &new_cert).unwrap();
        assert!(manager.verify(&new_cert, "api.example.com").is_ok());
    }
}
