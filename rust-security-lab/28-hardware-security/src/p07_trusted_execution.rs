//! # Lesson 07: Trusted Execution Environment (TEE)
//!
//! ## What is a TEE?
//!
//! A Trusted Execution Environment provides an isolated execution environment where:
//! - Code runs with hardware-enforced isolation from the rest of the system
//! - Data in the TEE is encrypted in memory (even the OS cannot read it)
//! - The TEE can attest its code to remote parties
//!
//! ## TEE Technologies
//!
//! | Technology | Vendor | Isolation Level |
//! |-----------|--------|-----------------|
//! | Intel SGX | Intel | Enclave in user space |
//! | ARM TrustZone | ARM | Secure World / Normal World |
//! | AMD SEV | AMD | VM-level encryption |
//! | Apple Secure Enclave | Apple | Separate processor |
//! | RISC-V Penglai | Open | Hardware-based |
//!
//! ## TEE vs Hypervisor
//!
//! A hypervisor isolates VMs but the hypervisor itself is trusted.
//! A TEE isolates code even from the OS and hypervisor — the hardware is the trust boundary.
//!
//! ## Attack: TEE Side Channels
//!
//! Despite hardware isolation, TEEs are vulnerable to:
//! - Cache-timing attacks (Flush+Reload, Prime+Probe)
//! - Speculative execution (Spectre variants)
//! - Power analysis (on embedded TEEs)
//!
//! Defense: constant-time algorithms, cache partitioning, noise injection.

use ring::digest;
use serde::{Deserialize, Serialize};

/// Simulated Trusted Execution Environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedExecutionEnv {
    /// The TEE's identity (measurement of loaded code).
    pub code_measurement: Vec<u8>,
    /// Isolated memory (encrypted in real TEEs).
    memory: Vec<u8>,
    /// Key sealing storage — keys bound to code measurement.
    sealed_keys: Vec<(Vec<u8>, Vec<u8>)>, // (sealed_data, binding_hash)
}

impl TrustedExecutionEnv {
    /// Exercise 1: Initialize a TEE with a code measurement.
    ///
    /// Hash the code to create the measurement, and initialize isolated memory.
    pub fn initialize(code: &[u8], memory_size: usize) -> Self {
        todo!("Initialize TEE with code measurement and isolated memory")
    }

    /// Exercise 2: Perform a secure computation inside the TEE.
    ///
    /// This simulates a function running in isolated memory. It takes
    /// input data, processes it (SHA-256 hash), and returns the result.
    ///
    /// In a real TEE, the input and output cross the isolation boundary
    /// through a controlled interface.
    pub fn secure_compute(&self, input: &[u8]) -> Vec<u8> {
        todo!("Compute SHA-256 of input inside the TEE")
    }

    /// Exercise 3: Seal a key to this TEE's code measurement.
    ///
    /// The key can only be unsealed by a TEE with the same code measurement.
    /// We simulate by XORing the key with SHA-256(measurement).
    pub fn seal_key(&mut self, key: &[u8]) {
        todo!("Seal key bound to TEE measurement")
    }

    /// Exercise 4: Unseal a key (only works if TEE measurement matches).
    ///
    /// Returns the unsealed key, or None if the binding doesn't match.
    pub fn unseal_key(&self, key_index: usize) -> Option<Vec<u8>> {
        todo!("Unseal key if TEE measurement matches")
    }

    /// Exercise 5: Generate an integrity report for remote verification.
    ///
    /// The report contains: SHA-256(measurement || nonce || memory_hash)
    /// This proves to a remote party what code is running and that memory is intact.
    pub fn generate_report(&self, nonce: &[u8]) -> Vec<u8> {
        todo!("Generate TEE integrity report")
    }

    /// Exercise 6: Check if the TEE memory has been tampered with.
    ///
    /// Compute a hash of all memory and compare against an expected value.
    /// Returns the memory hash for external verification.
    pub fn memory_hash(&self) -> Vec<u8> {
        todo!("Compute hash of TEE memory for integrity check")
    }
}

/// TEE lifecycle manager — controls creation, attestation, and destruction.
#[derive(Debug, Default)]
pub struct TeeManager {
    /// Active TEE instances.
    pub instances: Vec<TrustedExecutionEnv>,
    /// Known-good code measurements (whitelist).
    pub trusted_measurements: Vec<Vec<u8>>,
}

impl TeeManager {
    /// Exercise 7: Create and register a new TEE instance.
    ///
    /// Only creates the TEE if the code measurement is in the trusted list.
    /// Returns the index of the new TEE, or None if the code is not trusted.
    pub fn create_tee(&mut self, code: &[u8], memory_size: usize) -> Option<usize> {
        todo!("Create TEE only if code measurement is trusted")
    }

    /// Add a measurement to the trusted list.
    pub fn trust_measurement(&mut self, measurement: Vec<u8>) {
        self.trusted_measurements.push(measurement);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_initialization() {
        let code = b"trusted enclave code";
        let tee = TrustedExecutionEnv::initialize(code, 1024);
        assert_eq!(tee.code_measurement.len(), 32);
        assert_eq!(tee.memory.len(), 1024);
    }

    #[test]
    fn test_secure_compute() {
        let tee = TrustedExecutionEnv::initialize(b"code", 512);
        let result = tee.secure_compute(b"input data");
        let expected = digest::digest(&digest::SHA256, b"input data").as_ref().to_vec();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_seal_unseal_roundtrip() {
        let mut tee = TrustedExecutionEnv::initialize(b"code v1", 512);
        let key = b"encryption key material";
        tee.seal_key(key);
        let unsealed = tee.unseal_key(0).unwrap();
        assert_eq!(unsealed, key);
    }

    #[test]
    fn test_seal_binds_to_measurement() {
        let mut tee1 = TrustedExecutionEnv::initialize(b"code v1", 512);
        let mut tee2 = TrustedExecutionEnv::initialize(b"code v2", 512);

        let key = b"secret";
        tee1.seal_key(key);
        tee2.sealed_keys = tee1.sealed_keys.clone();

        let unsealed = tee2.unseal_key(0).unwrap();
        assert_ne!(unsealed, key, "Different measurement should not unseal correctly");
    }

    #[test]
    fn test_report_deterministic() {
        let tee = TrustedExecutionEnv::initialize(b"code", 512);
        let nonce = b"verifier-nonce";
        let r1 = tee.generate_report(nonce);
        let r2 = tee.generate_report(nonce);
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_memory_hash_deterministic() {
        let tee = TrustedExecutionEnv::initialize(b"code", 512);
        let h1 = tee.memory_hash();
        let h2 = tee.memory_hash();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_tee_manager_trusted_code() {
        let mut manager = TeeManager::default();
        let code = b"approved code";
        let measurement = digest::digest(&digest::SHA256, code).as_ref().to_vec();
        manager.trust_measurement(measurement);

        let idx = manager.create_tee(code, 512);
        assert!(idx.is_some());
    }

    #[test]
    fn test_tee_manager_rejects_untrusted_code() {
        let mut manager = TeeManager::default();
        let idx = manager.create_tee(b"unknown code", 512);
        assert!(idx.is_none(), "Untrusted code should be rejected");
    }
}
