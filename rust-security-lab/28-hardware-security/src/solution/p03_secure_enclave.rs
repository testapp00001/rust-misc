//! # Lesson 03: Secure Enclaves — SGX, TrustZone (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use serde::{Deserialize, Serialize};

/// Simulated enclave memory region.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveMemory {
    data: Vec<u8>,
    pub initialized: bool,
}

/// Simulated secure enclave.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureEnclave {
    pub measurement: Vec<u8>,
    memory: EnclaveMemory,
    pub sealed_secrets: Vec<u8>,
}

impl SecureEnclave {
    /// Create a new secure enclave from code. Measurement = SHA-256(code).
    pub fn create(code: &[u8], memory_size: usize) -> Self {
        let measurement = digest::digest(&digest::SHA256, code).as_ref().to_vec();
        Self {
            measurement,
            memory: EnclaveMemory {
                data: vec![0u8; memory_size],
                initialized: true,
            },
            sealed_secrets: Vec::new(),
        }
    }

    /// Write data into enclave protected memory at a given offset.
    pub fn memory_write(&mut self, offset: usize, data: &[u8]) -> bool {
        if offset + data.len() > self.memory.data.len() {
            return false;
        }
        self.memory.data[offset..offset + data.len()].copy_from_slice(data);
        true
    }

    /// Read data from enclave protected memory.
    pub fn memory_read(&self, offset: usize, len: usize) -> Option<Vec<u8>> {
        if offset + len > self.memory.data.len() {
            return None;
        }
        Some(self.memory.data[offset..offset + len].to_vec())
    }

    /// Seal a secret using XOR with measurement-derived key.
    pub fn seal_secret(&mut self, secret: &[u8]) {
        self.sealed_secrets = secret
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ self.measurement[i % self.measurement.len()])
            .collect();
    }

    /// Unseal a secret (reverse the XOR).
    pub fn unseal_secret(&self) -> Vec<u8> {
        self.sealed_secrets
            .iter()
            .enumerate()
            .map(|(i, &b)| b ^ self.measurement[i % self.measurement.len()])
            .collect()
    }

    /// Simulate a side-channel read of enclave memory.
    pub fn side_channel_read(&self, offset: usize, len: usize) -> Option<Vec<u8>> {
        // In a real attack, this bypasses enclave protections
        self.memory_read(offset, len)
    }

    /// Compute attestation report: SHA-256(measurement || nonce).
    pub fn attestation_report(&self, nonce: &[u8]) -> Vec<u8> {
        let mut combined = self.measurement.clone();
        combined.extend_from_slice(nonce);
        digest::digest(&digest::SHA256, &combined).as_ref().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enclave_creation() {
        let code = b"fn main() { process_secret(); }";
        let enclave = SecureEnclave::create(code, 1024);
        assert_eq!(enclave.measurement.len(), 32, "Measurement should be SHA-256 (32 bytes)");
        assert!(enclave.memory.initialized);
    }

    #[test]
    fn test_enclave_measurement_deterministic() {
        let code = b"enclave code";
        let e1 = SecureEnclave::create(code, 512);
        let e2 = SecureEnclave::create(code, 512);
        assert_eq!(e1.measurement, e2.measurement);
    }

    #[test]
    fn test_memory_write_read() {
        let code = b"enclave";
        let mut enclave = SecureEnclave::create(code, 256);
        let data = b"secret key material here";
        assert!(enclave.memory_write(10, data));
        let read_back = enclave.memory_read(10, data.len()).unwrap();
        assert_eq!(read_back, data);
    }

    #[test]
    fn test_memory_write_out_of_bounds() {
        let code = b"enclave";
        let mut enclave = SecureEnclave::create(code, 16);
        assert!(!enclave.memory_write(100, b"data"));
    }

    #[test]
    fn test_memory_read_out_of_bounds() {
        let code = b"enclave";
        let enclave = SecureEnclave::create(code, 16);
        assert!(enclave.memory_read(100, 10).is_none());
    }

    #[test]
    fn test_seal_unseal_roundtrip() {
        let code = b"enclave v1";
        let mut enclave = SecureEnclave::create(code, 512);
        let secret = b"top secret data";
        enclave.seal_secret(secret);
        let unsealed = enclave.unseal_secret();
        assert_eq!(unsealed, secret);
    }

    #[test]
    fn test_seal_binds_to_measurement() {
        let mut enclave1 = SecureEnclave::create(b"enclave v1", 512);
        let mut enclave2 = SecureEnclave::create(b"enclave v2", 512);
        let secret = b"shared secret";

        enclave1.seal_secret(secret);
        enclave2.sealed_secrets = enclave1.sealed_secrets.clone();

        let unsealed = enclave2.unseal_secret();
        assert_ne!(unsealed, secret, "Different enclave should not unseal correctly");
    }

    #[test]
    fn test_attestation_report() {
        let enclave = SecureEnclave::create(b"trusted code", 512);
        let nonce = b"verifier-challenge-123";
        let report = enclave.attestation_report(nonce);
        assert_eq!(report.len(), 32);

        let report2 = enclave.attestation_report(nonce);
        assert_eq!(report, report2);
    }

    #[test]
    fn test_side_channel_demonstrates_vulnerability() {
        let code = b"enclave";
        let mut enclave = SecureEnclave::create(code, 256);
        enclave.memory_write(0, b"SECRET");
        let leaked = enclave.side_channel_read(0, 6).unwrap();
        assert_eq!(leaked, b"SECRET");
    }
}
