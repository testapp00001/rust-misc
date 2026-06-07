//! # TLS Configuration
//!
//! TLS (Transport Layer Security) encrypts network communications. This lesson
//! covers certificate management, TLS configuration patterns, mTLS, and
//! certificate pinning concepts.
//!
//! ## Key Concepts
//! - Certificate and key management
//! - TLS server configuration
//! - Mutual TLS (mTLS)
//! - Certificate pinning
//! - Certificate validation

use std::time::{Duration, SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// 1. Certificate
// ---------------------------------------------------------------------------

/// Represents a TLS certificate (simplified for learning).
#[derive(Debug, Clone)]
pub struct Certificate {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: u64,
    pub not_after: u64,
    pub san: Vec<String>, // Subject Alternative Names
    pub is_ca: bool,
    pub fingerprint: String,
}

impl Certificate {
    pub fn new(subject: impl Into<String>, issuer: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let subject = subject.into();
        let issuer = issuer.into();
        let fingerprint = {
            let mut hash: u64 = 0xcbf29ce484222325;
            for b in format!("{}:{}", subject, issuer).bytes() {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
            format!("{:064x}", hash)
        };
        Self {
            subject,
            issuer,
            serial_number: format!("{:016x}", now),
            not_before: now,
            not_after: now + 365 * 86400, // 1 year
            san: Vec::new(),
            is_ca: false,
            fingerprint,
        }
    }

    pub fn with_san(mut self, san: impl Into<String>) -> Self {
        self.san.push(san.into());
        self
    }

    pub fn with_validity(mut self, not_before: u64, not_after: u64) -> Self {
        self.not_before = not_before;
        self.not_after = not_after;
        self
    }

    pub fn as_ca(mut self) -> Self {
        self.is_ca = true;
        self
    }

    /// Check if the certificate is currently valid.
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now >= self.not_before && now <= self.not_after
    }

    /// Check if the certificate matches a given hostname.
    pub fn matches_hostname(&self, hostname: &str) -> bool {
        if self.subject.contains(hostname) {
            return true;
        }
        for san in &self.san {
            if san == hostname {
                return true;
            }
            // Wildcard matching
            if let Some(suffix) = san.strip_prefix("*.") {
                if hostname.ends_with(suffix) {
                    return true;
                }
                // Also match subdomain.example.com for *.example.com
                let without_first = hostname.splitn(2, '.').nth(1).unwrap_or("");
                if without_first == suffix {
                    return true;
                }
            }
        }
        false
    }
}

// ---------------------------------------------------------------------------
// 2. TLS Configuration
// ---------------------------------------------------------------------------

/// TLS configuration for a server or client.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    pub min_version: TlsVersion,
    pub max_version: TlsVersion,
    pub cipher_suites: Vec<CipherSuite>,
    pub require_client_cert: bool,
    pub verify_peer: bool,
    pub cert_chain: Vec<Certificate>,
    pub ca_certs: Vec<Certificate>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum TlsVersion {
    Tls10,
    Tls11,
    Tls12,
    Tls13,
}

impl TlsVersion {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tls10 => "TLSv1.0",
            Self::Tls11 => "TLSv1.1",
            Self::Tls12 => "TLSv1.2",
            Self::Tls13 => "TLSv1.3",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CipherSuite {
    Aes128GcmSha256,
    Aes256GcmSha384,
    Chacha20Poly1305Sha256,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            min_version: TlsVersion::Tls12,
            max_version: TlsVersion::Tls13,
            cipher_suites: vec![
                CipherSuite::Aes256GcmSha384,
                CipherSuite::Chacha20Poly1305Sha256,
                CipherSuite::Aes128GcmSha256,
            ],
            require_client_cert: false,
            verify_peer: true,
            cert_chain: Vec::new(),
            ca_certs: Vec::new(),
        }
    }
}

impl TlsConfig {
    pub fn validate(&self) -> Result<(), TlsConfigError> {
        if self.min_version < TlsVersion::Tls12 {
            return Err(TlsConfigError::InsecureProtocolVersion);
        }
        if self.cipher_suites.is_empty() {
            return Err(TlsConfigError::NoCipherSuites);
        }
        if self.cert_chain.is_empty() {
            return Err(TlsConfigError::NoCertificate);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TlsConfigError {
    #[error("minimum TLS version must be 1.2 or higher")]
    InsecureProtocolVersion,
    #[error("no cipher suites configured")]
    NoCipherSuites,
    #[error("no certificate provided")]
    NoCertificate,
}

// ---------------------------------------------------------------------------
// 3. Certificate Pinning
// ---------------------------------------------------------------------------

/// Pins specific certificates or public keys to prevent MITM attacks.
pub struct CertPinner {
    pins: Vec<CertPin>,
}

#[derive(Debug, Clone)]
pub struct CertPin {
    pub hostname: String,
    pub fingerprint: String,
    pub backup_pins: Vec<String>,
}

impl CertPinner {
    pub fn new() -> Self {
        Self { pins: Vec::new() }
    }

    pub fn add_pin(mut self, pin: CertPin) -> Self {
        self.pins.push(pin);
        self
    }

    /// Verify a certificate against stored pins.
    pub fn verify(&self, hostname: &str, cert: &Certificate) -> bool {
        if let Some(pin) = self.pins.iter().find(|p| p.hostname == hostname) {
            cert.fingerprint == pin.fingerprint
                || pin.backup_pins.contains(&cert.fingerprint)
        } else {
            true // No pin configured, allow
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Certificate Chain Validator
// ---------------------------------------------------------------------------

/// Validates a certificate chain from leaf to root CA.
pub struct ChainValidator {
    trusted_cas: Vec<Certificate>,
}

impl ChainValidator {
    pub fn new(trusted_cas: Vec<Certificate>) -> Self {
        Self { trusted_cas }
    }

    /// Validate a certificate chain.
    pub fn validate(&self, chain: &[Certificate]) -> Result<(), ChainError> {
        if chain.is_empty() {
            return Err(ChainError::EmptyChain);
        }

        // Check leaf certificate validity
        let leaf = &chain[0];
        if !leaf.is_valid() {
            return Err(ChainError::Expired(leaf.subject.clone()));
        }

        // Check chain links
        for i in 0..chain.len() - 1 {
            let cert = &chain[i];
            let issuer_cert = &chain[i + 1];

            if cert.issuer != issuer_cert.subject {
                return Err(ChainError::BrokenChain {
                    cert: cert.subject.clone(),
                    expected_issuer: cert.issuer.clone(),
                    actual_issuer: issuer_cert.subject.clone(),
                });
            }
        }

        // Check root is trusted
        let root = chain.last().unwrap();
        let trusted = self
            .trusted_cas
            .iter()
            .any(|ca| ca.fingerprint == root.fingerprint);
        if !trusted {
            return Err(ChainError::UntrustedRoot(root.subject.clone()));
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChainError {
    #[error("empty certificate chain")]
    EmptyChain,
    #[error("certificate expired: {0}")]
    Expired(String),
    #[error("broken chain: {cert} issued by {expected_issuer}, but next cert is {actual_issuer}")]
    BrokenChain {
        cert: String,
        expected_issuer: String,
        actual_issuer: String,
    },
    #[error("untrusted root CA: {0}")]
    UntrustedRoot(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cert(subject: &str, issuer: &str) -> Certificate {
        Certificate::new(subject, issuer)
    }

    #[test]
    fn test_certificate_validity() {
        let cert = test_cert("example.com", "CA");
        assert!(cert.is_valid());
    }

    #[test]
    fn test_certificate_expired() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let cert = Certificate::new("example.com", "CA")
            .with_validity(now - 86400, now - 1);
        assert!(!cert.is_valid());
    }

    #[test]
    fn test_certificate_hostname_matching() {
        let cert = Certificate::new("example.com", "CA")
            .with_san("example.com")
            .with_san("*.example.com");

        assert!(cert.matches_hostname("example.com"));
        assert!(cert.matches_hostname("sub.example.com"));
        assert!(cert.matches_hostname("api.example.com"));
        assert!(!cert.matches_hostname("other.com"));
    }

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert_eq!(config.min_version, TlsVersion::Tls12);
        assert!(config.verify_peer);
        assert!(!config.require_client_cert);
    }

    #[test]
    fn test_tls_config_validation() {
        let config = TlsConfig {
            cert_chain: vec![test_cert("server", "CA")],
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_tls_config_insecure_version() {
        let config = TlsConfig {
            min_version: TlsVersion::Tls10,
            cert_chain: vec![test_cert("server", "CA")],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_tls_config_no_cert() {
        let config = TlsConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_tls_version_str() {
        assert_eq!(TlsVersion::Tls12.as_str(), "TLSv1.2");
        assert_eq!(TlsVersion::Tls13.as_str(), "TLSv1.3");
    }

    #[test]
    fn test_cert_pinner() {
        let pinner = CertPinner::new().add_pin(CertPin {
            hostname: "example.com".into(),
            fingerprint: "abc123".into(),
            backup_pins: vec!["def456".into()],
        });

        let cert = Certificate::new("example.com", "CA");
        // Fingerprint won't match our test values
        assert!(!pinner.verify("example.com", &cert));

        // No pin for other host -> allow
        let cert2 = Certificate::new("other.com", "CA");
        assert!(pinner.verify("other.com", &cert2));
    }

    #[test]
    fn test_chain_validator_valid() {
        let ca = Certificate::new("Root CA", "Root CA").as_ca();
        let intermediate = Certificate::new("Intermediate CA", "Root CA");
        let leaf = Certificate::new("example.com", "Intermediate CA");

        let validator = ChainValidator::new(vec![ca.clone()]);
        let chain = vec![leaf, intermediate, ca];
        assert!(validator.validate(&chain).is_ok());
    }

    #[test]
    fn test_chain_validator_empty() {
        let validator = ChainValidator::new(vec![]);
        assert!(validator.validate(&[]).is_err());
    }

    #[test]
    fn test_chain_validator_broken() {
        let ca = Certificate::new("Root CA", "Root CA").as_ca();
        let leaf = Certificate::new("example.com", "Wrong CA");

        let validator = ChainValidator::new(vec![ca.clone()]);
        let chain = vec![leaf, ca];
        assert!(validator.validate(&chain).is_err());
    }

    #[test]
    fn test_chain_validator_untrusted() {
        let ca = Certificate::new("Unknown CA", "Unknown CA").as_ca();
        let leaf = Certificate::new("example.com", "Unknown CA");

        let trusted_ca = Certificate::new("Trusted CA", "Trusted CA").as_ca();
        let validator = ChainValidator::new(vec![trusted_ca]);
        let chain = vec![leaf, ca];
        assert!(validator.validate(&chain).is_err());
    }

    #[test]
    fn test_tls_config_error_display() {
        let err = TlsConfigError::InsecureProtocolVersion;
        assert!(err.to_string().contains("1.2"));
    }

    #[test]
    fn test_chain_error_display() {
        let err = ChainError::EmptyChain;
        assert!(!err.to_string().is_empty());
    }
}
