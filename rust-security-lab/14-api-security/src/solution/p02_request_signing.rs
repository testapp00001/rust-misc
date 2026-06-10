//! # Lesson 02: HMAC Request Signing (Reference Solution)
//!
//! See the exercise file for full documentation and attack explanations.

use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};

type HmacSha256 = Hmac<Sha256>;

pub fn create_string_to_sign(method: &str, path: &str, timestamp: u64, body: &[u8]) -> String {
    let body_hash = hex::encode(Sha256::digest(body));
    format!("{}\n{}\n{}\n{}", method, path, timestamp, body_hash)
}

pub fn compute_signature(secret: &[u8], string_to_sign: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC accepts any key length");
    mac.update(string_to_sign.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub fn verify_signature(
    secret: &[u8],
    method: &str,
    path: &str,
    timestamp: u64,
    body: &[u8],
    provided_signature_hex: &str,
) -> bool {
    let sts = create_string_to_sign(method, path, timestamp, body);
    let expected = compute_signature(secret, &sts);

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

pub fn verify_signed_request(
    secret: &[u8],
    method: &str,
    path: &str,
    timestamp: u64,
    body: &[u8],
    signature_hex: &str,
    current_time: u64,
    max_age_seconds: u64,
) -> Result<(), &'static str> {
    // Check timestamp freshness (allow both past and small future skew)
    let diff = if current_time >= timestamp {
        current_time - timestamp
    } else {
        timestamp - current_time
    };
    if diff > max_age_seconds {
        return Err("expired");
    }

    if !verify_signature(secret, method, path, timestamp, body, signature_hex) {
        return Err("invalid_signature");
    }

    Ok(())
}

pub fn build_signed_headers(
    secret: &[u8],
    method: &str,
    path: &str,
    body: &[u8],
    timestamp: u64,
) -> HashMap<String, String> {
    let sts = create_string_to_sign(method, path, timestamp, body);
    let sig = compute_signature(secret, &sts);

    let mut headers = HashMap::new();
    headers.insert("X-Timestamp".to_string(), timestamp.to_string());
    headers.insert("X-Signature".to_string(), sig);
    headers
}

pub struct ReplayDetector {
    seen: HashSet<(u64, String)>,
}

impl ReplayDetector {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    pub fn check_and_record(&mut self, timestamp: u64, path: &str) -> bool {
        let key = (timestamp, path.to_string());
        if self.seen.contains(&key) {
            false
        } else {
            self.seen.insert(key);
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"my-secret-key-for-testing";

    #[test]
    fn test_string_to_sign_format() {
        let sts = create_string_to_sign("POST", "/api/transfer", 1700000000, b"amount=100");
        assert!(sts.contains("POST"));
        assert!(sts.contains("/api/transfer"));
        assert!(sts.contains("1700000000"));
        let body_hash = hex::encode(Sha256::digest(b"amount=100"));
        assert!(sts.contains(&body_hash));
    }

    #[test]
    fn test_string_to_sign_differs_with_different_body() {
        let sts1 = create_string_to_sign("POST", "/api/transfer", 1700000000, b"amount=100");
        let sts2 = create_string_to_sign("POST", "/api/transfer", 1700000000, b"amount=999");
        assert_ne!(sts1, sts2);
    }

    #[test]
    fn test_signature_deterministic() {
        let sig1 = compute_signature(TEST_SECRET, "test string");
        let sig2 = compute_signature(TEST_SECRET, "test string");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signature_differs_with_different_secret() {
        let sig1 = compute_signature(b"secret1", "test");
        let sig2 = compute_signature(b"secret2", "test");
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_verify_signature_valid() {
        let sts = create_string_to_sign("POST", "/api/data", 1700000000, b"hello");
        let sig = compute_signature(TEST_SECRET, &sts);
        assert!(verify_signature(TEST_SECRET, "POST", "/api/data", 1700000000, b"hello", &sig));
    }

    #[test]
    fn test_verify_signature_tampered_body() {
        let sts = create_string_to_sign("POST", "/api/data", 1700000000, b"hello");
        let sig = compute_signature(TEST_SECRET, &sts);
        assert!(!verify_signature(TEST_SECRET, "POST", "/api/data", 1700000000, b"tampered", &sig));
    }

    #[test]
    fn test_verify_signature_tampered_path() {
        let sts = create_string_to_sign("POST", "/api/data", 1700000000, b"hello");
        let sig = compute_signature(TEST_SECRET, &sts);
        assert!(!verify_signature(TEST_SECRET, "POST", "/api/admin", 1700000000, b"hello", &sig));
    }

    #[test]
    fn test_verify_signed_request_valid() {
        let sig = {
            let sts = create_string_to_sign("GET", "/api/users", 1700000000, b"");
            compute_signature(TEST_SECRET, &sts)
        };
        let result = verify_signed_request(
            TEST_SECRET, "GET", "/api/users", 1700000000, b"", &sig, 1700000000, 300,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_signed_request_expired() {
        let sig = {
            let sts = create_string_to_sign("GET", "/api/users", 1700000000, b"");
            compute_signature(TEST_SECRET, &sts)
        };
        let result = verify_signed_request(
            TEST_SECRET, "GET", "/api/users", 1700000000, b"", &sig, 1700000600, 300,
        );
        assert_eq!(result.unwrap_err(), "expired");
    }

    #[test]
    fn test_verify_signed_request_bad_signature() {
        let result = verify_signed_request(
            TEST_SECRET, "GET", "/api/users", 1700000000, b"", "deadbeef", 1700000000, 300,
        );
        assert_eq!(result.unwrap_err(), "invalid_signature");
    }

    #[test]
    fn test_build_signed_headers() {
        let headers = build_signed_headers(TEST_SECRET, "POST", "/api/data", b"body", 1700000000);
        assert_eq!(headers.get("X-Timestamp").unwrap(), "1700000000");
        assert!(headers.contains_key("X-Signature"));
        let sig = headers.get("X-Signature").unwrap();
        assert!(verify_signature(TEST_SECRET, "POST", "/api/data", 1700000000, b"body", sig));
    }

    #[test]
    fn test_replay_detector_new_request() {
        let mut detector = ReplayDetector::new();
        assert!(detector.check_and_record(1700000000, "/api/data"));
    }

    #[test]
    fn test_replay_detector_replay() {
        let mut detector = ReplayDetector::new();
        assert!(detector.check_and_record(1700000000, "/api/data"));
        assert!(!detector.check_and_record(1700000000, "/api/data"));
    }

    #[test]
    fn test_replay_detector_different_paths() {
        let mut detector = ReplayDetector::new();
        assert!(detector.check_and_record(1700000000, "/api/data"));
        assert!(detector.check_and_record(1700000000, "/api/other"));
    }
}
