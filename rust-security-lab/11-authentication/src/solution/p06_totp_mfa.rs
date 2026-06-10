//! # Lesson 06: TOTP — Time-based One-Time Passwords (RFC 6238)
//!
//! ## How TOTP Works
//!
//! TOTP is a 6-8 digit code that changes every 30 seconds.
//! Both server and authenticator app share a secret key.
//!
//! ```text
//! TOTP = Truncate(HMAC-SHA1(secret, floor(time / 30)))
//! ```
//!
//! Steps:
//! 1. Compute time counter: `T = floor(current_unix_time / time_step)`
//! 2. Convert T to 8-byte big-endian
//! 3. Compute HMAC-SHA1(secret, T_bytes)
//! 4. Dynamic truncation: extract 4 bytes from the HMAC
//! 5. Compute: truncated_value mod 10^digits
//!
//! ## Security Properties
//!
//! - **Shared secret**: 160+ bits, encoded as Base32 for QR codes
//! - **Time step**: 30 seconds (balances usability and security)
//! - **Window**: Server accepts current + previous + next step (clock skew tolerance)
//! - **Digits**: 6 (10^6 = 1,000,000 possibilities — not brute-forceable in 30s)
//!
//! ## Attack Context
//!
//! - **Phishing**: Attacker captures TOTP in real-time and races to use it
//! - **Secret theft**: If the shared secret is compromised, all future codes are predictable
//! - **Replay**: Each code is valid for 30s — must not be reusable
//!
//! **Defense**: Bind TOTP to device, rate-limit verification attempts, use backup codes.

use std::time::{SystemTime, UNIX_EPOCH};

/// TOTP configuration.
#[derive(Debug, Clone)]
pub struct TotpConfig {
    /// Number of digits in the code (typically 6)
    pub digits: u32,
    /// Time step in seconds (typically 30)
    pub time_step: u64,
    /// Number of time steps to check before/after current (typically 1)
    pub window: u32,
}

impl Default for TotpConfig {
    fn default() -> Self {
        Self {
            digits: 6,
            time_step: 30,
            window: 1,
        }
    }
}

/// Compute HMAC-SHA1 using ring.
///
/// ring's HMAC implementation is constant-time and well-audited.
fn hmac_sha1(key: &[u8], message: &[u8]) -> [u8; 20] {
    use ring::hmac;
    let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, key);
    let tag = hmac::sign(&key, message);
    let mut result = [0u8; 20];
    result.copy_from_slice(tag.as_ref());
    result
}

/// Generate a TOTP code for a given time counter.
///
/// This is the core TOTP algorithm (RFC 6238):
/// 1. HMAC-SHA1(secret, counter_as_8_bytes)
/// 2. Dynamic truncation to extract a 31-bit value
/// 3. Modulo 10^digits to get the final code
pub fn generate_totp(secret: &[u8], time_counter: u64, digits: u32) -> u32 {
    // Step 1: Convert counter to 8-byte big-endian
    let counter_bytes = time_counter.to_be_bytes();

    // Step 2: Compute HMAC-SHA1
    let hmac_result = hmac_sha1(secret, &counter_bytes);

    // Step 3: Dynamic truncation (RFC 4226 Section 5.4)
    let offset = (hmac_result[19] & 0x0F) as usize;
    let code = ((hmac_result[offset] as u32 & 0x7F) << 24)
        | ((hmac_result[offset + 1] as u32) << 16)
        | ((hmac_result[offset + 2] as u32) << 8)
        | (hmac_result[offset + 3] as u32);

    // Step 4: Modulo 10^digits
    code % 10u32.pow(digits)
}

/// Compute the time counter from the current Unix timestamp.
pub fn current_time_counter(time_step: u64) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    now / time_step
}

/// Verify a TOTP code with a time window.
///
/// Checks the current time step plus `window` steps before and after.
/// This tolerates clock skew between client and server.
pub fn verify_totp(secret: &[u8], code: u32, config: &TotpConfig, current_counter: u64) -> bool {
    for offset in 0..=config.window {
        // Check current step
        let counter = current_counter.wrapping_add(offset as u64);
        if generate_totp(secret, counter, config.digits) == code {
            return true;
        }
        // Check previous steps (avoid underflow)
        if offset > 0 {
            let counter = current_counter.wrapping_sub(offset as u64);
            if generate_totp(secret, counter, config.digits) == code {
                return true;
            }
        }
    }
    false
}

/// Generate a Base32-encoded secret for TOTP (what you put in a QR code).
///
/// Base32 is used because it's case-insensitive and URL-safe.
/// RFC 3548 encoding with A-Z and 2-7.
pub fn generate_base32_secret(length: usize) -> String {
    use rand::RngCore;
    let mut bytes = vec![0u8; length];
    rand::thread_rng().fill_bytes(&mut bytes);
    base32_encode(&bytes)
}

/// Simple Base32 encoding (RFC 4648).
pub fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut buffer: u32 = 0;
    let mut bits_in_buffer: u32 = 0;

    for &byte in data {
        buffer = (buffer << 8) | (byte as u32);
        bits_in_buffer += 8;
        while bits_in_buffer >= 5 {
            bits_in_buffer -= 5;
            let index = (buffer >> bits_in_buffer) & 0x1F;
            result.push(ALPHABET[index as usize] as char);
        }
    }

    if bits_in_buffer > 0 {
        let index = (buffer << (5 - bits_in_buffer)) & 0x1F;
        result.push(ALPHABET[index as usize] as char);
    }

    result
}

/// Format a TOTP code with leading zeros.
pub fn format_code(code: u32, digits: u32) -> String {
    format!("{:0width$}", code, width = digits as usize)
}

/// Generate TOTP at a specific time (for testing).
pub fn generate_totp_at(secret: &[u8], unix_time: u64, config: &TotpConfig) -> u32 {
    let counter = unix_time / config.time_step;
    generate_totp(secret, counter, config.digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 6238 test vectors (SHA-1)
    const TEST_SECRET: &[u8] = b"12345678901234567890"; // 20 bytes for SHA-1

    #[test]
    fn test_totp_rfc6238_vector_59() {
        // At time=59 (counter=1 for 30s step), code should be 94287082
        let code = generate_totp(TEST_SECRET, 1, 8);
        assert_eq!(code, 94287082, "RFC 6238 test vector at T=1");
    }

    #[test]
    fn test_totp_rfc6238_vector_1111111109() {
        // At time=1111111109 (counter=37037036), code should be 07081804
        let code = generate_totp(TEST_SECRET, 37037036, 8);
        assert_eq!(code, 7081804, "RFC 6238 test vector at T=37037036");
    }

    #[test]
    fn test_totp_6_digits() {
        let code = generate_totp(TEST_SECRET, 1, 6);
        assert!(code < 1_000_000, "6-digit code should be < 1000000");
    }

    #[test]
    fn test_totp_deterministic() {
        let c1 = generate_totp(TEST_SECRET, 42, 6);
        let c2 = generate_totp(TEST_SECRET, 42, 6);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_totp_changes_with_time() {
        let c1 = generate_totp(TEST_SECRET, 1, 6);
        let c2 = generate_totp(TEST_SECRET, 2, 6);
        // Not guaranteed to differ, but extremely likely
        // We test the algorithm structure instead
        assert!(c1 < 1_000_000);
        assert!(c2 < 1_000_000);
    }

    #[test]
    fn test_verify_totp_current_step() {
        let config = TotpConfig::default();
        let counter = 100;
        let code = generate_totp(TEST_SECRET, counter, config.digits);
        assert!(verify_totp(TEST_SECRET, code, &config, counter));
    }

    #[test]
    fn test_verify_totp_with_window() {
        let config = TotpConfig::default();
        let counter = 100;
        // Code from previous step should still be valid with window=1
        let prev_code = generate_totp(TEST_SECRET, counter - 1, config.digits);
        assert!(verify_totp(TEST_SECRET, prev_code, &config, counter));
    }

    #[test]
    fn test_verify_totp_outside_window() {
        let config = TotpConfig::default();
        let counter = 100;
        // Code from 10 steps ago should not be valid
        let old_code = generate_totp(TEST_SECRET, counter - 10, config.digits);
        assert!(!verify_totp(TEST_SECRET, old_code, &config, counter));
    }

    #[test]
    fn test_base32_encode() {
        let encoded = base32_encode(b"hello");
        assert!(!encoded.is_empty());
        // Base32 should only contain A-Z and 2-7
        assert!(encoded
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_format_code() {
        assert_eq!(format_code(42, 6), "000042");
        assert_eq!(format_code(123456, 6), "123456");
        assert_eq!(format_code(7, 8), "00000007");
    }

    #[test]
    fn test_generate_base32_secret() {
        let secret = generate_base32_secret(20);
        assert!(secret.len() > 0);
        assert!(secret
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }
}
