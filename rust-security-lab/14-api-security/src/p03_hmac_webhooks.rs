//! # Lesson 03: HMAC Webhook Verification
//!
//! ## What Are Webhooks?
//!
//! Webhooks are HTTP callbacks: when an event occurs (payment, push, deploy),
//! the server sends an HTTP POST to a URL you configured. But how do you know
//! the webhook really came from the expected service?
//!
//! ## The Problem
//!
//! An attacker who discovers your webhook URL can:
//! 1. Send fake webhook events (fake "payment completed" notifications)
//! 2. Replay old webhook events (duplicate processing)
//! 3. Modify webhook payloads in transit (tamper with data)
//!
//! ## HMAC Webhook Signatures
//!
//! The webhook provider and you share a secret. The provider signs each webhook:
//!
//! ```text
//! Provider:
//!   signature = HMAC-SHA256(webhook_secret, request_body)
//!   Headers:
//!     X-Webhook-Signature: hex(signature)
//!     X-Webhook-Timestamp: unix_epoch_seconds
//!
//! Your server:
//!   expected = HMAC-SHA256(webhook_secret, request_body)
//!   if signature == expected AND timestamp is recent:
//!       Process webhook
//!   else:
//!       Reject
//! ```
//!
//! This is used by Stripe, GitHub, Shopify, and most webhook providers.
//!
//! ## Attack: Signature Exclusion
//!
//! Some implementations allow unsigned webhooks for backward compatibility:
//!
//! ```rust
//! // DANGEROUS — processes unsigned webhooks
//! if let Some(sig) = header("X-Signature") {
//!     verify(sig, body);
//! }
//! process_webhook(body); // Always processes!
//! ```
//!
//! Always require signatures. If verification is optional, an attacker simply
//! omits the signature header.
//!
//! ## Defense
//!
//! 1. Always require a valid HMAC signature
//! 2. Use constant-time comparison
//! 3. Check timestamp freshness (reject webhooks older than 5 minutes)
//! 4. Use the raw request body (don't re-serialize JSON — whitespace differences break signatures)
//! 5. Use a shared secret that's unique per webhook endpoint

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::collections::HashMap;

type HmacSha256 = Hmac<Sha256>;

/// Exercise 1: Generate a webhook signature.
///
/// Given a webhook secret and the raw request body, compute the HMAC-SHA256
/// signature and return it as a hex string.
///
/// This is what the webhook sender (e.g., Stripe) does.
///
/// Hints:
/// - `HmacSha256::new_from_slice(secret)` creates the HMAC instance
/// - `.update(body)` feeds in the raw body bytes
/// - `.finalize().into_bytes()` produces the MAC
/// - `hex::encode(...)` converts to hex
pub fn generate_webhook_signature(secret: &[u8], body: &[u8]) -> String {
    todo!("Generate HMAC-SHA256 signature for a webhook body")
}

/// Exercise 2: Verify a webhook signature.
///
/// Recompute the expected signature and compare it to the provided one
/// using constant-time comparison.
///
/// Returns `true` if the signature is valid.
///
/// Hints:
/// - Compute the expected signature the same way as generate_webhook_signature
/// - Use `ring::constant_time::verify_slices_are_equal` for comparison
/// - Decode the provided hex signature with `hex::decode`
pub fn verify_webhook_signature(secret: &[u8], body: &[u8], provided_signature_hex: &str) -> bool {
    todo!("Verify a webhook signature using constant-time comparison")
}

/// Exercise 3: Verify a webhook with timestamp freshness.
///
/// In addition to signature verification, reject webhooks whose timestamp
/// is more than `max_age_seconds` old.
///
/// The timestamp is provided as a Unix epoch seconds value.
///
/// Returns:
/// - `Ok(())` if valid and fresh
/// - `Err("invalid_signature")` if signature doesn't match
/// - `Err("timestamp_expired")` if the webhook is too old
/// - `Err("timestamp_in_future")` if the timestamp is in the future (clock skew > 60s)
///
/// Hints:
/// - Check timestamp freshness first (cheaper than HMAC computation)
/// - `now.saturating_sub(timestamp) > max_age_seconds` → expired
/// - `timestamp > now + 60` → in the future (allow 60s clock skew)
/// - Then verify the signature
pub fn verify_webhook(
    secret: &[u8],
    body: &[u8],
    signature_hex: &str,
    timestamp: u64,
    current_time: u64,
    max_age_seconds: u64,
) -> Result<(), &'static str> {
    todo!("Verify webhook signature and timestamp")
}

/// Exercise 4: Parse webhook headers and verify.
///
/// Webhooks arrive as HTTP requests with headers. Given a map of headers,
/// extract the signature and timestamp, then verify.
///
/// Expected headers:
/// - `X-Webhook-Signature`: hex-encoded HMAC signature
/// - `X-Webhook-Timestamp`: Unix epoch seconds as a string
///
/// Returns:
/// - `Ok(())` if valid
/// - `Err("missing_signature")` if signature header is missing
/// - `Err("missing_timestamp")` if timestamp header is missing
/// - `Err("invalid_timestamp")` if timestamp can't be parsed as u64
/// - `Err("invalid_signature")` if signature verification fails
/// - `Err("timestamp_expired")` if webhook is too old
///
/// Hints:
/// - Use `headers.get("X-Webhook-Signature")` to extract the signature
/// - Parse timestamp with `str::parse::<u64>()`
/// - Delegate to `verify_webhook` for the actual verification
pub fn verify_webhook_from_headers(
    secret: &[u8],
    body: &[u8],
    headers: &HashMap<String, String>,
    current_time: u64,
    max_age_seconds: u64,
) -> Result<(), &'static str> {
    todo!("Parse headers and verify the webhook")
}

/// Exercise 5: Compute a webhook signature with a prefixed payload.
///
/// Some webhook providers (like GitHub) prefix the body with metadata:
///
/// ```text
/// payload = "{timestamp}.{body}"
/// signature = HMAC-SHA256(secret, payload)
/// ```
///
/// Implement this variant where the signed payload includes a timestamp prefix.
///
/// Hints:
/// - Format: `"{timestamp}.{body_as_utf8}"`
/// - Use the same HMAC computation as before
/// - body is raw bytes; convert to string for concatenation
pub fn generate_prefixed_signature(secret: &[u8], timestamp: u64, body: &[u8]) -> String {
    todo!("Generate HMAC signature over a timestamp-prefixed payload")
}

/// Exercise 6: Verify a GitHub-style webhook signature.
///
/// GitHub uses a specific format:
/// - Header: `X-Hub-Signature-256: sha256=<hex_signature>`
/// - Payload: the raw request body
///
/// Parse the `sha256=` prefix from the header value and verify the signature.
///
/// Hints:
/// - Strip the `sha256=` prefix (check it starts with this)
/// - The remainder is the hex-encoded signature
/// - Verify using the same HMAC computation
pub fn verify_github_webhook(secret: &[u8], body: &[u8], signature_header: &str) -> bool {
    todo!("Verify a GitHub-style webhook signature")
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
        // Should be valid hex
        assert!(hex::decode(&sig).is_ok());
        // SHA-256 HMAC is 32 bytes = 64 hex chars
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
        // Timestamp 120 seconds in the future (beyond 60s tolerance)
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
        // Compute the signature the way GitHub does
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
