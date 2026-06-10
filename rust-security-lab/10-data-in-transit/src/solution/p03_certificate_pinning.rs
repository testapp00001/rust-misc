//! # Lesson 03: Certificate Pinning (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pin {
    pub spki_hash: Vec<u8>,
    pub hostname: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct PinSet {
    pub hostname: String,
    pub pins: Vec<Pin>,
    pub backup_pins: Vec<Pin>,
    pub include_subdomains: bool,
}

#[derive(Debug, Clone)]
pub struct PinnedCertificate {
    pub subject_cn: String,
    pub public_key: Vec<u8>,
    pub issuer_cn: String,
}

impl PinnedCertificate {
    pub fn compute_pin(&self) -> Vec<u8> {
        digest::digest(&digest::SHA256, &self.public_key)
            .as_ref()
            .to_vec()
    }
}

pub fn create_pin(cert: &PinnedCertificate, hostname: &str, expires_at: u64) -> Pin {
    Pin {
        spki_hash: cert.compute_pin(),
        hostname: hostname.to_string(),
        expires_at,
    }
}

pub fn verify_pin(presented_cert: &PinnedCertificate, pin_set: &PinSet) -> Result<(), String> {
    let cert_pin = presented_cert.compute_pin();

    // Check primary pins
    for pin in &pin_set.pins {
        if pin.spki_hash == cert_pin {
            return Ok(());
        }
    }

    Err(format!(
        "Certificate pin does not match any primary pin for '{}'",
        pin_set.hostname
    ))
}

pub struct PinManager {
    pin_sets: HashMap<String, PinSet>,
}

impl PinManager {
    pub fn new() -> Self {
        PinManager {
            pin_sets: HashMap::new(),
        }
    }

    pub fn add_pin_set(&mut self, pin_set: PinSet) {
        self.pin_sets.insert(pin_set.hostname.clone(), pin_set);
    }

    pub fn verify(&self, cert: &PinnedCertificate, hostname: &str) -> Result<(), String> {
        let pin_set = self
            .pin_sets
            .get(hostname)
            .ok_or_else(|| format!("No pin set for hostname '{}'", hostname))?;
        verify_pin(cert, pin_set)
    }

    pub fn rotate_pins(&mut self, hostname: &str, new_cert: &PinnedCertificate) -> Result<(), String> {
        let pin_set = self
            .pin_sets
            .get_mut(hostname)
            .ok_or_else(|| format!("No pin set for hostname '{}'", hostname))?;

        // Move current primary pins to backup
        let old_pins: Vec<Pin> = pin_set.pins.drain(..).collect();
        pin_set.backup_pins.extend(old_pins);

        // Add new primary pin
        let new_pin = create_pin(new_cert, hostname, u64::MAX);
        pin_set.pins.push(new_pin);

        Ok(())
    }
}

pub fn simulate_pinning_attack(
    expected_pin: &PinSet,
    attacker_cert: &PinnedCertificate,
) -> Result<(), String> {
    verify_pin(attacker_cert, expected_pin)
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
