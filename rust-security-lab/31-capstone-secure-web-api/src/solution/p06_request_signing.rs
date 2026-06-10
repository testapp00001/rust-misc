//! # Lesson 06: HMAC Request Signing — Solution
//!
//! Canonical string, HMAC-SHA256, replay prevention.

use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct SignedRequest {
    pub method: String,
    pub path: String,
    pub timestamp: u64,
    pub body: Vec<u8>,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SignatureError {
    MissingSignature,
    InvalidSignatureFormat,
    SignatureMismatch,
    TimestampExpired {
        request_time: u64,
        server_time: u64,
        max_age_secs: u64,
    },
    TimestampFuture {
        request_time: u64,
        server_time: u64,
    },
}

pub fn hash_body(body: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(body);
    let result = hasher.finalize();
    hex::encode(result)
}

pub fn build_canonical_string(
    method: &str,
    path: &str,
    timestamp: u64,
    body: &[u8],
) -> String {
    let body_hash = hash_body(body);
    format!("{}\n{}\n{}\n{}", method, path, timestamp, body_hash)
}

pub fn sign_request(
    method: &str,
    path: &str,
    timestamp: u64,
    body: &[u8],
    secret: &[u8],
) -> String {
    let canonical = build_canonical_string(method, path, timestamp, body);
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(canonical.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}

pub fn create_signed_request(
    method: &str,
    path: &str,
    timestamp: u64,
    body: Vec<u8>,
    secret: &[u8],
) -> SignedRequest {
    let signature = sign_request(method, path, timestamp, &body, secret);
    SignedRequest {
        method: method.to_string(),
        path: path.to_string(),
        timestamp,
        body,
        signature,
    }
}

pub fn verify_request(
    request: &SignedRequest,
    secret: &[u8],
    server_time: u64,
    max_age_secs: u64,
) -> Result<(), SignatureError> {
    // Check timestamp is not too old
    if server_time > request.timestamp && (server_time - request.timestamp) > max_age_secs {
        return Err(SignatureError::TimestampExpired {
            request_time: request.timestamp,
            server_time,
            max_age_secs,
        });
    }

    // Check timestamp is not too far in the future (more than 60 seconds of skew)
    if request.timestamp > server_time && (request.timestamp - server_time) > 60 {
        return Err(SignatureError::TimestampFuture {
            request_time: request.timestamp,
            server_time,
        });
    }

    // Recompute expected signature
    let _expected = sign_request(
        &request.method,
        &request.path,
        request.timestamp,
        &request.body,
        secret,
    );

    // Decode the provided signature from hex
    let sig_bytes = hex::decode(&request.signature)
        .map_err(|_| SignatureError::InvalidSignatureFormat)?;

    // Compute HMAC for constant-time comparison
    let canonical = build_canonical_string(
        &request.method,
        &request.path,
        request.timestamp,
        &request.body,
    );
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(canonical.as_bytes());
    mac.verify_slice(&sig_bytes)
        .map_err(|_| SignatureError::SignatureMismatch)?;

    Ok(())
}

pub fn verify_request_middleware(
    method: &str,
    path: &str,
    timestamp: u64,
    body: &[u8],
    signature: &str,
    secret: &[u8],
    server_time: u64,
    max_age_secs: u64,
) -> Result<(), SignatureError> {
    let request = SignedRequest {
        method: method.to_string(),
        path: path.to_string(),
        timestamp,
        body: body.to_vec(),
        signature: signature.to_string(),
    };
    verify_request(&request, secret, server_time, max_age_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &[u8] = b"test-signing-secret-32-bytes!!";

    #[test]
    fn test_hash_body_deterministic() {
        let h1 = hash_body(b"hello world");
        let h2 = hash_body(b"hello world");
        assert_eq!(h1, h2);
        assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_body_different_inputs() {
        let h1 = hash_body(b"hello");
        let h2 = hash_body(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_build_canonical_string_format() {
        let cs = build_canonical_string("POST", "/api/users", 1700000000, b"body");
        assert!(cs.starts_with("POST\n"));
        assert!(cs.contains("\n/api/users\n"));
        assert!(cs.contains("\n1700000000\n"));
    }

    #[test]
    fn test_sign_and_verify() {
        let sig = sign_request("GET", "/api/users", 1700000000, b"", TEST_SECRET);
        let request = create_signed_request("GET", "/api/users", 1700000000, vec![], TEST_SECRET);
        assert_eq!(request.signature, sig);

        let result = verify_request(&request, TEST_SECRET, 1700000000, 300);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_wrong_secret() {
        let request = create_signed_request("GET", "/api/users", 1700000000, vec![], TEST_SECRET);
        let wrong_secret = b"wrong-signing-secret-32-bytes!!";
        let result = verify_request(&request, wrong_secret, 1700000000, 300);
        assert_eq!(result, Err(SignatureError::SignatureMismatch));
    }

    #[test]
    fn test_verify_expired_timestamp() {
        let request = create_signed_request("GET", "/api/users", 1700000000, vec![], TEST_SECRET);
        let result = verify_request(&request, TEST_SECRET, 1700000600, 300);
        assert!(matches!(result, Err(SignatureError::TimestampExpired { .. })));
    }

    #[test]
    fn test_verify_future_timestamp() {
        let request = create_signed_request("GET", "/api/users", 1700000400, vec![], TEST_SECRET);
        let result = verify_request(&request, TEST_SECRET, 1700000000, 300);
        assert!(matches!(result, Err(SignatureError::TimestampFuture { .. })));
    }

    #[test]
    fn test_verify_tampered_body() {
        let request = create_signed_request("POST", "/api/users", 1700000000, b"original".to_vec(), TEST_SECRET);
        let tampered = SignedRequest {
            method: request.method.clone(),
            path: request.path.clone(),
            timestamp: request.timestamp,
            body: b"tampered".to_vec(),
            signature: request.signature,
        };
        let result = verify_request(&tampered, TEST_SECRET, 1700000000, 300);
        assert_eq!(result, Err(SignatureError::SignatureMismatch));
    }

    #[test]
    fn test_verify_request_middleware_ok() {
        let sig = sign_request("GET", "/api/data", 1700000000, b"", TEST_SECRET);
        let result = verify_request_middleware(
            "GET", "/api/data", 1700000000, b"", &sig, TEST_SECRET, 1700000000, 300,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_request_middleware_bad_sig() {
        let result = verify_request_middleware(
            "GET", "/api/data", 1700000000, b"", "bad00000", TEST_SECRET, 1700000000, 300,
        );
        assert!(result.is_err());
    }
}
