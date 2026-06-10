//! # Lesson 02: Certificate Validation (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use ring::hmac;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub subject_cn: String,
    pub subject_alt_names: Vec<String>,
    pub issuer_cn: String,
    pub serial_number: Vec<u8>,
    pub not_before: u64,
    pub not_after: u64,
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub is_ca: bool,
}

pub struct CertificateAuthority {
    pub name: String,
    pub private_key: Vec<u8>,
    pub certificate: Certificate,
}

impl CertificateAuthority {
    pub fn new(name: &str) -> Self {
        let rng = ring::rand::SystemRandom::new();
        let mut private_key = vec![0u8; 32];
        rng.fill(&mut private_key).unwrap();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let certificate = Certificate {
            subject_cn: name.to_string(),
            subject_alt_names: vec![],
            issuer_cn: name.to_string(),
            serial_number: vec![1],
            not_before: now - 86400,
            not_after: now + 365 * 86400,
            public_key: private_key.iter().map(|b| b.wrapping_add(1)).collect(),
            signature: vec![],
            is_ca: true,
        };

        CertificateAuthority {
            name: name.to_string(),
            private_key,
            certificate,
        }
    }

    pub fn sign_certificate(
        &self,
        subject_cn: &str,
        subject_alt_names: Vec<String>,
        public_key: Vec<u8>,
        validity_days: u64,
        is_ca: bool,
    ) -> Certificate {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Sign by hashing (subject + issuer + public_key) with HMAC using CA's private key
        let signing_key = hmac::Key::new(hmac::HMAC_SHA256, &self.private_key);
        let mut data = Vec::new();
        data.extend_from_slice(subject_cn.as_bytes());
        data.extend_from_slice(self.name.as_bytes());
        data.extend_from_slice(&public_key);
        let signature = hmac::sign(&signing_key, &data).as_ref().to_vec();

        Certificate {
            subject_cn: subject_cn.to_string(),
            subject_alt_names,
            issuer_cn: self.name.clone(),
            serial_number: vec![rand::random::<u8>()],
            not_before: now - 86400,
            not_after: now + validity_days * 86400,
            public_key,
            signature,
            is_ca,
        }
    }
}

pub fn check_expiration(cert: &Certificate) -> Result<(), String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System time error: {}", e))?
        .as_secs();

    if now < cert.not_before {
        return Err(format!(
            "Certificate not yet valid: current={}, not_before={}",
            now, cert.not_before
        ));
    }
    if now > cert.not_after {
        return Err(format!(
            "Certificate expired: current={}, not_after={}",
            now, cert.not_after
        ));
    }
    Ok(())
}

pub fn check_hostname(cert: &Certificate, hostname: &str) -> Result<(), String> {
    // Check CN first
    if matches_hostname(&cert.subject_cn, hostname) {
        return Ok(());
    }
    // Check SANs
    for san in &cert.subject_alt_names {
        if matches_hostname(san, hostname) {
            return Ok(());
        }
    }
    Err(format!(
        "Hostname '{}' does not match certificate (CN={}, SANs={:?})",
        hostname, cert.subject_cn, cert.subject_alt_names
    ))
}

fn matches_hostname(pattern: &str, hostname: &str) -> bool {
    if pattern == hostname {
        return true;
    }
    // Wildcard matching: *.example.com matches sub.example.com but not example.com
    if let Some(suffix) = pattern.strip_prefix("*.") {
        if let Some(sub) = hostname.strip_suffix(suffix) {
            // sub should be something like "sub." — must have exactly one segment before the dot
            if sub.ends_with('.') && !sub.contains('.') {
                return true;
            }
            // Actually, let's check: hostname must be X.suffix where X has no dots
            if let Some(prefix) = hostname.strip_suffix(&format!(".{}", suffix)) {
                return !prefix.contains('.');
            }
        }
    }
    false
}

pub fn validate_chain(
    chain: &[Certificate],
    trusted_roots: &[Certificate],
) -> Result<(), String> {
    if chain.is_empty() {
        return Err("Empty certificate chain".to_string());
    }

    for i in 0..chain.len() - 1 {
        let cert = &chain[i];
        let issuer = &chain[i + 1];

        // Verify issuer chain
        if cert.issuer_cn != issuer.subject_cn {
            return Err(format!(
                "Chain break at index {}: issuer '{}' != next cert subject '{}'",
                i, cert.issuer_cn, issuer.subject_cn
            ));
        }
    }

    // Verify the last cert is a trusted root
    let root = chain.last().unwrap();
    let is_trusted = trusted_roots.iter().any(|r| r.subject_cn == root.subject_cn);
    if !is_trusted {
        return Err(format!(
            "Root certificate '{}' is not in trusted roots",
            root.subject_cn
        ));
    }

    Ok(())
}

pub fn validate_certificate(
    chain: &[Certificate],
    trusted_roots: &[Certificate],
    hostname: &str,
) -> Result<(), String> {
    if chain.is_empty() {
        return Err("Empty certificate chain".to_string());
    }

    check_expiration(&chain[0])?;
    check_hostname(&chain[0], hostname)?;
    validate_chain(chain, trusted_roots)?;

    Ok(())
}

pub fn simulate_mitm_attack(
    trusted_chain: &[Certificate],
    rogue_chain: &[Certificate],
    trusted_roots: &[Certificate],
) -> (bool, bool) {
    let bank_valid = validate_certificate(trusted_chain, trusted_roots, "bank.com").is_ok();
    let evil_valid = validate_certificate(rogue_chain, trusted_roots, "evil.com").is_ok();
    (bank_valid, evil_valid)
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

    #[test]
    fn test_ca_sign_certificate() {
        let ca = CertificateAuthority::new("Test CA");
        let cert = ca.sign_certificate("example.com", vec![], vec![42u8; 32], 365, false);
        assert_eq!(cert.subject_cn, "example.com");
        assert_eq!(cert.issuer_cn, "Test CA");
        assert!(!cert.is_ca);
        assert!(!cert.signature.is_empty());
    }

    #[test]
    fn test_full_mitm_simulation() {
        let trusted_ca = CertificateAuthority::new("Trusted CA");
        let rogue_ca = CertificateAuthority::new("Rogue CA");

        let bank_cert = trusted_ca.sign_certificate(
            "bank.com",
            vec!["bank.com".to_string()],
            vec![1u8; 32],
            365,
            false,
        );
        let evil_cert = rogue_ca.sign_certificate(
            "evil.com",
            vec!["evil.com".to_string()],
            vec![2u8; 32],
            365,
            false,
        );

        let trusted_root = trusted_ca.certificate.clone();
        let trusted_chain = vec![bank_cert, trusted_root.clone()];
        let rogue_chain = vec![evil_cert, rogue_ca.certificate.clone()];

        let (bank_valid, evil_valid) = simulate_mitm_attack(
            &trusted_chain,
            &rogue_chain,
            &[trusted_root],
        );
        assert!(bank_valid);
        assert!(!evil_valid);
    }
}
