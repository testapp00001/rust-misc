//! # Lesson 02: Certificate Validation
//!
//! ## X.509 Certificate Chain of Trust
//!
//! An X.509 certificate binds a public key to an identity (e.g., a domain name).
//! The chain of trust works like this:
//!
//! ```text
//! Root CA (trusted, pre-installed in OS/browser)
//!   └── Intermediate CA (signed by Root)
//!         └── Server Certificate (signed by Intermediate)
//!               └── Contains: domain name, public key, validity period
//! ```
//!
//! Validation checks:
//! 1. **Signature chain**: Each cert is signed by the next cert up the chain
//! 2. **Expiration**: Current time is within the cert's validity period
//! 3. **Hostname**: The cert's Subject/SAN matches the requested hostname
//! 4. **Revocation**: The cert has not been revoked (CRL/OCSP)
//!
//! ## Attack Scenario: Expired/Wrong-Hostname Certificate
//!
//! An attacker presents a valid certificate for `evil.com` when you connect to
//! `bank.com`. Without hostname verification, the TLS handshake succeeds because
//! the certificate is technically valid (signed by a real CA). The encrypted
//! channel is then with the attacker.
//!
//! ## Why This Matters
//!
//! Disabling certificate validation is the #1 TLS mistake. Every year, major
//! breaches trace back to `danger_accept_invalid_certs(true)` or equivalent.
//! Encryption without authentication is worthless — you are just encrypting
//! your data for the attacker.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// A simplified X.509 certificate representation.
///
/// Real X.509 certs use ASN.1/DER encoding. We use a simple struct
/// for educational purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Subject Common Name (e.g., "example.com")
    pub subject_cn: String,
    /// Subject Alternative Names (e.g., ["*.example.com", "example.com"])
    pub subject_alt_names: Vec<String>,
    /// Issuer Common Name (e.g., "Intermediate CA 1")
    pub issuer_cn: String,
    /// Serial number (unique per cert from this issuer)
    pub serial_number: Vec<u8>,
    /// Not-valid-before (Unix timestamp)
    pub not_before: u64,
    /// Not-valid-after (Unix timestamp)
    pub not_after: u64,
    /// Subject's public key (simplified as raw bytes)
    pub public_key: Vec<u8>,
    /// Signature from the issuer
    pub signature: Vec<u8>,
    /// Whether this is a CA certificate
    pub is_ca: bool,
}

/// A certificate authority that can sign certificates.
pub struct CertificateAuthority {
    pub name: String,
    pub private_key: Vec<u8>,
    pub certificate: Certificate,
}

impl CertificateAuthority {
    /// Create a new self-signed CA.
    ///
    /// Exercise: Generate a CA with a random key pair and self-signed certificate.
    pub fn new(name: &str) -> Self {
        todo!("Create a new CertificateAuthority")
    }

    /// Sign a certificate, creating a cert issued by this CA.
    ///
    /// Exercise: Create a new Certificate where issuer_cn matches this CA's name,
    /// and the signature is computed over the cert's contents using the CA's private key.
    pub fn sign_certificate(
        &self,
        subject_cn: &str,
        subject_alt_names: Vec<String>,
        public_key: Vec<u8>,
        validity_days: u64,
        is_ca: bool,
    ) -> Certificate {
        todo!("Implement certificate signing")
    }
}

/// Exercise: Validate that a certificate is currently valid (not expired).
///
/// Check that the current time is between not_before and not_after.
///
/// Hints:
/// - Use `SystemTime::now().duration_since(UNIX_EPOCH)` to get current Unix timestamp
/// - Compare with `cert.not_before` and `cert.not_after`
pub fn check_expiration(cert: &Certificate) -> Result<(), String> {
    todo!("Implement expiration check")
}

/// Exercise: Validate that a certificate matches a given hostname.
///
/// Rules:
/// - Exact match: cert CN or SAN equals hostname
/// - Wildcard match: "*.example.com" matches "sub.example.com" but NOT "example.com"
/// - Wildcard only matches one level: "*.example.com" does NOT match "a.b.example.com"
///
/// Hints:
/// - Check `subject_cn` first, then `subject_alt_names`
/// - For wildcard matching, split on '.' and compare segments
pub fn check_hostname(cert: &Certificate, hostname: &str) -> Result<(), String> {
    todo!("Implement hostname verification")
}

/// Exercise: Validate a certificate chain.
///
/// Verify that each certificate in the chain is signed by the next one,
/// ending at a trusted root.
///
/// Hints:
/// - Iterate through the chain (leaf at index 0, root at end)
/// - Verify each cert's issuer_cn matches the next cert's subject_cn
/// - Verify each cert's signature using the issuer's public key
/// - The last cert must be in the trusted_roots list
pub fn validate_chain(
    chain: &[Certificate],
    trusted_roots: &[Certificate],
) -> Result<(), String> {
    todo!("Implement chain validation")
}

/// Exercise: Full certificate validation.
///
/// Combines all checks: expiration, hostname, and chain validation.
///
/// Hints:
/// - Call `check_expiration` on the leaf cert (chain[0])
/// - Call `check_hostname` on the leaf cert
/// - Call `validate_chain` on the full chain
pub fn validate_certificate(
    chain: &[Certificate],
    trusted_roots: &[Certificate],
    hostname: &str,
) -> Result<(), String> {
    todo!("Implement full certificate validation")
}

/// Exercise: Simulate a MITM attack where an attacker presents a cert
/// for the wrong domain.
///
/// Create two CAs, one trusted and one rogue. Show that validation
/// rejects the rogue cert.
///
/// Hints:
/// - Create a trusted CA and a rogue CA
/// - Have the trusted CA sign a cert for "bank.com"
/// - Have the rogue CA sign a cert for "evil.com"
/// - Validate should succeed for "bank.com" and fail for "evil.com"
pub fn simulate_mitm_attack(
    trusted_chain: &[Certificate],
    rogue_chain: &[Certificate],
    trusted_roots: &[Certificate],
) -> (bool, bool) {
    todo!("Implement MITM simulation returning (bank_com_valid, evil_com_valid)")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now_unix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn make_test_cert(cn: &str, san: Vec<&str>, issuer: &str, valid: bool) -> Certificate {
        let now = now_unix();
        Certificate {
            subject_cn: cn.to_string(),
            subject_alt_names: san.into_iter().map(String::from).collect(),
            issuer_cn: issuer.to_string(),
            serial_number: vec![1, 2, 3],
            not_before: if valid { now - 86400 } else { now + 86400 },
            not_after: if valid { now + 365 * 86400 } else { now - 86400 },
            public_key: vec![4u8; 32],
            signature: vec![5u8; 64],
            is_ca: false,
        }
    }

    #[test]
    fn test_expiration_valid() {
        let cert = make_test_cert("example.com", vec![], "CA", true);
        assert!(check_expiration(&cert).is_ok());
    }

    #[test]
    fn test_expiration_expired() {
        let cert = make_test_cert("example.com", vec![], "CA", false);
        assert!(check_expiration(&cert).is_err());
    }

    #[test]
    fn test_hostname_exact_match() {
        let cert = make_test_cert("example.com", vec!["example.com"], "CA", true);
        assert!(check_hostname(&cert, "example.com").is_ok());
    }

    #[test]
    fn test_hostname_no_match() {
        let cert = make_test_cert("example.com", vec!["example.com"], "CA", true);
        assert!(check_hostname(&cert, "evil.com").is_err());
    }

    #[test]
    fn test_hostname_wildcard_match() {
        let cert = make_test_cert("*.example.com", vec!["*.example.com"], "CA", true);
        assert!(check_hostname(&cert, "sub.example.com").is_ok());
    }

    #[test]
    fn test_hostname_wildcard_no_match_base() {
        let cert = make_test_cert("*.example.com", vec!["*.example.com"], "CA", true);
        assert!(check_hostname(&cert, "example.com").is_err());
    }

    #[test]
    fn test_hostname_wildcard_no_match_multi_level() {
        let cert = make_test_cert("*.example.com", vec!["*.example.com"], "CA", true);
        assert!(check_hostname(&cert, "a.b.example.com").is_err());
    }

    #[test]
    fn test_chain_validation_basic() {
        let leaf = Certificate {
            subject_cn: "example.com".to_string(),
            subject_alt_names: vec![],
            issuer_cn: "Test CA".to_string(),
            serial_number: vec![1],
            not_before: 0,
            not_after: u64::MAX,
            public_key: vec![1u8; 32],
            signature: vec![],
            is_ca: false,
        };
        let root = Certificate {
            subject_cn: "Test CA".to_string(),
            subject_alt_names: vec![],
            issuer_cn: "Test CA".to_string(),
            serial_number: vec![2],
            not_before: 0,
            not_after: u64::MAX,
            public_key: vec![2u8; 32],
            signature: vec![],
            is_ca: true,
        };
        let result = validate_chain(&[leaf, root.clone()], &[root]);
        assert!(result.is_ok());
    }
}
