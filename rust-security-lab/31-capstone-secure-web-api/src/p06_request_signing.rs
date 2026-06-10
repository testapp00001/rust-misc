//! # Lesson 06: HMAC Request Signing
//!
//! ## What is Request Signing?
//!
//! Request signing ensures that a request:
//! 1. **Has not been tampered with** (integrity)
//! 2. **Came from the expected sender** (authenticity)
//! 3. **Is not a replay of an old request** (freshness)
//!
//! Even if an attacker can observe the request (e.g., on a network they control),
//! they cannot modify it or replay it without knowing the shared secret.
//!
//! ## Signing Process
//!
//! ```text
//! Client:
//!   1. Build canonical string: METHOD\nPATH\nTIMESTAMP\nBODY_HASH
//!   2. Compute signature = HMAC-SHA256(canonical_string, shared_secret)
//!   3. Send request with headers:
//!      X-Signature: <hex(signature)>
//!      X-Timestamp: <unix_timestamp>
//!
//! Server:
//!   1. Extract signature and timestamp from headers
//!   2. Check timestamp is within acceptable window (e.g., 5 minutes)
//!   3. Recompute canonical string from the request
//!   4. Recompute HMAC and compare (constant-time)
//!   5. Reject if mismatch or timestamp too old (replay prevention)
//! ```
//!
//! ## Attack Context
//!
//! - **Request tampering**: Attacker modifies the body. Defense: body hash in signature.
//! - **Replay attack**: Attacker resends a captured valid request. Defense: timestamp window.
//! - **Signature bypass**: Attacker sends requests without signatures. Defense: require signature.
//! - **Timing attack**: Attacker measures comparison time to guess signature byte by byte.
//!   Defense: use constant-time comparison (the `hmac` crate does this automatically).

use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};

type HmacSha256 = Hmac<Sha256>;

/// A signed request with all the components needed for verification.
#[derive(Debug, Clone)]
pub struct SignedRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path (e.g., "/api/users")
    pub path: String,
    /// Unix timestamp when the request was signed
    pub timestamp: u64,
    /// Request body bytes (empty for GET requests)
    pub body: Vec<u8>,
    /// The HMAC-SHA256 signature (hex-encoded)
    pub signature: String,
}

/// Errors during signature verification.
#[derive(Debug, Clone, PartialEq)]
pub enum SignatureError {
    /// Signature header is missing
    MissingSignature,
    /// Signature is not valid hex
    InvalidSignatureFormat,
    /// Signature does not match the computed value
    SignatureMismatch,
    /// Timestamp is outside the acceptable window (replay prevention)
    TimestampExpired {
        /// The request timestamp
        request_time: u64,
        /// The current server time
        server_time: u64,
        /// The maximum allowed age in seconds
        max_age_secs: u64,
    },
    /// The request timestamp is too far in the future (clock skew protection)
    TimestampFuture {
        request_time: u64,
        server_time: u64,
    },
}

/// Exercise 1: Compute the SHA-256 hash of a body.
///
/// Return the hex-encoded hash. For an empty body, return the hash of empty bytes.
/// This hash becomes part of the canonical string.
pub fn hash_body(body: &[u8]) -> String {
    todo!("Compute SHA-256 hash of request body")
}

/// Exercise 2: Build the canonical string for signing.
///
/// Format: "{METHOD}\n{PATH}\n{TIMESTAMP}\n{BODY_HASH}"
///
/// Each component is on its own line, separated by newlines.
/// The body_hash is the hex-encoded SHA-256 of the body bytes.
pub fn build_canonical_string(
    method: &str,
    path: &str,
    timestamp: u64,
    body: &[u8],
) -> String {
    todo!("Build canonical string for request signing")
}

/// Exercise 3: Sign a request using HMAC-SHA256.
///
/// Steps:
/// 1. Build the canonical string
/// 2. Compute HMAC-SHA256 of the canonical string using the secret
/// 3. Return the hex-encoded signature
pub fn sign_request(
    method: &str,
    path: &str,
    timestamp: u64,
    body: &[u8],
    secret: &[u8],
) -> String {
    todo!("Sign a request with HMAC-SHA256")
}

/// Exercise 4: Create a signed request.
///
/// Build a SignedRequest with the given parameters and a computed signature.
pub fn create_signed_request(
    method: &str,
    path: &str,
    timestamp: u64,
    body: Vec<u8>,
    secret: &[u8],
) -> SignedRequest {
    todo!("Create a signed request with computed signature")
}

/// Exercise 5: Verify a signed request.
///
/// Steps:
/// 1. Check that the timestamp is within `max_age_secs` of `server_time`
///    (not too old AND not more than 60 seconds in the future)
/// 2. Recompute the expected signature
/// 3. Compare signatures (constant-time via the hmac crate)
/// 4. Return Ok(()) if valid, or the appropriate SignatureError
pub fn verify_request(
    request: &SignedRequest,
    secret: &[u8],
    server_time: u64,
    max_age_secs: u64,
) -> Result<(), SignatureError> {
    todo!("Verify a signed request")
}

/// Exercise 6: Build a request signing middleware function.
///
/// Given raw request components (method, path, timestamp, body), the signature
/// from the request header, the secret, the current server time, and max age:
///
/// 1. Build a SignedRequest from the components
/// 2. Verify it
/// 3. Return Ok(()) or the error
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
    todo!("Implement request signing middleware")
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
        // Server time is 600 seconds later, max age is 300
        let result = verify_request(&request, TEST_SECRET, 1700000600, 300);
        assert!(matches!(result, Err(SignatureError::TimestampExpired { .. })));
    }

    #[test]
    fn test_verify_future_timestamp() {
        let request = create_signed_request("GET", "/api/users", 1700000400, vec![], TEST_SECRET);
        // Server time is earlier, more than 60 seconds of skew
        let result = verify_request(&request, TEST_SECRET, 1700000000, 300);
        assert!(matches!(result, Err(SignatureError::TimestampFuture { .. })));
    }

    #[test]
    fn test_verify_tampered_body() {
        let request = create_signed_request("POST", "/api/users", 1700000000, b"original".to_vec(), TEST_SECRET);
        // Create a new request with different body but same signature
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
