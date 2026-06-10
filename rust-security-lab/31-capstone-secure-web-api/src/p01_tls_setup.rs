//! # Lesson 01: TLS Configuration
//!
//! ## What is TLS?
//!
//! Transport Layer Security (TLS) encrypts communication between client and server.
//! TLS 1.3 is the current standard, removing all legacy weaknesses.
//!
//! ## Key Concepts
//!
//! - **Certificate chains**: Server cert signed by intermediate CA, signed by root CA
//! - **Cipher suites**: TLS 1.3 has only 5 cipher suites, all using AEAD
//! - **Forward secrecy**: Ephemeral keys mean compromising the long-term key does not
//!   compromise past sessions
//!
//! ## TLS 1.3 Cipher Suites
//!
//! | Suite | Key Exchange | AEAD | Hash |
//! |-------|-------------|------|------|
//! | TLS_AES_128_GCM_SHA256 | x25519/P-256 | AES-128-GCM | SHA-256 |
//! | TLS_AES_256_GCM_SHA384 | x25519/P-384 | AES-256-GCM | SHA-384 |
//! | TLS_CHACHA20_POLY1305_SHA256 | x25519 | ChaCha20-Poly1305 | SHA-256 |
//!
//! ## Attack Context
//!
//! - **Downgrade attacks**: Attacker forces weaker TLS version. Defense: TLS 1.3 removes
//!   negotiation of version (downgrade is detected and rejected).
//! - **Certificate spoofing**: Attacker presents a fake certificate. Defense: Certificate
//!   pinning, proper chain validation.
//! - **Weak cipher suites**: Attacker exploits known weaknesses in RC4, DES, 3DES, CBC mode.
//!   Defense: TLS 1.3 only allows AEAD ciphers.

use serde::{Deserialize, Serialize};

/// Supported TLS protocol versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsVersion {
    /// TLS 1.2 — still widely supported but should be migrated away from
    Tls12,
    /// TLS 1.3 — current standard, preferred
    Tls13,
}

/// TLS cipher suite identifiers for TLS 1.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CipherSuite {
    /// AES-128-GCM with SHA-256
    Aes128GcmSha256,
    /// AES-256-GCM with SHA-384
    Aes256GcmSha384,
    /// ChaCha20-Poly1305 with SHA-256
    Chacha20Poly1305Sha256,
}

/// Represents a loaded TLS certificate and its private key.
#[derive(Debug, Clone)]
pub struct TlsCertificate {
    /// PEM-encoded certificate chain (server cert + intermediates)
    pub cert_pem: Vec<u8>,
    /// PEM-encoded private key
    pub key_pem: Vec<u8>,
    /// The subject Common Name (CN) from the certificate
    pub subject_cn: String,
    /// Certificate expiry as Unix timestamp
    pub not_after: u64,
}

/// TLS server configuration.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Minimum TLS version allowed
    pub min_version: TlsVersion,
    /// Allowed cipher suites (in preference order)
    pub cipher_suites: Vec<CipherSuite>,
    /// The server certificate
    pub certificate: TlsCertificate,
    /// Whether to require client certificates (mutual TLS)
    pub require_client_cert: bool,
    /// PEM-encoded CA certificates for client cert verification
    pub client_ca_pem: Option<Vec<u8>>,
}

/// Exercise 1: Create a default TLS 1.3 configuration.
///
/// Build a TlsConfig that:
/// - Requires TLS 1.3 minimum
/// - Allows all three TLS 1.3 cipher suites in order: AES-256-GCM, AES-128-GCM, ChaCha20
/// - Does not require client certificates
///
/// Use the provided `cert` parameter for the certificate.
pub fn create_tls_config(cert: TlsCertificate) -> TlsConfig {
    todo!("Create a TLS 1.3 configuration")
}

/// Exercise 2: Validate a TLS configuration for security.
///
/// Check that:
/// 1. Minimum version is TLS 1.3
/// 2. At least one cipher suite is configured
/// 3. The certificate has not expired (compare `not_after` to `current_time`)
/// 4. The certificate key PEM is not empty
///
/// Return Ok(()) if valid, or an Err with a description of the problem.
pub fn validate_tls_config(config: &TlsConfig, current_time: u64) -> Result<(), String> {
    todo!("Validate TLS configuration security")
}

/// Exercise 3: Check if a cipher suite is considered strong.
///
/// Strong cipher suites use AEAD and are approved for TLS 1.3.
/// All three TLS 1.3 cipher suites are strong.
///
/// Return true if the cipher suite is strong, false otherwise.
pub fn is_strong_cipher(cipher: &CipherSuite) -> bool {
    todo!("Check cipher suite strength")
}

/// Exercise 4: Build a certificate fingerprint for pinning.
///
/// Compute the SHA-256 hash of the DER-encoded certificate (given as PEM bytes).
/// For this exercise, simply compute SHA-256 of the raw cert_pem bytes.
///
/// Return the hex-encoded fingerprint.
pub fn certificate_fingerprint(cert_pem: &[u8]) -> String {
    todo!("Compute certificate fingerprint")
}

/// Exercise 5: Build a mutual TLS (mTLS) configuration.
///
/// Modify the given TLS config to:
/// - Require client certificates
/// - Set the client CA for verification
///
/// Return a new TlsConfig with these settings applied.
pub fn configure_mtls(mut config: TlsConfig, client_ca_pem: Vec<u8>) -> TlsConfig {
    todo!("Configure mutual TLS")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cert() -> TlsCertificate {
        TlsCertificate {
            cert_pem: b"-----BEGIN CERTIFICATE-----\nMIIB...sample...\n-----END CERTIFICATE-----".to_vec(),
            key_pem: b"-----BEGIN PRIVATE KEY-----\nMIIE...sample...\n-----END PRIVATE KEY-----".to_vec(),
            subject_cn: "example.com".to_string(),
            not_after: 2000000000, // Far future
        }
    }

    #[test]
    fn test_create_tls_config_defaults() {
        let config = create_tls_config(sample_cert());
        assert_eq!(config.min_version, TlsVersion::Tls13);
        assert_eq!(config.cipher_suites.len(), 3);
        assert_eq!(config.cipher_suites[0], CipherSuite::Aes256GcmSha384);
        assert_eq!(config.cipher_suites[1], CipherSuite::Aes128GcmSha256);
        assert_eq!(config.cipher_suites[2], CipherSuite::Chacha20Poly1305Sha256);
        assert!(!config.require_client_cert);
    }

    #[test]
    fn test_validate_config_good() {
        let config = create_tls_config(sample_cert());
        assert!(validate_tls_config(&config, 1700000000).is_ok());
    }

    #[test]
    fn test_validate_config_expired_cert() {
        let mut cert = sample_cert();
        cert.not_after = 1000000000; // Expired
        let config = create_tls_config(cert);
        assert!(validate_tls_config(&config, 1700000000).is_err());
    }

    #[test]
    fn test_validate_config_empty_key() {
        let mut cert = sample_cert();
        cert.key_pem = Vec::new();
        let config = create_tls_config(cert);
        assert!(validate_tls_config(&config, 1700000000).is_err());
    }

    #[test]
    fn test_strong_cipher_suites() {
        assert!(is_strong_cipher(&CipherSuite::Aes128GcmSha256));
        assert!(is_strong_cipher(&CipherSuite::Aes256GcmSha384));
        assert!(is_strong_cipher(&CipherSuite::Chacha20Poly1305Sha256));
    }

    #[test]
    fn test_certificate_fingerprint_deterministic() {
        let cert = sample_cert();
        let fp1 = certificate_fingerprint(&cert.cert_pem);
        let fp2 = certificate_fingerprint(&cert.cert_pem);
        assert_eq!(fp1, fp2);
        assert!(!fp1.is_empty());
        // Should be hex-encoded (only hex chars)
        assert!(fp1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_certificate_fingerprint_different_certs() {
        let fp1 = certificate_fingerprint(b"cert-a");
        let fp2 = certificate_fingerprint(b"cert-b");
        assert_ne!(fp1, fp2);
    }

    #[test]
    fn test_configure_mtls() {
        let config = create_tls_config(sample_cert());
        let mtls_config = configure_mtls(config, b"ca-pem-data".to_vec());
        assert!(mtls_config.require_client_cert);
        assert!(mtls_config.client_ca_pem.is_some());
    }
}
