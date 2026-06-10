//! # Lesson 07: DNS Security
//!
//! ## What is DNS?
//!
//! The Domain Name System (DNS) translates human-readable names (example.com)
//! into IP addresses (93.184.216.34). DNS was designed in 1983 with no security.
//! Queries and responses are unencrypted and unauthenticated.
//!
//! ## Attack Scenario: DNS Spoofing
//!
//! An attacker intercepts a DNS query and responds with a fake IP address.
//! The client connects to the attacker's server instead of the real one.
//!
//! ```text
//! Client: "What is the IP for bank.com?"
//! Attacker: "It's 10.0.0.1" (attacker's server)
//! Client connects to attacker, thinking it's the bank
//! ```
//!
//! ## DNS Security Solutions
//!
//! 1. **DNSSEC (DNS Security Extensions)**: Signs DNS records with cryptographic
//!    signatures. Clients can verify the response is authentic and untampered.
//!
//! 2. **DoH (DNS over HTTPS)**: DNS queries sent over HTTPS (port 443).
//!    Encrypted, authenticated via TLS, and blends with web traffic.
//!
//! 3. **DoT (DNS over TLS)**: DNS queries sent over TLS (port 853).
//!    Encrypted and authenticated, but uses a dedicated port (easy to block).
//!
//! ## Why This Matters
//!
//! If DNS is compromised, TLS alone cannot help. The client connects to the
//! wrong IP, and the attacker can use a valid certificate for that IP
//! (or the client might skip validation). DNS security is a prerequisite
//! for transport security.

use ring::digest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A DNS record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: DnsType,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DnsType {
    A,     // IPv4 address
    AAAA,  // IPv6 address
    CNAME, // Canonical name (alias)
    TXT,   // Text record
}

/// A DNSSEC-signed record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedDnsRecord {
    pub record: DnsRecord,
    /// SHA-256 hash of the record data (simplified signature)
    pub signature: Vec<u8>,
    /// Key tag identifying the signing key
    pub key_tag: u16,
    /// Signer's domain name
    pub signer: String,
}

/// Exercise: Simulate a DNS spoofing attack.
///
/// The attacker intercepts a DNS query and returns a fake IP.
///
/// Hints:
/// - Take a DNS cache and a target domain
/// - Replace the real IP with the attacker's IP
/// - Return the spoofed record
pub fn spoof_dns_record(
    cache: &HashMap<String, DnsRecord>,
    target_domain: &str,
    attacker_ip: &str,
) -> DnsRecord {
    todo!("Implement DNS spoofing simulation")
}

/// Exercise: Sign a DNS record (simplified DNSSEC).
///
/// Create a signed record by hashing the record data.
///
/// Hints:
/// - Serialize the record to bytes (JSON or manual encoding)
/// - Hash with SHA-256
/// - Return a SignedDnsRecord with the hash as signature
pub fn sign_dns_record(record: &DnsRecord, signer: &str, key_tag: u16) -> SignedDnsRecord {
    todo!("Implement DNS record signing")
}

/// Exercise: Verify a signed DNS record.
///
/// Hints:
/// - Recompute the hash of the record
/// - Compare with the stored signature
pub fn verify_dns_record(signed: &SignedDnsRecord) -> Result<(), String> {
    todo!("Implement DNS record verification")
}

/// Exercise: Simulate DNS over HTTPS (DoH) query encoding.
///
/// DoH sends DNS queries as HTTPS POST requests with content-type "application/dns-message".
/// The DNS message is encoded in a specific wire format.
///
/// For this exercise, encode a simple DNS query as JSON (simplified).
///
/// Hints:
/// - Create a JSON representation of the DNS query
/// - Include the domain name and record type
/// - Base64-encode the result (simulating HTTPS body)
pub fn encode_doh_query(domain: &str, record_type: DnsType) -> String {
    todo!("Implement DoH query encoding")
}

/// Exercise: Decode a DoH response.
///
/// Hints:
/// - Base64-decode the input
/// - Parse the JSON
/// - Return the DNS record
pub fn decode_doh_response(encoded: &str) -> Result<DnsRecord, String> {
    todo!("Implement DoH response decoding")
}

/// Exercise: Detect DNS spoofing by verifying the record against a trusted source.
///
/// Compare the response from a potentially spoofed source with a trusted cache.
///
/// Hints:
/// - Check if the IP in the response matches the trusted cache
/// - Check if the response is DNSSEC-signed and the signature is valid
pub fn detect_spoofing(
    response: &DnsRecord,
    trusted_cache: &HashMap<String, DnsRecord>,
) -> Result<(), String> {
    todo!("Implement spoofing detection")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_record() -> DnsRecord {
        DnsRecord {
            name: "example.com".to_string(),
            record_type: DnsType::A,
            value: "93.184.216.34".to_string(),
            ttl: 3600,
        }
    }

    #[test]
    fn test_spoof_dns_record() {
        let mut cache = HashMap::new();
        cache.insert("bank.com".to_string(), DnsRecord {
            name: "bank.com".to_string(),
            record_type: DnsType::A,
            value: "1.2.3.4".to_string(),
            ttl: 300,
        });

        let spoofed = spoof_dns_record(&cache, "bank.com", "10.0.0.1");
        assert_eq!(spoofed.value, "10.0.0.1");
        assert_eq!(spoofed.name, "bank.com");
    }

    #[test]
    fn test_sign_and_verify() {
        let record = example_record();
        let signed = sign_dns_record(&record, "example.com", 12345);
        assert!(verify_dns_record(&signed).is_ok());
    }

    #[test]
    fn test_tampered_record_fails_verification() {
        let record = example_record();
        let mut signed = sign_dns_record(&record, "example.com", 12345);
        signed.record.value = "10.0.0.1".to_string(); // Tamper
        assert!(verify_dns_record(&signed).is_err());
    }

    #[test]
    fn test_doh_encode_decode_roundtrip() {
        let encoded = encode_doh_query("example.com", DnsType::A);
        let decoded = decode_doh_response(&encoded).unwrap();
        assert_eq!(decoded.name, "example.com");
        assert_eq!(decoded.record_type, DnsType::A);
    }

    #[test]
    fn test_detect_spoofing_clean() {
        let record = example_record();
        let mut trusted = HashMap::new();
        trusted.insert("example.com".to_string(), record.clone());
        assert!(detect_spoofing(&record, &trusted).is_ok());
    }

    #[test]
    fn test_detect_spoofing_detected() {
        let record = example_record();
        let mut trusted = HashMap::new();
        trusted.insert("example.com".to_string(), DnsRecord {
            name: "example.com".to_string(),
            record_type: DnsType::A,
            value: "5.6.7.8".to_string(), // Different IP
            ttl: 3600,
        });
        assert!(detect_spoofing(&record, &trusted).is_err());
    }

    #[test]
    fn test_dnssec_signature_uniqueness() {
        let r1 = example_record();
        let mut r2 = r1.clone();
        r2.ttl = 7200;
        let s1 = sign_dns_record(&r1, "ca.example.com", 1);
        let s2 = sign_dns_record(&r2, "ca.example.com", 1);
        assert_ne!(s1.signature, s2.signature, "Different records should have different signatures");
    }
}
