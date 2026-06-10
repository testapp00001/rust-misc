//! # Lesson 02: HMAC Request Signing
//!
//! ## Why Sign Requests?
//!
//! When your API receives a request, how do you know:
//! 1. **The sender is authentic** — not an impersonator?
//! 2. **The body wasn't tampered with** — in transit?
//! 3. **The request isn't a replay** — of an older valid request?
//!
//! HMAC request signing solves all three problems.
//!
//! ## How It Works
//!
//! ```text
//! Client:
//!   timestamp = current_time()
//!   body_hash = SHA256(request_body)
//!   string_to_sign = method + "\n" + path + "\n" + timestamp + "\n" + body_hash
//!   signature = HMAC-SHA256(shared_secret, string_to_sign)
//!   Headers:
//!     X-Timestamp: timestamp
//!     X-Signature: hex(signature)
//!
//! Server:
//!   Extract timestamp, signature from headers
//!   Verify timestamp is within acceptable window (e.g., 5 minutes)
//!   Recompute expected signature using same formula
//!   Compare signatures using constant-time comparison
//! ```
//!
//! ## Attack: Replay Attacks
//!
//! Without the timestamp check, an attacker can:
//! 1. Capture a valid signed request
//! 2. Replay it later (e.g., to repeat a payment)
//! 3. The signature is still valid!
//!
//! The timestamp window prevents this: requests older than N minutes are rejected.
//!
//! ## Attack: Signature Exclusion
//!
//! Some APIs make the mistake of only signing the body. An attacker can:
//! - Change the HTTP method (GET → DELETE)
//! - Change the URL path (/users → /admin)
//!
//! Always sign: method + path + timestamp + body_hash
//!
//! ## Defense
//!
//! 1. Sign method + path + timestamp + body hash
//! 2. Use constant-time comparison for signature verification
//! 3. Reject requests outside a tight timestamp window (3-5 minutes)
//! 4. Use HTTPS so the signing secret isn't exposed

use hmac::{Hmac, Mac};
use sha2::{Sha256, Digest};

type HmacSha256 = Hmac<Sha256>;

/// Exercise 1: Create a "string to sign" from request components.
///
/// The string to sign is the canonical representation of the request that both
/// the client and server compute independently. Format:
///
/// ```text
/// "{method}\n{path}\n{timestamp}\n{body_hash}"
/// ```
///
/// Where `body_hash` is the hex-encoded SHA-256 hash of the body.
///
/// Hints:
/// - Use `sha2::Sha256` to hash the body
/// - Use `hex::encode` to convert the hash to hex
/// - Format with newlines between each component
pub fn create_string_to_sign(method: &str, path: &str, timestamp: u64, body: &[u8]) -> String {
    todo!("Create the canonical string to sign")
}

/// Exercise 2: Compute an HMAC-SHA256 signature.
///
/// Given a shared secret and the string to sign, compute the HMAC-SHA256
/// and return it as a hex-encoded string.
///
/// Hints:
/// - `HmacSha256::new_from_slice(secret)` creates a new HMAC instance
/// - `.update(data)` feeds in the data
/// - `.finalize().into_bytes()` gets the result
/// - `hex::encode(...)` converts to hex string
pub fn compute_signature(secret: &[u8], string_to_sign: &str) -> String {
    todo!("Compute HMAC-SHA256 signature")
}

/// Exercise 3: Verify a request signature.
///
/// Recompute the expected signature from the request components and compare
/// it to the provided signature using constant-time comparison.
///
/// Returns `true` if the signature is valid, `false` otherwise.
///
/// Hints:
/// - Recompute the string_to_sign and expected signature
/// - Use `ring::constant_time::verify_slices_are_equal` for constant-time comparison
/// - Convert the hex signature string to bytes with `hex::decode`
pub fn verify_signature(
    secret: &[u8],
    method: &str,
    path: &str,
    timestamp: u64,
    body: &[u8],
    provided_signature_hex: &str,
) -> bool {
    todo!("Verify the request signature using constant-time comparison")
}

/// Exercise 4: Verify a request with timestamp freshness check.
///
/// In addition to verifying the signature, reject requests where the timestamp
/// is too old (more than `max_age_seconds` from `current_time`).
///
/// Returns:
/// - `Ok(())` if the signature is valid AND the timestamp is fresh
/// - `Err("expired")` if the timestamp is too old
/// - `Err("invalid_signature")` if the signature doesn't match
///
/// Hints:
/// - First check: `current_time.saturating_sub(timestamp) <= max_age_seconds`
/// - Also check: `timestamp.saturating_sub(current_time) <= max_age_seconds` (future requests)
/// - Then verify the signature
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
    todo!("Verify signature and check timestamp freshness")
}

/// Exercise 5: Build a complete signed request header set.
///
/// Given the secret, method, path, body, and timestamp, return a HashMap
/// containing all the headers needed for a signed request:
/// - "X-Timestamp": the timestamp as a string
/// - "X-Signature": the hex-encoded HMAC signature
///
/// This is the client-side counterpart to verify_signed_request.
///
/// Hints:
/// - Call create_string_to_sign and compute_signature
/// - Insert into a HashMap
pub fn build_signed_headers(
    secret: &[u8],
    method: &str,
    path: &str,
    body: &[u8],
    timestamp: u64,
) -> std::collections::HashMap<String, String> {
    todo!("Build the complete set of signed request headers")
}

/// Exercise 6: Detect a replay attack.
///
/// Maintain a set of seen (timestamp, path) pairs. If a request with the same
/// pair is seen again, it's a replay. Even if the signature is valid, reject it.
///
/// Returns `true` if this is a new (non-replay) request, `false` if it's a replay.
///
/// Hints:
/// - Use a `HashSet<(u64, String)>` to track seen requests
/// - Check if the pair exists before inserting
/// - This is a simplified model; production systems use nonce-based approaches
pub struct ReplayDetector {
    seen: std::collections::HashSet<(u64, String)>,
}

impl ReplayDetector {
    pub fn new() -> Self {
        todo!("Initialize the replay detector")
    }

    /// Returns true if this is a NEW request, false if it's a replay.
    pub fn check_and_record(&mut self, timestamp: u64, path: &str) -> bool {
        todo!("Check if this request has been seen before and record it")
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
        // body_hash should be hex SHA-256
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
        // Current time is 600 seconds later, max age is 300
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
        // Verify the signature is correct
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
        assert!(!detector.check_and_record(1700000000, "/api/data"), "Should detect replay");
    }

    #[test]
    fn test_replay_detector_different_paths() {
        let mut detector = ReplayDetector::new();
        assert!(detector.check_and_record(1700000000, "/api/data"));
        assert!(detector.check_and_record(1700000000, "/api/other"));
    }
}
