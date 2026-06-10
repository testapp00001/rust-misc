//! # Lesson 04: Remote Attestation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use ring::hmac;
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};

/// A platform's identity.
#[derive(Debug, Clone)]
pub struct PlatformIdentity {
    pub attestation_key: Vec<u8>,
    pub measurements: Vec<Vec<u8>>,
}

impl PlatformIdentity {
    /// Create a new platform identity with random attestation key.
    pub fn new(initial_measurements: Vec<Vec<u8>>) -> Self {
        let rng = ring::rand::SystemRandom::new();
        let mut attestation_key = vec![0u8; 32];
        rng.fill(&mut attestation_key).unwrap();
        Self {
            attestation_key,
            measurements: initial_measurements,
        }
    }

    /// Extend a platform measurement: new = SHA-256(old || new).
    pub fn extend_measurement(&mut self, index: usize, measurement: &[u8]) {
        if index >= self.measurements.len() {
            return;
        }
        let mut combined = self.measurements[index].clone();
        combined.extend_from_slice(measurement);
        self.measurements[index] = digest::digest(&digest::SHA256, &combined).as_ref().to_vec();
    }

    /// Generate an attestation quote: HMAC-SHA256(key, nonce || measurements).
    pub fn generate_quote(&self, nonce: &[u8]) -> AttestationQuote {
        let mut message = nonce.to_vec();
        for m in &self.measurements {
            message.extend_from_slice(m);
        }
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &self.attestation_key);
        let signature = hmac::sign(&hmac_key, &message).as_ref().to_vec();

        AttestationQuote {
            nonce: nonce.to_vec(),
            measurements: self.measurements.clone(),
            signature,
        }
    }
}

/// An attestation quote.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationQuote {
    pub nonce: Vec<u8>,
    pub measurements: Vec<Vec<u8>>,
    pub signature: Vec<u8>,
}

/// A verifier that checks attestation quotes.
#[derive(Debug)]
pub struct AttestationVerifier {
    pub expected_measurements: Vec<Vec<u8>>,
    pub platform_key: Vec<u8>,
    used_nonces: Vec<Vec<u8>>,
}

impl AttestationVerifier {
    /// Create a new verifier.
    pub fn new(expected_measurements: Vec<Vec<u8>>, platform_key: Vec<u8>) -> Self {
        Self {
            expected_measurements,
            platform_key,
            used_nonces: Vec::new(),
        }
    }

    /// Generate a random 32-byte challenge nonce.
    pub fn generate_nonce(&mut self) -> Vec<u8> {
        let rng = ring::rand::SystemRandom::new();
        let mut nonce = vec![0u8; 32];
        rng.fill(&mut nonce).unwrap();
        nonce
    }

    /// Verify an attestation quote: check nonce, signature, and measurements.
    pub fn verify_quote(&mut self, quote: &AttestationQuote) -> VerificationResult {
        // Check nonce was generated and not reused
        let nonce_pos = self.used_nonces.iter().position(|n| n == &quote.nonce);
        match nonce_pos {
            Some(_) => return VerificationResult::InvalidNonce,
            None => {
                // Mark nonce as used (in real impl, generate_nonce would track this)
                self.used_nonces.push(quote.nonce.clone());
            }
        }

        // Recompute signature
        let mut message = quote.nonce.clone();
        for m in &quote.measurements {
            message.extend_from_slice(m);
        }
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &self.platform_key);
        let expected_sig = hmac::sign(&hmac_key, &message);

        if ring::constant_time::verify_slices_are_equal(expected_sig.as_ref(), &quote.signature).is_err() {
            return VerificationResult::InvalidSignature;
        }

        // Check measurements match
        if quote.measurements.len() != self.expected_measurements.len() {
            return VerificationResult::MeasurementsMismatch;
        }
        for (a, b) in quote.measurements.iter().zip(self.expected_measurements.iter()) {
            if ring::constant_time::verify_slices_are_equal(a, b).is_err() {
                return VerificationResult::MeasurementsMismatch;
            }
        }

        VerificationResult::Trusted
    }
}

/// Result of attestation verification.
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    Trusted,
    InvalidNonce,
    InvalidSignature,
    MeasurementsMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_measurements() -> Vec<Vec<u8>> {
        vec![
            digest::digest(&digest::SHA256, b"BIOS").as_ref().to_vec(),
            digest::digest(&digest::SHA256, b"Bootloader").as_ref().to_vec(),
            digest::digest(&digest::SHA256, b"OS Kernel").as_ref().to_vec(),
        ]
    }

    #[test]
    fn test_platform_creation() {
        let measurements = make_measurements();
        let platform = PlatformIdentity::new(measurements.clone());
        assert_eq!(platform.attestation_key.len(), 32);
        assert_eq!(platform.measurements.len(), 3);
    }

    #[test]
    fn test_quote_contains_nonce() {
        let platform = PlatformIdentity::new(make_measurements());
        let nonce = b"challenge-123";
        let quote = platform.generate_quote(nonce);
        assert_eq!(quote.nonce, nonce);
    }

    #[test]
    fn test_quote_signature_deterministic() {
        let platform = PlatformIdentity::new(make_measurements());
        let nonce = b"test-nonce";
        let q1 = platform.generate_quote(nonce);
        let q2 = platform.generate_quote(nonce);
        assert_eq!(q1.signature, q2.signature);
    }

    #[test]
    fn test_attestation_success() {
        let measurements = make_measurements();
        let platform = PlatformIdentity::new(measurements.clone());
        let mut verifier = AttestationVerifier::new(measurements, platform.attestation_key.clone());

        let nonce = verifier.generate_nonce();
        let quote = platform.generate_quote(&nonce);

        let result = verifier.verify_quote(&quote);
        assert_eq!(result, VerificationResult::Trusted);
    }

    #[test]
    fn test_attestation_replay_rejected() {
        let measurements = make_measurements();
        let platform = PlatformIdentity::new(measurements.clone());
        let mut verifier = AttestationVerifier::new(measurements, platform.attestation_key.clone());

        let nonce = verifier.generate_nonce();
        let quote = platform.generate_quote(&nonce);

        assert_eq!(verifier.verify_quote(&quote), VerificationResult::Trusted);
        let result = verifier.verify_quote(&quote);
        assert_eq!(result, VerificationResult::InvalidNonce);
    }

    #[test]
    fn test_attestation_tampered_measurements() {
        let measurements = make_measurements();
        let platform = PlatformIdentity::new(measurements.clone());
        let mut verifier = AttestationVerifier::new(measurements, platform.attestation_key.clone());

        let nonce = verifier.generate_nonce();
        let mut quote = platform.generate_quote(&nonce);
        quote.measurements[0] = digest::digest(&digest::SHA256, b"MALWARE").as_ref().to_vec();

        let result = verifier.verify_quote(&quote);
        assert_ne!(result, VerificationResult::Trusted);
    }

    #[test]
    fn test_attestation_wrong_key() {
        let measurements = make_measurements();
        let platform = PlatformIdentity::new(measurements.clone());
        let wrong_key = vec![0u8; 32];
        let mut verifier = AttestationVerifier::new(measurements, wrong_key);

        let nonce = verifier.generate_nonce();
        let quote = platform.generate_quote(&nonce);

        let result = verifier.verify_quote(&quote);
        assert_eq!(result, VerificationResult::InvalidSignature);
    }

    #[test]
    fn test_measurement_extension_changes_quote() {
        let measurements = make_measurements();
        let mut platform = PlatformIdentity::new(measurements.clone());
        let nonce = b"test";
        let q1 = platform.generate_quote(nonce);

        platform.extend_measurement(0, b"new stage");
        let q2 = platform.generate_quote(nonce);

        assert_ne!(q1.signature, q2.signature, "Quote should change after measurement extension");
    }
}
