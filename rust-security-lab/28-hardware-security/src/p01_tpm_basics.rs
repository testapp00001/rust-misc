//! # Lesson 01: TPM Basics — Trusted Platform Module
//!
//! ## What is a TPM?
//!
//! A Trusted Platform Module (TPM) is a hardware chip (or firmware implementation) that provides:
//! - **Secure key generation and storage**: Keys are generated inside the TPM and never leave it
//! - **Platform integrity measurement**: PCR (Platform Configuration Register) stores boot measurements
//! - **Remote attestation**: Sign PCR values to prove platform state to a remote party
//! - **Sealed storage**: Encrypt data that can only be decrypted when PCRs match expected values
//!
//! ## TPM 2.0 PCR Registers
//!
//! PCR registers are extended (not overwritten) with measurements at each boot stage:
//! - PCR[0]: BIOS/UEFI firmware
//! - PCR[1]: BIOS/UEFI configuration
//! - PCR[2]: Option ROMs
//! - PCR[3]: Option ROM configuration
//! - PCR[4]: Bootloader (MBR)
//! - PCR[7]: Secure Boot policy
//! - PCR[8]: OS bootloader
//! - PCR[9]: OS kernel
//!
//! Extension: `PCR[new] = Hash(PCR[old] || measurement)`
//!
//! ## Attack: TPM Reset
//!
//! An attacker with physical access might try to reset the TPM to clear PCR values,
//! allowing a modified boot chain to appear valid. Defense: bind keys to specific PCR
//! values using TPM2 policy sessions.
//!
//! ## Sealed Storage
//!
//! Data can be "sealed" to specific PCR values:
//! ```
//! seal(data, expected_pcrs) → sealed_blob
//! unseal(sealed_blob) → data  // only if current PCRs match expected_pcrs
//! ```

use ring::digest;
use serde::{Deserialize, Serialize};

/// Simulated PCR (Platform Configuration Register) bank.
/// In a real TPM, there are 24+ PCR registers. We simulate 8 for learning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcrBank {
    /// Each PCR holds a 32-byte (SHA-256) digest.
    pub registers: Vec<Vec<u8>>,
}

impl PcrBank {
    /// Exercise 1: Create a new PCR bank with `num_regs` registers,
    /// each initialized to 32 zero bytes (SHA-256 digest length).
    pub fn new(num_regs: usize) -> Self {
        todo!("Create PCR bank with num_regs zero-initialized registers")
    }

    /// Exercise 2: Extend a PCR register with a measurement.
    ///
    /// TPM extension formula: PCR[i] = SHA-256(PCR[i] || measurement)
    ///
    /// This is an append-only operation — you cannot reset a PCR to a previous state.
    ///
    /// Hints:
    /// - Concatenate current PCR value with the measurement
    /// - Hash the concatenation with SHA-256
    /// - Replace the PCR value with the new hash
    /// - Use `ring::digest::digest(&digest::SHA256, &combined)`
    pub fn extend(&mut self, index: usize, measurement: &[u8]) {
        todo!("Implement PCR extend: PCR[i] = SHA-256(PCR[i] || measurement)")
    }

    /// Exercise 3: Read the current value of a PCR register.
    ///
    /// Returns None if the index is out of bounds.
    pub fn read(&self, index: usize) -> Option<&[u8]> {
        todo!("Read PCR register value by index")
    }

    /// Exercise 4: Generate an attestation quote.
    ///
    /// A quote is a signed snapshot of PCR values. In a real TPM, the TPM signs
    /// the PCR values with an attestation key. Here we simulate by computing
    /// SHA-256 over all PCR values concatenated together.
    ///
    /// Returns the hash of all PCR values concatenated in order.
    ///
    /// Hints:
    /// - Concatenate all PCR values: PCR[0] || PCR[1] || ... || PCR[n]
    /// - Hash the concatenation
    pub fn quote(&self) -> Vec<u8> {
        todo!("Generate attestation quote from all PCR values")
    }

    /// Exercise 5: Verify that a quote matches the current PCR state.
    ///
    /// Recompute the quote and compare with the provided quote.
    /// Use constant-time comparison to prevent timing attacks.
    pub fn verify_quote(&self, expected_quote: &[u8]) -> bool {
        todo!("Verify attestation quote using constant-time comparison")
    }
}

/// Simulated sealed data — encrypted blob bound to specific PCR values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedBlob {
    /// The PCR values at sealing time (snapshot).
    pub pcr_snapshot: Vec<Vec<u8>>,
    /// The "encrypted" data (simulated with XOR against PCR-derived key).
    pub ciphertext: Vec<u8>,
}

/// Exercise 6: Seal data to the current PCR state.
///
/// In a real TPM, sealing encrypts data such that it can only be decrypted
/// when the PCR values match the sealed snapshot. We simulate this by:
/// 1. Snapshot the current PCR values
/// 2. Derive a key from the PCR snapshot (SHA-256 of all PCRs concatenated)
/// 3. XOR the data with the derived key
///
/// Hints:
/// - Get all PCR values concatenated and hash them to get a 32-byte key
/// - XOR each byte of data with the corresponding byte of the key (cycling)
pub fn seal(pcr_bank: &PcrBank, data: &[u8]) -> SealedBlob {
    todo!("Seal data to current PCR state")
}

/// Exercise 7: Unseal a blob, verifying PCR values match.
///
/// Returns the decrypted data only if the current PCR values match the snapshot.
/// Returns None if PCRs have changed (boot chain was tampered with).
///
/// Hints:
/// - Compare current PCR values with the snapshot (byte-by-byte)
/// - If they match, derive the same key and XOR to decrypt
pub fn unseal(pcr_bank: &PcrBank, blob: &SealedBlob) -> Option<Vec<u8>> {
    todo!("Unseal data only if PCR values match the snapshot")
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
