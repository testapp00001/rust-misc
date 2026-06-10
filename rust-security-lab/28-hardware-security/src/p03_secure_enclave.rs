//! # Lesson 03: Secure Enclaves — SGX, TrustZone, Isolated Execution
//!
//! ## What is a Secure Enclave?
//!
//! A secure enclave is a protected region of memory and execution that even the OS
//! cannot access. The CPU enforces isolation at the hardware level.
//!
//! ### Intel SGX (Software Guard Extensions)
//! - Creates "enclaves" in user-space memory
//! - Memory is encrypted and integrity-protected by the CPU
//! - OS, hypervisor, and other processes cannot read enclave memory
//! - Enclave code can be attested remotely
//!
//! ### ARM TrustZone
//! - Splits the processor into "Secure World" and "Normal World"
//! - Secure World has access to all memory; Normal World cannot access Secure memory
//! - Used in mobile devices (Android Keystore, Apple Secure Enclave)
//!
//! ## Attack: Side-Channel on Enclaves
//!
//! Even with hardware isolation, enclaves are vulnerable to side-channel attacks:
//! - **Cache timing**: Attacker observes cache access patterns to infer enclave behavior
//! - **Speculative execution**: Spectre/Meltdown can leak data across isolation boundaries
//! - **Page table monitoring**: OS controls page tables and can observe enclave page faults
//!
//! Defense: constant-time code, cache line flushing, oblivious RAM (ORAM) techniques.

use ring::digest;
use serde::{Deserialize, Serialize};

/// Simulated enclave memory region — isolated from the "OS".
/// In a real SGX enclave, this memory is encrypted by the CPU.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnclaveMemory {
    /// Encrypted memory content (simulated — we store plaintext but mark it as "protected").
    data: Vec<u8>,
    /// Whether the enclave is currently initialized and running.
    initialized: bool,
}

/// Simulated secure enclave.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureEnclave {
    /// Enclave identity (measurement of enclave code).
    pub measurement: Vec<u8>,
    /// Protected memory region.
    memory: EnclaveMemory,
    /// Sealed secrets that only this enclave can access.
    sealed_secrets: Vec<u8>,
}

impl SecureEnclave {
    /// Exercise 1: Create a new secure enclave from code.
    ///
    /// Compute the enclave measurement (hash of the code) to establish identity.
    /// The measurement is SHA-256 of the code bytes.
    ///
    /// Hints:
    /// - Hash the code with `ring::digest::digest(&digest::SHA256, code)`
    /// - Initialize memory as a zero-filled vector of `memory_size` bytes
    /// - Set initialized to true
    pub fn create(code: &[u8], memory_size: usize) -> Self {
        todo!("Create enclave with measurement and isolated memory")
    }

    /// Exercise 2: Write data into enclave memory at a given offset.
    ///
    /// In a real enclave, this memory is encrypted — the OS cannot read it.
    /// Returns false if the write would go out of bounds.
    pub fn memory_write(&mut self, offset: usize, data: &[u8]) -> bool {
        todo!("Write to enclave protected memory")
    }

    /// Exercise 3: Read data from enclave memory at a given offset.
    ///
    /// Returns None if the read would go out of bounds.
    pub fn memory_read(&self, offset: usize, len: usize) -> Option<Vec<u8>> {
        todo!("Read from enclave protected memory")
    }

    /// Exercise 4: Seal a secret inside the enclave.
    ///
    /// "Sealing" binds data to the enclave's identity. Only an enclave with
    /// the same measurement can unseal it. We simulate this by XORing the
    /// secret with a key derived from the enclave measurement.
    ///
    /// Hints:
    /// - Use the measurement as a key seed
    /// - XOR each byte of data with the corresponding byte of measurement (cycling)
    /// - Store the result in sealed_secrets
    pub fn seal_secret(&mut self, secret: &[u8]) {
        todo!("Seal secret inside enclave using measurement-derived key")
    }

    /// Exercise 5: Unseal a secret (only if enclave measurement matches).
    ///
    /// Reverses the seal operation. Returns the original secret.
    pub fn unseal_secret(&self) -> Vec<u8> {
        todo!("Unseal secret using measurement-derived key")
    }

    /// Exercise 6: Simulate a side-channel attack.
    ///
    /// An attacker observes memory access patterns. Given the enclave memory
    /// and an offset, the attacker can read whatever is at that offset
    /// (simulating a cache-timing side channel).
    ///
    /// This demonstrates why enclave code must be written to avoid
    /// secret-dependent memory access patterns.
    pub fn side_channel_read(&self, offset: usize, len: usize) -> Option<Vec<u8>> {
        todo!("Simulate side-channel memory read (demonstrate the vulnerability)")
    }

    /// Exercise 7: Compute the enclave's attestation report.
    ///
    /// An attestation report binds the enclave identity (measurement) to
    /// a nonce provided by the verifier. We simulate this as:
    /// SHA-256(measurement || nonce)
    ///
    /// This proves to a remote party that a specific enclave is running.
    pub fn attestation_report(&self, nonce: &[u8]) -> Vec<u8> {
        todo!("Compute attestation report: SHA-256(measurement || nonce)")
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

        // Different measurement means different key — unseal should produce garbage
        let unsealed = enclave2.unseal_secret();
        assert_ne!(unsealed, secret, "Different enclave should not unseal correctly");
    }

    #[test]
    fn test_attestation_report() {
        let enclave = SecureEnclave::create(b"trusted code", 512);
        let nonce = b"verifier-challenge-123";
        let report = enclave.attestation_report(nonce);
        assert_eq!(report.len(), 32);

        // Same nonce should produce same report
        let report2 = enclave.attestation_report(nonce);
        assert_eq!(report, report2);
    }

    #[test]
    fn test_side_channel_demonstrates_vulnerability() {
        let code = b"enclave";
        let mut enclave = SecureEnclave::create(code, 256);
        enclave.memory_write(0, b"SECRET");
        // Side channel can read enclave memory (demonstrates the attack)
        let leaked = enclave.side_channel_read(0, 6).unwrap();
        assert_eq!(leaked, b"SECRET");
    }
}
