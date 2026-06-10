//! # Lesson 07: Trusted Execution Environment (TEE) (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use serde::{Deserialize, Serialize};

/// Simulated Trusted Execution Environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedExecutionEnv {
    pub code_measurement: Vec<u8>,
    memory: Vec<u8>,
    /// (sealed_data, measurement_at_seal_time)
    pub sealed_keys: Vec<(Vec<u8>, Vec<u8>)>,
}

impl TrustedExecutionEnv {
    /// Initialize a TEE with a code measurement.
    pub fn initialize(code: &[u8], memory_size: usize) -> Self {
        let code_measurement = digest::digest(&digest::SHA256, code).as_ref().to_vec();
        Self {
            code_measurement,
            memory: vec![0u8; memory_size],
            sealed_keys: Vec::new(),
        }
    }

    /// Perform SHA-256 computation inside the TEE.
    pub fn secure_compute(&self, input: &[u8]) -> Vec<u8> {
        digest::digest(&digest::SHA256, input).as_ref().to_vec()
    }

    /// Seal a key bound to TEE measurement: XOR with SHA-256(measurement).
    /// Stores the sealed data along with the measurement at seal time.
    pub fn seal_key(&mut self, key: &[u8]) {
        let binding = digest::digest(&digest::SHA256, &self.code_measurement)
            .as_ref()
            .to_vec();
        let sealed: Vec<u8> = key
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ binding[i % binding.len()])
            .collect();
        self.sealed_keys.push((sealed, self.code_measurement.clone()));
    }

    /// Unseal a key. Derives the binding from the CURRENT code_measurement.
    /// If the measurement has changed since sealing, the XOR key will differ
    /// and the result will be garbage (but always the same length).
    pub fn unseal_key(&self, key_index: usize) -> Option<Vec<u8>> {
        let (sealed, _seal_measurement) = self.sealed_keys.get(key_index)?;
        let binding = digest::digest(&digest::SHA256, &self.code_measurement)
            .as_ref()
            .to_vec();
        Some(
            sealed
                .iter()
                .enumerate()
                .map(|(i, &b)| b ^ binding[i % binding.len()])
                .collect(),
        )
    }

    /// Generate integrity report: SHA-256(measurement || nonce || memory_hash).
    pub fn generate_report(&self, nonce: &[u8]) -> Vec<u8> {
        let mem_hash = self.memory_hash();
        let mut combined = self.code_measurement.clone();
        combined.extend_from_slice(nonce);
        combined.extend_from_slice(&mem_hash);
        digest::digest(&digest::SHA256, &combined).as_ref().to_vec()
    }

    /// Compute hash of TEE memory for integrity verification.
    pub fn memory_hash(&self) -> Vec<u8> {
        digest::digest(&digest::SHA256, &self.memory).as_ref().to_vec()
    }
}

/// TEE lifecycle manager.
#[derive(Debug, Default)]
pub struct TeeManager {
    pub instances: Vec<TrustedExecutionEnv>,
    pub trusted_measurements: Vec<Vec<u8>>,
}

impl TeeManager {
    /// Create a TEE only if code measurement is trusted.
    pub fn create_tee(&mut self, code: &[u8], memory_size: usize) -> Option<usize> {
        let measurement = digest::digest(&digest::SHA256, code).as_ref().to_vec();
        if !self.trusted_measurements.contains(&measurement) {
            return None;
        }
        let tee = TrustedExecutionEnv::initialize(code, memory_size);
        self.instances.push(tee);
        Some(self.instances.len() - 1)
    }

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
