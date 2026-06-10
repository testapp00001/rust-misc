//! # Lesson 03: HMAC Webhook Verification (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_webhook_signature(secret: &[u8], body: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC accepts any key length");
    mac.update(body);
    hex::encode(mac.finalize().into_bytes())
}

pub fn verify_webhook_signature(secret: &[u8], body: &[u8], provided_signature_hex: &str) -> bool {
    let expected = generate_webhook_signature(secret, body);

    let expected_bytes = match hex::decode(&expected) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let provided_bytes = match hex::decode(provided_signature_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };

    ring::constant_time::verify_slices_are_equal(&expected_bytes, &provided_bytes).is_ok()
}

pub fn verify_webhook(
    secret: &[u8],
    body: &[u8],
    signature_hex: &str,
    timestamp: u64,
    current_time: u64,
    max_age_seconds: u64,
) -> Result<(), &'static str> {
    // Check timestamp freshness
    if timestamp > current_time + 60 {
        return Err("timestamp_in_future");
    }
    if current_time.saturating_sub(timestamp) > max_age_seconds {
        return Err("timestamp_expired");
    }

    // Verify signature
    if !verify_webhook_signature(secret, body, signature_hex) {
        return Err("invalid_signature");
    }

    Ok(())
}

pub fn verify_webhook_from_headers(
    secret: &[u8],
    body: &[u8],
    headers: &HashMap<String, String>,
    current_time: u64,
    max_age_seconds: u64,
) -> Result<(), &'static str> {
    let sig = match headers.get("X-Webhook-Signature") {
        Some(s) => s,
        None => return Err("missing_signature"),
    };

    let ts_str = match headers.get("X-Webhook-Timestamp") {
        Some(t) => t,
        None => return Err("missing_timestamp"),
    };

    let timestamp: u64 = ts_str.parse().map_err(|_| "invalid_timestamp")?;

    verify_webhook(secret, body, sig, timestamp, current_time, max_age_seconds)
}

pub fn generate_prefixed_signature(secret: &[u8], timestamp: u64, body: &[u8]) -> String {
    let payload = format!("{}.{}", timestamp, String::from_utf8_lossy(body));
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC accepts any key length");
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn verify_github_webhook(secret: &[u8], body: &[u8], signature_header: &str) -> bool {
    let sig_hex = match signature_header.strip_prefix("sha256=") {
        Some(h) => h,
        None => return false,
    };

    verify_webhook_signature(secret, body, sig_hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEBHOOK_SECRET: &[u8] = b"whsec_test_secret_key_12345";

    #[test]
    fn test_generate_signature_deterministic() {
        let sig1 = generate_webhook_signature(WEBHOOK_SECRET, b"hello");
        let sig2 = generate_webhook_signature(WEBHOOK_SECRET, b"hello");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_generate_signature_hex_format() {
        let sig = generate_webhook_signature(WEBHOOK_SECRET, b"test");
        assert!(hex::decode(&sig).is_ok());
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_verify_signature_valid() {
        let sig = generate_webhook_signature(WEBHOOK_SECRET, b"payment_data");
        assert!(verify_webhook_signature(WEBHOOK_SECRET, b"payment_data", &sig));
    }

    #[test]
    fn test_verify_signature_tampered() {
        let sig = generate_webhook_signature(WEBHOOK_SECRET, b"payment_data");
        assert!(!verify_webhook_signature(WEBHOOK_SECRET, b"tampered_data", &sig));
    }

    #[test]
    fn test_verify_signature_wrong_secret() {
        let sig = generate_webhook_signature(WEBHOOK_SECRET, b"payment_data");
        assert!(!verify_webhook_signature(b"wrong_secret", b"payment_data", &sig));
    }

    #[test]
    fn test_verify_webhook_valid() {
        let body = b"event=payment&amount=100";
        let sig = generate_webhook_signature(WEBHOOK_SECRET, body);
        let result = verify_webhook(WEBHOOK_SECRET, body, &sig, 1700000000, 1700000000, 300);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_webhook_expired() {
        let body = b"event=payment";
        let sig = generate_webhook_signature(WEBHOOK_SECRET, body);
        let result = verify_webhook(WEBHOOK_SECRET, body, &sig, 1700000000, 1700000600, 300);
        assert_eq!(result.unwrap_err(), "timestamp_expired");
    }

    #[test]
    fn test_verify_webhook_future() {
        let body = b"event=payment";
        let sig = generate_webhook_signature(WEBHOOK_SECRET, body);
        let result = verify_webhook(WEBHOOK_SECRET, body, &sig, 1700000120, 1700000000, 300);
        assert_eq!(result.unwrap_err(), "timestamp_in_future");
    }

    #[test]
    fn test_verify_webhook_bad_signature() {
        let body = b"event=payment";
        let result = verify_webhook(WEBHOOK_SECRET, body, "deadbeef", 1700000000, 1700000000, 300);
        assert_eq!(result.unwrap_err(), "invalid_signature");
    }

    #[test]
    fn test_verify_from_headers_valid() {
        let body = b"event=payment&amount=100";
        let sig = generate_webhook_signature(WEBHOOK_SECRET, body);
        let mut headers = HashMap::new();
        headers.insert("X-Webhook-Signature".to_string(), sig);
        headers.insert("X-Webhook-Timestamp".to_string(), "1700000000".to_string());
        let result = verify_webhook_from_headers(WEBHOOK_SECRET, body, &headers, 1700000000, 300);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_from_headers_missing_signature() {
        let mut headers = HashMap::new();
        headers.insert("X-Webhook-Timestamp".to_string(), "1700000000".to_string());
        let result = verify_webhook_from_headers(WEBHOOK_SECRET, b"body", &headers, 1700000000, 300);
        assert_eq!(result.unwrap_err(), "missing_signature");
    }

    #[test]
    fn test_verify_from_headers_missing_timestamp() {
        let mut headers = HashMap::new();
        headers.insert("X-Webhook-Signature".to_string(), "abc".to_string());
        let result = verify_webhook_from_headers(WEBHOOK_SECRET, b"body", &headers, 1700000000, 300);
        assert_eq!(result.unwrap_err(), "missing_timestamp");
    }

    #[test]
    fn test_verify_from_headers_bad_timestamp() {
        let mut headers = HashMap::new();
        headers.insert("X-Webhook-Signature".to_string(), "abc".to_string());
        headers.insert("X-Webhook-Timestamp".to_string(), "not_a_number".to_string());
        let result = verify_webhook_from_headers(WEBHOOK_SECRET, b"body", &headers, 1700000000, 300);
        assert_eq!(result.unwrap_err(), "invalid_timestamp");
    }

    #[test]
    fn test_prefixed_signature() {
        let sig = generate_prefixed_signature(WEBHOOK_SECRET, 1700000000, b"hello");
        assert!(hex::decode(&sig).is_ok());
        assert_eq!(sig.len(), 64);
    }

    #[test]
    fn test_github_webhook_verify() {
        let body = b"payload data";
        let mut mac = HmacSha256::new_from_slice(WEBHOOK_SECRET).unwrap();
        mac.update(body);
        let sig_hex = hex::encode(mac.finalize().into_bytes());
        let header = format!("sha256={}", sig_hex);
        assert!(verify_github_webhook(WEBHOOK_SECRET, body, &header));
    }

    #[test]
    fn test_github_webhook_invalid_prefix() {
        assert!(!verify_github_webhook(WEBHOOK_SECRET, b"body", "md5=abc123"));
    }

    #[test]
    fn test_github_webhook_bad_signature() {
        assert!(!verify_github_webhook(WEBHOOK_SECRET, b"body", "sha256=deadbeef"));
    }
}
