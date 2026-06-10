//! # Lesson 01: TPM Basics — Trusted Platform Module (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use serde::{Deserialize, Serialize};

/// Simulated PCR (Platform Configuration Register) bank.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcrBank {
    pub registers: Vec<Vec<u8>>,
}

impl PcrBank {
    /// Create a new PCR bank with `num_regs` registers, each initialized to 32 zero bytes.
    pub fn new(num_regs: usize) -> Self {
        Self {
            registers: vec![vec![0u8; 32]; num_regs],
        }
    }

    /// Extend a PCR register: PCR[i] = SHA-256(PCR[i] || measurement)
    pub fn extend(&mut self, index: usize, measurement: &[u8]) {
        if index >= self.registers.len() {
            return;
        }
        let mut combined = Vec::with_capacity(32 + measurement.len());
        combined.extend_from_slice(&self.registers[index]);
        combined.extend_from_slice(measurement);
        self.registers[index] = digest::digest(&digest::SHA256, &combined).as_ref().to_vec();
    }

    /// Read the current value of a PCR register.
    pub fn read(&self, index: usize) -> Option<&[u8]> {
        self.registers.get(index).map(|v| v.as_slice())
    }

    /// Generate an attestation quote: SHA-256 of all PCR values concatenated.
    pub fn quote(&self) -> Vec<u8> {
        let mut combined = Vec::new();
        for reg in &self.registers {
            combined.extend_from_slice(reg);
        }
        digest::digest(&digest::SHA256, &combined).as_ref().to_vec()
    }

    /// Verify that a quote matches the current PCR state using constant-time comparison.
    pub fn verify_quote(&self, expected_quote: &[u8]) -> bool {
        let current = self.quote();
        ring::constant_time::verify_slices_are_equal(&current, expected_quote).is_ok()
    }
}

/// Simulated sealed data — encrypted blob bound to specific PCR values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedBlob {
    pub pcr_snapshot: Vec<Vec<u8>>,
    pub ciphertext: Vec<u8>,
}

fn derive_key_from_pcrs(pcr_bank: &PcrBank) -> Vec<u8> {
    let mut combined = Vec::new();
    for reg in &pcr_bank.registers {
        combined.extend_from_slice(reg);
    }
    digest::digest(&digest::SHA256, &combined).as_ref().to_vec()
}

/// Seal data to the current PCR state using XOR with a PCR-derived key.
pub fn seal(pcr_bank: &PcrBank, data: &[u8]) -> SealedBlob {
    let key = derive_key_from_pcrs(pcr_bank);
    let ciphertext: Vec<u8> = data.iter().enumerate().map(|(i, &b)| b ^ key[i % key.len()]).collect();
    SealedBlob {
        pcr_snapshot: pcr_bank.registers.clone(),
        ciphertext,
    }
}

/// Unseal a blob, verifying PCR values match. Returns None if PCRs changed.
pub fn unseal(pcr_bank: &PcrBank, blob: &SealedBlob) -> Option<Vec<u8>> {
    // Verify PCRs match the snapshot
    if pcr_bank.registers.len() != blob.pcr_snapshot.len() {
        return None;
    }
    for (current, snapshot) in pcr_bank.registers.iter().zip(blob.pcr_snapshot.iter()) {
        if ring::constant_time::verify_slices_are_equal(current, snapshot).is_err() {
            return None;
        }
    }
    // Derive the same key and decrypt
    let key = derive_key_from_pcrs(pcr_bank);
    let plaintext: Vec<u8> = blob.ciphertext.iter().enumerate().map(|(i, &b)| b ^ key[i % key.len()]).collect();
    Some(plaintext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcr_bank_initialization() {
        let bank = PcrBank::new(8);
        assert_eq!(bank.registers.len(), 8);
        for reg in &bank.registers {
            assert_eq!(reg.len(), 32);
            assert!(reg.iter().all(|&b| b == 0), "PCR should be zero-initialized");
        }
    }

    #[test]
    fn test_pcr_extend_changes_value() {
        let mut bank = PcrBank::new(8);
        let before = bank.read(0).unwrap().to_vec();
        bank.extend(0, b"BIOS measurement");
        let after = bank.read(0).unwrap().to_vec();
        assert_ne!(before, after, "PCR value should change after extend");
    }

    #[test]
    fn test_pcr_extend_deterministic() {
        let mut bank1 = PcrBank::new(8);
        let mut bank2 = PcrBank::new(8);
        bank1.extend(0, b"measurement");
        bank2.extend(0, b"measurement");
        assert_eq!(bank1.read(0), bank2.read(0), "Same measurement should produce same PCR value");
    }

    #[test]
    fn test_pcr_extend_order_matters() {
        let mut bank1 = PcrBank::new(8);
        let mut bank2 = PcrBank::new(8);
        bank1.extend(0, b"A");
        bank1.extend(0, b"B");
        bank2.extend(0, b"B");
        bank2.extend(0, b"A");
        assert_ne!(bank1.read(0), bank2.read(0), "PCR extension order should matter");
    }

    #[test]
    fn test_pcr_read_out_of_bounds() {
        let bank = PcrBank::new(8);
        assert!(bank.read(8).is_none(), "Out-of-bounds read should return None");
        assert!(bank.read(100).is_none());
    }

    #[test]
    fn test_quote_deterministic() {
        let mut bank = PcrBank::new(8);
        bank.extend(0, b"boot");
        bank.extend(1, b"config");
        let q1 = bank.quote();
        let q2 = bank.quote();
        assert_eq!(q1, q2, "Quote should be deterministic");
    }

    #[test]
    fn test_quote_changes_with_pcr() {
        let mut bank = PcrBank::new(8);
        let q1 = bank.quote();
        bank.extend(0, b"new measurement");
        let q2 = bank.quote();
        assert_ne!(q1, q2, "Quote should change when PCR changes");
    }

    #[test]
    fn test_seal_unseal_roundtrip() {
        let mut bank = PcrBank::new(8);
        bank.extend(0, b"BIOS v1.0");
        bank.extend(1, b"Secure Boot enabled");

        let secret = b"disk encryption key";
        let blob = seal(&bank, secret);

        let unsealed = unseal(&bank, &blob);
        assert_eq!(unsealed, Some(secret.to_vec()));
    }

    #[test]
    fn test_unseal_fails_on_pcr_change() {
        let mut bank = PcrBank::new(8);
        bank.extend(0, b"BIOS v1.0");

        let secret = b"disk encryption key";
        let blob = seal(&bank, secret);

        // Simulate a tampered boot — extend PCR with different measurement
        bank.extend(0, b"modified boot");

        let result = unseal(&bank, &blob);
        assert!(result.is_none(), "Unseal should fail when PCRs have changed");
    }
}
