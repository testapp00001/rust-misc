//! # Lesson 07: DNS Security (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use ring::digest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: DnsType,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DnsType {
    A,
    AAAA,
    CNAME,
    TXT,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedDnsRecord {
    pub record: DnsRecord,
    pub signature: Vec<u8>,
    pub key_tag: u16,
    pub signer: String,
}

pub fn spoof_dns_record(
    cache: &HashMap<String, DnsRecord>,
    target_domain: &str,
    attacker_ip: &str,
) -> DnsRecord {
    if let Some(original) = cache.get(target_domain) {
        DnsRecord {
            name: original.name.clone(),
            record_type: original.record_type.clone(),
            value: attacker_ip.to_string(),
            ttl: original.ttl,
        }
    } else {
        DnsRecord {
            name: target_domain.to_string(),
            record_type: DnsType::A,
            value: attacker_ip.to_string(),
            ttl: 300,
        }
    }
}

pub fn sign_dns_record(record: &DnsRecord, signer: &str, key_tag: u16) -> SignedDnsRecord {
    let data = serde_json::to_vec(record).unwrap();
    let signature = digest::digest(&digest::SHA256, &data).as_ref().to_vec();

    SignedDnsRecord {
        record: record.clone(),
        signature,
        key_tag,
        signer: signer.to_string(),
    }
}

pub fn verify_dns_record(signed: &SignedDnsRecord) -> Result<(), String> {
    let data = serde_json::to_vec(&signed.record)
        .map_err(|e| format!("Serialization error: {}", e))?;
    let computed = digest::digest(&digest::SHA256, &data);

    if ring::constant_time::verify_slices_are_equal(computed.as_ref(), &signed.signature).is_err() {
        return Err("DNS record signature verification failed".to_string());
    }
    Ok(())
}

pub fn encode_doh_query(domain: &str, record_type: DnsType) -> String {
    let query = serde_json::json!({
        "name": domain,
        "type": format!("{:?}", record_type),
    });
    let json = serde_json::to_string(&query).unwrap();
    base64::engine::general_purpose::STANDARD.encode(json.as_bytes())
}

pub fn decode_doh_response(encoded: &str) -> Result<DnsRecord, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    let record: DnsRecord =
        serde_json::from_slice(&bytes).map_err(|e| format!("JSON parse error: {}", e))?;
    Ok(record)
}

pub fn detect_spoofing(
    response: &DnsRecord,
    trusted_cache: &HashMap<String, DnsRecord>,
) -> Result<(), String> {
    match trusted_cache.get(&response.name) {
        Some(trusted) => {
            if trusted.value == response.value {
                Ok(())
            } else {
                Err(format!(
                    "DNS spoofing detected: expected '{}' but got '{}'",
                    trusted.value, response.value
                ))
            }
        }
        None => Err(format!(
            "No trusted record for '{}'",
            response.name
        )),
    }
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
        cache.insert(
            "bank.com".to_string(),
            DnsRecord {
                name: "bank.com".to_string(),
                record_type: DnsType::A,
                value: "1.2.3.4".to_string(),
                ttl: 300,
            },
        );

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
        signed.record.value = "10.0.0.1".to_string();
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
        trusted.insert(
            "example.com".to_string(),
            DnsRecord {
                name: "example.com".to_string(),
                record_type: DnsType::A,
                value: "5.6.7.8".to_string(),
                ttl: 3600,
            },
        );
        assert!(detect_spoofing(&record, &trusted).is_err());
    }

    #[test]
    fn test_dnssec_signature_uniqueness() {
        let r1 = example_record();
        let mut r2 = r1.clone();
        r2.ttl = 7200;
        let s1 = sign_dns_record(&r1, "ca.example.com", 1);
        let s2 = sign_dns_record(&r2, "ca.example.com", 1);
        assert_ne!(
            s1.signature, s2.signature,
            "Different records should have different signatures"
        );
    }
}
