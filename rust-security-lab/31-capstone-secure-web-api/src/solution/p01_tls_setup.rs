//! # Lesson 01: TLS Configuration — Solution
//!
//! Certificate loading, TLS 1.3, cipher suite selection.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Supported TLS protocol versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsVersion {
    Tls12,
    Tls13,
}

/// TLS cipher suite identifiers for TLS 1.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CipherSuite {
    Aes128GcmSha256,
    Aes256GcmSha384,
    Chacha20Poly1305Sha256,
}

/// Represents a loaded TLS certificate and its private key.
#[derive(Debug, Clone)]
pub struct TlsCertificate {
    pub cert_pem: Vec<u8>,
    pub key_pem: Vec<u8>,
    pub subject_cn: String,
    pub not_after: u64,
}

/// TLS server configuration.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub min_version: TlsVersion,
    pub cipher_suites: Vec<CipherSuite>,
    pub certificate: TlsCertificate,
    pub require_client_cert: bool,
    pub client_ca_pem: Option<Vec<u8>>,
}

pub fn create_tls_config(cert: TlsCertificate) -> TlsConfig {
    TlsConfig {
        min_version: TlsVersion::Tls13,
        cipher_suites: vec![
            CipherSuite::Aes256GcmSha384,
            CipherSuite::Aes128GcmSha256,
            CipherSuite::Chacha20Poly1305Sha256,
        ],
        certificate: cert,
        require_client_cert: false,
        client_ca_pem: None,
    }
}

pub fn validate_tls_config(config: &TlsConfig, current_time: u64) -> Result<(), String> {
    if config.min_version != TlsVersion::Tls13 {
        return Err("Minimum TLS version must be 1.3".to_string());
    }
    if config.cipher_suites.is_empty() {
        return Err("At least one cipher suite must be configured".to_string());
    }
    if config.certificate.not_after <= current_time {
        return Err("Certificate has expired".to_string());
    }
    if config.certificate.key_pem.is_empty() {
        return Err("Certificate private key is empty".to_string());
    }
    Ok(())
}

pub fn is_strong_cipher(cipher: &CipherSuite) -> bool {
    // All TLS 1.3 cipher suites are AEAD-based and considered strong
    matches!(
        cipher,
        CipherSuite::Aes128GcmSha256
            | CipherSuite::Aes256GcmSha384
            | CipherSuite::Chacha20Poly1305Sha256
    )
}

pub fn certificate_fingerprint(cert_pem: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(cert_pem);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn configure_mtls(mut config: TlsConfig, client_ca_pem: Vec<u8>) -> TlsConfig {
    config.require_client_cert = true;
    config.client_ca_pem = Some(client_ca_pem);
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cert() -> TlsCertificate {
        TlsCertificate {
            cert_pem: b"-----BEGIN CERTIFICATE-----\nMIIB...sample...\n-----END CERTIFICATE-----".to_vec(),
            key_pem: b"-----BEGIN PRIVATE KEY-----\nMIIE...sample...\n-----END PRIVATE KEY-----".to_vec(),
            subject_cn: "example.com".to_string(),
            not_after: 2000000000,
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
        cert.not_after = 1000000000;
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
